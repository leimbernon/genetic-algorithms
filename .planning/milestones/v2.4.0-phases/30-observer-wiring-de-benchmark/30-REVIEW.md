---
phase: 30-observer-wiring-de-benchmark
reviewed: 2026-05-02T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - src/engines/de/engine.rs
  - src/engines/scatter/engine.rs
  - src/engines/cellular/engine.rs
  - src/engines/alps/engine.rs
  - tests/engines/de/test_de.rs
  - tests/engines/scatter/test_scatter.rs
  - tests/engines/cellular/test_cellular.rs
  - tests/engines/alps/test_alps.rs
  - benches/de.rs
findings:
  critical: 3
  warning: 4
  info: 2
  total: 9
status: issues_found
---

# Phase 30: Code Review Report

**Reviewed:** 2026-05-02T00:00:00Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase 30 wires the five `GaObserver` lifecycle hooks into four new engines (DE, Scatter, Cellular, ALPS) and adds a DE-vs-GA benchmark. The observer wiring pattern is consistent and mechanically correct: all five hooks are present in every engine, `notify()` is zero-cost when `observer` is `None`, and the per-generation constraints (CellularEngine fires `on_new_best` at most once, AlpsEngine stats are merged across all layers) are satisfied.

Three correctness defects require attention before shipping:

1. `DeEngine::find_best` and `ScatterEngine::find_best` index `pop[0]` unconditionally — a panic if the caller supplies an empty population or if the init function returns fewer elements than expected.
2. The termination-cause classification in all three deterministic engines (DE, Scatter, ALPS) is wrong when early-stopping fires on the last generation, causing `FitnessTargetReached` to be reported as `GenerationLimitReached`.
3. `AlpsEngine` initialises `best_fitness` to `f64::NAN` then immediately calls `is_better(prev_best_fitness = NaN, …)` at the end of generation 0; the NaN guard in `is_better` only protects the `current` argument, so the comparison `is_better(best_fitness, NaN)` fires `on_new_best` unconditionally on the first generation regardless of whether the fitness actually improved, producing a spurious notification.

---

## Critical Issues

### CR-01: Panic on empty population in `DeEngine::find_best` and `ScatterEngine::find_best`

**File:** `src/engines/de/engine.rs:258`, `src/engines/scatter/engine.rs:276`

**Issue:** Both `find_best` implementations index `pop[0]` without a length guard. `DeEngine::run` calls this immediately after `(self.init_fn)(pop_size)` (line 105) — if the caller's `init_fn` returns an empty `Vec`, the very next call at line 112 panics with an index-out-of-bounds. The `ScatterEngine` assertion on line 105 only covers `pool`, not `ref_set`; `ref_set` can be shorter than expected when `pool.len() < quality_count` and `remaining` is exhausted, but `find_best` is called on `ref_set` at line 126 without checking emptiness.

**Fix:**
```rust
fn find_best(&self, pop: &[U]) -> (usize, f64) {
    assert!(!pop.is_empty(), "find_best called on empty population");
    let mut best_idx = 0;
    let mut best_fit = pop[0].fitness();
    for (i, ind) in pop.iter().enumerate().skip(1) {
        if self.is_better(ind.fitness(), best_fit) {
            best_fit = ind.fitness();
            best_idx = i;
        }
    }
    (best_idx, best_fit)
}
```

Alternatively, return `Option<(usize, f64)>` (matching `AlpsEngine::find_best`) and handle the empty case at call sites.

---

### CR-02: Wrong `TerminationCause` when early-stopping fires on the final generation

**File:** `src/engines/de/engine.rs:244-248`, `src/engines/scatter/engine.rs:194-198`, `src/engines/alps/engine.rs:312-316`

**Issue:** All three engines use this classification pattern:

```rust
let cause = if generations < self.config.max_generations {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
```

When the loop runs to completion (`gen == max_generations - 1`) and the fitness target is met on the last iteration, `break` is executed, `generations` is incremented to equal `max_generations`, and the condition `generations < max_generations` is `false`. The cause is therefore reported as `GenerationLimitReached` even though a target was reached. Downstream consumers (observers, callers) receive misleading termination information. The test `test_de_observer_fires_5_hooks` avoids this edge case by setting a hard generation limit with no fitness target, so the bug is not caught by the test suite.

**Fix:** Track the cause explicitly with a bool or enum during the loop:

```rust
let mut target_reached = false;
for gen in 0..self.config.max_generations {
    // ... evolution ...
    if let Some(target) = self.config.fitness_target {
        if self.reached_target(best_fitness, target) {
            target_reached = true;
            break;
        }
    }
}
let cause = if target_reached {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
```

---

### CR-03: `AlpsEngine` fires spurious `on_new_best` on generation 0 due to NaN propagation

**File:** `src/engines/alps/engine.rs:137-163`, `src/engines/alps/engine.rs:275`

**Issue:** `best_fitness` is initialised using a `fold` starting with `f64::NAN` (line 140). At the top of the first iteration `prev_best_fitness = best_fitness` captures this NaN (line 163). After evolution, any real fitness value passes `is_better(best_fitness, NaN)`. `is_better` guards only when `current.is_nan()` returns true (line 373), meaning `is_better(real_value, NaN)` returns `true`. Then `is_better(best_fitness, prev_best_fitness)` at line 275 is `is_better(real, NaN)` which is also `true`, unconditionally firing `on_new_best` on the first generation. This is a correctness defect: the observer contract is that `on_new_best` fires only when the global best actually improved compared to the previously reported best, but here it fires without any real improvement having occurred between `prev_best_fitness` and the new value.

The fix also matters because `prev_best_fitness` is set from `best_fitness` which was computed before the main loop starts, so after a real initial evaluation `best_fitness` should never be NaN. The root problem is choosing NaN as the seed for the fold rather than using the direction-appropriate sentinel:

