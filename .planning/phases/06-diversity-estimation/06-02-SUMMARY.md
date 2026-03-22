---
phase: 06-diversity-estimation
plan: 02
subsystem: ga-execution-loop
tags: [diversity, extension, dynamic-mutation, loop-reorder, integration-tests]

# Dependency graph
requires:
  - "06-01: GenerationStats.diversity field"
provides:
  - "GA generation loop collects stats before dynamic mutation and extension trigger"
  - "Extension trigger reads gen_stats.diversity (not inline std-dev)"
  - "Dynamic mutation reads gen_stats.diversity (not compute_cardinality)"
  - "Integration tests: diversity populated in stats, extension uses diversity"
affects:
  - ga-execution-loop
  - extension-trigger
  - dynamic-mutation

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Stats collected after niching and best-chromosome, before all diversity-consuming subsystems"
    - "Single diversity signal computed once per generation, read by multiple subsystems"

key-files:
  created: []
  modified:
    - src/ga.rs
    - tests/test_ga.rs
    - tests/extension/test_extension.rs

key-decisions:
  - "Niching and best-chromosome moved before stats collection so diversity reflects final population state"
  - "Extension trigger simplified: removed n > 1.0 guard (std-dev already handles edge cases in GenerationStats)"
  - "Dynamic mutation comment updated from 'cardinality' to 'diversity' to reflect new signal"

patterns-established:
  - "Reorder generation loop: niching -> best_chromosome -> stats -> dynamic_mutation -> extension -> checkpoint -> callback -> stopping"

requirements-completed: [DIV-02, DIV-03]

# Metrics
duration: 5min
completed: 2026-03-20
---

# Phase 6 Plan 02: Diversity Estimation — Loop Reorder and Wiring Summary

**Reordered GA generation loop to collect stats once before subsystems, then wired extension trigger and dynamic mutation to read gen_stats.diversity instead of computing independent signals**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-20T19:26:31Z
- **Completed:** 2026-03-20T19:31:02Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Moved niching and best-chromosome blocks before stats collection in `src/ga.rs`
- Stats are now computed once per generation at a stable point (after population is finalized)
- Dynamic mutation block updated: removed `compute_cardinality` call, reads `gen_stats.diversity` instead
- Extension trigger block updated: removed inline `fitness_vals`/`std_dev` computation, reads `gen_stats.diversity` with simplified condition (no `n > 1.0` guard needed)
- Both log messages updated: "cardinality" -> "diversity" in dynamic mutation; "fitness_std_dev" -> "diversity" in extension
- Added `test_ga_stats_diversity_populated` to `tests/test_ga.rs`: verifies diversity >= 0.0 and diversity == fitness_std_dev for every generation, and at least one generation has diversity > 0.0
- Added `ga_extension_triggers_on_diversity` to `tests/extension/test_extension.rs`: uses uniform-fitness population (diversity = 0.0) with threshold = 1.0 to guarantee extension fires, verifies GA completes and stats are populated

## Task Commits

1. **Task 1: Reorder GA loop — move stats before subsystems** - `b188d26` (feat)
2. **Task 2: Add integration tests for diversity in GA stats and extension trigger** - `d738366` (test)

## Files Created/Modified

- `src/ga.rs` - Reordered generation loop; dynamic mutation and extension trigger read gen_stats.diversity
- `tests/test_ga.rs` - Added `test_ga_stats_diversity_populated` integration test
- `tests/extension/test_extension.rs` - Added `ga_extension_triggers_on_diversity` integration test

## Decisions Made

- Niching and best-chromosome must happen BEFORE stats so diversity reflects the final post-niching population state for the generation
- Extension trigger's `if n > 1.0` guard removed — `GenerationStats::from_fitness_values` already handles the empty/single-element case by returning 0.0 std-dev, and 0.0 < threshold is a valid trigger
- Dynamic mutation's `compute_cardinality` (unique gene ratio) replaced by `gen_stats.diversity` (fitness std-dev) — this unifies the diversity signal per the plan's intent

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- One diversity signal per generation, read by all subsystems
- Requirements DIV-02 and DIV-03 fulfilled
- Foundation ready for future plans to replace fitness std-dev with a richer diversity metric

---
*Phase: 06-diversity-estimation*
*Completed: 2026-03-20*

## Self-Check: PASSED

- `src/ga.rs` modified: FOUND
- `tests/test_ga.rs` modified: FOUND
- `tests/extension/test_extension.rs` modified: FOUND
- Commit `b188d26`: FOUND
- Commit `d738366`: FOUND
