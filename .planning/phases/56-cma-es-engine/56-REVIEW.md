---
phase: 56-cma-es-engine
reviewed: 2026-06-01T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - examples/cma_es_rastrigin.rs
  - src/engines/cma/configuration.rs
  - src/engines/cma/engine.rs
  - src/engines/cma/mod.rs
  - src/engines/de/crossover.rs
  - src/engines/de/engine.rs
  - src/engines/de/gene.rs
  - src/engines/de/mod.rs
  - src/engines/de/mutation.rs
  - src/engines/scatter/engine.rs
  - src/lib.rs
  - src/traits.rs
  - src/traits/real_gene.rs
  - tests/engines/cma/test_cma.rs
  - tests/gp.rs
  - tests/test_engines.rs
  - tests/test_variable_length.rs
  - tests/traits/test_self_adaptive.rs
findings:
  critical: 1
  warning: 5
  info: 5
  total: 11
status: issues_found
---

# Phase 56: Code Review Report

**Reviewed:** 2026-06-01
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

Phase 56 delivers the CMA-ES engine (`src/engines/cma/`), touches DE and Scatter engines via the `RealGene` trait relocation, and wires in the new `cma` public module. The CMA-ES algorithm implementation is algorithmically sound and follows Hansen's reference algorithm (arXiv:1604.00772) correctly. The eigendecomposition, covariance update, step-size adaptation, and evolution path updates are all consistent with the reference. The observer lifecycle hooks are correct.

Critical issues found: one panic-path in `DeEngine::run` on empty population (no guard, unlike `CmaEngine` which has an explicit guard). Several warnings cover a dead orphan file, a DE mutation correctness gap (best/current-to-best strategies can draw the best individual as a difference vector), a weak test that always passes, a double `init_fn` call on the auto-size path, and a subtle algorithmic deviation in L-SHADE. Info items cover doc count inconsistency, variable-length fitness silently truncated in Scatter, and magic numbers.

---

## Critical Issues

### CR-01: `DeEngine::run` panics with unclear message on empty population

**File:** `src/engines/de/engine.rs:87-94`

