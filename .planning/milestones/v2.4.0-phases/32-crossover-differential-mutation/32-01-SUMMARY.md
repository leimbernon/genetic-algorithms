---
phase: 32-crossover-differential-mutation
plan: "01"
subsystem: crossover-operators
tags:
  - crossover
  - permutation
  - erx
  - edge-recombination
dependency_graph:
  requires: []
  provides:
    - Crossover::EdgeRecombination operator
  affects:
    - src/operations.rs
    - src/operations/crossover.rs
tech_stack:
  added:
    - ERX algorithm (Whitley 1989) — union adjacency map, fewest-neighbours heuristic, D-06 random fallback
  patterns:
    - enum + factory dispatch (existing pattern)
    - HashMap<i32, HashSet<i32>> adjacency structure for ERX
key_files:
  created:
    - src/operations/crossover/edge_recombination.rs
    - tests/operations/test_crossover_edge_recombination.rs
  modified:
    - src/operations.rs
    - src/operations/crossover.rs
    - tests/test_operations.rs
decisions:
  - "Adjacency map keyed by gene ID (i32) rather than index — index-agnostic, handles arbitrary gene sets cleanly"
  - "Clone adjacency map before first child build so second child gets an independent copy"
  - "fewest-unvisited-neighbours tie-break favours first iterator element — deterministic, acceptable for genetic diversity"
  - "D-06 fallback uses random_range on remaining ids vec — consistent with pmx.rs RNG pattern"
metrics:
  duration: "~7 minutes"
  completed_date: "2026-05-06"
  tasks_completed: 1
  tasks_total: 1
  files_created: 2
  files_modified: 3
---

# Phase 32 Plan 01: Edge Recombination Crossover (ERX) Summary

## One-liner

ERX permutation crossover via union adjacency map and fewest-neighbours heuristic, with D-06/D-07/D-08 validation guards.

## What Was Built

Added `Crossover::EdgeRecombination` to the operator enum and implemented the canonical Whitley 1989 ERX algorithm:

1. **`src/operations.rs`** — New `EdgeRecombination` variant added after `Rejuvenate` in the `Crossover` enum.

2. **`src/operations/crossover/edge_recombination.rs`** — Full ERX implementation:
   - `erx<U: ChromosomeT>()` — validates inputs (D-07 length, D-08 uniqueness + same gene set), builds union adjacency map, produces 2 children
   - `erx_build_child()` — iterative construction using fewest-unvisited-neighbours heuristic; D-06 random fallback when adjacency exhausted
   - All STRIDE mitigations from threat model applied (T-32-01 loop bound, T-32-02 permutation validation, T-32-03 length guard)

3. **`src/operations/crossover.rs`** — Added `pub mod edge_recombination`, `pub use self::edge_recombination::erx`, and `Crossover::EdgeRecombination => erx(...)` arms in both `impl CrossoverOperator` blocks.

4. **`tests/operations/test_crossover_edge_recombination.rs`** — 7 tests covering all required cases: two-children, length preservation, valid permutations (50 iterations), error on different lengths, error on len<2, error on duplicate IDs, D-06 fallback.

5. **`tests/test_operations.rs`** — Added `mod test_crossover_edge_recombination` to the test module list.

## Verification

- `cargo build` — clean
- `cargo test --test test_operations test_crossover_edge_recombination` — 7/7 pass
- `cargo test` — 730 passed, 23 ignored (full suite, no regressions)
- `cargo clippy --all-targets -- -D warnings` — no issues

## Commits

| Hash | Description |
|------|-------------|
| 7c87ace | feat(32-01): add Crossover::EdgeRecombination (ERX) operator |

## Deviations from Plan

None — plan executed exactly as written. The test target structure required adding the module to `tests/test_operations.rs` (the umbrella test file), which was implied by the project's existing pattern but not explicitly stated in the plan. This is standard procedure, not a deviation.

## Known Stubs

None — ERX is fully functional with real data; no hardcoded placeholders.

## Threat Flags

No new trust boundaries introduced. All four STRIDE mitigations from the plan's threat model are applied inline in `erx()` and `erx_build_child()`.

## Self-Check: PASSED

- `src/operations/crossover/edge_recombination.rs` — FOUND
- `tests/operations/test_crossover_edge_recombination.rs` — FOUND
- Commit `7c87ace` — FOUND
- All 7 ERX tests pass
- Full suite 730 tests green
