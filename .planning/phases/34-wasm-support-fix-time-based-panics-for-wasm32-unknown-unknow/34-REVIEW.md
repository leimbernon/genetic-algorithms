---
phase: 34-wasm-support-fix-time-based-panics-for-wasm32-unknown-unknown
reviewed: 2026-05-07T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - .github/workflows/wasm-check.yml
  - src/engines/ga.rs
  - src/engines/nsga2/mod.rs
  - src/observe/reporter/duration.rs
  - tests/observe/observer/test_observer.rs
  - tests/wasm_smoke.rs
findings:
  critical: 1
  warning: 3
  info: 1
  total: 5
status: issues_found
---

# Phase 34: Code Review Report

**Reviewed:** 2026-05-07T00:00:00Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 34 cfg-gates `std::time::Instant` and `rayon` usages for `wasm32-unknown-unknown`. The structural approach is sound: un-gated `Instant` imports (needed for `Option<Instant>` type annotations), cfg-gated `Instant::now()` sites, and duplicated `par_iter` / `iter` blocks. The CI workflow and duration reporter are correct.

Four issues surface across the six files. The most serious is a missing `Differential` mutation variant in the wasm32 branch of `nsga2/mod.rs::create_offspring`, which silently falls through to the generic path and would produce incorrect mutation behaviour. A massive code-duplication warning stems from `parent_crossover` in `ga.rs` being copy-pasted entirely rather than extracted behind a conditional; and the `wasm_smoke.rs` comment block contains a copy-paste duplicate sentence. There is also a test reliability issue with `test_mutation_timing_nonzero`.

---

## Critical Issues

### CR-01: `Differential` mutation variant absent from wasm32 path in `nsga2/mod.rs::create_offspring`

**File:** `src/engines/nsga2/mod.rs:447-480`
**Issue:** The wasm32 sequential path of `create_offspring` is the `else` / `iterator` path in `create_offspring`. Actually `create_offspring` is not cfg-gated: there is only **one** `create_offspring` implementation and it is entirely shared. However, the mutation dispatch inside `create_offspring` (lines 450-479) handles `Cauchy`, `LevyFlight`, and `Polynomial` as named variants but has **no arm for `Differential`**. The non-wasm GA path in `ga.rs` (lines 1443-1450 and 1488-1495) has an explicit `Differential` arm. In `nsga2/mod.rs` the `Differential` mutation method will silently fall through to the generic `factory_with_params` call at line 472-479, which ignores the `differential_f` parameter and cannot perform the cross-population vector difference that Differential Evolution requires (it has no access to the population slice). This is both a logic error (wrong mutation applied) and a latent panic risk depending on how `factory_with_params` handles the Differential variant internally.

**Fix:**
```rust
// In create_offspring, inside the per-child mutation block, add before the else:
} else if mutation_config.method == crate::operations::Mutation::Differential {
    let f = mutation_config.differential_f.unwrap_or(0.5);
    // population slice is available as `population` in create_offspring's scope
    crate::operations::mutation::differential::differential_mutation(
        child,
        &population.iter().map(|ind| &ind.chromosome).cloned().collect::<Vec<_>>(),
        // Use 0 as a fallback target index — or thread the index through
        0,
        f,
    )?;
}
```
Note: the real fix may require restructuring `create_offspring` to thread the parent index through, or document that `Differential` is explicitly unsupported in NSGA-II and return a `GaError::ConfigurationError` early in `validate()`.

---

## Warnings

### WR-01: Massive duplication — `parent_crossover` body copy-pasted verbatim for wasm32

**File:** `src/engines/ga.rs:1362-1735`
**Issue:** The entire inner body of `parent_crossover` (roughly 180 lines of crossover + mutation + fitness logic) is duplicated: once under `#[cfg(not(target_arch = "wasm32"))]` using `par_iter` and once under `#[cfg(target_arch = "wasm32")]` using `iter`. The only difference between the two blocks is `parents.par_iter()` vs `parents.iter()`. Any future bug fix or logic change in one branch must be manually mirrored to the other, and the history of phase 33 fixes (WR-03, WR-04 for Polynomial/Cauchy/LevyFlight) was already applied to both — one miss would produce a silent behavioural divergence between wasm32 and native builds.

