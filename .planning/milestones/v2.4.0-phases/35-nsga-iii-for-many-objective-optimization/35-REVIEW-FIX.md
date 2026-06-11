---
phase: 35-nsga-iii-for-many-objective-optimization
fixed_at: 2026-05-08T00:00:00Z
review_path: .planning/phases/35-nsga-iii-for-many-objective-optimization/35-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 35: Code Review Fix Report

**Fixed at:** 2026-05-08T00:00:00Z
**Source review:** .planning/phases/35-nsga-iii-for-many-objective-optimization/35-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: Empty custom reference-point list passes `validate()` then panics at runtime

**Files modified:** `src/engines/nsga3/mod.rs`
**Commit:** (fix(35): CR-01)
**Applied fix:** Added `points.is_empty()` guard inside the `Some(points)` arm of the reference-point match in `validate()`. Returns `Err(InvalidNsga3Configuration("reference points list must not be empty"))` before the per-point dimension loop executes.

---

### CR-02: `multi_objective` shared module depends on `nsga2::configuration` (inverted dependency)

**Files modified:** `src/engines/multi_objective/mod.rs`, `src/engines/multi_objective/non_dominated_sort.rs`, `src/engines/multi_objective/pareto.rs`, `src/engines/nsga2/configuration.rs`
**Commit:** (fix(35): CR-02)
**Applied fix:** Moved the canonical `ObjectiveDirection` definition into `src/engines/multi_objective/mod.rs`. Replaced the definition in `nsga2/configuration.rs` with `pub use crate::multi_objective::ObjectiveDirection` (backward-compat re-export). Updated `non_dominated_sort.rs` and `pareto.rs` to import via `super::ObjectiveDirection`. All existing public paths continue to resolve. `nsga3::configuration` already re-exports from `nsga2::configuration` so that path is unaffected.

---

### WR-01: `validate()` does not reject `with_reference_points_auto(0)`

**Files modified:** `src/engines/nsga3/mod.rs`, `src/engines/nsga3/configuration.rs`
**Commit:** (fix(35): WR-01)
**Applied fix:** Added a `pub(crate) reference_points_auto_p()` accessor on `Nsga3Configuration` (the field is private). Added a `p == 0` guard in `validate()` using this accessor, returning `Err(InvalidNsga3Configuration("Das-Dennis subdivision count p must be >= 1"))`.

---

### WR-02: `effective_reference_points()` called redundantly in `run()`

**Files modified:** `src/engines/nsga3/mod.rs`
**Commit:** (fix(35): WR-02)
**Applied fix:** Added a private `validate_and_get_ref_points()` method that runs all validation checks and materialises the reference points in a single `effective_reference_points()` call, returning `Ok(Vec<Vec<f64>>)`. `run()` now calls `validate_and_get_ref_points()?` instead of the previous `validate()?` + separate `effective_reference_points()` pair. Eliminates the double Das-Dennis recursion on every `run()` invocation.

---

### WR-03: `test_nsga3_validate_mismatched_objective_directions` is missing

**Files modified:** `tests/engines/nsga3/test_nsga3.rs`
**Commit:** (fix(35): WR-03)
**Applied fix:** Added `test_nsga3_validate_mismatched_objective_directions` test. Passes a 1-element `objective_directions` vec against `num_objectives=3` and asserts the error message contains `"objective_directions"`. Also added `ObjectiveDirection` to the test file's use imports. Test passes.

---

### WR-04: Stale doc-comment references to `CONTEXT.md` and `RESEARCH.md`

**Files modified:** `src/engines/nsga3/configuration.rs`, `src/engines/nsga3/mod.rs`
**Commit:** (fix(35): WR-04)
**Applied fix:** Removed `"see CONTEXT.md D-07"` from both `with_reference_points_auto` and `with_reference_points` doc comments; replaced with self-contained prose. Replaced `"RESEARCH.md Pitfall 1"` in `normalize_st` with an inline explanation: "to prevent division by zero when all individuals collapse onto the ideal point".

---

_Fixed: 2026-05-08T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
