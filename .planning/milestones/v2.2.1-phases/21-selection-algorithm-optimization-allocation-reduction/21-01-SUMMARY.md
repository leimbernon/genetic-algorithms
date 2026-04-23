---
phase: 21-selection-algorithm-optimization-allocation-reduction
plan: 01
subsystem: operations
tags: [selection, performance, binary-search, partition-point, rank-selection, boltzmann-selection]

# Dependency graph
requires: []
provides:
  - "O(log n) roulette-wheel sampling in Rank Selection via partition_point"
  - "O(log n) roulette-wheel sampling in Boltzmann Selection via partition_point"
affects: [performance-optimizations, selection-operators]

# Tech tracking
tech-stack:
  added: []
  patterns: ["partition_point(|&x| x < r).min(n-1) idiom for binary-search roulette-wheel sampling"]

key-files:
  created: []
  modified:
    - src/operations/selection/rank.rs
    - src/operations/selection/boltzmann.rs

key-decisions:
  - "partition_point(predicate).min(n-1) replaces iter().position(predicate).unwrap_or(n-1) — semantically equivalent but O(log n)"
  - ".min(n-1) clamp is essential: partition_point returns n when all elements satisfy the predicate (float drift), which would be an out-of-bounds index"
  - "Predicate must be cp < r (strict less-than) — partition_point returns the first index where predicate is false, i.e., first where cp >= r"

patterns-established:
  - "Binary-search roulette-wheel: cumulative.partition_point(|&x| x < r).min(n-1)"

requirements-completed: [ALGO-03, ALGO-04]

# Metrics
duration: 5min
completed: 2026-03-31
---

# Phase 21 Plan 01: Selection Algorithm Optimization Summary

**Rank and Boltzmann roulette-wheel sampling changed from O(n) iter().position() to O(log n) partition_point(), reducing per-sample inner-loop cost from linear to logarithmic**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-31T11:38:30Z
- **Completed:** 2026-03-31T11:40:23Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Replaced O(n) linear scan in `rank_selection` with `partition_point(|&(_, cp)| cp < r).min(n-1)`
- Replaced O(n) linear scan in `boltzmann_selection` with `partition_point(|&cp| cp < r).min(n-1)`
- All 57 selection tests pass unchanged — behaviour is identical

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace linear scan with partition_point in Rank and Boltzmann selection** - `e29041e` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/operations/selection/rank.rs` - roulette-wheel sampling switched to partition_point binary search
- `src/operations/selection/boltzmann.rs` - roulette-wheel sampling switched to partition_point binary search

## Decisions Made
- `partition_point` is called directly on the Vec (not on `.iter()`), which is required for the method to exist on slices
- Predicate is `cp < r` (strict) so that partition_point returns the first index where `cp >= r` — matching the prior `position(|cp| cp >= r)` semantics exactly
- `.min(n-1)` clamp replaces `.unwrap_or(n-1)` — handles the float-drift edge case where all cumulative values are less than r, which would make partition_point return n (out-of-bounds)
- Removed the inline comment "Find the first individual whose cumulative probability >= r" from rank.rs as partition_point is self-documenting

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing clippy failure (`too-many-arguments` in an unrelated function) and pre-existing niching test compilation error — both out of scope and not caused by this plan's changes.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plans 21-02 and 21-03 are independent and can proceed immediately
- Both selection operators now use O(log n) sampling — inner selection loop is O(k log n) instead of O(k*n)

---
*Phase: 21-selection-algorithm-optimization-allocation-reduction*
*Completed: 2026-03-31*
