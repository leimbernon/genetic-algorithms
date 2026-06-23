---
phase: 49
plan: 01
subsystem: traits
tags: [strategy-trait, dyn-dispatch, ga-engine, re-exports]
dependency_graph:
  requires: []
  provides: [Strategy<U> trait, impl Strategy<U> for Ga<U>]
  affects: [src/traits, src/lib.rs, src/engines/ga.rs]
tech_stack:
  added: []
  patterns: [enum-factory, dyn-trait, fully-qualified-method-call]
key_files:
  created:
    - src/traits/strategy.rs
  modified:
    - src/engines/ga.rs
    - src/traits.rs
    - src/lib.rs
decisions:
  - Strategy<U> is dyn-safe: U is trait type parameter, not method-level generic
  - Ga<U> impl delegates run() to Ga::run() via fully-qualified syntax to avoid unconditional recursion
  - Strategy<U> for Ga<U> bounds match full Ga::run() where clause for correctness
metrics:
  duration: ~10m
  completed: 2026-05-22
  tasks: 3
  files: 4
---

# Phase 49 Plan 01: Strategy Trait + Ga<U> Impl + Re-exports Summary

## Status: COMPLETE

## What was built

JWT auth with refresh rotation using jose library — wait, wrong template text.

Strategy<U> dyn-safe trait enabling runtime algorithm swapping across GA, hill-climbing, and permutation search engines via `Box<dyn Strategy<U>>`.

- Created `src/traits/strategy.rs` with dyn-safe `Strategy<U>` trait (two methods: `run()` returning `Result<(), GaError>` and `best()` returning `Option<&U>`)
- Added `impl Strategy<U> for Ga<U>` at end of `src/engines/ga.rs` using `Ga::run(self)` fully-qualified call to avoid unconditional recursion
- Updated `src/traits.rs`: added `pub mod strategy; pub use strategy::Strategy;` after `operator_compat` and added `Strategy` to the Key items doc table
- Updated `src/lib.rs`: added `pub use traits::Strategy;` after `pub use traits::OperatorCompat;`

## Verification results

- `cargo build`: PASS
- `cargo clippy`: PASS (no issues)
- `cargo check --target wasm32-unknown-unknown`: PASS

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unconditional recursion in Strategy::run()**
- **Found during:** Task 2 implementation (caught by compiler warning on first build)
- **Issue:** `self.run()` inside `impl Strategy<U> for Ga<U>` resolved to the `Strategy::run` method itself (infinite recursion), not the inherent `Ga::run` method
- **Fix:** Changed to `Ga::run(self).map(|_| ())` using fully-qualified syntax so Rust resolves to the inherent method
- **Files modified:** `src/engines/ga.rs`
- **Commit:** 67ff0f3

**2. [Rule 2 - Missing critical functionality] Expanded Strategy<U> impl bounds to match Ga::run() requirements**
- **Found during:** Task 2 — compiler reported 4 missing trait bounds when using `Ga::run(self)`
- **Issue:** Plan specified `U: LinearChromosome + Send + Sync + 'static + Clone + MaybeDeserialize` but `Ga::run()` also requires `Debug`, `ValueMutable`, `MaybeSerialize`, `OperatorCompat`, and `U::Gene: 'static + Debug`
- **Fix:** Added all required bounds to the impl block where clause to match `Ga::run()`'s full constraint set
- **Files modified:** `src/engines/ga.rs`
- **Commit:** 67ff0f3

## Key decisions

- `Strategy<U>` is dyn-safe: `U` is the trait type parameter, not a method-level generic
- `Ga<U>` impl delegates `run()` to existing `Ga::run()`, discarding the `&Population` return via `.map(|_| ())`
- `best()` checks `best_chromosome_is_set` (a `pub(crate)` field accessible within the crate) before returning a reference to `best_chromosome`

## Known Stubs

None — the trait and impl are fully functional.

## Threat Flags

None — this change adds a trait definition and a trait impl; no new network endpoints, auth paths, file access, or schema changes.

## Self-Check: PASSED

- `src/traits/strategy.rs`: FOUND
- `src/engines/ga.rs` (impl Strategy): FOUND (lines 2788-2809)
- `src/traits.rs` (pub mod strategy + pub use): FOUND
- `src/lib.rs` (pub use traits::Strategy): FOUND
- Commit 67ff0f3: FOUND
