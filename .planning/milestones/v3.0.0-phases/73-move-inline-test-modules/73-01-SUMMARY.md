---
phase: 73-move-inline-test-modules
plan: 01
subsystem: testing
tags: [indicators, test-harness, cfg-test, multi-objective]

# Dependency graph
requires:
  - phase: 72-audit-ignored-doctests
    provides: "Baseline test counts and ignored-doctest inventory"
provides:
  - "30 indicator integration tests now wired and running under cargo test"
  - "Zero #[cfg(test)] blocks in src/engines/multi_objective/indicators/"
affects: [73-move-inline-test-modules]

# Tech tracking
tech-stack:
  added: []
  patterns: [harness-wiring, mirrored-subdirectory-mod-declaration]

key-files:
  created: []
  modified:
    - tests/test_engines.rs
    - tests/engines/multi_objective/indicators/test_generational_distance.rs
    - tests/engines/multi_objective/indicators/test_hypervolume.rs
    - tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs
    - tests/engines/multi_objective/indicators/test_spread.rs
    - src/engines/multi_objective/indicators/generational_distance.rs
    - src/engines/multi_objective/indicators/hypervolume.rs
    - src/engines/multi_objective/indicators/inverted_generational_distance.rs
    - src/engines/multi_objective/indicators/spread.rs

key-decisions:
  - "Used (1.1, 1.1) reference point for HV ZDT1 test to ensure strict domination of boundary point (0, 1.0)"
  - "Used ZDT1 n=5 vs n=10 for GD subset test to avoid exact-subset trivial zero GD"

patterns-established:
  - "mod multi_objective { mod indicators { ... } } pattern in test_engines.rs for indicator wiring"

requirements-completed: []

# Metrics
duration: 3min
completed: 2026-06-19
---

# Phase 73 Plan 01: Wire Indicator Tests + Delete Inline Blocks Summary

**Wired 30 multi-objective indicator integration tests into test_engines.rs harness and removed 23 redundant inline #[cfg(test)] blocks from 4 indicator source files**

## Performance

- **Duration:** 3 min
- **Started:** 2026-06-19T06:43:09Z
- **Completed:** 2026-06-19T06:46:22Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Wired 4 indicator test files (GD, HV, IGD, Spread) into test_engines.rs harness — 30 tests now run
- Deleted 23 inline #[cfg(test)] blocks from 4 indicator source files (generational_distance, hypervolume, inverted_generational_distance, spread)
- Zero #[cfg(test)] blocks remain in src/engines/multi_objective/indicators/
- Fixed pre-existing broken GaError import paths in all 4 indicator test files
- Fixed 2 pre-existing test data bugs (HV reference point domination, GD exact-subset triviality)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire indicators into test_engines.rs harness** - `b6fb53e` (feat)
2. **Task 2: Delete inline #[cfg(test)] blocks from 4 indicator source files** - `631977c` (refactor)

## Files Created/Modified
- `tests/test_engines.rs` — Added `mod multi_objective { mod indicators { ... } }` block inside `mod engines`
- `tests/engines/multi_objective/indicators/test_generational_distance.rs` — Fixed GaError import path; fixed GD subset test data
- `tests/engines/multi_objective/indicators/test_hypervolume.rs` — Fixed GaError import path; fixed HV ZDT1 reference point
- `tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs` — Fixed GaError import path
- `tests/engines/multi_objective/indicators/test_spread.rs` — Fixed GaError import path
- `src/engines/multi_objective/indicators/generational_distance.rs` — Deleted 6-test inline block
- `src/engines/multi_objective/indicators/hypervolume.rs` — Deleted 6-test inline block
- `src/engines/multi_objective/indicators/inverted_generational_distance.rs` — Deleted 6-test inline block
- `src/engines/multi_objective/indicators/spread.rs` — Deleted 5-test inline block

## Decisions Made
- Used (1.1, 1.1) reference point for HV ZDT1 test — the ZDT1 front starts at (0, 1.0) which equals the reference point (1.0, 1.0) in f2, violating strict domination required by hypervolume()
- Used ZDT1 n=5 vs n=10 for GD subset test — n=10 vs n=1000 produced exact subsets (999/9=111 integer), making GD trivially zero

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed broken GaError import paths in 4 indicator test files**
- **Found during:** Task 1 (wiring tests into harness)
- **Issue:** Test files used `use genetic_algorithms::GaError;` but GaError is not re-exported at crate root — correct path is `genetic_algorithms::error::GaError`
- **Fix:** Changed import to `use genetic_algorithms::error::GaError;` in all 4 files
- **Files modified:** test_generational_distance.rs, test_hypervolume.rs, test_inverted_generational_distance.rs, test_spread.rs
- **Verification:** `cargo test --test test_engines` compiles and passes
- **Committed in:** b6fb53e (Task 1)

**2. [Rule 1 - Bug] Fixed test_hypervolume_zdt1 reference point domination**
- **Found during:** Task 1 (test execution)
- **Issue:** ZDT1 front starts at (0, 1.0). With reference point (1.0, 1.0), f2 equals ref f2 (1.0 == 1.0), so point is not strictly dominated — hypervolume() returns error
- **Fix:** Changed reference point to (1.1, 1.1) which strictly dominates all ZDT1 points
- **Files modified:** test_hypervolume.rs
- **Verification:** test_hypervolume_zdt1 passes
- **Committed in:** b6fb53e (Task 1)

**3. [Rule 1 - Bug] Fixed test_gd_zdt1_subset trivial zero GD**
- **Found during:** Task 1 (test execution)
- **Issue:** ZDT1 n=10 front is an exact subset of n=1000 front (999/9=111 integer), making GD = 0.0, violating assert!(result > 0.0)
- **Fix:** Changed to n=5 vs n=10 (9/4 not integer, not exact subset)
- **Files modified:** test_generational_distance.rs
- **Verification:** test_gd_zdt1_subset passes with positive GD
- **Committed in:** b6fb53e (Task 1)

---

**Total deviations:** 3 auto-fixed (3 bugs in pre-existing test files)
**Impact on plan:** All fixes necessary for tests to compile and pass. No scope creep — all changes are in test files.

## Issues Encountered
- Pre-existing `cargo clippy --all-targets -- -D warnings` failure due to `useless_vec!` warnings in indicator test files — these are pre-existing and not introduced by this plan's changes

## Known Stubs
None

## Threat Flags
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Indicator tests wired and passing — ready for next plan in Phase 73 (AOS, local_search, levy_flight, benchmarks migrations)
- 4 of 10 inline test blocks removed from src/

---
*Phase: 73-move-inline-test-modules*
*Completed: 2026-06-19*
