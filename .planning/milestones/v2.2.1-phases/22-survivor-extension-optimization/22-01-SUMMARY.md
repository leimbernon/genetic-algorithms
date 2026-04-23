---
phase: 22-survivor-extension-optimization
plan: 01
subsystem: performance
tags: [rust, genetic-algorithms, selection, extension, O(n), partitioning]

# Dependency graph
requires:
  - phase: 21-selection-niching-optimization
    provides: O(log n) selection sampling and O(n) fitness sharing in ga.rs
provides:
  - O(n) elite reinsertion via select_nth_unstable_by in reinsert_elite (src/ga.rs)
  - O(n) single-pass top-2 scan in mass_genesis (src/operations/extension/mass_genesis.rs)
affects: [ga.rs, mass_genesis, elitism, extension]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "select_nth_unstable_by for worst-k partitioning (O(n) replacement for sort in hot paths)"
    - "Single-pass min-2 tracking with best_idx/second_idx swap+truncate pattern"

key-files:
  created: []
  modified:
    - src/ga.rs
    - src/operations/extension/mass_genesis.rs

key-decisions:
  - "reinsert_elite uses select_nth_unstable_by with worst-first comparator: Maximization=natural order (lower=worst first), Minimization=reversed (higher=worst first)"
  - "mass_genesis single-pass handles best_idx==0 swap displacement via second_idx correction before second swap"
  - "Added .take(k) guard in reinsert_elite for elitism count exceeding population size edge case"

patterns-established:
  - "Worst-k partitioning pattern: select_nth_unstable_by(k-1, worst_first_cmp) places k worst at indices 0..k"
  - "Top-2 single-pass: initialize from [0],[1], scan rest updating best_idx/second_idx, then swap+truncate"

requirements-completed:
  - ALGO-05
  - ALGO-06

# Metrics
duration: 12min
completed: 2026-03-31
---

# Phase 22 Plan 01: Survivor/Extension O(n) Algorithm Optimization Summary

**O(n) partitioning replaces O(n log n) sort in elite reinsertion and O(n) single-pass scan replaces sort+truncate in mass genesis**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-03-31T17:00:00Z
- **Completed:** 2026-03-31T17:12:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- `reinsert_elite` in `src/ga.rs` now uses `select_nth_unstable_by` (O(n)) to place the k worst chromosomes at indices 0..k, then overwrites them — replaces an O(n log n) full sort called every generation with elitism
- `mass_genesis` in `src/operations/extension/mass_genesis.rs` now uses a single O(n) loop tracking `best_idx`/`second_idx`, followed by two swaps and a truncate — replaces an O(n log n) sort
- All 33 ga tests and 3 mass_genesis tests pass with zero failures

## Task Commits

Each task was committed atomically:

1. **Task 1: O(n) elite reinsertion in reinsert_elite** - `7527a01` (perf)
2. **Task 2: O(n) single-pass top-2 scan in mass_genesis** - `2e2d62c` (perf)

## Files Created/Modified
- `src/ga.rs` - reinsert_elite now uses select_nth_unstable_by instead of sort_by
- `src/operations/extension/mass_genesis.rs` - mass_genesis now uses single-pass best_idx/second_idx tracking

## Decisions Made
- Worst-first comparator for `select_nth_unstable_by`: Maximization uses natural order (lower fitness = worst, placed first); Minimization/FixedFitness uses reversed order (higher fitness = worst, placed first)
- `best_idx==0` displacement fix: after `chromosomes.swap(0, best_idx)`, if `second_idx` was 0 it now points to `best_idx` position — fixed with `if second_idx == 0 { second_idx = best_idx; }` before the second swap
- Added `.take(k)` in the overwrite loop to guard against elite count exceeding population size (edge case triggered existing test `test_elitism_count_exceeding_population_does_not_panic`)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added .take(k) guard in reinsert_elite overwrite loop**
- **Found during:** Task 1 (O(n) elite reinsertion in reinsert_elite)
- **Issue:** Initial implementation iterated all of `elite` without capping at `k`, causing index-out-of-bounds when `elite.len() > chromosomes.len()` — test `test_elitism_count_exceeding_population_does_not_panic` caught this
- **Fix:** Added `.take(k)` to the iterator so overwrite only touches indices 0..k, matching the original `take(count)` guard
- **Files modified:** src/ga.rs
- **Verification:** `test_elitism_count_exceeding_population_does_not_panic` passes; full suite passes
- **Committed in:** `7527a01` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug)
**Impact on plan:** Required for correctness on edge case; no scope creep.

## Issues Encountered
- Initial reinsert_elite implementation omitted `.take(k)`, caught immediately by existing test — fixed in same task commit

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 22 Plan 01 complete: O(n) hot-path algorithms in place for elitism and extension
- Phase 22 Plan 02 (if it exists) can proceed; both changes are backwards-compatible with zero API surface changes

---
*Phase: 22-survivor-extension-optimization*
*Completed: 2026-03-31*