**Fix:** Extract the per-pair closure into a named function and cfg-gate only the iterator:
```rust
fn process_pair<U>(...) -> Result<Vec<U>, GaError> { /* single copy of logic */ }

#[cfg(not(target_arch = "wasm32"))]
let results: Vec<_> = parents.par_iter().map(|(k,v)| process_pair(...)).collect();
#[cfg(target_arch = "wasm32")]
let results: Vec<_> = parents.iter().map(|(k,v)| process_pair(...)).collect();
```

### WR-02: `test_mutation_timing_nonzero` and `test_fitness_eval_timing_nonzero` assert `>= Duration::ZERO` — always true, hides real intent

**File:** `tests/observe/observer/test_observer.rs:511-515` and `528-532`
**Issue:** The assertion `d.unwrap() >= Duration::ZERO` is a tautology — `Duration` is unsigned and can never be negative. The comment acknowledges this ("we accept `Some(Duration::ZERO)` as passing") but that means the test does not actually verify that timing is plausible. On a very fast CI machine `t.elapsed()` can legitimately return `Duration::ZERO`, so no new information is gained beyond "the hook fired". The real guard `d.is_some()` already covers that. The misleading assertion creates false confidence that timing is being validated.

**Fix:** Either remove the tautological `>=` assertion entirely and keep only `is_some()`, or strengthen it to actually assert a non-zero duration — but only if it is safe to do so with a known-slow workload:
```rust
// Option A: honest minimal assertion
assert!(d.is_some(), "on_mutation_complete should have been called");

// Option B: if workload is large enough to guarantee non-zero on any machine
assert!(d.unwrap() > Duration::ZERO, "Duration should be non-zero");
```

### WR-03: `wasm_smoke.rs` has no `with_problem_solving` — relies on default `Minimization`, test comment says any termination is acceptable but assertion is too weak

**File:** `tests/wasm_smoke.rs:27-51`
**Issue:** The smoke test does not call `with_problem_solving`, so the GA runs under the default `ProblemSolving::Minimization` (fitness target = 0.0). With `count_ones` as the fitness function and random initialization, it is possible for a chromosome to have 0 true bits, making `limit_reached` return `true` on the first generation and terminating via `FitnessTargetReached` rather than the generation limit. This means `max_duration_secs` time-limit code path is never reached even conceptually — the test's stated goal ("exercises the non-wasm32 native path of the cfg-gated time-limit check") is not met when the run exits early. It also means on the first generation `stagnation_count` tracking and convergence checking are bypassed, so much of the wasm32 cfg-gated code goes untested.

**Fix:** Add `with_problem_solving(ProblemSolving::Maximization)` so the run always completes all 5 generations:
```rust
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::traits::ConfigurationT;

let mut ga: Ga<BinaryChromosome> = Ga::new()
    .with_population_size(8)
    .with_genes_per_chromosome(8)
    // ...
    .with_problem_solving(ProblemSolving::Maximization)  // add this
    .with_max_generations(5)
    // ...
```

---

## Info

### IN-01: Duplicate comment in `wasm_smoke.rs`

**File:** `tests/wasm_smoke.rs:46-49`
**Issue:** Lines 46-47 and 48-49 are identical comments left by a copy-paste:
```
// Reaching this line proves no panic from Instant::now() (cfg-gated)
// and no panic from rayon (cfg-gated). Any clean termination is acceptable.
// Reaching this line proves no panic from Instant::now() (cfg-gated)
// and no panic from rayon (cfg-gated). Any clean termination is acceptable.
```
**Fix:** Delete the duplicate pair (lines 48-49).

---

_Reviewed: 2026-05-07T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
