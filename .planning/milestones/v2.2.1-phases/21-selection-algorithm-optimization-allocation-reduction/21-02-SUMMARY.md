---
phase: 21-selection-algorithm-optimization-allocation-reduction
plan: 02
subsystem: niching
tags: [fitness-sharing, niching, allocation-reduction, on-the-fly, O(n)-memory]

# Dependency graph
requires:
  - phase: 21-selection-algorithm-optimization-allocation-reduction
    provides: Context and research for allocation reduction patterns in niching
provides:
  - apply_fitness_sharing_with_dna public function in src/niching/sharing.rs that computes fitness sharing in O(n) memory
  - Three correctness and behavioral tests in tests/niching/test_niching_sharing.rs
affects: [22, 23, 24, ga-niching-integration]

# Tech tracking
tech-stack:
  added: []
  patterns: ["On-the-fly distance computation in double loop replaces O(n^2) pre-allocated matrix"]

key-files:
  created: []
  modified:
    - src/niching/sharing.rs
    - tests/niching/test_niching_sharing.rs

key-decisions:
  - "apply_fitness_sharing_with_dna allocates only O(n) niche_counts Vec — eliminates O(n^2) distance matrix in hot generation loop"
  - "Old apply_fitness_sharing and compute_distance_matrix left unchanged — no breaking API changes"
  - "TDD: tests committed in RED phase before implementation, verified failing, then implemented GREEN"

patterns-established:
  - "On-the-fly distance computation: iterate (i,j) pairs inline, accumulate niche_counts[i] directly without storing distances"

requirements-completed: [ALLOC-02]

# Metrics
duration: 7min
completed: 2026-03-31
---

# Phase 21 Plan 02: Selection Algorithm Optimization — Allocation Reduction (Niching) Summary

**apply_fitness_sharing_with_dna eliminates the O(n^2) distance matrix allocation in fitness sharing by computing Hamming/custom distances on-the-fly into an O(n) niche_counts buffer**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-31T11:39:05Z
- **Completed:** 2026-03-31T11:46:45Z
- **Tasks:** 1 (TDD: 2 commits — test RED + feat GREEN)
- **Files modified:** 2

## Accomplishments
- Added `apply_fitness_sharing_with_dna` as a public function in `src/niching/sharing.rs` — functionally equivalent to `compute_distance_matrix + apply_fitness_sharing` but O(n) memory
- Three behavioral tests confirm: correctness vs matrix approach (within 1e-10), empty input safety, and distant-individuals unchanged-fitness invariant
- Old public API (`apply_fitness_sharing`, `compute_distance_matrix`, `sharing_function`) preserved unchanged — zero breaking changes

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Failing tests for apply_fitness_sharing_with_dna** - `2dd152c` (test)
2. **Task 1 GREEN: Implement apply_fitness_sharing_with_dna** - `70c9d64` (feat)

**Plan metadata:** (docs commit — see below)

_Note: TDD tasks have multiple commits (test RED → feat GREEN)_

## Files Created/Modified
- `src/niching/sharing.rs` - Added `apply_fitness_sharing_with_dna` function (56 lines) after `compute_distance_matrix`
- `tests/niching/test_niching_sharing.rs` - Added 3 new tests and updated import to include new function

## Decisions Made
- Used `niche_counts: Vec<f64>` of size n instead of accumulating into a single variable per outer iteration — cleaner separation between accumulation and application passes, matches existing `apply_fitness_sharing` structure
- Kept `raw_fitnesses: Vec<f64>` clone (same as existing function) to allow in-place update without overwriting values mid-loop

## Deviations from Plan

None — plan executed exactly as written.

The pre-existing `clippy::too_many_arguments` warning in `src/ga.rs` (function `parent_crossover`) was confirmed as pre-existing before our changes and logged as out-of-scope.

## Issues Encountered
- Pre-existing clippy `-D warnings` failure in `src/ga.rs::parent_crossover` (8 args, limit is 7). Confirmed pre-existing via `git stash` verification. Not fixed — out of scope for this plan.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `apply_fitness_sharing_with_dna` is ready to be wired into the GA niching integration in `src/ga.rs` as a replacement call site for `compute_distance_matrix + apply_fitness_sharing`
- All 3 niching plans (21-01, 21-02, 21-03) can proceed independently

---
*Phase: 21-selection-algorithm-optimization-allocation-reduction*
*Completed: 2026-03-31*
