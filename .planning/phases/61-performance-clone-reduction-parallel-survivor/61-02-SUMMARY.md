---
phase: 61-performance-clone-reduction-parallel-survivor
plan: "02"
subsystem: operations/survivor
tags: [performance, rayon, parallel, survivor, wasm]
dependency_graph:
  requires: []
  provides: [parallel-survivor-sort]
  affects: [src/operations/survivor/fitness.rs, src/operations/survivor/mu_plus_lambda.rs, src/operations/survivor/age.rs, src/operations/survivor/mu_comma_lambda.rs]
tech_stack:
  added: []
  patterns: [dual-cfg rayon par_sort_unstable_by with wasm sequential fallback]
key_files:
  modified:
    - src/operations/survivor/fitness.rs
    - src/operations/survivor/mu_plus_lambda.rs
    - src/operations/survivor/age.rs
    - src/operations/survivor/mu_comma_lambda.rs
decisions:
  - "Unstable sort accepted per D-06: previous sort_by was already non-deterministic on fitness ties in practice"
  - "sort_by_key(Reverse) in age.rs converted to explicit comparator (b.age().cmp(&a.age())) to use par_sort_unstable_by without needing par_sort_unstable_by_key"
  - "DeterministicCrowding excluded per D-08: order-dependent operator not suitable for parallel sort"
metrics:
  duration: "~8 minutes"
  completed: "2026-06-08"
  tasks_completed: 2
  files_modified: 4
---

# Phase 61 Plan 02: Parallel Survivor Sort Summary

Parallelized all four sort-based survivor operators with `par_sort_unstable_by` (native) and `sort_unstable_by` (WASM fallback) via dual-cfg gates.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Parallelize fitness.rs and mu_plus_lambda.rs | 429d4b0 | fitness.rs, mu_plus_lambda.rs |
| 2 | Parallelize age.rs and mu_comma_lambda.rs | cdbbbbe | age.rs, mu_comma_lambda.rs |

## What Was Built

Applied `par_sort_unstable_by` (with sequential `sort_unstable_by` WASM fallback) to the four sort-based survivor operators per D-06/D-07/D-08/D-09. Each file received:

1. `#[cfg(not(target_arch = "wasm32"))] use rayon::prelude::*;` import
2. Dual-cfg sort blocks replacing `sort_by` / `sort_by_key` calls:
   - Non-wasm: `chromosomes.par_sort_unstable_by(...)` via rayon
   - Wasm: `chromosomes.sort_unstable_by(...)` sequential fallback

**Site counts:**
- `fitness.rs`: 2 sort sites converted (fitness branch + FixedFitness branch)
- `mu_plus_lambda.rs`: 2 sort sites converted (fitness branch + FixedFitness branch)
- `age.rs`: 1 sort site converted (sort_by_key(Reverse) converted to explicit comparator)
- `mu_comma_lambda.rs`: 2 sort sites converted (fitness branch + FixedFitness branch)

`DeterministicCrowding` was NOT touched — it is order-dependent and explicitly excluded per D-08.

## Verification Results

- `cargo test --test test_operations`: 365 passed
- `cargo test --features serde`: 1247 passed, 46 ignored
- `cargo check --target wasm32-unknown-unknown`: clean
- `cargo clippy --all-targets -- -D warnings`: no issues

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — internal sort optimization, no new network endpoints or trust boundaries.

## Self-Check: PASSED

- src/operations/survivor/fitness.rs: modified with 2 par_sort_unstable_by sites
- src/operations/survivor/mu_plus_lambda.rs: modified with 2 par_sort_unstable_by sites
- src/operations/survivor/age.rs: modified with 1 par_sort_unstable_by site
- src/operations/survivor/mu_comma_lambda.rs: modified with 2 par_sort_unstable_by sites
- Commit 429d4b0: exists (Task 1)
- Commit cdbbbbe: exists (Task 2)
