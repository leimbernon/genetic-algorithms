---
phase: 58-eda-umda-engine
fixed_at: 2026-06-04T00:00:00Z
review_path: .planning/phases/58-eda-umda-engine/58-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 6
skipped: 1
status: partial
---

# Phase 58: Code Review Fix Report

**Fixed at:** 2026-06-04T00:00:00Z
**Source review:** .planning/phases/58-eda-umda-engine/58-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (CR-01, CR-02, WR-01, WR-02, WR-03, WR-04, WR-05)
- Fixed: 6 (CR-01, CR-02, WR-01, WR-02, WR-04, WR-05)
- Skipped: 1 (WR-03)

## Fixed Issues

### CR-01 + CR-02: FixedFitness sort comparator bug in truncation selection

**Files modified:** `src/engines/eda/engine.rs`
**Commit:** see fix(58) engine.rs commit
**Applied fix:** Replaced the binary `is_maximization` flag + if/else sort branches in both
`EdaEngine::run()` and `EdaRealEngine::run()` with a three-way `cmp` closure that mirrors
`is_better` for all three `ProblemSolving` variants:
- `Maximization`: `b_fit.partial_cmp(&a_fit)` (descending — top-k highest)
- `Minimization`: `a_fit.partial_cmp(&b_fit)` (ascending — top-k lowest)
- `FixedFitness`: ascending `|fitness - target|` distance (top-k closest to target)

The single `indices.select_nth_unstable_by` call now uses this closure, replacing the
duplicated if/else block.

### WR-01: Gaussian variance unbiased estimator

**Files modified:** `src/engines/eda/engine.rs`
**Commit:** included in engine.rs commit
**Applied fix:** Changed `estimate_gaussian` from biased population variance (`/ n`) to
unbiased sample variance (`/ (n - 1.0)`) with a guard for `n <= 1.0` (returns `0.0`
variance in that case, resulting in a `1e-6` std floor).

### WR-02: find_best empty population guard

**Files modified:** `src/engines/eda/engine.rs`
**Commit:** included in engine.rs commit
**Applied fix:** Added `assert!(!pop.is_empty(), "EdaEngine::find_best called with empty population")`
to `EdaEngine::find_best` and `assert!(!pop.is_empty(), "EdaRealEngine::find_best called with empty population")`
to `EdaRealEngine::find_best`. Both guards fire before the unconditional `pop[0]` access.

### WR-04: best_model tracks the best-generation model, not last-generation

**Files modified:** `src/engines/eda/engine.rs`
**Commit:** included in engine.rs commit
**Applied fix:** Added `best_model` variable (initialized to the pre-loop model) in both
`run()` methods. Updated `best_model = learned_model.clone()` inside the `is_better` branch
alongside `best_fitness` / `best` updates. `EdaResult` now returns `learned_model: best_model`
instead of `learned_model: learned_model`, so callers receive the model from the generation
that produced the best individual.

### WR-05: lib.rs engine count updated from 12 to 13

**Files modified:** `src/lib.rs`
**Commit:** see fix(58) lib.rs commit
**Applied fix:** Updated the `## Engines (12 total)` heading to `## Engines (13 total)` and
the introductory sentence `This crate offers 12 optimization engines` to `13 optimization engines`.
Added `EdaEngine<U>` / `EdaRealEngine<U>` row to the engines summary table and two rows to
the "When to Use Which Engine" table (one for binary/Bernoulli, one for continuous/Gaussian).

## Skipped Issues

### WR-03: Code duplication between EdaEngine and EdaRealEngine

**File:** `src/engines/eda/engine.rs:107-682`
**Reason:** Skipped per user instruction — "If WR-03 fix risks breaking compilation, skip it
and document in the commit message." The refactor requires extracting shared `is_better`,
`reached_target`, `find_best`, and `notify` implementations into a shared helper. This
is a non-trivial structural change (trait object vs free function, lifetime constraints,
`RealGene` bound isolation) that carries meaningful refactor risk. The critical and warning
fixes (CR-01, CR-02, WR-01, WR-02, WR-04) were applied to both structs individually.
**Original issue:** Verbatim duplication of ~300 lines of logic between `EdaEngine<U>` and
`EdaRealEngine<U>`. Any future bug fix must be applied in two places.

---

_Fixed: 2026-06-04T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
