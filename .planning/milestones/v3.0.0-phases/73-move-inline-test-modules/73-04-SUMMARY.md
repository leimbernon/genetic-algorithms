---
phase: 73-move-inline-test-modules
plan: 04
subsystem: testing
tags: [benchmarks, dtlz, zdt, single_objective, feature-gated, serde]

requires:
  - phase: 72-doctest-cleanup
    provides: clean doctest baseline
provides:
  - benchmark integration tests under tests/benchmarks/
  - cleaned benchmark source files with no inline test blocks
affects: [benchmarks, testing]

tech-stack:
  added: []
  patterns: [feature-gated-integration-test, serde-flat-test-pattern]

key-files:
  created:
    - tests/test_benchmarks.rs
    - tests/benchmarks/dtlz.rs
    - tests/benchmarks/zdt.rs
    - tests/benchmarks/single_objective.rs
  modified:
    - src/benchmarks/dtlz.rs
    - src/benchmarks/zdt.rs
    - src/benchmarks/single_objective.rs

key-decisions:
  - "Flattened serde nested mod into top-level #[cfg(all(feature = \"benchmarks\", feature = \"serde\"))] tests"
  - "Used bounds().len() and evaluate() for serde roundtrip assertions instead of private fields"

patterns-established:
  - "Feature-gated integration test: #[cfg(feature = \"benchmarks\")] on each test fn"
  - "Serde flat pattern: #[cfg(all(feature = \"benchmarks\", feature = \"serde\"))] replaces nested mod serde_tests"

requirements-completed: []

duration: 3min
completed: 2026-06-19
---

# Phase 73 Plan 04: Move inline benchmark test blocks Summary

**Feature-gated benchmark integration tests (30 non-serde + 10 serde) migrated from src/benchmarks/ to tests/benchmarks/ with flattened serde pattern**

## Performance

- **Duration:** 3 min
- **Started:** 2026-06-19T06:43:07Z
- **Completed:** 2026-06-19T06:46:18Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Created tests/test_benchmarks.rs harness and tests/benchmarks/ directory with dtlz.rs, zdt.rs, single_objective.rs
- All 40 tests pass under --features benchmarks,serde (30 non-serde + 10 serde)
- Zero #[cfg(test)] blocks remain in src/benchmarks/
- Source files compile clean with and without benchmarks feature

## Task Commits

Each task was committed atomically:

1. **Task 1: Create tests/test_benchmarks.rs harness + three feature-gated benchmark test files** - `0167679` (test)
2. **Task 2: Delete the inline #[cfg(test)] blocks from the three benchmark source files** - `c762147` (refactor)

## Files Created/Modified
- `tests/test_benchmarks.rs` - New top-level harness: `mod benchmarks { mod dtlz; mod single_objective; mod zdt; }`
- `tests/benchmarks/dtlz.rs` - 13 non-serde + 4 serde DTLZ tests (feature-gated)
- `tests/benchmarks/zdt.rs` - 10 non-serde + 3 serde ZDT tests (feature-gated)
- `tests/benchmarks/single_objective.rs` - 7 non-serde + 3 serde single-objective tests (feature-gated)
- `src/benchmarks/dtlz.rs` - Inline test block removed (577-line deletion)
- `src/benchmarks/zdt.rs` - Inline test block removed
- `src/benchmarks/single_objective.rs` - Inline test block removed

## Decisions Made
- Flattened serde nested `mod serde_tests` into top-level `#[cfg(all(feature = "benchmarks", feature = "serde"))]` test functions per D-10 / Pitfall 6 pattern
- Used `bounds().len()` and `evaluate()` roundtrip assertions instead of private struct fields (`n_vars`, `n_obj`, `alpha`) for serde tests

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed private field access in migrated serde tests**
- **Found during:** Task 1 (Create test files)
- **Issue:** Original inline tests accessed private struct fields (`n_vars`, `n_obj`, `alpha`, `s.bounds.len()`) via `use super::*;` — not accessible from integration tests
- **Fix:** Replaced with public API assertions: `bounds().len()` for dimension, `evaluate()` roundtrip for structural equality, `n` (pub) for Sphere
- **Files modified:** tests/benchmarks/dtlz.rs, tests/benchmarks/zdt.rs, tests/benchmarks/single_objective.rs
- **Verification:** All 40 tests pass with --features benchmarks,serde
- **Committed in:** 0167679 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minimal — adaptation required for integration test visibility boundaries. No scope creep.

## Issues Encountered
None beyond the private-field adaptation above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 73 Plan 04 complete — all 10 inline test blocks from issue #266 have been migrated across plans 01-04
- Ready for phase completion verification

---
*Phase: 73-move-inline-test-modules*
*Completed: 2026-06-19*
