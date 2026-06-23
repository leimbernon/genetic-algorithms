---
phase: 83-wire-lexicase-selection-into-ga-run-close-sel-02-sel-03-trai
plan: "01"
subsystem: ga-engine
tags: [lexicase, selection, vector-fitness, ga-run]
status: complete

dependency_graph:
  requires:
    - src/operations/selection.rs (factory_lexicase — already complete from Phase 50)
    - src/traits/VectorFitness (already complete from Phase 55)
  provides:
    - Ga::run_lexicase()
    - Ga::run_lexicase_with_callback()
    - Lexicase ConfigurationError guard in run_with_callback
  affects:
    - src/engines/ga/mod.rs

tech_stack:
  added: []
  patterns:
    - VectorFitness-constrained impl block extension
    - matches! macro for early error guard
    - Full generation loop duplication (accepted; TODO comment for future consolidation)

key_files:
  created: []
  modified:
    - src/engines/ga/mod.rs

decisions:
  - "run_lexicase_with_callback duplicates run_with_callback body (accepted for Phase 83; TODO comment left for future consolidation via parameterized inner loop)"
  - "num_parents = 2 is enforced implicitly — factory_lexicase does not accept num_parents argument; documented in comment rather than dead let binding (clippy -D warnings)"
  - "RealValuedMutation bound added to VectorFitness impl block to match base impl block requirements for generation::parent_crossover"

metrics:
  duration: "~10 minutes"
  completed: "2026-06-23"
  tasks: 2
  files: 1
---

# Phase 83 Plan 01: Wire Lexicase Selection into Ga::run — Summary

Wired `Selection::Lexicase` and `Selection::EpsilonLexicase` into the full GA run path by adding `run_lexicase()` and `run_lexicase_with_callback()` on the `VectorFitness`-constrained `impl<U> Ga<U>` block, and added an early `matches!` guard in `run_with_callback` that returns a clear `ConfigurationError` naming `run_lexicase` for lexicase variants.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| T01 | Add lexicase error guard to run_with_callback | c062777 | src/engines/ga/mod.rs |
| T02 | Add run_lexicase and run_lexicase_with_callback | eaea603 | src/engines/ga/mod.rs |

## What Was Built

**T01** — Inserted a `matches!` guard immediately after `ValidatorFactory::validate` in `run_with_callback`. When `Selection::Lexicase` or `Selection::EpsilonLexicase` is configured, the standard `run()` / `run_with_callback()` path now returns `GaError::ConfigurationError` whose message contains `run_lexicase`, directing users to the correct entry point. The base `impl<U> Ga<U>` where-clause at ~line 773 remains unchanged — no `VectorFitness` bound was added to the base block.

**T02** — Extended the `VectorFitness`-constrained `impl<U> Ga<U>` block:
- Added `+ crate::traits::RealValuedMutation` to its where-clause (required by `generation::parent_crossover`)
- `run_lexicase(&mut self)` — thin wrapper mirroring the existing `run()` turbofish pattern
- `run_lexicase_with_callback<F>` — full generation loop identical to `run_with_callback` except:
  1. No T01 lexicase guard (this is the legitimate entry point)
  2. Selection call replaced with `crate::operations::selection::factory_lexicase(&mut self.population.chromosomes, ...)` — passes `&mut [U]` for scalar fitness sync (D-04); drops the `num_parents` argument (factory_lexicase enforces 2-parent groups internally)
  3. All existing `cfg` gates (`wasm32`, `parallel`) preserved byte-identical

## Verification Results

- `cargo build` — clean
- `cargo clippy --all-targets -- -D warnings` — no issues
- `cargo check --target wasm32-unknown-unknown` — clean
- `cargo test` — 1577 passed, 6 ignored
- `grep fn run_lexicase src/engines/ga/mod.rs` — found at lines 2505, 2523
- `grep factory_lexicase src/engines/ga/mod.rs` — found at lines 2492, 2721
- Base impl block where-clause at ~line 773 — confirmed VectorFitness absent

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused `let num_parents = 2;` binding**
- **Found during:** T02 compilation — `cargo build` emitted `unused_variables` warning
- **Issue:** The plan specified `let num_parents = 2;` to document the forced 2-parent decision, but `factory_lexicase` does not accept a `num_parents` argument, making the binding dead. This would fail `clippy -D warnings`.
- **Fix:** Removed the `let num_parents = 2;` statement; moved the Pitfall 3 rationale into an inline comment above the `factory_lexicase` call explaining that the 2-parent constraint is enforced internally by `factory_lexicase`.
- **Files modified:** src/engines/ga/mod.rs
- **Commit:** eaea603

## Known Stubs

None — all methods are fully implemented and call production code paths.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All changes are within the existing `src/engines/ga/mod.rs` module. Threat mitigations T-83-01 and T-83-02 from the plan's threat register are implemented:
- T-83-01: Early `matches!` guard in `run_with_callback` rejects Lexicase/EpsilonLexicase with a clear ConfigurationError
- T-83-02: `factory_lexicase` enforces 2-parent groups internally; the lexicase path cannot desync on multi-parent crossover config

## Self-Check: PASSED

- [x] `src/engines/ga/mod.rs` exists and contains `fn run_lexicase` and `fn run_lexicase_with_callback`
- [x] Commits c062777 and eaea603 exist in git log
- [x] 1577 tests pass; clippy clean; wasm32 check clean
