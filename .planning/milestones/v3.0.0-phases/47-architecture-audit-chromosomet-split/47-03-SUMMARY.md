---
phase: 47-architecture-audit-chromosomet-split
plan: 03
subsystem: engines, examples, tests
tags:
  - rust
  - engines
  - bound-change
  - breaking-change
  - pr-gate

dependency_graph:
  requires:
    - "47-01 (LinearChromosome trait definition)"
    - "47-02 (operator layer + chromosome implementors migration)"
  provides:
    - "All 12 engine orchestrators constrained to U: LinearChromosome (ARCH-02)"
    - "PR 1 phase verification gate: all 5 checks GREEN"
    - "Examples and test suite compile against new trait boundary"
  affects:
    - "PR 1 (ARCH-01 + ARCH-02) — now mergeable to milestone/v3.0.0"
    - "All downstream feature branches (Phase 48+) unblocked"

tech_stack:
  added: []
  patterns:
    - "Two-impl-block split for custom test chromosomes (ChromosomeT eval + LinearChromosome flat-slice)"
    - "LinearChromosome import required in all user code calling dna()/set_dna()/dna_mut()"

key_files:
  created: []
  modified:
    - src/engines/cellular/engine.rs
    - src/engines/alps/engine.rs
    - src/engines/nsga2/mod.rs
    - src/engines/nsga3/mod.rs
    - src/engines/moead/mod.rs
    - src/engines/spea2/mod.rs
    - src/hall_of_fame.rs
    - examples/aos_demo.rs
    - examples/constrained_g1.rs
    - examples/feature_selection.rs
    - examples/hall_of_fame_demo.rs
    - examples/island_model.rs
    - examples/job_scheduling.rs
    - examples/niching.rs
    - examples/onemax_extension.rs
    - tests/engines/island/test_island_migration.rs
    - tests/engines/island/test_island_nsga2.rs
    - tests/engines/test_ga.rs
    - tests/engines/warm_starting/test_warm_starting.rs
    - tests/types/chromosomes/test_binary.rs
    - tests/types/chromosomes/test_list.rs
    - "tests/operations/test_crossover*.rs (6 files)"
    - "tests/operations/test_mutation*.rs (8 files)"
    - "tests/validators/*.rs (2 files)"
    - "tests/observe/*.rs (2 files)"
    - tests/test_constraints.rs
    - tests/extension/test_extension.rs
    - tests/initializers/test_initializers.rs
    - tests/engines/alps/test_alps.rs
    - tests/engines/cellular/test_cellular.rs
    - tests/engines/de/test_de.rs
    - tests/engines/hall_of_fame/test_hall_of_fame.rs
    - tests/engines/scatter/test_scatter.rs

decisions:
  - "nsga3 helper functions (nsga3_environmental_selection, normalize_st) also changed to LinearChromosome since the overall pattern is consistent and LinearChromosome is a supertrait of ChromosomeT (strictly more capable)"
  - "alleles_can_be_repeated and needs_unique_ids reads in multi-obj engines intentionally left in place — removal is ARCH-04, targeted at PR 2"
  - "hall_of_fame.rs doc comment ChromosomeT → LinearChromosome fixed to eliminate rustdoc unresolved-link warning"

metrics:
  duration: "~11 minutes"
  completed_date: "2026-05-20"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 51
---

# Phase 47 Plan 03: Engine Bound Upgrade to LinearChromosome — Summary

**One-liner:** Upgraded all 12 engine orchestrators (including cellular, alps, nsga2, nsga3, moead, spea2, and the 5 already updated in 47-02) to `U: LinearChromosome` and fixed 51 test/example files to import `LinearChromosome` in scope, making the full PR 1 verification gate GREEN.

## What Was Built

### Task 1: Single-population and alt-metaheuristic engine bound upgrade

Updated the two remaining alt-metaheuristic engines that 47-02 had not yet touched:

- **`src/engines/cellular/engine.rs`**: Changed `use crate::traits::{ChromosomeT, FitnessFn}` → `{LinearChromosome, FitnessFn}`. All struct bounds and impl blocks updated. The engine calls `.dna()` through the fitness function and uses `ValueMutable` (which requires `LinearChromosome`), so the bound is correct.

- **`src/engines/alps/engine.rs`**: Same transformation. ALPS calls `.dna()` on individuals for fitness evaluation, `.set_age()` / `.age()` for layer promotion, and `ValueMutable` for crossover/mutation — all require `LinearChromosome`.

The other 3 alt-metaheuristic engines (`de/engine.rs`, `scatter/engine.rs`) and `ga.rs` were already updated in 47-02 (HallOfFame forced that issue early — see 47-02 SUMMARY deviation 2).

`cargo check --lib` and `cargo check --target wasm32-unknown-unknown --lib` both GREEN after Task 1.

### Task 2: Multi-objective + island engine bound upgrade + PR 1 gate

Updated the 4 multi-objective engines that 47-02 had not yet touched:

- **`src/engines/nsga2/mod.rs`**: `ChromosomeT → LinearChromosome` on `Nsga2Ga<U>` struct and all impl blocks.
- **`src/engines/nsga3/mod.rs`**: Same, including helper functions `nsga3_environmental_selection<U>` and `normalize_st<U>` (upgraded for consistency — they hold `ParetoIndividual<U>` which wraps `U: LinearChromosome`).
- **`src/engines/moead/mod.rs`**: `MoeaDGa<U>` struct and impl blocks.
- **`src/engines/spea2/mod.rs`**: `Spea2Ga<U>` struct and impl blocks.