**Fix:**
```rust
// Replace the NaN-seeded fold with a direction-aware sentinel.
let mut best_fitness = match self.config.problem_solving {
    ProblemSolving::Minimization | ProblemSolving::FixedFitness => f64::MAX,
    ProblemSolving::Maximization => f64::MIN,
};
for ind in &layers[0] {
    if self.is_better(ind.fitness(), best_fitness) {
        best_fitness = ind.fitness();
    }
}
```

This ensures `prev_best_fitness` on iteration 0 is a real sentinel value and `on_new_best` fires only when a genuine improvement occurs.

---

## Warnings

### WR-01: `AlpsEngine::keep_best` uses `FixedFitness` as maximization

**File:** `src/engines/alps/engine.rs:357-369`

**Issue:** The `match` in `keep_best` has two arms: `Minimization` sorts ascending, and the wildcard `_` sorts descending. This means `FixedFitness` sorts the same as `Maximization` (descending by raw fitness), not by proximity to the target. For a `FixedFitness` problem the best survivors are the ones whose fitness is closest to the target, not the ones with the highest raw value. This produces incorrect survivor selection for `FixedFitness` problems.

**Fix:**
```rust
ProblemSolving::Minimization => { /* ascending */ }
ProblemSolving::Maximization => { /* descending */ }
ProblemSolving::FixedFitness => {
    let t = self.config.fitness_target.unwrap_or(0.0);
    pop.sort_unstable_by(|a, b| {
        (a.fitness() - t).abs().partial_cmp(&(b.fitness() - t).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
```

---

### WR-02: `ScatterEngine::local_search_improve` does not update fitness after reverting

**File:** `src/engines/scatter/engine.rs:236-257`

**Issue:** When the perturbation is accepted, `set_fitness` is called correctly (line 249). When it is rejected, the gene is reverted via `set_gene` (line 254) but `set_fitness` is never called to revert it. This leaves `ind.fitness()` holding the value from the last accepted step, which is correct only if fitness is updated immediately on acceptance. However, `current_fitness` is a local variable tracking the running fitness, while `ind.fitness()` holds the value from the last `set_fitness` call. After the loop, the chromosome's stored fitness is stale whenever the last local-search step was a rejection. The stale value propagates into the reference-set stats reported to `on_generation_end`.

**Fix:** After the loop, ensure the chromosome's stored fitness matches `current_fitness`:
```rust
// End of local_search_improve:
ind.set_fitness(current_fitness);
```

---

### WR-03: `CellularEngine` silently skips cells with no neighbors instead of treating the grid as a 1-element population

**File:** `src/engines/cellular/engine.rs:179-181`

**Issue:** `neighbors()` for `VonNeumann` and `Linear` on a 1×1 grid returns an empty vec after `dedup`/`retain` removes self. The inner loop then hits `continue` at line 181, skipping every cell. The engine completes all generations with zero evolution and returns the initial population unchanged. There is no warning, no error, and the grid size constraint is not validated before the run. While a 1×1 grid is pathological, a 1×N or N×1 grid with `VonNeumann` neighborhood and `cols=1` similarly collapses some neighbors to self, losing valid neighbors. The issue should be caught at construction or at run-start.

**Fix:** Add a validation in `run()` before the main loop:
```rust
assert!(
    rows * cols >= 2,
    "CellularEngine requires a grid with at least 2 cells"
);
```

---

### WR-04: `AlpsEngine` on_generation_end stats can be empty when all layers are empty

**File:** `src/engines/alps/engine.rs:293-300`

**Issue:** `fitness_values` is built by flattening all layer iterators (line 294-297). If all layers are empty (which can happen when all individuals age out before injection refills layer 0), `fitness_values` is empty. `GenerationStats::from_fitness_values` is called with an empty slice. Whether that function panics or produces a degenerate stats object depends on its implementation, which is out of scope for this file, but the call site does not guard against it.

**Fix:** Guard before the stats call:
```rust
if !fitness_values.is_empty() {
    let gen_stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
    stats_history.push(gen_stats);
    self.notify(|obs| obs.on_generation_end(stats_history.last().unwrap()));
}
```

---

## Info

### IN-01: Observer test for ALPS does not verify the `on_new_best` upper bound (at-most-once-per-generation)

**File:** `tests/engines/alps/test_alps.rs:327`

**Issue:** `test_alps_observer_fires_5_hooks` checks `new_best >= 1` but does not assert `new_best <= max_gens`. The equivalent Cellular test (`test_cellular_observer_fires_5_hooks`, line 292-294) explicitly checks both bounds. The ALPS engine does fire `on_new_best` at most once per generation (the check is correct at line 275 of the engine), but the test does not verify this contract, making future regressions harder to catch.

**Fix:** Add the upper-bound assertion:
```rust
assert!(
    data.new_best.load(Ordering::Relaxed) <= max_gens,
    "on_new_best must fire at most once per generation"
);
```

---

### IN-02: Benchmark `bench_de_vs_ga` uses an unseeded RNG, producing non-reproducible results

**File:** `benches/de.rs:19`

**Issue:** `make_pop` calls `rand::rng()` (the global thread-local RNG, line 19) without seeding. Each benchmark iteration starts with different initial populations, which inflates variance in the `de_vs_ga` comparison. The test helpers in all four test files use `rng::set_seed(Some(seed))` for reproducibility; the benchmark does not. This makes performance comparisons between runs unreliable.

**Fix:** Seed the benchmark RNG for reproducible populations, or explicitly document that population randomness is intentional and use `criterion`'s statistical analysis to absorb the variance.

---

_Reviewed: 2026-05-02T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
