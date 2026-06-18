---
phase: 70-replace-operator-downcasting
plan: 02
subsystem: operations
tags: [rust, mutation, trait-dispatch, downcasting, compile-time]

# Dependency graph
requires:
  - phase: 70-01
    provides: RealValuedMutation trait with 5 default-error methods and Range<T> impl
provides:
  - Zero downcast/Any references in mutation.rs
  - All 5 real-valued mutation operators dispatch via RealValuedMutation trait methods
  - RealValuedMutation impls for all chromosome types (Binary, List, Unique, MultiRange, MultiUnique)
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [compile-time-trait-dispatch, blanket-trait-impl-per-type]

key-files:
  created: []
  modified:
    - src/operations/mutation.rs
    - src/engines/ga/generation.rs
    - src/engines/ga/mod.rs
    - src/traits/real_valued_mutation.rs
    - src/types/chromosomes/binary.rs
    - src/types/chromosomes/list.rs
    - src/types/chromosomes/unique.rs
    - src/types/chromosomes/multi_range.rs
    - src/types/chromosomes/multi_unique.rs

key-decisions:
  - "RealValuedMutation added as bound to mutate(), factory(), factory_with_params(), factory_with_chromosome_length(), factory_self_adaptive()"
  - "RealValuedMutation added to Ga<U>, Strategy<U>, and all multi-objective engine impl blocks"
  - "Explicit RealValuedMutation impls for all chromosome types rather than blanket impl (conflicts with Range<T> specific impl)"

patterns-established:
  - "Trait-bound propagation: adding a bound to mutation dispatch requires it on all caller impl blocks"

requirements-completed: []

# Metrics
duration: 11min
completed: 2026-06-18
---

# Phase 70 Plan 02: Mutation.rs Refactor Summary

**Replaced runtime downcasting with RealValuedMutation trait dispatch — zero downcast/Any/try_* references in mutation.rs, all 5 operators dispatch via compile-time trait methods**

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-18T08:34:55Z
- **Completed:** 2026-06-18T08:46:01Z
- **Tasks:** 2
- **Files modified:** 28

## Accomplishments
- Removed 5 `try_*` functions (118 lines) and `try_type!` macro from mutation.rs
- Removed `std::any::Any` and `RangeChromosome` imports from mutation.rs
- Replaced 5 match arms with `RealValuedMutation` trait method calls
- Added `RealValuedMutation` bound to `mutate()`, `factory()`, `factory_with_params()`, `factory_with_chromosome_length()`, `factory_self_adaptive()`
- Added `RealValuedMutation` impls for all chromosome types (Binary, List, Unique, MultiRange, MultiUnique) and all test/example types
- Propagated `RealValuedMutation` bound to `Ga<U>`, `Strategy<U>`, and all multi-objective engine impl blocks

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor mutation.rs to use RealValuedMutation trait dispatch** - `810d94c` (feat)
2. **Task 2: Full verification gate — clippy, fmt, doc-tests, WASM** - `55dd46b` (test)

## Files Created/Modified
- `src/operations/mutation.rs` - Removed downcasting code, added trait dispatch
- `src/engines/ga/generation.rs` - Added RealValuedMutation bound to parent_crossover
- `src/engines/ga/mod.rs` - Added RealValuedMutation bound to Ga<U> impl blocks
- `src/traits/real_valued_mutation.rs` - Updated doc-comment
- `src/types/chromosomes/binary.rs` - Added RealValuedMutation impl
- `src/types/chromosomes/list.rs` - Added RealValuedMutation impl
- `src/types/chromosomes/unique.rs` - Added RealValuedMutation impl
- `src/types/chromosomes/multi_range.rs` - Added RealValuedMutation impl
- `src/types/chromosomes/multi_unique.rs` - Added RealValuedMutation impl
- `tests/structures.rs` - Added RealValuedMutation impls for test types
- Multiple test and example files - Added RealValuedMutation impls

## Decisions Made
- RealValuedMutation bound added to mutation dispatch functions (not just mutate())
- Explicit impls for each chromosome type rather than blanket impl (avoids conflict with Range<T> specific impl)
- Ga<U> and all multi-objective engine impl blocks require RealValuedMutation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cascading RealValuedMutation bound propagation**
- **Found during:** Task 1 (mutation.rs refactor)
- **Issue:** Adding RealValuedMutation bound to mutate() cascaded to parent_crossover, Ga<U>, Strategy<U>, and all multi-objective engine impl blocks. Also required RealValuedMutation impls for all chromosome types (Binary, List, Unique, MultiRange, MultiUnique, and ~15 test/example types).
- **Fix:** Added RealValuedMutation impls (default error-returning) for all chromosome types. Added RealValuedMutation bound to all affected impl blocks.
- **Files modified:** src/engines/ga/mod.rs, src/engines/ga/generation.rs, src/types/chromosomes/*.rs, tests/*.rs, examples/*.rs, benches/*.rs
- **Verification:** cargo test passes (268 tests), cargo clippy clean
- **Committed in:** 810d94c (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Cascading bound propagation was necessary for correctness. All existing tests pass identically.

## Issues Encountered
None beyond the deviation above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 70 is complete: mutation.rs has zero downcast/Any/try_* references
- All 5 real-valued mutation operators dispatch via RealValuedMutation trait methods
- All 268 existing tests pass identically
- Phase 70 can be marked complete in ROADMAP.md

---
*Phase: 70-replace-operator-downcasting*
*Completed: 2026-06-18*
