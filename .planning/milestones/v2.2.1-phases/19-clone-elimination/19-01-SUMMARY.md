---
phase: 19-clone-elimination
plan: 01
subsystem: performance
tags: [rust, genetic-algorithms, clones, mutation, crossover, in-place]

# Dependency graph
requires: []
provides:
  - "Deferred parent cloning in parent_crossover() — clones only in fallback else branch"
  - "In-place swap mutation via dna_mut().swap()"
  - "In-place inversion mutation via dna_mut()[..].reverse()"
  - "In-place scramble mutation via dna_mut().swap()"
affects: [20-fitness-caching, 21-alloc-reduction]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Borrow-first, clone-only-when-needed: get references from population, clone only in fallback branch"
    - "dna_mut() for zero-allocation in-place slice operations (swap, reverse)"

key-files:
  created: []
  modified:
    - src/ga.rs
    - src/operations/mutation/swap.rs
    - src/operations/mutation/inversion.rs
    - src/operations/mutation/scramble.rs

key-decisions:
  - "Pass &U references to crossover::factory and aga_probability instead of owned clones"
  - "Clone parents only in the else fallback branch (crossover probability not met), not unconditionally"
  - "Use slice::swap() and slice::reverse() on dna_mut() instead of per-gene clone+set_gene"

patterns-established:
  - "Deferred cloning: acquire references first, clone only when ownership transfer is required"
  - "In-place mutation: prefer dna_mut() slice operations over index-based gene-by-gene clones"

requirements-completed: [CLONE-01, CLONE-04]

# Metrics
duration: 7min
completed: 2026-03-29
---

# Phase 19 Plan 01: Clone Elimination (Crossover + Mutations) Summary

**Deferred parent cloning in hot crossover path and converted swap/inversion/scramble to zero-allocation in-place dna_mut() operations**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-29T17:30:39Z
- **Completed:** 2026-03-29T17:33:50Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Eliminated 2 unconditional `.clone()` calls per parent pair in `parent_crossover()` — parents are now borrowed references; cloning occurs only in the fallback else branch when crossover probability is not met
- Replaced 4-line clone+set_gene swap pattern with single `chromosome.dna_mut().swap(i, j)` call
- Replaced per-gene clone loop in inversion with single `individual.dna_mut()[lower..=upper].reverse()` call
- Replaced per-gene clone+set_gene scramble loop body with `chromosome.dna_mut().swap(i, random_index)`
- All tests pass (22/22), including serde feature flag; clippy clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Defer parent clones in parent_crossover()** - `c68f818` (feat)
2. **Task 2: In-place swap, inversion, and scramble mutations** - `c9f65a3` (feat)

**Plan metadata:** (docs commit — see final)

## Files Created/Modified
- `src/ga.rs` - Removed unconditional `.clone()` from chromosomes.get(); updated crossover::factory and aga_probability calls to pass references directly; added `.clone()` in else fallback branch
- `src/operations/mutation/swap.rs` - Replaced gene_1/gene_2 clone + set_gene with dna_mut().swap()
- `src/operations/mutation/inversion.rs` - Replaced per-gene clone loop with dna_mut() slice reverse
- `src/operations/mutation/scramble.rs` - Replaced per-gene clone+set_gene with dna_mut().swap() in loop

## Decisions Made
- Used `parent_1` / `parent_2` (not `&parent_1` / `&parent_2`) when passing to `crossover::factory` and `aga_probability` because parents are already `&U` references after removing the `.clone()`
- Kept observer notification points and all other ga.rs logic completely untouched

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 19 plan 01 complete; CLONE-01 and CLONE-04 requirements fulfilled
- Hot crossover path and three index-based mutation operators are now allocation-free for the common case
- Ready to proceed to remaining phase 19 plans (fitness caching, allocation reduction)

---
*Phase: 19-clone-elimination*
*Completed: 2026-03-29*
