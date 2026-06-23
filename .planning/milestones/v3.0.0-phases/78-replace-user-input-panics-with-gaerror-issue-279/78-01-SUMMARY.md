---
phase: 78-replace-user-input-panics-with-gaerror-issue-279
plan: "01"
subsystem: error-handling
tags: [error, mutex, panic-safety, recoverable-errors]
dependency_graph:
  requires: []
  provides: [GaError::InternalError, cache_snapshot_result, cache_fill_stats_result]
  affects: [src/engines/ga/generation.rs, src/engines/ga/cache.rs, src/engines/ga/batch.rs, src/engines/ga/mod.rs]
tech_stack:
  added: []
  patterns: [map_err propagation, Result return types for mutex operations]
key_files:
  created: []
  modified:
    - src/error.rs
    - src/engines/ga/generation.rs
    - src/engines/ga/cache.rs
    - src/engines/ga/batch.rs
    - src/engines/ga/mod.rs
    - tests/test_error.rs
decisions:
  - "GaError::InternalError(String) placed after TreeSizeExceeded — last variant, additive only"
  - "cache_snapshot and cache_fill_stats return Result so callers propagate via ?"
  - "All mutex poison messages are stable string literals for grep-ability"
metrics:
  duration: "5min"
  completed: "2026-06-20"
  tasks_completed: 3
  files_modified: 6
status: complete
---

# Phase 78 Plan 01: Add GaError::InternalError and Convert Mutex Panics Summary

## One-liner

GaError::InternalError(String) variant added; all 12 mutex panic sites in ga/ converted to recoverable map_err propagation.

## What Was Built

Added the `GaError::InternalError(String)` variant to the crate error enum as the foundation for converting runtime panics (primarily from poisoned mutexes) into recoverable errors. Converted all 12 panic-producing mutex lock sites across three files:

- **Task 1 (TDD):** Added `InternalError(String)` to `GaError` with Display arm `"Internal error: {msg}"`. Added 4 tests to `tests/test_error.rs` covering construction, display, clone/PartialEq, and Debug format.
- **Task 2:** Replaced 8 `lock().unwrap()` calls in `generation.rs` (AOS state and reward accumulator mutexes) with `map_err(|_| GaError::InternalError(...))?`. No signature changes to `parent_crossover`.
- **Task 3:** Changed `cache_snapshot` return type to `Result<(u64, u64), GaError>` and `cache_fill_stats` to `Result<(), GaError>`. Converted 2 `lock().expect()` calls in `batch.rs`. Updated callers in `mod.rs` with `?`.

## Commits

| Task | Commit | Files |
|------|--------|-------|
| Task 1: Add InternalError variant + tests | 54afc97 | src/error.rs, tests/test_error.rs |
| Task 2: Convert AOS mutex unwraps | 83a1ef4 | src/engines/ga/generation.rs |
| Task 3: Convert cache mutex expects | dc843e0 | src/engines/ga/cache.rs, src/engines/ga/batch.rs, src/engines/ga/mod.rs |

## Verification

- `cargo build`: exits 0
- `cargo test`: 1560 passed, 6 ignored
- `grep -c "InternalError(String)" src/error.rs`: 1
- `grep lock().unwrap() generation.rs` (non-comment): 0 matches
- `grep lock().expect cache.rs batch.rs`: 0 matches

## Deviations from Plan

None — plan executed exactly as written. All 8 AOS mutex sites and 4 cache mutex sites converted; return types updated exactly as specified.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

- src/error.rs: FOUND — contains InternalError(String) variant and Display arm
- src/engines/ga/generation.rs: FOUND — zero lock().unwrap() remaining
- src/engines/ga/cache.rs: FOUND — zero lock().expect() remaining, returns Result
- src/engines/ga/batch.rs: FOUND — zero lock().expect() remaining
- src/engines/ga/mod.rs: FOUND — callers propagate with ?
- tests/test_error.rs: FOUND — 4 new tests passing
- Commits: 54afc97, 83a1ef4, dc843e0 — all present in git log
