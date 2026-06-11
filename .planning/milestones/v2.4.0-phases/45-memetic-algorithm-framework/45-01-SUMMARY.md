---
phase: 45-memetic-algorithm-framework
plan: 01
subsystem: operators
tags: [memetic, local-search, hill-climbing, lamarckian, baldwinian]

# Dependency graph
requires:
  - phase: 44-standard-benchmark-functions
    provides: Inline test structure patterns for operator modules
provides:
  - LocalSearchOperator trait (6th operator, matching 5-operator pattern)
  - LocalSearch enum with HillClimbing variant and dispatch
  - HillClimbingConfig with step_size=0.1, max_iterations=20 defaults
  - LocalSearchApplicationStrategy enum (AllOffspring, BestN, Probabilistic, EveryNGenerations)
  - LocalSearchMode enum (Lamarckian, Baldwinian)
  - GaError::LocalSearchError variant
  - Factory functions (factory, factory_with_config)
affects: [45-02, 45-03, 45-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "LocalSearchOperator trait with generic improve() matching operator trait pattern"
    - "Enum-based factory dispatch (not Box<dyn> due to generic method incompatibility)"
    - "Runtime type downcast to RangeChromosome<f64> with unsafe gene perturbation"

key-files:
  created:
    - src/operations/local_search.rs
  modified:
    - src/traits/operators.rs
    - src/traits.rs
    - src/error.rs
    - src/operations.rs

key-decisions:
  - "Factory functions return concrete types (LocalSearch enum or HillClimbingConfig), not Box<dyn LocalSearchOperator>, because generic methods make the trait not dyn-compatible"
  - "HillClimbing uses ranges[0].0/ranges[0].1 for clamping (not non-existent min/max fields), following SBX pattern"
  - "HillClimbing uses crate::rng::make_rng() for project consistency, not rand::thread_rng()"
  - "Only RangeChromosome<f64> supported (single-type downcast), not f32/i32/i64 multi-dispatch"

patterns-established:
  - "Pattern: Implement LocalSearchOperator on LocalSearch enum for variant dispatch"
  - "Pattern: Implement LocalSearchOperator on HillClimbingConfig for actual logic"
  - "Pattern: Runtime Any::is::<> type check returning GaError::LocalSearchError"

requirements-completed:
  - MEM-01

# Metrics
duration: 18min
completed: 2026-05-14
---

# Phase 45 Plan 01: Local Search Operator Foundation Summary

**LocalSearchOperator trait, HillClimbing implementation with runtime downcast to RangeChromosome<f64>, and supporting enums (application strategy, mode) — the full foundation layer for memetic algorithm local search**

## Performance

- **Duration:** 18 min
- **Completed:** 2026-05-14
- **Tasks:** 3
- **Files modified:** 5 (1 created, 4 modified)

## Accomplishments
- LocalSearchOperator trait added as the 6th operator (matching existing 5-operator pattern)
- LocalSearch enum with HillClimbing variant and enum-level dispatch
- HillClimbingConfig with step_size and max_iterations defaults (0.1, 20)
- LocalSearchApplicationStrategy (AllOffspring, BestN, Probabilistic, EveryNGenerations)
- LocalSearchMode (Lamarckian, Baldwinian) with Lamarckian default
- hill climbing uses runtime Any::is::<RangeChromosome<f64>>() check with unsafe pointer gene mutation, returns LocalSearchError for unsupported types
- GaError::LocalSearchError variant with Display impl
- Factory functions (factory, factory_with_config) returning concrete types
- Module wiring in operations.rs, re-exports in traits.rs, serde derives on all types
- 7 inline tests pass (improvement count, unsupported type, empty DNA, defaults, factory)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add LocalSearchOperator trait** - `21729dd` (feat)
2. **Task 2: Create local_search.rs module** - `e1e430b` (feat)
3. **Task 3: Wire module declarations and re-exports** - `dd9e09c` (feat)

## Files Created/Modified
- `src/operations/local_search.rs` - New module: LocalSearch enum, HillClimbingConfig, LocalSearchApplicationStrategy, LocalSearchMode, factory functions, HillClimbing implementation
- `src/traits/operators.rs` - Added LocalSearchOperator trait definition
- `src/traits.rs` - Added LocalSearchOperator to re-exports
- `src/error.rs` - Added LocalSearchError variant with Display arm
- `src/operations.rs` - Added pub mod local_search; and pub use re-exports

## Decisions Made
- **Factory returns concrete type, not Box<dyn>**: The `LocalSearchOperator` trait has a generic `improve<U>()` method, making it not `dyn`-compatible. The plan's `Box<dyn LocalSearchOperator>` pattern cannot compile. Changed to match the existing codebase pattern: `impl LocalSearchOperator for LocalSearch` enum directly, with factory functions returning `LocalSearch` or `HillClimbingConfig`.
- **Range gene field access**: The plan's HillClimbing code referenced `gene.min`/`gene.max` but the actual `RangeGene<f64>` struct has `ranges: Arc<[(T, T)]>`. Adapted to use `ranges[0].0`/`ranges[0].1` for clamping (following SBX pattern).
- **RNG**: Used `crate::rng::make_rng()` instead of `rand::thread_rng()` for project consistency.
- **Single-type downcast**: HillClimbing only supports `RangeChromosome<f64>` (not f32/i32/i64 multi-dispatch like SBX), matching the plan's intent.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Factory functions cannot return Box<dyn LocalSearchOperator>**
- **Found during:** Task 2 (Create local_search.rs module)
- **Issue:** The plan specifies `pub fn factory(op: LocalSearch) -> Box<dyn LocalSearchOperator>` but `LocalSearchOperator::improve()` has generic type parameters, making the trait not `dyn`-compatible. Compilation fails with E0038.
- **Fix:** Changed factory functions to return concrete types (`LocalSearch` enum for `factory()`, `HillClimbingConfig` for `factory_with_config()`). Implemented `LocalSearchOperator` on the `LocalSearch` enum directly for variant dispatch (matching `Crossover`/`CrossoverOperator` pattern).
- **Files modified:** src/operations/local_search.rs
- **Verification:** `cargo check --lib` passes, tests pass
- **Committed in:** e1e430b (Task 2 commit)

**2. [Rule 1 - Bug] RangeGene struct uses `ranges` not `min`/`max` fields**
- **Found during:** Task 2 (HillClimbing implementation)
- **Issue:** The plan's HillClimbing implementation code references `gene.min` and `gene.max` but the actual `RangeGene<f64>` struct has `ranges: Arc<[(T, T)]>` with no `min`/`max` fields.
- **Fix:** Changed clamping to use `gene.ranges[0].0` / `gene.ranges[0].1` with `is_empty()` check, following the same pattern as SBX crossover.
- **Files modified:** src/operations/local_search.rs
- **Verification:** Compiles and tests pass
- **Committed in:** e1e430b (Task 2 commit)

**3. [Rule 3 - Blocking] Missing LocalSearchOperator re-export in traits.rs**
- **Found during:** Task 2 compilation
- **Issue:** `crate::traits::LocalSearchOperator` import failed because the `traits` module root (`src/traits.rs`) didn't re-export the new trait.
- **Fix:** Added `LocalSearchOperator` to the `pub use operators::{}` block in `src/traits.rs`.
- **Files modified:** src/traits.rs
- **Verification:** Compilation passes
- **Committed in:** e1e430b (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep. Plan intent preserved.

## Issues Encountered
- None - deviations were straightforward auto-fixes. All followed existing codebase patterns.

## Threat Surface Scan
No new threat flags. Risk dispositions matched implementation:
- T-45-01 (T): Mitigated via Any::is::<RangeChromosome<f64>>() type check
- T-45-02 (I): Mitigated via enum discriminant dispatch
- T-45-03 (D): Accepted - fitness fn passed per-call

## Stub Check
No stubs found. HillClimbing implementation is fully functional; all types have proper defaults.

## Known Stubs
None - all features implemented functionally.

## WASM Compatibility
No code in this plan uses `par_iter()`, `std::time::Instant::now()`, or other WASM-incompatible constructs. Pre-existing WASM compilation failures are from `getrandom` crate, not from this plan's changes.

## Next Phase Readiness
- LocalSearchOperator foundation is complete and ready for integration into the GA loop (Plan 45-02)
- HillClimbingConfig wire-up to a `LocalSearchConfiguration` struct in `src/configuration.rs` is the next step
- Application strategy and mode enums are defined but not yet consumed - ready for downstream wiring

## Self-Check: PASSED

All verification criteria met:
- `cargo check --lib` passes (0 errors)
- `cargo test --lib operations::local_search` passes (7/7)
- `cargo test --features serde --lib operations::local_search` passes (7/7)
- `pub trait LocalSearchOperator` found in operators.rs: 1 match
- `LocalSearchError` found in error.rs: 2 matches (variant + Display)
- `pub mod local_search` found in operations.rs: 1 match
- `pub use local_search` found in operations.rs: 1 match
- All 3 commits found in git history
- `src/operations/local_search.rs` exists with all 7 type definitions

---
*Phase: 45-memetic-algorithm-framework*
*Completed: 2026-05-14*
