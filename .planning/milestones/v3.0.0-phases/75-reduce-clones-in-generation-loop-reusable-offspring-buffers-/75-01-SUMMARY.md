---
phase: 75-reduce-clones-in-generation-loop-reusable-offspring-buffers
plan: "01"
subsystem: operations
tags: [copy-derive, mutation, configuration, zero-cost, prerequisite]
dependency_graph:
  requires: []
  provides: [Mutation:Copy, MutationConfiguration:Copy]
  affects: [src/operations.rs, src/configuration.rs]
tech_stack:
  added: []
  patterns: [derive-widening]
key_files:
  created: []
  modified:
    - src/operations.rs
    - src/configuration.rs
decisions:
  - "D-01: All 8 *Params structs derive Copy — every field is Option<f64>, soundness trivial"
  - "D-02: MutationConfiguration derives Copy — prerequisite Mutation:Copy satisfied by D-01; consistent with CrossoverConfiguration and LimitConfiguration"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-19T09:25:06Z"
status: complete
---

# Phase 75 Plan 01: Copy-derive on Mutation and MutationConfiguration Summary

Add `Copy` to `Mutation`, all 8 `*Params` payload structs, and `MutationConfiguration` — a zero-runtime-cost derive widening that lets Plan 02 delete every `mutation_method.clone()` call site.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add Copy derive to 8 *Params structs and Mutation enum (D-01) | 03f4832 | src/operations.rs |
| 2 | Add Copy derive to MutationConfiguration (D-02) | f644eb4 | src/configuration.rs |

## What Was Built

`Mutation`, `CreepParams`, `GaussianParams`, `PolynomialParams`, `NonUniformParams`, `DifferentialParams`, `CauchyParams`, `LevyFlightParams`, `SelfAdaptiveGaussianParams`, and `MutationConfiguration` all now derive `Copy`. Every field in these types is `Option<f64>`, `bool`, or `Mutation` — all of which are `Copy`, so the derives are sound. No behavior changed; this plan only widens the derive attribute lists.

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

- `cargo build` — zero errors
- `cargo test --test test_operations` — 396 passed, 1 ignored
- `cargo clippy --all-targets` — zero warnings

## Threat Flags

None. Internal type-derive change only; no external input, no new I/O, no new dependencies. Serde round-trip is unaffected (field layout unchanged).

## Known Stubs

None.

## Self-Check: PASSED

- src/operations.rs modified: FOUND (03f4832)
- src/configuration.rs modified: FOUND (f644eb4)
- `grep -c 'Copy' src/operations.rs` = 14 (9 new Copy derives added)
- `pub enum Mutation` preceded by derive containing `Clone` and `Copy`: CONFIRMED
- `pub struct MutationConfiguration` preceded by derive containing `Clone` and `Copy`: CONFIRMED
