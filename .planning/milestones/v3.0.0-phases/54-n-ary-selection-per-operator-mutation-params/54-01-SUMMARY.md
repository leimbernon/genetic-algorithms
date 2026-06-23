---
phase: 54-n-ary-selection-per-operator-mutation-params
plan: "01"
subsystem: selection
tags: [breaking-change, api-refactor, n-ary-selection, multi-parent]
dependency_graph:
  requires: []
  provides: [SelectionOperator-nary-api, selection-factory-nary, ga-nary-dispatch]
  affects: [ga-loop, island-engine, gp-engine, cellular-engine]
tech_stack:
  added: []
  patterns: [N-ary parent groups via Vec<Vec<usize>>, group.len() dispatch for multi-parent crossover]
key_files:
  created: []
  modified:
    - src/traits/operators.rs
    - src/operations/selection.rs
    - src/operations/selection/tournament.rs
    - src/operations/selection/random.rs
    - src/operations/selection/fitness_proportionate.rs
    - src/operations/selection/rank.rs
    - src/operations/selection/boltzmann.rs
    - src/operations/selection/truncation.rs
    - src/operations/selection/clearing.rs
    - src/operations/selection/lexicase.rs
    - src/engines/ga.rs
    - src/engines/island/mod.rs
    - src/engines/gp/engine.rs
    - src/engines/cellular/engine.rs
    - tests/operations/test_selection.rs
    - tests/operations/test_selection_boltzmann.rs
    - tests/operations/test_selection_clearing.rs
    - tests/operations/test_selection_lexicase.rs
    - tests/operations/test_selection_lexicase_diversity.rs
    - tests/operations/test_selection_rank.rs
    - tests/operations/test_selection_truncation.rs
decisions:
  - "N-ary groups represented as Vec<Vec<usize>> — inner Vec length equals num_parents"
  - "group.len() > 2 dispatches to factory_multi_parent_dispatch; == 2 uses standard factory"
  - "Redundant UNDX/SPX/PCX match block removed from parent_crossover — dispatch purely by group size"
  - "Lexicase always produces groups of 2 (factory_lexicase hardcodes num_parents=2)"
  - "Island, GP, and Cellular engines pass num_parents=2 (standard 2-parent crossover only)"
metrics:
  duration_minutes: 45
  completed_date: "2026-05-28"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 21
---

# Phase 54 Plan 01: N-ary Selection API — SelectionOperator Returns Vec<Vec<usize>> Summary

N-ary selection API generalized from 2-tuple parent pairs to variable-length parent groups; `SelectionOperator::select` and `selection::factory` now return `Vec<Vec<usize>>` where each inner `Vec` has `num_parents` elements, enabling UNDX/SPX/PCX multi-parent crossover to flow through a single unified selection call.

## What Was Built

### Task 1: SelectionOperator Trait + All Selection Functions (commit bc46d97)

- `SelectionOperator::select` signature changed: added `num_parents: usize` parameter after `number_of_threads`; return type changed from `Vec<(usize, usize)>` to `Vec<Vec<usize>>`
- `selection::factory` signature updated: added trailing `num_parents: usize`; returns `Result<Vec<Vec<usize>>, GaError>`
- `factory_lexicase` updated: returns `Vec<Vec<usize>>` (always groups of 2 for lexicase)
- All 10 internal selection functions updated:
  - `random`, `roulette_wheel_selection`, `stochastic_universal_sampling`, `tournament`, `rank_selection`, `boltzmann_selection`, `truncation_selection`, `clearing_selection`, `lexicase_selection`, `epsilon_lexicase_selection`
  - Each now accepts `num_parents: usize` and builds `Vec<Vec<usize>>` groups of that size
  - Existing WASM cfg gates preserved (rayon `par_iter` stays behind `#[cfg(not(target_arch = "wasm32"))]`)

### Task 2: Engine Call Sites + N=3 Test (commit 6673283)

- `src/engines/ga.rs`:
  - Derives `num_parents` from crossover config before selection call: `Crossover::Undx/Spx/Pcx { num_parents } => num_parents`, all others `=> 2`
  - `parent_crossover` parameter changed: `parents: &[(usize, usize)]` → `parents: &[Vec<usize>]`
  - Closure renamed `process_pair` → accepts `group: &Vec<usize>`; extracts `key = group[0]`, `value = group[1]`; asserts `group.len() >= 2` (T-54-01)
  - Dispatch: `if group.len() > 2 { factory_multi_parent_dispatch }` else `{ factory }` — removes redundant UNDX/SPX/PCX match block
  - `select_parents_lexicase` return type updated to `Vec<Vec<usize>>`
- `src/engines/island/mod.rs`: passes `num_parents=2`; iterates `for group in &parent_pairs { let idx_a = group[0]; let idx_b = group[1]; }`
- `src/engines/gp/engine.rs`: passes `num_parents=2`; iterates `for group in &pairs { let (i, j) = (group[0], group[1]); }`
- `src/engines/cellular/engine.rs`: passes `num_parents=2` to direct trait call; destructures `group[0]`/`group[1]`
- All 7 selection test files updated to new API signatures and `Vec<usize>` iteration patterns
- New test `test_factory_returns_groups_of_num_parents`: asserts N=3 produces groups of exactly 3; N=2 produces groups of exactly 2

## Verification Results

| Check | Result |
|-------|--------|
| `cargo test` | 1140 passed, 35 ignored |
| `cargo test --features serde` | 1180 passed, 35 ignored |
| `cargo clippy` | No issues found |
| `cargo check --target wasm32-unknown-unknown` | Compiled successfully |
| `grep -c "Vec<(usize, usize)>" src/operations/selection.rs` | 0 |
| `grep -n "num_parents" src/traits/operators.rs` | Present in signature |
| N=3 group-size test | Passes |

## Deviations from Plan

None - plan executed exactly as written. The only minor observation is that the N=3 test was added to `test_selection.rs` at the end (the plan specified `tests/operations/test_selection.rs` which is where it landed).

## Known Stubs

None. All functionality is wired; no placeholders remain.

## Threat Flags

None. Pure internal trait/API refactor — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Self-Check

PASSED:
- All modified files exist and compile
- Commits bc46d97 and 6673283 verified in git log
- `cargo test` passes (1140/1180 tests)
- No `Vec<(usize, usize)>` return types remain in selection layer
- N=3 group-size test (`test_factory_returns_groups_of_num_parents`) passes
