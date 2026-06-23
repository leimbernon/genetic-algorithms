---
phase: 78
status: issues
findings: 5
blocking: 0
reviewed: 2026-06-20
---

# Code Review — Phase 78: Replace User-Input Panics with GaError

## Summary

5 findings: 2 confirmed bugs, 3 plausible/advisory. No blockers, but the CMA `expect()` and `on_run_end` bypass are real correctness issues that should be fixed.

## Findings

### [C1] CONFIRMED — `batch_evaluate_pop().expect()` panic inside `CmaEngine::run()`
**File:** `src/engines/cma/engine.rs` ~line 682  
**Severity:** High  
`run()` returns `Result<CmaResult<U>, GaError>` but calls `self.batch_evaluate_pop(&mut pop).expect("batch_evaluate_pop failed on initial population")` inside the restart loop. Any batch evaluator error (fitness cache lock poison, evaluator failure) panics instead of propagating `GaError`. Inconsistent with the surrounding `?` propagation pattern throughout `run()`.  
**Fix:** Replace `.expect(...)` with `?`.

### [C2] CONFIRMED — `CellularEngine::select()` swallows ALL `Err` variants
**File:** `src/engines/cellular/engine.rs` ~line 206  
**Severity:** High  
The `Err(_)` catch on the `select()` call swallows every error kind, including `GaError::InternalError` from a poisoned fitness-cache mutex. The intent is to handle only Lexicase incompatibility, but any infrastructure failure silently degrades the engine to always pairing cell 0 with cell 1, with no error surfaced to the caller.  
**Fix:** Match on `Err(GaError::SelectionError(_))` only; propagate all other variants.

### [C3] CONFIRMED — Mutex `?` early-return bypasses `on_run_end` in PSO/EDA/CMA
**File:** `src/engines/pso/engine.rs` ~lines 372, 489; `src/engines/eda/engine.rs`; `src/engines/cma/engine.rs`  
**Severity:** Medium  
All three engines have `?` early-return sites from poisoned mutex locks inside the generation loop. On early return, `on_run_end` (after the loop) is never called, leaving observers with an unmatched `on_run_start` event. Any observer tracking run lifetime (metrics, checkpoint flush) will leak state.  
**Fix:** Use a guard (`defer`-style or `Drop` impl) or catch the error and call `on_run_end` before returning.

### [P1] PLAUSIBLE — `assert!(n > 0)` hard panic inside `CmaEngine::run()`
**File:** `src/engines/cma/engine.rs` ~line 624  
**Severity:** Medium  
`run()` returns `Result` but `assert!(n > 0, "CmaEngine: chromosomes must have non-zero DNA length")` will hard-panic (not return `Err`) if `init_fn` returns chromosomes with zero-length DNA. The empty-population guard above it does not prevent this.  
**Fix:** Replace with `if n == 0 { return Err(GaError::InitializationError(...)); }`.

### [P2] PLAUSIBLE — `AlpsEngine::run()` returns bare `AlpsResult<U>`, not `Result`
**File:** `src/engines/alps/engine.rs` ~line 142  
**Severity:** Low  
`new()` was migrated to `Result` but `run()` still returns `AlpsResult<U>`. If `init_fn` violates its contract and returns an empty Vec, `layers[0][0].clone()` at line ~171 panics and cannot be caught by callers. Asymmetric with PSO/EDA/CMA where `run()` also returns `Result`.  
**Fix:** Migrate `AlpsEngine::run()` to return `Result<AlpsResult<U>, GaError>` in a follow-on phase.

## Not Findings

- `ox_build_child` `collect::<Result<Vec<_>,_>>()` — correct Rust pattern, sound.
- Cellular fallback `[0,1]` OOB — refuted; guarded by `neighbor_idxs.is_empty()` continue.
- Missing `#[must_use]` on `run()` — refuted; `Result` is `#[must_use]` at stdlib type level.
- EDA `find_best()` `assert!` — refuted; always guarded by prior empty-population `Err` return.
