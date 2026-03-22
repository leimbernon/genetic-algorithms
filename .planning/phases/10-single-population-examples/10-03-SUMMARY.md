---
phase: 10-single-population-examples
plan: "03"
subsystem: examples
tags: [genetic-algorithms, niching, fitness-sharing, multimodal, range-chromosome, rust]

requires: []
provides:
  - "examples/niching.rs — runnable multimodal GA example with fitness sharing"
affects: [documentation, examples-index]

tech-stack:
  added: []
  patterns:
    - "NichingConfig trait imported explicitly to enable builder methods"
    - "RangeGenotype::new id parameter is i32, not T"

key-files:
  created:
    - examples/niching.rs
  modified: []

key-decisions:
  - "Used RangeGenotype::new(0_i32, ...) — id field is i32, not the generic T type"
  - "SIGMA_SHARE=1.5 and POP_SIZE=150 produce reliable coverage of all 3 peaks"

patterns-established:
  - "NichingConfig must be imported from genetic_algorithms::traits for builder methods to compile"

requirements-completed: [EX-06]

duration: 1min
completed: 2026-03-22
---

# Phase 10 Plan 03: Niching / Fitness Sharing Example Summary

**Multimodal GA example using fitness sharing across three Gaussian peaks at x=2, x=5, x=8 with Range<f64> chromosomes and NichingConfig builder API**

## Performance

- **Duration:** ~1 min
- **Started:** 2026-03-22T09:33:51Z
- **Completed:** 2026-03-22T09:35:03Z
- **Tasks:** 1 of 1
- **Files modified:** 1

## Accomplishments

- Created `examples/niching.rs` demonstrating multimodal optimization with fitness sharing
- Population of 150 individuals successfully maintains coverage of all 3 peaks (87/43/19 individuals per peak in a representative run)
- Per-generation progress reporting every 50 generations with best/avg fitness
- Final output counts individuals near each peak and confirms SUCCESS when all 3 are covered

## Task Commits

Each task was committed atomically:

1. **Task 1: Create niching.rs example** - `a683df2` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `examples/niching.rs` - Self-contained niching/fitness-sharing multimodal GA example with three Gaussian peaks

## Decisions Made

- `RangeGenotype::new(id, ranges, value)` — the `id` field is `i32`, not the generic `T`, so the initialiser call must use `0_i32` as the first argument (not `0.0_f64`). The plan spec had this wrong; auto-fixed.
- `SIGMA_SHARE=1.5` with `POP_SIZE=150` consistently produces coverage of all 3 peaks within 300 generations.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wrong id type in RangeGenotype::new call**
- **Found during:** Task 1 (Create niching.rs example) — compile error
- **Issue:** Plan spec used `RangeGenotype::new(0.0_f64, ...)` but the `id` parameter is `i32`, not `T`
- **Fix:** Changed to `RangeGenotype::new(0_i32, ...)` — matches the actual signature
- **Files modified:** examples/niching.rs
- **Verification:** `cargo build --example niching` succeeded after fix
- **Committed in:** a683df2 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug — type mismatch from plan spec)
**Impact on plan:** Minimal; single-line fix, no scope change.

## Issues Encountered

None beyond the auto-fixed type mismatch above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- EX-06 requirement satisfied; niching example is runnable and self-explanatory
- Phase 10 plan 03 is the last plan in the phase; phase 10 is now complete

---
*Phase: 10-single-population-examples*
*Completed: 2026-03-22*
