---
phase: 61-performance-clone-reduction-parallel-survivor
reviewed: 2026-06-09T00:00:00Z
depth: standard
files_reviewed: 24
files_reviewed_list:
  - Cargo.toml
  - benches/rastrigin.rs
  - src/engines/cma/engine.rs
  - src/engines/eda/engine.rs
  - src/engines/ga.rs
  - src/engines/gp/engine.rs
  - src/engines/hill_climb/engine.rs
  - src/engines/permutate/engine.rs
  - src/engines/pso/engine.rs
  - src/observe/observer/composite.rs
  - src/observe/observer/log.rs
  - src/observe/observer/metrics_observer.rs
  - src/observe/observer/mod.rs
  - src/observe/observer/tracing_observer.rs
  - src/operations/survivor/age.rs
  - src/operations/survivor/fitness.rs
  - src/operations/survivor/mu_comma_lambda.rs
  - src/operations/survivor/mu_plus_lambda.rs
  - tests/engines/cma/test_cma.rs
  - tests/engines/eda/test_eda.rs
  - tests/engines/hill_climb/test_hill_climb.rs
  - tests/engines/permutate/test_permutate.rs
  - tests/engines/pso/test_pso.rs
  - tests/observe/observer/test_observer.rs
findings:
  critical: 2
  warning: 4
  info: 2
  total: 8
status: issues_found
---

# Phase 61: Code Review Report

**Reviewed:** 2026-06-09
**Depth:** standard
**Files Reviewed:** 24
**Status:** issues_found

## Summary

Phase 61 introduced four changes: parallel `par_sort_unstable_by` in survivor operators (with WASM sequential fallback), a `&U` borrow change on `on_new_best`, removal of a clone in `CompositeObserver` fan-out, and a rastrigin benchmark harness. The WASM gating pattern for `par_sort` is correctly applied across all four survivor operators. The `on_new_best(&U)` migration is complete and consistent across all seven engines. The clone removal in `CompositeObserver` is correct. However, a pre-existing but now clearly visible logic inversion in `age_based` survivor selection is a correctness blocker, and the fitness survivor has an incorrect sort-then-truncate direction for non-FixedFitness that silently keeps wrong individuals in Minimization mode.

---

## Critical Issues

### CR-01: `age_based` survivor sorts descending by age then keeps the front — retains OLDEST, not youngest

**File:** `src/operations/survivor/age.rs:23`

**Issue:** The comparator `b.age().cmp(&a.age())` produces a descending sort (highest age at index 0). `truncate(population_size)` then keeps the front of the vector. The doc comment on line 3 says "Retains the youngest individuals (lowest age)"; the code does the exact opposite and retains the oldest. Any GA configured with `Survivor::Age` will preferentially discard fresh offspring and keep long-lived parents each generation, inverting the intended selection pressure.

**Fix:**
```rust
// Change comparator from descending to ascending so youngest (lowest age) land at front:
#[cfg(not(target_arch = "wasm32"))]
chromosomes.par_sort_unstable_by(|a, b| a.age().cmp(&b.age()));
#[cfg(target_arch = "wasm32")]
chromosomes.sort_unstable_by(|a, b| a.age().cmp(&b.age()));
// truncate(population_size) then keeps the correct (youngest) front
```

---

### CR-02: `fitness_based` Minimization path sorts descending then drains from front — correct in isolation but the pairing with `par_sort_unstable_by` is non-deterministic for equal-fitness chromosomes, and `mu_comma_lambda`/`mu_plus_lambda` share the same asymmetric truncate pattern without sorting by a secondary key

**File:** `src/operations/survivor/fitness.rs:36-79`

**Issue:** For Minimization, the sort is `b.fitness().partial_cmp(&a.fitness())` — descending — so the worst (highest) fitness individuals land at indices 0..excess. Then `drain(0..excess)` discards them and keeps the tail (lowest fitness, best). This is algorithmically correct. However, `par_sort_unstable_by` is used, which is **non-deterministic in tie-breaking** across parallel threads. When many chromosomes share the same fitness (common at convergence), the subset of survivors kept is unpredictable and varies between runs even with the same seed. For a library advertising reproducibility via `rng_seed`, this is a behavioral correctness defect. The sequential `sort_unstable_by` on WASM is also non-deterministic in ties, but at least it is deterministic given the same input order, which parallel sort is not.

**Fix:** Use a secondary sort key (e.g., chromosome age or index) to break ties deterministically:
```rust
chromosomes.par_sort_unstable_by(|a, b| {
    b.fitness()
        .partial_cmp(&a.fitness())
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| a.age().cmp(&b.age())) // younger first among equals
});
```
Apply the same secondary key to `mu_comma_lambda.rs:45`, `mu_plus_lambda.rs:35`, and both FixedFitness branches.

---

## Warnings

### WR-01: `on_new_best` in `ga.rs` fires AFTER `on_generation_end` relative to the call in the observer

**File:** `src/engines/ga.rs:2285`

**Issue:** In `run_with_callback`, the call order within a generation is:
1. `on_generation_end` (line ~2210)
2. `on_new_best` (line 2285) — inside the stagnation-check block that runs after `stats.push` and the `on_generation_end` notify

