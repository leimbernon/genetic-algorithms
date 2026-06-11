---
phase: 11-advanced-mode-examples
plan: 02
subsystem: examples
tags: [island-model, rastrigin, continuous-optimization, multi-population, ring-topology, heterogeneous-mutation]

# Dependency graph
requires:
  - phase: 10-single-population-examples
    provides: RangeChromosome/RangeGenotype pattern and Rastrigin fitness function style
provides:
  - Island model multi-population example using IslandGa with heterogeneous configs
affects: [11-advanced-mode-examples, documentation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "IslandGa with heterogeneous configs via with_heterogeneous_configs() + direct field assignment"
    - "GaConfiguration built via Default::default() with direct field mutation for IslandGa"

key-files:
  created:
    - examples/island_model.rs
  modified: []

key-decisions:
  - "IslandGa::run() is used directly — evolve_islands_one_generation() and global_best() are private, so no per-migration progress reporting"
  - "alleles_can_be_repeated=true passed to range_random_initialization for continuous Rastrigin (dimensions > allele entries)"

patterns-established:
  - "IslandGa builder: with_heterogeneous_configs -> with_alleles -> with_initialization_fn -> with_fitness_fn -> build -> run"
  - "Per-island mutation diversity: [0.01, 0.05, 0.10, 0.20] covers exploitation to broad exploration"

requirements-completed: [EX-03]

# Metrics
duration: 1min
completed: 2026-03-22
---

# Phase 11 Plan 02: Island Model Rastrigin 20D Example Summary

**IslandGa example with 4 heterogeneous islands (mutation probs 0.01–0.20) on Rastrigin 20D via Ring topology migration**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-22T10:33:48Z
- **Completed:** 2026-03-22T10:34:52Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Created `examples/island_model.rs` (160 lines) demonstrating IslandGa with 4 islands
- Heterogeneous mutation rates (0.01, 0.05, 0.10, 0.20) balance exploitation and exploration across islands
- Ring topology migration every 10 generations with 2 migrants per event
- API limitation (private evolve_islands_one_generation/global_best) documented in doc block and code comment

## Task Commits

1. **Task 1: Create Island Model Rastrigin 20D example** - `b54153a` (feat)

**Plan metadata:** _(pending docs commit)_

## Files Created/Modified

- `examples/island_model.rs` - Island model GA example: 4-island Rastrigin 20D with heterogeneous mutation rates, Ring topology

## Decisions Made

- Used `IslandGa::run()` directly since `evolve_islands_one_generation()` and `global_best()` are private — per-migration progress is not available via the public API. Documented limitation in `/*!` doc block.
- Passed `Some(true)` for `alleles_can_be_repeated` in `range_random_initialization` since 20-dimensional chromosome uses a single allele template (repeated across all dimensions).

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Island model example complete and verified with `cargo clippy` and `cargo run`
- Ready to proceed to Plan 03 (NSGA-II or remaining advanced examples)

---
*Phase: 11-advanced-mode-examples*
*Completed: 2026-03-22*
