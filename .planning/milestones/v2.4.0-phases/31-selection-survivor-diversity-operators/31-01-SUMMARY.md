---
phase: 31
plan: "01"
subsystem: operations
tags: [selection, survivor, diversity, clearing, deterministic-crowding]
dependency_graph:
  requires: []
  provides: [Selection::Clearing, Survivor::DeterministicCrowding]
  affects: [src/operations.rs, src/operations/selection.rs, src/operations/survivor.rs, src/configuration.rs, src/traits/configuration.rs]
tech_stack:
  added: []
  patterns: [enum+factory dispatch, Fisher-Yates random pairing, Hamming distance genotype comparison]
key_files:
  created:
    - src/operations/selection/clearing.rs
    - src/operations/survivor/deterministic_crowding.rs
    - tests/operations/test_selection_clearing.rs
    - tests/operations/test_survivor_deterministic_crowding.rs
  modified:
    - src/operations.rs
    - src/operations/selection.rs
    - src/operations/survivor.rs
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/engines/ga.rs
    - tests/test_operations.rs
    - tests/observe/test_serde.rs
decisions:
  - "Clearing niche distance uses fitness space (|f_a - f_b|) — generic across all chromosome types, no gene-type constraints"
  - "DeterministicCrowding offspring identified by age()==0 — no API changes needed"
  - "niche_radius: f64 added to SelectionConfiguration with default 0.1 — consistent with boltzmann_temperature co-location pattern"
  - "Hamming distance on gene IDs uses min(len_a, len_b) — safe for variable-length chromosomes"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-04"
  tasks_completed: 3
  files_changed: 12
---

# Phase 31 Plan 01: Selection & Survivor Diversity Operators Summary

Implemented two diversity-promoting operators: `Selection::Clearing` and `Survivor::DeterministicCrowding`. Both follow the existing enum + factory dispatch pattern with no new trait definitions and no breaking changes.

## What Was Built

### Selection::Clearing

Niche-based selection operator that prevents any single fitness peak from dominating the mating pool. The algorithm sorts individuals by fitness (descending), identifies the best individual in each fitness-space niche as the winner, clears all other individuals within `niche_radius` of that winner, then randomly pairs the eligible pool.

- Distance metric: `|f_a - f_b|` in fitness space (generic across all chromosome types)
- `niche_radius: f64` added to `SelectionConfiguration` (default `0.1`)
- `with_niche_radius()` builder method added to `SelectionConfig` trait and both impls (`GaConfiguration`, `Ga<U>`)
- Factory dispatch: `Selection::Clearing` uses `niche_radius` from configuration; `SelectionOperator` match arm uses the default value

### Survivor::DeterministicCrowding

Replacement-based survivor strategy that maintains multiple fitness peaks by competing offspring only against similar parents. Each offspring (identified by `age() == 0`) is paired with the available parent having the lowest Hamming distance on gene IDs; the fitter of the pair survives. Unpaired offspring survive unconditionally.

- Distance metric: Hamming on `GeneT::id()` values, comparing `min(len_a, len_b)` positions
- No configuration struct changes needed
- Fully integrated into `SurvivorOperator` trait and factory dispatch

## Commits

| Hash    | Message |
|---------|---------|
| 55fbfd8 | feat(31-01): add Clearing selection operator |
| f84a073 | feat(31-01): add DeterministicCrowding survivor operator |
| 154a0bf | test(31-01): add tests for Clearing selection and DeterministicCrowding survivor |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed test assertions for Clearing niche winner ordering**
- **Found during:** Task 3 (test run)
- **Issue:** Initial tests assumed index 0 (fitness 10.0) would be the niche winner, but the algorithm correctly selects the highest-fitness individual first (index 1, fitness 10.05) as the winner, clearing index 0.
- **Fix:** Updated two test assertions to reflect the correct winner (index 1 = 10.05) and cleared individual (index 0 = 10.0)
- **Files modified:** tests/operations/test_selection_clearing.rs

**2. [Rule 1 - Bug] Fixed missing niche_radius field in test_serde.rs struct initializer**
- **Found during:** Task 3 (serde feature test run)
- **Issue:** `tests/observe/test_serde.rs` manually constructs `SelectionConfiguration` without `..Default::default()`; adding the new `niche_radius` field caused a compile error under `--features serde`
- **Fix:** Added `niche_radius: 0.1` to the struct literal
- **Files modified:** tests/observe/test_serde.rs

## Test Coverage

- 9 tests for `clearing_selection`: niche semantics, radius edge cases (zero/large), 2-chromosome minimum, factory dispatch, enum dispatch
- 10 tests for `deterministic_crowding`: empty pop, all-parents, all-offspring, offspring wins, parent wins, tie resolution, Hamming-based matching, different DNA lengths, multi-pair scenario, enum dispatch
- All 753 tests pass (including `--features serde`)

## Known Stubs

None — both operators are fully wired with real implementations.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes. Both operators are pure in-memory computation.

## Self-Check: PASSED

- src/operations/selection/clearing.rs: FOUND
- src/operations/survivor/deterministic_crowding.rs: FOUND
- tests/operations/test_selection_clearing.rs: FOUND
- tests/operations/test_survivor_deterministic_crowding.rs: FOUND
- Commit 55fbfd8: FOUND
- Commit f84a073: FOUND
- Commit 154a0bf: FOUND
