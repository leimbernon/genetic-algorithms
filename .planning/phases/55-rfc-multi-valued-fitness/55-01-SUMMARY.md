---
phase: 55-rfc-multi-valued-fitness
plan: 01
subsystem: traits
tags: [rust, traits, rename, vector-fitness, multi-case-fitness]

# Dependency graph
requires: []
provides:
  - VectorFitness trait at src/traits/vector_fitness.rs
  - genetic_algorithms::VectorFitness crate-root re-export
  - fitness_values() and set_fitness_values() methods replacing case_fitness/set_case_fitness
affects:
  - 55-02 (downstream operators referencing MultiCaseFitness)
  - 55-03 (chromosome implementors)
  - 55-04 (multi-objective engines)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Hard rename with git mv preserves git history for trait file renames"
    - "No alias bridge: old symbol completely removed, downstream errors are intentional Wave 1 outcome"

key-files:
  created:
    - src/traits/vector_fitness.rs
    - tests/traits/test_vector_fitness.rs
  modified:
    - src/traits.rs
    - src/lib.rs
    - tests/test_traits.rs

key-decisions:
  - "D-09: Hard rename, no MultiCaseFitness alias bridge — forces all downstream sites to update in Waves 2-4"
  - "D-02: No default impl on VectorFitness — lifetime mismatch prevents deriving from scalar fitness(); each implementor must store Vec<f64>"
  - "D-05: VectorFitness re-exported at crate root via pub use traits::VectorFitness"

patterns-established:
  - "VectorFitness: supertrait of ChromosomeT carrying Vec<f64> for both lexicase and multi-objective engines"

requirements-completed: [TRAITS-01]

# Metrics
duration: 8min
completed: 2026-05-30
---

# Phase 55 Plan 01: VectorFitness Trait Rename Summary

**Hard rename of MultiCaseFitness → VectorFitness with git mv; methods case_fitness/set_case_fitness → fitness_values/set_fitness_values; crate-root re-export updated; downstream compile errors are expected Wave 1 outcome**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-30T08:26:00Z
- **Completed:** 2026-05-30T08:34:29Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Renamed `src/traits/multi_case_fitness.rs` → `src/traits/vector_fitness.rs` via `git mv` (preserves history)
- Renamed trait `MultiCaseFitness` → `VectorFitness`; methods `case_fitness` → `fitness_values`, `set_case_fitness` → `set_fitness_values`; param `scores` → `values`
- Extended rustdoc: covers lexicase + all MO engines (NSGA-II/III, MOEA/D, SPEA2, SMS-EMOA, IBEA); added `# No default impl` section explaining lifetime constraint
- Updated `src/traits.rs` module declaration and re-export; updated `src/lib.rs` crate-root re-export
- Created `tests/traits/test_vector_fitness.rs` with `VfTestChromosome` fixture, roundtrip test, and re-export accessibility test; wired into `tests/test_traits.rs`

## Task Commits

1. **Task 1: Rename trait file + module declaration + lib.rs re-export** - `17376d6` (feat)
2. **Task 2: Trait roundtrip + re-export baseline test** - `6dcfbf8` (test)

## Files Created/Modified

- `src/traits/vector_fitness.rs` — New file (renamed from multi_case_fitness.rs); contains `pub trait VectorFitness: ChromosomeT` with `fitness_values()` and `set_fitness_values()`
- `src/traits/multi_case_fitness.rs` — Deleted (git mv)
- `src/traits.rs` — `pub mod vector_fitness;` + `pub use vector_fitness::VectorFitness;` (replaced multi_case_fitness refs)
- `src/lib.rs` — `VectorFitness` in `pub use traits::{...}` (replaced `MultiCaseFitness`)
- `tests/traits/test_vector_fitness.rs` — New; two tests: roundtrip and re-export
- `tests/test_traits.rs` — Added `mod test_vector_fitness;` to traits mod block

## Decisions Made

- **D-09 (hard rename, no alias):** `MultiCaseFitness` symbol completely removed from `src/` with no backward-compat alias. Downstream compile errors are the intended outcome, forcing all call sites to update in Waves 2-4.
- **D-02 (no default impl):** `VectorFitness` provides no default body for either method. The trait rustdoc's `# No default impl` section explains the lifetime mismatch reason.
- **D-05 (crate-root re-export):** `genetic_algorithms::VectorFitness` resolves correctly; verified by `test_vector_fitness_reexport`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. `cargo check` confirms the expected downstream errors (unresolved import `MultiCaseFitness` in operators and engine modules) which will be fixed in Waves 2-4.

## Known Stubs

None. This plan only renames a trait definition and adds baseline tests. No data wiring or rendering involved.

## Next Phase Readiness

- Wave 1 foundation complete. `src/traits/vector_fitness.rs` is the authoritative trait definition for all subsequent waves.
- Downstream errors at call sites in `src/operations/selection/`, `src/engines/ga.rs`, and MO engine modules will be resolved in Plans 55-02 through 55-04.
- Tests in `tests/traits/test_vector_fitness.rs` will go GREEN once Wave 2-3 chromosome implementors are updated (Plan 55-03).

---
*Phase: 55-rfc-multi-valued-fitness*
*Completed: 2026-05-30*
