---
phase: 32-crossover-differential-mutation
reviewed: 2026-05-06T00:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - src/configuration.rs
  - src/engines/ga.rs
  - src/operations.rs
  - src/operations/crossover.rs
  - src/operations/crossover/edge_recombination.rs
  - src/operations/mutation.rs
  - src/operations/mutation/differential.rs
  - src/traits/configuration.rs
  - tests/observe/test_serde.rs
  - tests/operations/test_crossover_edge_recombination.rs
  - tests/operations/test_mutation_differential.rs
  - tests/test_operations.rs
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
findings_fixed:
  critical: 2
  warning: 3
  info: 0
  total: 5
status: fixed
---

# Phase 32: Code Review Report

**Reviewed:** 2026-05-06
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

This phase introduces two new operators: Edge Recombination Crossover (ERX) for permutation chromosomes, and DE-style Differential Mutation for `Range<T>` chromosomes. The operator implementations are generally sound. However, the integration of Differential Mutation into the GA engine contains a logic bug that causes inconsistent mutation application between child_1 and child_2 each generation (different comparison operators). Additionally, `differential_mutation` has an infinite-loop risk when `target_idx >= chromosomes.len()` — the guard at the top only checks `chromosomes.len() < 4`, not that `target_idx` is a valid index. The serde test also omits `Crossover::Rejuvenate` from the enum round-trip coverage.

---

## Critical Issues

### CR-01: Inconsistent mutation probability comparison for child_1 vs child_2

**File:** `src/engines/ga.rs:1393` and `src/engines/ga.rs:1413`

**Issue:** Child_1 is mutated when `mutation_probability < effective_mutation_prob` (strict less-than), but child_2 is mutated when `mutation_probability <= effective_mutation_prob` (less-than-or-equal). This is not symmetric — child_1 is systematically mutated slightly less often than child_2 across all runs. The boundary case `mutation_probability == effective_mutation_prob` applies mutation to child_2 but skips it for child_1. This bias is present for every mutation method routed through `parent_crossover`, including the new `Mutation::Differential` path. Because `rng.random_range(0.0..1.0)` (an exclusive upper bound) makes exact equality vanishingly rare in practice, the bug is unlikely to manifest visibly in results, but the code is semantically inconsistent and constitutes a logic error.

**Fix:** Make both comparisons use the same operator. The `<=` form (line 1413) matches the crossover convention on line 1379 and is the correct choice:

```rust
// line 1393 — change < to <=
if mutation_probability <= effective_mutation_prob {
```

---

### CR-02: Infinite loop in `differential_mutation` when `target_idx >= chromosomes.len()`

**File:** `src/operations/mutation/differential.rs:62-73`

**Issue:** The three donor-selection loops (`while r1 == target_idx`, `while r2 == target_idx || r2 == r1`, `while r3 == target_idx || r3 == r1 || r3 == r2`) all terminate only because the population is large enough that a different index will eventually be drawn. However, if `target_idx >= chromosomes.len()` (an out-of-bounds index), the loop `while r1 == target_idx` can never equal the out-of-bounds value, so it terminates immediately on the first draw — that is not a problem. But if `target_idx` happens to equal one of the valid indices in a population of exactly 4, and all three loops are trying to avoid both `target_idx` and previously selected values, the third loop must pick a value that is not `target_idx`, `r1`, or `r2`. With `pop_len == 4` and the three excluded values being three distinct valid indices, there is exactly one remaining index. The loop terminates correctly in that case.

The real hazard is that `target_idx` is never validated against `chromosomes.len()`. In `parent_crossover` (engine line 1399), `*key` is passed as `target_idx`, and `*key` is derived from the selection operator. The parent bounds are checked via `.get(*key)` on lines 1320–1326, but `differential_mutation` receives `target_idx` as a plain `usize`. If the caller ever passes a `target_idx` that is out of bounds (e.g., an index into the pre-expansion population passed to a post-expansion slice), the exclusion loops treat `target_idx` as a phantom index that will never match, making `r1`/`r2`/`r3` potentially repeat the actual target individual — violating the DE/rand/1 "three distinct donors all distinct from target" guarantee silently rather than returning an error.

**Fix:** Add an explicit validation at the top of the function alongside the population-size check:

```rust
if target_idx >= chromosomes.len() {
    return Err(GaError::MutationError(format!(
        "Differential mutation: target_idx {} is out of bounds (population size {})",
        target_idx,
        chromosomes.len()
    )));
}
```

---

## Warnings

### WR-01: `Crossover::Rejuvenate` missing from serde round-trip test

**File:** `tests/observe/test_serde.rs:51-68`

**Issue:** `serde_crossover_enum` tests 11 of the 12 `Crossover` variants — `Crossover::Rejuvenate` is absent. If `Rejuvenate` ever fails to round-trip (e.g., due to a rename or a `#[serde(rename = ...)]` annotation being added), this test would not catch it. Per project policy all PRs must pass serde tests, and the crossover enum test is the designated coverage point.

