---
phase: 32-crossover-differential-mutation
plan: "03"
subsystem: engine-dispatch
tags: [mutation, differential-evolution, engine, serde, builder]
dependency_graph:
  requires: [32-01, 32-02]
  provides: [CRS-01, MUT-04]
  affects: [src/engines/ga.rs, src/configuration.rs, src/traits/configuration.rs, tests/observe/test_serde.rs]
tech_stack:
  added: []
  patterns: [enum-dispatch, builder-fluent, differential-evolution]
key_files:
  created: []
  modified:
    - src/engines/ga.rs
    - src/traits/configuration.rs
    - src/configuration.rs
    - tests/observe/test_serde.rs
decisions:
  - "Added with_differential_f to GaConfiguration (MutationConfig impl) when removing provisional trait default — both implementors needed the method"
  - "Removed provisional default body from MutationConfig::with_differential_f so implementors are forced to provide real bodies"
metrics:
  duration_mins: 15
  completed: "2026-05-06"
  tasks_completed: 2
  tasks_total: 2
---

# Phase 32 Plan 03: Engine Wiring for Differential Mutation Summary

Wire `Mutation::Differential` into the standard `Ga<U>` engine dispatch and update serde tests for new enum variants and `differential_f` configuration field.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Ga::with_differential_f and Differential dispatch | 5a17961 | src/engines/ga.rs, src/traits/configuration.rs, src/configuration.rs |
| 2 | Update serde tests for new variants and differential_f field | 38f3060 | tests/observe/test_serde.rs |

## What Was Built

- **`MutationConfig::with_differential_f` trait method** — Removed provisional default body (`{ let _ = f; self }`) so all implementors must provide real implementations.
- **`GaConfiguration::with_differential_f`** — Sets `mutation_configuration.differential_f = Some(f)` on the bare config type.
- **`Ga<U>::with_differential_f`** — Sets `configuration.mutation_configuration.differential_f = Some(f)` on the engine builder.
- **`parent_crossover` Differential dispatch** — Two branches added (one per child), gated on `Mutation::Differential`. Calls `crate::operations::mutation::differential::differential_mutation` with `*key` (parent_1 index) for child_1 and `*value` (parent_2 index) for child_2. Non-Differential paths unchanged.
- **Serde tests** — `Crossover::EdgeRecombination` added to `serde_crossover_enum`; `Mutation::Differential` added to `serde_mutation_enum`. The `differential_f: None` field was already present in the `serde_ga_configuration_with_values` struct literal from Plan 02.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing impl] Added with_differential_f to GaConfiguration**
- **Found during:** Task 1 — when removing the provisional default body from the trait, `GaConfiguration` (which implements `MutationConfig`) also needed the method or the build would fail.
- **Issue:** Plan 03 only mentioned adding the impl to `Ga<U>`, but `GaConfiguration` is also an implementor and relied on the provisional default.
- **Fix:** Added `fn with_differential_f(mut self, f: f64) -> Self` to the `impl MutationConfig for GaConfiguration` block in `src/configuration.rs`.
- **Files modified:** `src/configuration.rs`
- **Commit:** 5a17961

## Verification Results

- `cargo build` — clean
- `cargo test` — 736 passed, 23 ignored
- `cargo test --features serde` — 766 passed, 23 ignored
- `cargo clippy --all-targets -- -D warnings` — clean (no issues)
- `cargo doc --no-deps` — 1 pre-existing warning on `SelectionConfiguration::niche_radius` unresolved link (not introduced by this plan)

## Known Stubs

None. All dispatch paths wire to real operator implementations.

## Threat Flags

None. All changes are internal dispatch wiring and test updates; no new network endpoints, auth paths, or schema changes at trust boundaries.

## Self-Check: PASSED

- `src/engines/ga.rs` — FOUND (modified, committed at 5a17961)
- `src/traits/configuration.rs` — FOUND (modified, committed at 5a17961)
- `src/configuration.rs` — FOUND (modified, committed at 5a17961)
- `tests/observe/test_serde.rs` — FOUND (modified, committed at 38f3060)
- Commit 5a17961 — FOUND in git log
- Commit 38f3060 — FOUND in git log
