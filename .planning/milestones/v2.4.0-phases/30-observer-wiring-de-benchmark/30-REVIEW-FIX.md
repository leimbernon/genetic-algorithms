---
phase: 30-observer-wiring-de-benchmark
fixed_at: 2026-05-02T00:00:00Z
review_path: .planning/phases/30-observer-wiring-de-benchmark/30-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 30: Code Review Fix Report

**Fixed at:** 2026-05-02T00:00:00Z
**Source review:** .planning/phases/30-observer-wiring-de-benchmark/30-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: Panic on empty population in `DeEngine::find_best` and `ScatterEngine::find_best`

**Files modified:** `src/engines/de/engine.rs`, `src/engines/scatter/engine.rs`
**Commit:** 9b7e6df
**Applied fix:** Added `assert!(!pop.is_empty(), "find_best called on empty population")` as the first statement in both `find_best` implementations.

---

### CR-02: Wrong `TerminationCause` when early-stopping fires on the final generation

**Files modified:** `src/engines/de/engine.rs`, `src/engines/scatter/engine.rs`, `src/engines/alps/engine.rs`
**Commit:** dfae691
**Applied fix:** Introduced a `let mut target_reached = false` boolean before the main loop in all three engines. Set `target_reached = true` before `break` in the early-stopping block, then replaced the `generations < max_generations` condition with `target_reached` to determine `TerminationCause`.

---

### CR-03: `AlpsEngine` fires spurious `on_new_best` on generation 0 due to NaN propagation

**Files modified:** `src/engines/alps/engine.rs`
**Commit:** f99d1c7
**Applied fix:** Replaced the `f64::NAN`-seeded fold with a direction-aware sentinel: `f64::MAX` for Minimization/FixedFitness and `f64::MIN` for Maximization. The initial `best_fitness` is then computed with an explicit loop using `is_better`, ensuring `prev_best_fitness` on generation 0 is never NaN.

---

### WR-01: `AlpsEngine::keep_best` uses `FixedFitness` as maximization

**Files modified:** `src/engines/alps/engine.rs`
**Commit:** d3185f8
**Applied fix:** Replaced the wildcard `_` arm in the `match self.config.problem_solving` block with explicit `Maximization` and `FixedFitness` arms. The `FixedFitness` arm sorts by proximity to the target using `(fitness - t).abs()`.

---

### WR-02: `ScatterEngine::local_search_improve` does not update fitness after reverting

**Files modified:** `src/engines/scatter/engine.rs`
**Commit:** 1c18771
**Applied fix:** Added `ind.set_fitness(current_fitness)` immediately after the local-search loop to ensure the chromosome's stored fitness always matches the final accepted value, regardless of whether the last step was accepted or reverted.

---

### WR-03: `CellularEngine` silently skips cells with no neighbors instead of treating the grid as a 1-element population

**Files modified:** `src/engines/cellular/engine.rs`
**Commit:** 9cf5779
**Applied fix:** Added `assert!(rows * cols >= 2, "CellularEngine requires a grid with at least 2 cells")` at the start of `run()`, before population initialisation.

---

### WR-04: `AlpsEngine` on_generation_end stats can be empty when all layers are empty

**Files modified:** `src/engines/alps/engine.rs`
**Commit:** 7dd60e7
**Applied fix:** Wrapped the `GenerationStats::from_fitness_values` call and observer notification inside `if !fitness_values.is_empty()` to prevent a potentially panicking call with an empty slice.

---

_Fixed: 2026-05-02T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
