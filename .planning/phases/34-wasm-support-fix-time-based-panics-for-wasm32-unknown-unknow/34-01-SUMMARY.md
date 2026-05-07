---
phase: 34-wasm-support-fix-time-based-panics-for-wasm32-unknown-unknow
plan: "01"
subsystem: observe/reporter
tags: [wasm, cfg-target-arch, reporter, duration]
dependency_graph:
  requires: []
  provides: [DurationReporter wasm32-compatible]
  affects: [src/observe/reporter/duration.rs]
tech_stack:
  added: []
  patterns: [cfg(not(target_arch = "wasm32")), cfg(target_arch = "wasm32")]
key_files:
  modified:
    - src/observe/reporter/duration.rs
decisions:
  - "cfg-gate Instant import and all call sites; return Duration::ZERO on wasm32 (D-02, D-03)"
  - "Suppress on_start warning on wasm32 since start field does not exist there"
metrics:
  duration: "3 minutes"
  completed: "2026-05-07T12:34:45Z"
  tasks_completed: 1
  tasks_total: 1
---

# Phase 34 Plan 01: DurationReporter WASM cfg-gate Summary

**One-liner:** cfg-gated all six `std::time::Instant` sites in DurationReporter so wasm32 builds compile and return `Duration::ZERO` without panicking.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | cfg-gate Instant usage in DurationReporter | af5ce4c | src/observe/reporter/duration.rs |

## What Was Built

Applied six `#[cfg(not(target_arch = "wasm32"))]` guards and one `#[cfg(target_arch = "wasm32")]` guard in `src/observe/reporter/duration.rs`:

1. **Import gate** — `use std::time::Instant` is now cfg-gated; only `Duration` is always imported.
2. **Struct field gate** — `start: Option<Instant>` field only exists on native targets.
3. **Constructor gate** — `new()` initializes `start: None` only on native; wasm32 struct has no fields.
4. **`on_start` body gate** — `Instant::now()` call wrapped in `cfg(not(wasm32))` block; on wasm32 it is a no-op.
5. **Elapsed computation gate** — native uses `self.start.map(...).unwrap_or(Duration::ZERO)`; wasm32 always assigns `Duration::ZERO`.
6. **Missing `on_start` warning gate** — the `log::warn!` guard is suppressed on wasm32 since the `start` field does not exist there.

## Verification Results

- `cargo build` (native): exit 0
- `cargo test reporter`: 21 passed, 0 failures
- `cargo check --target wasm32-unknown-unknown`: no `Instant`-related errors in `duration.rs` or any DurationReporter code paths
- Remaining wasm32 errors are pre-existing `getrandom` backend issues (out of scope for plan 01; addressed in subsequent plans)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — all threat mitigations from the threat register (T-34-01) are implemented: `Instant::now()` is unreachable on wasm32 targets.

## Self-Check: PASSED

- [x] `src/observe/reporter/duration.rs` exists and contains all 6 cfg gates
- [x] Commit af5ce4c exists: `fix(34-01): cfg-gate Instant usage in DurationReporter for wasm32`
- [x] No modifications to STATE.md or ROADMAP.md