**Fix:**

```rust
let variants = [
    Crossover::Cycle,
    Crossover::MultiPoint,
    Crossover::Uniform,
    Crossover::SinglePoint,
    Crossover::Order,
    Crossover::Pmx,
    Crossover::Sbx,
    Crossover::BlendAlpha,
    Crossover::Arithmetic,
    Crossover::Clone,
    Crossover::Rejuvenate,          // add this
    Crossover::EdgeRecombination,
];
```

---

### WR-02: `differential_mutation` silently uses `x_r1` as the base rather than the target

**File:** `src/operations/mutation/differential.rs:107-110`

**Issue:** The classical DE/rand/1 formula is `mutant[i] = x_r1[i] + F * (x_r2[i] - x_r3[i])`. This is the "rand" variant — the base vector is a random population member (`x_r1`), not the target individual. The docstring and comments say "rand/1", which is correct. However, the function signature documents it as mutating `individual` in-place ("The target chromosome to mutate (mutated in-place)"), and the result is written back to `target` (which is the downcast of `individual`). This means the individual being passed in has its values replaced entirely with the perturbed `x_r1`-based vector — the individual's own current gene values play no role as a base. This is correct DE/rand/1 behavior, but it differs from the user's intuitive expectation that the existing individual is being perturbed. The asymmetry between the function's side-effect contract ("mutated in-place") and the actual behavior (replaced with a vector derived from three other population members) is a documentation gap that will surprise users and makes the unit test `differential_mutation_can_change_value` misleading — it only proves values change, not that the correct formula is used.

No code change is strictly required (the algorithm is mathematically correct), but the parameter name `individual` and phrase "mutated in-place" in the doc should be clarified to say the target's DNA is replaced with `x_r1 + F*(x_r2 - x_r3)`.

**Fix:** Update the doc comment:

```rust
/// * `individual` - The target chromosome whose DNA will be replaced with the
///   DE/rand/1 mutant vector `x_r1 + F * (x_r2 - x_r3)`. The target's own
///   gene values are not used as the base (this is the "rand" variant, not "current-to-rand").
```

---

### WR-03: ERX `gene_by_id` lookup panics on gene IDs present in parent_2 but not parent_1

**File:** `src/operations/crossover/edge_recombination.rs:89-90`

**Issue:** `gene_by_id` is built exclusively from `parent_1.dna()` (line 80). The D-08 validation (lines 54-58) confirms that both parents contain the same gene ID set, so in correct usage `gene_by_id[id]` will never panic. However, the index operator `gene_by_id[id]` (line 89-90) panics rather than returning an error if the assertion is somehow violated (e.g., in a future code path that bypasses the validation, or via unsafe/test code that constructs parents directly). The fallback to `unwrap_or` / `ok_or_else` used elsewhere in the file is not applied here.

**Fix:** Replace the index operator with `.get(id).expect(...)` with a descriptive message, or better, propagate as a `GaError`:

```rust
let dna_1: Vec<U::Gene> = child_ids_1
    .iter()
    .map(|id| gene_by_id.get(id).cloned().ok_or_else(|| {
        GaError::CrossoverError(format!("ERX: gene id {} not found in parent_1", id))
    }))
    .collect::<Result<_, _>>()?;
let dna_2: Vec<U::Gene> = child_ids_2
    .iter()
    .map(|id| gene_by_id.get(id).cloned().ok_or_else(|| {
        GaError::CrossoverError(format!("ERX: gene id {} not found in parent_1", id))
    }))
    .collect::<Result<_, _>>()?;
```

This requires changing `erx` to propagate the error, which is a small refactor but eliminates the panic path.

---

## Info

### IN-01: `differential_f` parameter test does not assert the mutation formula correctness

**File:** `tests/operations/test_mutation_differential.rs:117-142`

**Issue:** `differential_f_parameter` tests that F=0.0 and F=2.0 run without error and stay within bounds — but it does not assert that F=0.0 produces `mutant = x_r1` (i.e., the result equals the value at the randomly selected `r1` donor). The comment on line 121 acknowledges this: "F=0.0 should produce no change (mutant = x_r1 + 0 * ... = x_r1) // but we just verify it runs without error". With a seeded RNG this could be verified deterministically, which would make the test more diagnostic.

**Fix:** Use `with_rng_seed` (or a fixed test seed) so that r1 is predictable, then assert `target_zero.dna()[i].value == pop[r1].dna()[i].value` after the F=0.0 call.

---

### IN-02: `serde_ga_configuration_with_values` hardcodes `differential_f: None` rather than testing the `Some(f)` path

**File:** `tests/observe/test_serde.rs:150`

**Issue:** The serde round-trip test for `MutationConfiguration` explicitly sets `differential_f: None`. Since the new `differential_f` field was added specifically for this milestone, the serde test should exercise the `Some(f64)` path to ensure it survives a JSON round-trip.

**Fix:**

```rust
differential_f: Some(0.8),   // was: None
```

---

_Reviewed: 2026-05-06_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