**Issue:** `DeEngine::run()` calls `(self.init_fn)(pop_size)` at line 87 and immediately calls `self.find_best(&pop)` at line 94 without any emptiness guard. `find_best` unconditionally accesses `pop[0]` (line 223). If `init_fn` returns an empty `Vec` (e.g., when `pop_size = 0` via a misconfigured `DeConfiguration`, or when the user's closure returns fewer items than requested), this panics with an index-out-of-bounds message rather than a meaningful error. `CmaEngine` already has an explicit guard and panic message for the same scenario — `DeEngine` should be consistent.

**Fix:**
```rust
let mut pop: Vec<U> = (self.init_fn)(pop_size);
if pop.is_empty() {
    panic!("DeEngine: init_fn returned an empty population");
}
for ind in &mut pop {
```

---

## Warnings

### WR-01: `src/engines/de/gene.rs` is an orphan file — dead code with a false compatibility claim

**File:** `src/engines/de/gene.rs:1-3`

**Issue:** The file comment claims it provides "internal module-path compatibility during the transition" from a previous trait location. However, `de/mod.rs` does **not** declare `pub mod gene`, so `gene.rs` is never compiled. Any code attempting to use `crate::engines::de::gene::RealGene` or `genetic_algorithms::de::gene::RealGene` would still fail to compile. The stated purpose is unachievable with the current module structure. The file is invisible dead code.

**Fix:** Remove `src/engines/de/gene.rs` entirely. If backward path compatibility was genuinely needed for the public API, add `pub mod gene;` to `src/engines/de/mod.rs` — but the trait is already re-exported as `genetic_algorithms::traits::RealGene` and `genetic_algorithms::RealGene`, so no compatibility shim is needed.

---

### WR-02: DE `Best1` and `CurrentToBest1` strategies can include `best_idx` as a difference-vector donor

**File:** `src/engines/de/mutation.rs:71-91`

**Issue:** `pick_distinct` only excludes the target index `i` from the random donor pool. For `Best1` (`mutant_from_base(pop[best_idx], pop[rs[0]], pop[rs[1]], ...)`) and `CurrentToBest1`, `rs[0]` or `rs[1]` can equal `best_idx`. For `Best1` this means `v = best + F*(best - rs[1])` or `v = best + F*(rs[0] - best)`, which degenerates the difference vector. For `CurrentToBest1` the additional difference `F*(rs[0] - rs[1])` can contain `best` twice. While not a crash, this deviates from the algorithm specification where `r1 ≠ r2 ≠ best ≠ i`. This reduces diversity and can cause premature convergence.

**Fix:** In `mutate`, pass an additional exclusion set to `pick_distinct` for strategies that use `best_idx`:
```rust
DeMutationStrategy::Best1 => {
    let rs = pick_distinct_excluding(rng, pop.len(), &[i, best_idx], 2);
    // ...
}
DeMutationStrategy::CurrentToBest1 => {
    let rs = pick_distinct_excluding(rng, pop.len(), &[i, best_idx], 2);
    // ...
}
```
Alternatively, add a post-pick check that replaces collisions with fresh random indices.

---

### WR-03: `CmaEngine::run` calls `init_fn` twice when `population_size == 0`

**File:** `src/engines/cma/engine.rs:428-462`

**Issue:** When `population_size == 0` (the auto-size default from `CmaConfiguration::default()`), `run()` calls `init_fn(1)` to peek at the problem dimension, then calls `init_fn(lambda)` again to build the real population. If the user's `init_fn` has side effects — e.g., calling `rng::set_seed(Some(N))` (as the example does in `examples/cma_es_rastrigin.rs`) — the seed is reset on the second call and the sequence differs from a single call with the correct size. This is surprising and hard to debug. `CmaConfiguration::default()` is a common entry point (e.g., `CmaConfiguration::default().with_sigma0(0.5)`), so real users will hit this.

**Fix:** Infer `n` from the config or documentation contract, or change the API so `init_fn` receives just the dimension and the engine builds the population internally. As a minimal fix, document prominently in `CmaEngine::new` that `init_fn` may be called twice when `population_size == 0`:
```rust
/// * `init_fn` — called with `population_size`. NOTE: if `population_size == 0`,
///   this function is called twice: once with `1` to determine the problem dimension,
///   and once with the auto-computed `lambda`. Avoid side effects inside `init_fn`.
```

---

### WR-04: `test_cma_observer_new_best` (CMA-05) always passes regardless of optimization quality

**File:** `tests/engines/cma/test_cma.rs:196-218`

**Issue:** The test asserts `spy.new_best_count >= 1`. This passes unconditionally because `CmaEngine::run` fires `on_new_best` before the main loop starts (line 500 of `engine.rs`) for the initial best individual. The test does not distinguish between "the engine fires the initial hook" and "the engine actually finds a better solution during optimization". A genuine regression where the in-loop `on_new_best` notification is removed would go undetected.

**Fix:** Split into two assertions: one for the initial notification (≥ 1) and one for optimization improvement (the final population best should be better than the initial best over 200 generations of sphere minimization):
```rust
// CMA must fire at least once (initial best)
assert!(spy.new_best_count.load(Ordering::SeqCst) >= 1, "...");
// Fitness must have improved (otherwise on_new_best during optimization is not tested)
assert!(
    result.best_fitness < initial_best_fitness,
    "Engine should improve on sphere within 200 generations"
);
```

---

### WR-05: `LShadeState::update` does not advance `k` when no improvements occur in a generation

**File:** `src/engines/de/mutation.rs:298-307`

**Issue:** The write index `k` only advances when `!self.s_f.is_empty()` (i.e., at least one successful trial occurred). In standard L-SHADE, `k` advances every generation regardless of success. The current implementation means that if many consecutive generations produce no improvements, the same memory slot `m_f[k]` / `m_cr[k]` never gets updated, and the history ring stalls. This can cause all individuals to draw from stale memory entries during convergence phases, reducing the adaptive benefit of the history.

**Fix:**
```rust
pub fn update(&mut self) {
    let h = self.m_f.len();
    if !self.s_f.is_empty() {
        self.m_f[self.k] = lehmer_mean(&self.s_f);
        self.m_cr[self.k] = arithmetic_mean(&self.s_cr);
    }
    // Always advance k, even on unsuccessful generations (standard L-SHADE behavior)
    self.k = (self.k + 1) % h;
    self.s_f.clear();
    self.s_cr.clear();
}
```

---

## Info

### IN-01: `lib.rs` engine count is inconsistent — "13" in intro vs "12 total" in section header

**File:** `src/lib.rs:4` and `src/lib.rs:58`

**Issue:** Line 4 says "Provides 13 optimization engines" while the section heading at line 58 says "## Engines (12 total)". The actual module count (including `hill_climb`, `permutate`, `cma`, and the two engines sharing a table row) is higher than both numbers. The discrepancy creates confusion for new users.

**Fix:** Audit the actual engine count including `HillClimbEngine` and `PermutateEngine` (which appear in `lib.rs` exports at lines 359, 361 but are absent from the table), then update both references to the same correct number.

---

### IN-02: `euclidean_distance` in `ScatterEngine` silently truncates variable-length chromosomes

**File:** `src/engines/scatter/engine.rs:296-305`

**Issue:** `euclidean_distance` computes distance over `a.len().min(b.len())` dimensions. For two chromosomes of lengths 3 and 7, only the first 3 genes are compared. This means longer chromosomes are never penalised for their extra genes in the diversity metric, and the diverse-selection step of the reference set construction can be skewed. The function has no comment explaining this truncation contract.

**Fix:** Add a doc comment explaining the truncation policy. If Scatter Search is not intended to support variable-length chromosomes, add an assertion `debug_assert_eq!(a.len(), b.len())`. If it is intended, consider padding with zeros or documenting the trade-off.

---

### IN-03: `CmaEngine::find_best` and `DeEngine::find_best` / `ScatterEngine::find_best` are copy-pasted identically

**File:** `src/engines/cma/engine.rs:406-416`, `src/engines/de/engine.rs:221-231`, `src/engines/scatter/engine.rs:244-254`

**Issue:** The three `find_best` methods are identical in structure and semantics. Any future bug fix (e.g., the empty-slice UB) must be applied three times.

**Fix:** Extract to a shared utility function in a common module, or to a default method on a `BestTracking<U>` trait.

---

### IN-04: `CMA-07` and `CMA-08` tests provide no runtime assertion and give false confidence

**File:** `tests/engines/cma/test_cma.rs:265-282`

**Issue:** Both tests create a closure `let _: fn(CmaConfiguration) = |_| {}` and return immediately with the comment "No assertion needed — if this file compiles, the cma module wiring is intact." A compile-time check does not verify runtime behavior, and the tests could be removed without affecting correctness coverage. They occupy test slots (CMA-07, CMA-08) without adding value.

**Fix:** Either remove these tests or replace them with minimal runtime smoke tests that actually exercise the DE and Scatter engines with `RangeGene<f64>` to verify end-to-end behavior post-rename.

---

### IN-05: Magic number `1e-6` used in `FixedFitness` stopping criterion without named constant

**File:** `src/engines/cma/engine.rs:401`, `src/engines/de/engine.rs:272`, `src/engines/scatter/engine.rs:291`

**Issue:** The tolerance `1e-6` for `ProblemSolving::FixedFitness` stopping is a magic number duplicated across three engines. If the tolerance needs adjustment, it must be changed in all three places.

**Fix:** Define a shared constant:
```rust
/// Tolerance for FixedFitness stopping criterion.
const FIXED_FITNESS_TOLERANCE: f64 = 1e-6;
```

---

_Reviewed: 2026-06-01_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
