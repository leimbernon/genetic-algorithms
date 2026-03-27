---
phase: 16-sub-traits
plan: "02"
subsystem: observer
tags: [rust, observer, island-model, rayon, arc, parallel]

# Dependency graph
requires:
  - phase: 16-sub-traits-01
    provides: IslandGaObserver and Nsga2Observer trait definitions in src/observer/mod.rs
provides:
  - IslandGa<U> observer field (Option<Arc<dyn IslandGaObserver<U> + Send + Sync>>)
  - with_observer() builder method on IslandGa<U>
  - notify() dispatch helper on IslandGa<U>
  - on_island_run_start hook (fires before generation loop)
  - on_island_run_end hook (fires on both normal exit and fitness-target early return)
  - on_island_generation_end hook (fires per island per generation inside par_iter_mut)
  - on_migration_triggered hook (fires after each migration)
  - Zero log!() calls in src/island/mod.rs
affects: [16-sub-traits-03, examples, integration-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Clone-once-before-parallel: Arc<dyn Trait> cloned once before par_iter_mut, moved into closure"
    - "notify() helper: #[inline] FnOnce dispatch — zero cost when observer is None"

key-files:
  created: []
  modified:
    - src/island/mod.rs
    - tests/island/test_island.rs

key-decisions:
  - "island_id=0 passed to run-level hooks (on_island_run_start, on_island_run_end) — IslandGa is a single logical run, not per-island-thread; future refactor can add per-island run hooks"
  - "gen: usize added as explicit parameter to private evolve_islands_one_generation() — cleaner than storing generation in struct"
  - "test_island_ga_validate_empty_configs updated to use with_heterogeneous_configs([]) — observer field is private, struct literal syntax no longer valid"

patterns-established:
  - "Clone-once-before-parallel pattern for Arc<dyn Trait> in rayon par_iter_mut closures"

requirements-completed: [SUB-01]

# Metrics
duration: 15min
completed: 2026-03-27
---

# Phase 16 Plan 02: IslandGa Observer Integration Summary

**IslandGa<U> now fires four lifecycle hooks via IslandGaObserver: run-start, run-end (both paths), per-island generation, and migration — with Arc clone-once-before-parallel pattern for rayon safety**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-27T08:40:00Z
- **Completed:** 2026-03-27T08:55:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Observer field `Option<Arc<dyn IslandGaObserver<U> + Send + Sync>>` added to IslandGa struct
- `with_observer()` builder and `notify()` helper added, matching the `Ga<U>` pattern from Plan 13
- All 4 lifecycle hooks wired at correct call sites in `run()` and `evolve_islands_one_generation()`
- Clone-once-before-parallel pattern used for rayon safety in `par_iter_mut`
- All `log!()` calls and `use log::{debug, info}` removed from `src/island/mod.rs`
- Existing test updated to use public constructor API (private field broke struct literal)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add observer field, with_observer(), notify() to IslandGa + hook calls + remove log!()** - `5ed3e3a` (feat)

## Files Created/Modified
- `src/island/mod.rs` - Added observer field, with_observer(), notify(), 4 hook call sites; removed log calls
- `tests/island/test_island.rs` - Fixed test_island_ga_validate_empty_configs to use public API

## Decisions Made
- `island_id=0` passed to run-level hooks — the run-level observer hooks conceptually track the island model as a single entity, not individual island threads
- Added `gen: usize` parameter to the private `evolve_islands_one_generation()` method — needed for `on_island_generation_end` without storing generation in struct state
- Test fix used `with_heterogeneous_configs(island_config, vec![])` to test empty configs validation since the new private `observer` field broke struct literal construction in external tests

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed test_island_ga_validate_empty_configs broken by private observer field**
- **Found during:** Task 1 (verification — cargo test)
- **Issue:** `tests/island/test_island.rs` used struct literal syntax `IslandGa::<Binary> { ... }` without the `observer` field — fails to compile because `observer` is private and missing from the literal
- **Fix:** Replaced struct literal with `IslandGa::<Binary>::with_heterogeneous_configs(island_config, vec![])` which tests the same empty-configs validation path
- **Files modified:** tests/island/test_island.rs
- **Verification:** `cargo test` passes (22 passed, 0 failed)
- **Committed in:** 5ed3e3a (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Necessary fix — test was using struct literal that broke when a private field was added. Public API is the correct pattern for external tests. No scope creep.

## Issues Encountered
- Pre-existing `log!()` calls exist in `src/island/nsga2.rs` and `src/island/migration.rs` (5 total). These are out of scope for plan 16-02 which modifies only `src/island/mod.rs`. Deferred to phase 16-03 or a future log-migration task. Logged to deferred-items.

## Next Phase Readiness
- `IslandGaObserver` fully integrated into `IslandGa<U>` — ready for plan 16-03 (Nsga2Observer integration)
- Log calls in `island/nsga2.rs` and `island/migration.rs` remain as pre-existing technical debt
- `LogObserver` already implements `IslandGaObserver` (from plan 16-01) — users can attach it immediately

---
*Phase: 16-sub-traits*
*Completed: 2026-03-27*
