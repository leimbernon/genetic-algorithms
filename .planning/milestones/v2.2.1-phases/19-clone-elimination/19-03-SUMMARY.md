---
phase: 19-clone-elimination
plan: 03
subsystem: operations
tags: [crossover, clone-elimination, performance, genetic-algorithms]

# Dependency graph
requires:
  - phase: 19-clone-elimination
    provides: "ChromosomeT::new() default construction pattern for zero-clone child initialization"
provides:
  - "All 10 crossover operators construct children via U::new() or RangeChromosome::new() instead of parent.clone()"
  - "CLONE-02 requirement fully satisfied across entire crossover layer"
affects:
  - operations/crossover
  - performance-optimizations

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "U::new() + set_dna(Cow::Owned(dna)) pattern for generic crossover child construction"
    - "RangeChromosome::<T>::new() + set_dna(Cow::Owned(dna)) for concrete numeric crossover operators"
    - "U::new() + set_dna(Cow::Borrowed(parent.dna())) for rejuvenate (DNA preserved, metadata not)"

key-files:
  created: []
  modified:
    - src/operations/crossover/multipoint.rs
    - src/operations/crossover/uniform_crossover.rs
    - src/operations/crossover/cycle.rs
    - src/operations/crossover/single_point.rs
    - src/operations/crossover/rejuvenate.rs
    - src/operations/crossover/order.rs
    - src/operations/crossover/pmx.rs
    - src/operations/crossover/sbx.rs
    - src/operations/crossover/blend_alpha.rs
    - src/operations/crossover/arithmetic.rs

key-decisions:
  - "Children start from default state (U::new()) rather than inheriting parent fitness/age metadata — fitness is always re-evaluated before selection so parent metadata is wasted work"
  - "Rejuvenate operator uses Cow::Borrowed(parent.dna()) to share DNA slice without copying, avoiding even the DNA allocation"

patterns-established:
  - "Crossover child construction: never clone parent structs — use U::new() + set_dna()"
  - "Numeric crossover: use RangeChromosome::<T>::new() matching the concrete type parameter"

requirements-completed: [CLONE-02]

# Metrics
duration: 10min
completed: 2026-03-30
---

# Phase 19 Plan 03: Clone-Free Child Construction in All Crossover Operators Summary

**Eliminated parent.clone() from all 10 crossover operators — children now constructed via U::new() + set_dna(), avoiding fitness closure and metadata inheritance**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-03-30T17:50:00Z
- **Completed:** 2026-03-30T17:58:00Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- All 7 generic crossover operators (multipoint, uniform, cycle, single_point, rejuvenate, order, pmx) now use `U::new()` for child construction
- All 3 concrete numeric crossover operators (sbx, blend_alpha, arithmetic) now use `RangeChromosome::<T>::new()` for child construction
- Rejuvenate operator specifically uses `Cow::Borrowed(parent.dna())` to share the DNA slice without allocation — most efficient option since DNA is not recomputed
- CLONE-02 requirement fully satisfied: zero parent.clone() calls in the entire crossover layer

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace parent.clone() with U::new() in generic crossover operators** - `3270a38` (feat)
2. **Task 2: Replace parent.clone() with RangeChromosome::new() in numeric crossover operators** - `43d735f` (feat)

## Files Created/Modified

- `src/operations/crossover/multipoint.rs` - U::new() for child construction
- `src/operations/crossover/uniform_crossover.rs` - U::new() for child construction
- `src/operations/crossover/cycle.rs` - U::new() for child construction
- `src/operations/crossover/single_point.rs` - U::new() for child construction
- `src/operations/crossover/rejuvenate.rs` - U::new() + set_dna(Cow::Borrowed) for child construction
- `src/operations/crossover/order.rs` - U::new() for child construction
- `src/operations/crossover/pmx.rs` - U::new() for child construction
- `src/operations/crossover/sbx.rs` - RangeChromosome::<T>::new() for child construction
- `src/operations/crossover/blend_alpha.rs` - RangeChromosome::<T>::new() for child construction
- `src/operations/crossover/arithmetic.rs` - RangeChromosome::<T>::new() for child construction

## Decisions Made

- Children are constructed from default state (`U::new()`) rather than cloned parents because: (1) fitness is always re-evaluated before selection so parent fitness is irrelevant to children, (2) age should start at 0 for all children, (3) cloning the fitness function closure and other parent metadata is pure waste
- For the rejuvenate operator specifically, `Cow::Borrowed(parent.dna())` is used to share the DNA slice without any allocation — this is valid because the borrow only needs to last through `set_dna()` which clones/installs the data

## Deviations from Plan

None - plan executed exactly as written. Task 1 (generic operators) had already been completed in a previous session (commit 3270a38). Task 2 (numeric operators) had changes already applied in the working tree and was committed in this session.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CLONE-02 requirement fully satisfied — all crossover operators are clone-free for child construction
- All existing tests pass unchanged (22 tests, cargo test + cargo test --features serde)
- No public API changes
- Phase 19 clone-elimination work continues with any remaining plans

---
*Phase: 19-clone-elimination*
*Completed: 2026-03-30*
