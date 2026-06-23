---
phase: 78-replace-user-input-panics-with-gaerror-issue-279
plan: "02"
subsystem: engines/eda, engines/pso, engines/cma
tags: [error-handling, gaerror, panic-elimination, breaking-change]
dependency_graph:
  requires: [78-01]
  provides: [EDA-run-Result, PSO-run-Result, CMA-run-Result]
  affects: [engines/eda/engine.rs, engines/pso/engine.rs, engines/cma/engine.rs]
tech_stack:
  added: []
  patterns: [Result-propagation, map_err-GaError, ok_or_else-GaError]
key_files:
  created: []
  modified:
    - src/engines/eda/engine.rs
    - src/engines/pso/engine.rs
    - src/engines/cma/engine.rs
decisions:
  - "EdaEngine::run(), EdaRealEngine::run(), PsoEngine::run(), CmaEngine::run() all return Result<...Result<U>, GaError> — breaking change under v3.0.0"
  - "Empty-population user init_fn returns Err(GaError::InitializationError) — not a panic"
  - "CMA defensive fallback (global_best None) returns Err(GaError::InternalError) via ok_or_else"
  - "All fitness-cache lock() sites use .map_err(|_| GaError::InternalError(...))?  propagation"
metrics:
  duration: "~8min"
  completed: "2026-06-20"
  tasks_completed: 3
  files_modified: 3
status: complete
---

# Phase 78 Plan 02: Convert EDA/PSO/CMA Engine run() to Result Summary

EDA, PSO, and CMA engines now return `Result<...Result<U>, GaError>` from `run()`. All six empty-population `panic!` sites and all ten fitness-cache `lock().expect()` sites in these three files are eliminated. Every error is now recoverable from the caller — a user-supplied `init_fn` that returns an empty population yields `Err(GaError::InitializationError)`, and a poisoned mutex yields `Err(GaError::InternalError)`.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Convert EDA engine run() to Result | a3d4a9f | src/engines/eda/engine.rs |
| 2 | Convert PSO engine run() to Result | 56ef0bd | src/engines/pso/engine.rs |
| 3 | Convert CMA engine run() to Result | f4c3f36 | src/engines/cma/engine.rs |

## Changes Made

### Task 1: EDA Engine (src/engines/eda/engine.rs)

- Added `use crate::error::GaError` import
- Changed `EdaEngine::run()` signature: `EdaResult<U>` → `Result<EdaResult<U>, GaError>`
- Changed `EdaRealEngine::run()` signature: `EdaResult<U>` → `Result<EdaResult<U>, GaError>`
- Replaced `panic!("EdaEngine: init_fn returned an empty population")` with `Err(GaError::InitializationError(...))`
- Replaced `panic!("EdaRealEngine: init_fn returned an empty population")` with `Err(GaError::InitializationError(...))`
- Converted 4 `lock().expect("fitness cache lock poisoned")` calls to `.map_err(|_| GaError::InternalError(...))?`
- Wrapped success `EdaResult { ... }` in `Ok(...)` at both method return points

### Task 2: PSO Engine (src/engines/pso/engine.rs)

- Added `use crate::error::GaError` import
- Changed `PsoEngine::run()` signature: `PsoResult<U>` → `Result<PsoResult<U>, GaError>`
- Replaced `panic!("PsoEngine: init_fn returned an empty population")` with `Err(GaError::InitializationError(...))`
- Converted 2 `lock().expect("fitness cache lock poisoned")` calls to `.map_err(|_| GaError::InternalError(...))?`
- Wrapped success `PsoResult { ... }` in `Ok(...)`

### Task 3: CMA Engine (src/engines/cma/engine.rs)

`GaError` was already imported from Plan 01.

- Changed `CmaEngine::run()` signature: `CmaResult<U>` → `Result<CmaResult<U>, GaError>`
- Replaced 3 empty-pop `panic!` sites with `Err(GaError::InitializationError(...))`:
  - peek-population guard (base message)
  - first-init guard (`(first init)` message)
  - restart-init guard (`(restart)` message)
- Converted defensive fallback `global_best.unwrap_or_else(|| panic!(...))` to `global_best.ok_or_else(|| GaError::InternalError(...))?`
- Converted 4 `lock().expect(...)` calls to `.map_err(|_| GaError::InternalError(...))?`:
  - Two in `batch_evaluate_pop()` (D-06 partition cache acquire)
  - One in the inner generation loop (D-07 pre-generation snapshot)
  - One in the statistics section (D-07 post-generation stats)
- Wrapped success `CmaResult { ... }` in `Ok(...)`

## Verification

```
grep -rn 'panic!' src/engines/eda/engine.rs src/engines/pso/engine.rs src/engines/cma/engine.rs
# → no output (0 matches)

grep -rn 'lock().expect' src/engines/eda/engine.rs src/engines/pso/engine.rs src/engines/cma/engine.rs
# → no output (0 matches)

cargo build --lib
# → Finished dev profile
```

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

No new security-relevant surface introduced. All changes are error-path conversions within existing code.

## Self-Check: PASSED

- src/engines/eda/engine.rs: modified (verified via `cargo build --lib`)
- src/engines/pso/engine.rs: modified (verified via `cargo build --lib`)
- src/engines/cma/engine.rs: modified (verified via `cargo build --lib`)
- Commits a3d4a9f, 56ef0bd, f4c3f36 confirmed in `git log`