`sms_emoa/mod.rs`, `ibea/mod.rs`, `island/mod.rs` were already done in 47-02.

**Intentionally preserved** (PR 2 / ARCH-04 territory):
```
nsga2/mod.rs:444:  let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;
moead/mod.rs:554:  let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;
nsga3/mod.rs:491:  let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;
spea2/mod.rs:592:  let alleles_can_repeat = self.ga_config.limit_configuration.alleles_can_be_repeated;
```

### PR 1 Phase Verification Gate Results

| Check | Result |
|-------|--------|
| `cargo test` (default features) | 982 passed, 30 ignored |
| `cargo test --features serde` | 1018 passed, 30 ignored |
| `cargo clippy --all-features -- -D warnings` | No issues |
| `cargo check --target wasm32-unknown-unknown` | Clean |
| `cargo doc --no-deps --all-features` (zero warnings) | Zero warnings |

**ARCH-01 + ARCH-02 fully satisfied. PR 1 is mergeable to `milestone/v3.0.0`.**

## PR 1 Boundary

PR 1 covers plans 47-01, 47-02, 47-03:
- ARCH-01: `ChromosomeT` trait shrunk to 5 evaluation methods
- ARCH-02: `LinearChromosome` supertrait + all operator/engine files updated

PR 1 scope ends here. Deferred to PR 2:
- ARCH-04: `GaConfiguration` encapsulation (`pub(crate)` fields + accessors)
- ARCH-05: `ChromosomeLength` enum
- ARCH-06: `StoppingCriteria` struct removal / flattening
- Removal of `alleles_can_be_repeated` and `needs_unique_ids` fields from `LimitConfiguration`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 51 files missing `LinearChromosome` import after trait split**
- **Found during:** Task 2, running `cargo test`
- **Issue:** The `ChromosomeT` → `LinearChromosome` split moved `dna()`, `set_dna()`, `dna_mut()`, `set_fitness_fn()` to `LinearChromosome`. All existing examples and test files only imported `ChromosomeT` and called these methods. After the split, the trait must be in scope for the methods to be accessible.
- **Fix:** Added `LinearChromosome` to the import block of 10 examples and 37 test files. For `test_island_migration.rs` and `test_island_nsga2.rs`, the custom chromosome types implemented `dna()` etc. inside `impl ChromosomeT` — these were split into two impl blocks (`ChromosomeT` for eval + `LinearChromosome` for flat-slice), mirroring the production chromosome split from 47-02.
- **Files modified:** 51 files across `examples/`, `tests/`

**2. [Rule 1 - Bug] `hall_of_fame.rs` doc comment had unresolved link to `ChromosomeT`**
- **Found during:** Task 2, `cargo doc --no-deps --all-features` step
- **Issue:** `HallOfFame<U: LinearChromosome>` struct had a doc comment `/// * U -- must implement [`ChromosomeT`]` which created a rustdoc warning (unresolved link). CLAUDE.md requires zero rustdoc warnings.
- **Fix:** Updated doc comment to reference `[`LinearChromosome`]`.
- **Files modified:** `src/hall_of_fame.rs`

## Deferred Items (PR 2)

1. `alleles_can_be_repeated` field on `LimitConfiguration` — read in 6 multi-obj engine files. Removal is ARCH-04 / PR 2.
2. `needs_unique_ids` field on `LimitConfiguration` — same.
3. `GaConfiguration` field encapsulation (currently `pub` fields, not `pub(crate)` + accessors) — ARCH-04 / PR 2.
4. `ChromosomeLength` enum introduction — ARCH-05 / PR 2.
5. `StoppingCriteria` struct flattening — ARCH-06 / PR 2.

## Known Stubs

None. All engine bounds are real `LinearChromosome` constraints backed by full trait implementations.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All changes are trait bound refactoring and import additions. No new packages installed.

## Self-Check: PASSED

- `src/engines/cellular/engine.rs` — FOUND with `U: LinearChromosome` bounds and `LinearChromosome` import
- `src/engines/alps/engine.rs` — FOUND with `U: LinearChromosome` bounds
- `src/engines/nsga2/mod.rs` — FOUND with `U: LinearChromosome` and `use crate::traits::{LinearChromosome, InitializationFn}`
- `src/engines/nsga3/mod.rs` — FOUND with `U: LinearChromosome` on struct, impl, and helper functions
- `src/engines/moead/mod.rs` — FOUND with `U: LinearChromosome`
- `src/engines/spea2/mod.rs` — FOUND with `U: LinearChromosome`
- `alleles_can_be_repeated` preserved in nsga2, nsga3, moead, spea2 — CONFIRMED (4 occurrences)
- Task 1 commit `e70f271` — FOUND
- Task 2 commit `7da7695` — FOUND
- `cargo test` — 982 passed, 30 ignored — CONFIRMED GREEN
- `cargo test --features serde` — 1018 passed, 30 ignored — CONFIRMED GREEN
- `cargo clippy --all-features -- -D warnings` — No issues — CONFIRMED GREEN
- `cargo check --target wasm32-unknown-unknown` — Clean — CONFIRMED GREEN
- `cargo doc --no-deps --all-features` — Zero warnings — CONFIRMED GREEN
