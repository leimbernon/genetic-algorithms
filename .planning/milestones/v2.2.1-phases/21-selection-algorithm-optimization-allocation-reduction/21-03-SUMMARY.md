---
phase: 21-selection-algorithm-optimization-allocation-reduction
plan: 03
subsystem: ga
tags: [rust, niching, fitness-sharing, allocation-reduction, performance]

# Dependency graph
requires:
  - phase: 21-02
    provides: apply_fitness_sharing_with_dna function in src/niching/sharing.rs
provides:
  - Single fitness_values Vec collection per generation (before niching block)
  - On-the-fly niching via apply_fitness_sharing_with_dna (no O(n^2) matrix)
  - Stats block reuses same fitness_values Vec (no duplicate collection)
affects: [performance-benchmarks, ga-generation-loop]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Collect-once-reuse: single fitness_values Vec collected before niching and reused by stats"
    - "On-the-fly fitness sharing: apply_fitness_sharing_with_dna eliminates intermediate O(n^2) matrix"

key-files:
  created: []
  modified:
    - src/ga.rs

key-decisions:
  - "Single let mut fitness_values before niching block replaces two separate collections — niching modifies in-place, stats reads result"
  - "apply_fitness_sharing_with_dna replaces compute_distance_matrix + apply_fitness_sharing two-step — eliminates O(n^2) distance matrix allocation in generation loop"

patterns-established:
  - "Fitness values collected once per generation at a canonical location before niching; reused downstream"

requirements-completed: [ALLOC-01, ALLOC-02]

# Metrics
duration: 8min
completed: 2026-03-31
---

# Phase 21 Plan 03: Selection Algorithm Optimization — Fitness Collection Merge Summary

**Merged redundant fitness_values Vec collections in ga.rs and replaced compute_distance_matrix+apply_fitness_sharing two-step with on-the-fly apply_fitness_sharing_with_dna, eliminating 1 O(n) allocation and 1 O(n^2) matrix allocation per generation when niching is enabled.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-31T11:50:00Z
- **Completed:** 2026-03-31T11:58:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Fitness values are now collected exactly once per generation at a canonical location before the niching block
- Niching block uses `apply_fitness_sharing_with_dna` (on-the-fly, O(n) niche_counts Vec only — no O(n^2) distance matrix)
- Stats block reuses the same `fitness_values` Vec; duplicate `.collect()` call removed
- Observable behavior preserved: stats see post-niching fitness values when niching is enabled (same as before)

## Task Commits

Each task was committed atomically:

1. **Task 1: Merge fitness collection and switch to on-the-fly niching in ga.rs** - `5179618` (feat)

**Plan metadata:** (docs commit to follow)

## Files Created/Modified
- `src/ga.rs` - Merged fitness collection; switched niching from two-step matrix pattern to on-the-fly apply_fitness_sharing_with_dna; removed duplicate stats-block collection

## Decisions Made
- Outer `let mut fitness_values` inserted immediately after `recalculate_aga()` — natural position before niching so niching can modify in-place and stats read the final result
- Pre-existing `clippy::too_many_arguments` warning on `parent_crossover` is out of scope (existed before this plan)

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered
- `test_reporter_on_new_best_fires` failed once during parallel `cargo test` run — confirmed flaky (passes in isolation and in subsequent full-suite runs). Pre-existing issue unrelated to this plan's changes.

## Next Phase Readiness
- Phase 21 complete (all 3 plans executed)
- ga.rs generation loop now has minimal allocations: no O(n^2) matrix, single fitness Vec, merged collection point
- No blockers for subsequent phases

---
*Phase: 21-selection-algorithm-optimization-allocation-reduction*
*Completed: 2026-03-31*
