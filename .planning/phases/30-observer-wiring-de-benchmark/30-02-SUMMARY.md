---
plan: 30-02
phase: 30-observer-wiring-de-benchmark
status: complete
completed: 2026-05-02
subsystem: engines
tags: [observer, cellular, alps, lifecycle-hooks]
dependency-graph:
  requires: []
  provides: [OBS-03, OBS-04]
  affects: [src/engines/cellular/engine.rs, src/engines/alps/engine.rs]
tech-stack:
  added: []
  patterns: [GaObserver-Arc-dyn, notify-inline-closure, GenerationStats-from-fitness-values]
key-files:
  created: []
  modified:
    - src/engines/cellular/engine.rs
    - src/engines/alps/engine.rs
    - tests/engines/cellular/test_cellular.rs
    - tests/engines/alps/test_alps.rs
decisions:
  - "AlpsEngine prev_best_fitness snapshot placed at generation start (before inner evolution loop) so that on_new_best fires when any evolution within the generation improves the global best"
  - "CellularEngine prev_best_fitness snapshot placed before inner row/col sweep per plan spec"
metrics:
  duration: ~8 minutes
  completed: 2026-05-02
  tasks: 2
  files: 4
---

# Phase 30 Plan 02: Wire GaObserver into CellularEngine and AlpsEngine Summary

## What Was Built

GaObserver lifecycle hooks wired into CellularEngine and AlpsEngine. Both engines now accept an optional `Arc<dyn GaObserver<U> + Send + Sync>` via `with_observer()` and fire all 5 hooks: `on_run_start`, `on_generation_start`, `on_new_best`, `on_generation_end`, and `on_run_end`. The observer field defaults to `None` — existing code paths are unaffected.

Key correctness properties enforced:
- **CellularEngine**: `on_new_best` fires at most once per generation (not per-cell) by snapshotting `prev_best_fitness` before the inner row/col sweep and comparing after replacements.
- **AlpsEngine**: `on_generation_end` receives stats merged across all layers (D-06); `on_new_best` fires based on global best across all layers (D-07), with snapshot taken at generation start before any evolution.

## Key Files Created/Modified

- `src/engines/cellular/engine.rs` — Added `observer` field, `with_observer()` in first impl block, `notify()` + 5 hook call sites in second impl block; renamed `_gen` to `gen`; added `stats_history` and `is_maximization` locals.
- `src/engines/alps/engine.rs` — Added `observer` field, `with_observer()` in first impl block, `notify()` + 5 hook call sites in second impl block; added `stats_history`, `is_maximization`, and merged-layer fitness collection.
- `tests/engines/cellular/test_cellular.rs` — Added `CellularSpyData`, `CellularSpyObserver`, `test_cellular_observer_fires_5_hooks` (with `on_new_best <= max_gens` assertion), `test_cellular_no_observer_no_panic`.
- `tests/engines/alps/test_alps.rs` — Added `AlpsSpyData`, `AlpsSpyObserver`, `test_alps_observer_fires_5_hooks`, `test_alps_no_observer_no_panic`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] AlpsEngine prev_best_fitness snapshot placement**
- **Found during:** Task 2 test run — `test_alps_observer_fires_5_hooks` failed with "on_new_best must fire at least once"
- **Issue:** Plan spec placed the snapshot immediately before the global tracking block (lines 241+). However, the inner evolution loop (lines 158-206 of original) already updates `best_fitness` for offspring improvements. Snapshotting after the inner loop meant `prev_best_fitness` equaled the already-updated `best_fitness`, so the comparison was always false.
- **Fix:** Moved `let prev_best_fitness = best_fitness;` to the very start of the generation body (after `on_generation_start`), before any evolution runs. This correctly captures the pre-generation best.
- **Files modified:** `src/engines/alps/engine.rs`
- **Commit:** e8f9b01

**2. [Rule 3 - Blocking] Wrong import path for GaObserver**
- **Found during:** Task 1 first compile — `use crate::observe::observer::GaObserver` failed
- **Issue:** `lib.rs` maps the physical path `observe/observer/mod.rs` to the module name `observer`, so the crate-internal path is `crate::observer::GaObserver` not `crate::observe::observer::GaObserver`.
- **Fix:** Corrected import to `use crate::observer::GaObserver;` in both engine files.
- **Files modified:** `src/engines/cellular/engine.rs`, `src/engines/alps/engine.rs`

## Self-Check

- [x] `cargo test --test test_engines` passes (171 tests, 2 ignored)
- [x] CellularEngine on_new_best fires at most once per generation (not per-cell) — asserted in test
- [x] AlpsEngine on_generation_end uses merged stats across all layers (D-06)
- [x] AlpsEngine on_new_best based on global best across all layers (D-07)
- [x] `cargo clippy -- -D warnings` passes (no issues)

## Self-Check: PASSED