All other engines (CMA, EDA, GP, PSO, HillClimb, Permutate) fire `on_new_best` before `on_generation_end`. The inconsistency means observers that track "best fitness at generation end" will see a stale value for `Ga<U>` when a new best occurs in the same generation. The test `test_extension_fires_before_generation_end` does not catch this because it tests `on_extension_triggered` ordering, not `on_new_best`.

**Fix:** Move the `on_new_best` notify to before the `on_generation_end` call, mirroring the pattern in all other engines. Or at minimum document the deliberate divergence.

---

### WR-02: Rastrigin benchmark has no `required-features` guard and no `benchmarks` feature gate, but Cargo.toml declares it without one either

**File:** `Cargo.toml:107-108` and `benches/rastrigin.rs:38,77`

**Issue:** The rastrigin benchmark entry in `Cargo.toml` has `harness = false` but no `required-features` field. The existing `de` and `metrics_observer` benches both guard themselves with `required-features`. Without a guard, `cargo bench` will try to compile `rastrigin.rs` even without the `benchmarks` feature, but more importantly `#[cfg(not(tarpaulin_include))]` on the helper functions is the only protection, which only hides from tarpaulin — it does not gate the bench from normal compilation. This is only a quality issue if a user runs `cargo bench` on a minimal feature set and hits unexpected public API exposure, but the benchmark directly accesses internal field `c.dna` (line 50) via `RangeChromosome::new()` without going through a builder pattern consistent with the rest of the codebase.

**Fix:** Either add `required-features = ["benchmarks"]` to the `[[bench]]` block in `Cargo.toml`, or document that direct field access on `RangeChromosome` is intentional in benchmarks.

---

### WR-03: `bench_with_input` passes `&dim` but the setup closure already captures it, so `|mut ga| ga.run()` runs to completion — warm-up iterations include full GA runs

**File:** `benches/rastrigin.rs:84-97`

**Issue:** `iter_batched` with `BatchSize::SmallInput` is correct for one-shot setup cost amortization. However, `max_generations = 50` with `population_size = 500` means each benchmark iteration runs a full 50-generation GA. At `dim=50`, this is a substantial wall-clock cost per sample. Criterion will run this many times and may time out or produce unreliable samples on CI. More importantly, the benchmark measures end-to-end GA runtime rather than isolating the survivor operator, which is what Phase 61 actually changed. The benchmark does not serve as a targeted regression test for the parallelized sort.

**Fix:** Either reduce `max_generations` to 5-10 for CI stability, or rename the group to clarify it measures full GA throughput rather than the survivor operator specifically. Add a dedicated `survivor` bench entry (already exists at `benches/survivor.rs`) that exercises only `fitness_based` with representative population sizes.

---

### WR-04: `TracingObserver` may silently drop span entries when `Mutex::lock` fails due to a poisoned lock

**File:** `src/observe/observer/tracing_observer.rs:95-98`, `105-108`

**Issue:** Every hook uses `.lock().ok()` and silently ignores a poisoned mutex. If `on_generation_start` panics mid-hook and poisons `gen_span`, all subsequent hooks in that generation will get `None` for their entered guard and emit events without the correct span parent. Because the lock is acquired with `.ok()`, no error is propagated or logged. On WASM, where panics unwind differently, this is less likely, but on native targets a user hook or panic in a downstream subscriber could cause silent span-hierarchy corruption.

**Fix:** Use `.lock().unwrap_or_else(|e| e.into_inner())` (poisoned-lock recovery) to always get the inner value, or at minimum add a `#[allow(clippy::mutex_atomic)]` comment explaining the intentional silent-drop policy and its implications.

---

## Info

### IN-01: `Cargo.toml` rastrigin bench entry appears between `alps`/`cellular` and `metrics_observer` — insertion order inconsistency

**File:** `Cargo.toml:107-108`

**Issue:** All other bench entries are grouped logically (operators, then engines). The rastrigin bench was inserted between `cellular` (line 104) and `metrics_observer` (line 119) rather than alongside the other engine benches or at the end. This is a minor ordering inconsistency that has no functional impact but will produce slightly confusing output in `cargo bench --list`.

**Fix:** Move the `[[bench]] name = "rastrigin"` block adjacent to `ga_run` and `island_ga` where full-GA benchmarks are grouped.

---

### IN-02: No test for `on_new_best` receiving `&U` (borrow) vs the old owned `U` — the signature change is only indirectly tested

**File:** `tests/observe/observer/test_observer.rs:79`

**Issue:** The `SpyObserver::on_new_best` in the test file receives `_best: &BinaryChromosome` and does nothing with it beyond incrementing a counter. There is no test that actually reads from the `&U` reference (e.g., calls `.fitness()`, `.dna()`) to verify the reference is valid and not dangling. Since this is a breaking API change for v3.0.0, a test that exercises the reference content would provide stronger evidence of correctness and catch any lifetime issue.

**Fix:** In `test_observer_on_new_best_fires`, after the run, verify the content of the last-seen best chromosome by recording it inside the observer:
```rust
fn on_new_best(&self, _gen: usize, best: &BinaryChromosome) {
    self.data.new_best.fetch_add(1, Ordering::Relaxed);
    let f = best.fitness();
    assert!(f.is_finite(), "on_new_best received chromosome with non-finite fitness");
}
```

---

_Reviewed: 2026-06-09_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
