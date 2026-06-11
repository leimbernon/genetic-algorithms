---
phase: 18-observer-api-polish
plan: 01
subsystem: observer
tags: [observer, tracing, composability, hooks, timing, ordering]

# Dependency graph
requires:
  - phase: 17-composite-and-metrics
    provides: AllObserver blanket impl, CompositeObserver, IslandGaObserver and Nsga2Observer traits
  - phase: 15-tracingobserver
    provides: TracingObserver struct with GaObserver impl

provides:
  - TracingObserver satisfies AllObserver (composable via CompositeObserver::add)
  - on_extension_triggered fires before on_generation_end within same generation
  - on_mutation_complete and on_fitness_evaluation_complete receive real non-zero Duration

affects:
  - observer-api-polish
  - any future plan using TracingObserver in CompositeObserver
  - EXT-01 (per-operator timing separation, v2.3+)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Empty impl blocks satisfy AllObserver blanket impl bounds (same pattern as MetricsObserver)"
    - "Hook ordering: extension block before reporter+on_generation_end restores pre-v2.2.0 semantics"

key-files:
  created: []
  modified:
    - src/observer/tracing_observer.rs
    - src/ga.rs

key-decisions:
  - "elapsed (combined crossover+mutation+fitness time) passed to on_mutation_complete and on_fitness_evaluation_complete — per-operator separation deferred to EXT-01 (v2.3+)"
  - "Extension block moved before reporter and on_generation_end — restores pre-v2.2.0 hook ordering where on_extension_triggered fires within the same generation context as on_generation_end"
  - "Unused std::time::Duration import removed from ga.rs after Duration::ZERO replacement"

patterns-established:
  - "Empty IslandGaObserver/Nsga2Observer impls unlock AllObserver blanket impl for any GaObserver implementor"

requirements-completed: [OBS-01, LOG-01, TRAC-01, COMP-01, COMP-02]

# Metrics
duration: 10min
completed: 2026-03-28
---

# Phase 18 Plan 01: Observer API Polish Summary

**TracingObserver made composable via AllObserver blanket impl; ga.rs extension hook ordering and Duration::ZERO timing bugs fixed.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-03-28T16:20:24Z
- **Completed:** 2026-03-28T16:29:39Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- TracingObserver now implements IslandGaObserver and Nsga2Observer, satisfying the AllObserver blanket impl and allowing Arc<TracingObserver> to be passed to CompositeObserver::add
- Extension block in ga.rs moved before reporter and on_generation_end, restoring pre-v2.2.0 hook ordering (on_extension_triggered fires before on_generation_end within same generation)
- Duration::ZERO replaced with real elapsed time for on_mutation_complete and on_fitness_evaluation_complete hooks

## Task Commits

Each task was committed atomically:

1. **Task 1: Add IslandGaObserver and Nsga2Observer impls to TracingObserver** - `21aab9d` (feat)
2. **Task 2: Fix Duration::ZERO bug and hook ordering in ga.rs** - `2fd8b5d` (fix)

## Files Created/Modified
- `src/observer/tracing_observer.rs` - Updated use statement; added empty IslandGaObserver and Nsga2Observer impl blocks
- `src/ga.rs` - Replaced Duration::ZERO with elapsed in mutation/fitness hooks; moved extension block before reporter+on_generation_end; removed unused Duration import

## Decisions Made
- `elapsed` from the combined crossover+mutation+fitness timer is passed to both `on_mutation_complete` and `on_fitness_evaluation_complete`. Per-operator timers require splitting `parent_crossover` into separate calls, deferred to EXT-01 (v2.3+).
- Extension block now precedes reporter and on_generation_end, exactly matching the pre-v2.2.0 call order documented in the audit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused `std::time::Duration` import from ga.rs**
- **Found during:** Task 2 (Fix Duration::ZERO bug)
- **Issue:** After replacing `Duration::ZERO` with `elapsed`, the `use std::time::Duration` import became unused, producing a compiler warning
- **Fix:** Removed the import line
- **Files modified:** src/ga.rs
- **Verification:** `cargo clippy` clean, all tests pass
- **Committed in:** `2fd8b5d` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug/warning cleanup)
**Impact on plan:** Minor cleanup only, directly caused by the Duration::ZERO replacement. No scope creep.

## Issues Encountered
- `test_reporter_on_new_best_fires` appeared to fail once under `cargo test --features serde` but passed consistently on re-run and in isolation — confirmed pre-existing flaky test unrelated to these changes.

## Next Phase Readiness
- Plan 18-01 requirements fulfilled: TracingObserver composable, hook ordering correct, timing non-zero
- Plan 18-02 can proceed (remaining phase 18 work)

---
*Phase: 18-observer-api-polish*
*Completed: 2026-03-28*
