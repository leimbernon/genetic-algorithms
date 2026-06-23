---
phase: 55-rfc-multi-valued-fitness
plan: "03"
subsystem: selection
tags:
  - rust
  - lexicase
  - rename
  - mechanical
dependency_graph:
  requires:
    - 55-01  # VectorFitness trait defined
    - 55-02  # VectorFitness implemented on all chromosome types
  provides:
    - lexicase operator using VectorFitness + fitness_values()
    - factory_lexicase with VectorFitness bound
    - Ga::select_parents_lexicase with VectorFitness where clause
  affects:
    - src/operations/selection/lexicase.rs
    - src/operations/selection.rs
    - src/engines/ga.rs
tech_stack:
  added: []
  patterns:
    - mechanical rename — enum/factory pattern unchanged
key_files:
  modified:
    - src/operations/selection/lexicase.rs
    - src/operations/selection.rs
    - src/engines/ga.rs
decisions:
  - Kept all algorithm logic identical — pure rename, zero behavioral change
  - Updated error message strings and doc comments in addition to trait bounds and method calls
metrics:
  duration: "~8 minutes"
  completed: "2026-05-30T08:46:35Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
---

# Phase 55 Plan 03: Lexicase MultiCaseFitness to VectorFitness Rename Summary

**One-liner:** Mechanical rename of `MultiCaseFitness` → `VectorFitness` and `case_fitness()` → `fitness_values()` across all three lexicase code paths.

## What Was Done

Eliminated the `MultiCaseFitness` symbol from all lexicase code paths in `src/operations/` and `src/engines/ga.rs`. Three files updated with zero algorithmic change.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Update lexicase.rs — trait import, bounds, method calls | f140475 | src/operations/selection/lexicase.rs |
| 2 | Update selection.rs factory_lexicase + ga.rs select_parents_lexicase | d2bcc34 | src/operations/selection.rs, src/engines/ga.rs |

## Changes by File

**src/operations/selection/lexicase.rs:**
- `use crate::traits::{ChromosomeT, MultiCaseFitness}` → `use crate::traits::{ChromosomeT, VectorFitness}`
- `compute_mad_epsilons<U: MultiCaseFitness>` → `<U: VectorFitness>`
- `select_one_winner<U: MultiCaseFitness>` → `<U: VectorFitness>`
- `where U: ChromosomeT + MultiCaseFitness` → `+ VectorFitness` (both pub functions)
- All `.case_fitness()[...]` → `.fitness_values()[...]` (7 call sites)
- Doc comment updated

**src/operations/selection.rs:**
- Import updated to `VectorFitness`
- `factory_lexicase<U: ChromosomeT + MultiCaseFitness>` → `+ VectorFitness`
- Error message: `"case_fitness() is empty — call set_case_fitness..."` → `"fitness_values() is empty — call set_fitness_values..."`
- Error message: `"NaN in case_fitness at chromosome {}"` → `"NaN in fitness_values at chromosome {}"`
- Panic message: `"do not support MultiCaseFitness"` → `"do not support VectorFitness"`
- Config error message: `"does not support MultiCaseFitness bound"` → `"VectorFitness bound"`
- `.case_fitness().to_vec()` → `.fitness_values().to_vec()` in D-04 sync loop
- Doc comments updated

**src/engines/ga.rs:**
- Import: `MultiCaseFitness` replaced with `VectorFitness` in traits import list
- `impl<U> Ga<U> where U: LinearChromosome + MultiCaseFitness` → `+ VectorFitness`
- Two doc comments in `select_parents_lexicase` updated

## Verification Results

- `grep -rn "MultiCaseFitness|case_fitness|set_case_fitness" src/operations/ src/engines/ga.rs | grep -v '^\s*//'` → 0 matches
- `cargo check` — no errors from the three modified files
- Remaining `cargo check` errors are confined to MO engine files (handled in plans 04 + 05)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — this plan performs a mechanical rename with no new code paths, network endpoints, or auth changes.

## Self-Check: PASSED

- src/operations/selection/lexicase.rs — modified and committed at f140475
- src/operations/selection.rs — modified and committed at d2bcc34
- src/engines/ga.rs — modified and committed at d2bcc34
- Zero `MultiCaseFitness` / `case_fitness` in live code across all three files
