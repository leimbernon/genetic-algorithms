---
phase: 47-architecture-audit-chromosomet-split
plan: 02
subsystem: traits, operators, chromosomes
tags:
  - rust
  - traits
  - bound-change
  - refactor
  - breaking-change

dependency_graph:
  requires:
    - "47-01 (ChromosomeT split — new LinearChromosome trait)"
  provides:
    - "Binary, Range<T>, ListChromosome<T> implement both ChromosomeT and LinearChromosome (ARCH-02)"
    - "All DNA-access operators use U: LinearChromosome bound (ARCH-02)"
    - "ValueMutable: LinearChromosome supertrait (ARCH-02)"
    - "Wave 0 tests from 47-01 now GREEN"
  affects:
    - "ga.rs, island, nsga2 orchestrators (47-03 will do deeper refactoring)"
    - "All user code that implements ChromosomeT without LinearChromosome"

tech_stack:
  added: []
  patterns:
    - "Two-impl-block split: impl ChromosomeT (eval) + impl LinearChromosome (flat-slice)"
    - "Supertrait bound propagation across operator dispatch layer"
    - "Selective ChromosomeT retention for fitness-only operators"

key_files:
  created: []
  modified:
    - src/types/chromosomes/binary.rs
    - src/types/chromosomes/range.rs
    - src/types/chromosomes/list.rs
    - tests/structures.rs
    - src/operations/mutation.rs
    - src/operations/crossover.rs
    - src/operations/crossover/single_point.rs
    - src/operations/crossover/multipoint.rs
    - src/operations/crossover/uniform_crossover.rs
    - src/operations/crossover/cycle.rs
    - src/operations/crossover/order.rs
    - src/operations/crossover/pmx.rs
    - src/operations/crossover/edge_recombination.rs
    - src/operations/crossover/arithmetic.rs
    - src/operations/crossover/blend_alpha.rs
    - src/operations/crossover/sbx.rs
    - src/operations/crossover/clone.rs
    - src/operations/crossover/rejuvenate.rs
    - src/operations/mutation/swap.rs
    - src/operations/mutation/inversion.rs
    - src/operations/mutation/scramble.rs
    - src/operations/mutation/insertion.rs
    - src/operations/mutation/bit_flip.rs
    - src/operations/mutation/differential.rs
    - src/operations/mutation/cauchy.rs
    - src/operations/mutation/levy_flight.rs
    - src/operations/mutation/gaussian.rs
    - src/operations/mutation/polynomial.rs
    - src/operations/mutation/uniform.rs
    - src/operations/mutation/non_uniform.rs
    - src/operations/mutation/creep.rs
    - src/operations/mutation/value.rs
    - src/operations/mutation/list_value.rs
    - src/operations/survivor.rs
    - src/operations/survivor/deterministic_crowding.rs
    - src/operations/extension/mod.rs
    - src/operations/extension/mass_deduplication.rs
    - src/operations/extension/mass_degeneration.rs
    - src/operations/local_search.rs
    - src/traits/operators.rs
    - src/traits/common.rs
    - src/traits/linear_chromosome.rs
    - src/validators/generic_validator.rs
    - src/validators/validator_factory.rs
    - src/hall_of_fame.rs
    - src/engines/ga.rs
    - src/engines/island/mod.rs
    - src/engines/island/nsga2.rs
    - src/engines/de/engine.rs
    - src/engines/de/mutation.rs
    - src/engines/ibea/mod.rs
    - src/engines/scatter/engine.rs
    - src/engines/sms_emoa/mod.rs

decisions:
  - "SurvivorOperator::select_survivors and ExtensionOperator::apply_extension trait methods upgraded to U: LinearChromosome (DeterministicCrowding and MassDeduplication require dna() — forcing the dispatch method to be LinearChromosome)"
  - "Engine struct bounds (ga.rs, island, nsga2, de, ibea, scatter, sms_emoa) upgraded to LinearChromosome in this plan — not deferred to 47-03 — because HallOfFame<U: LinearChromosome> forced the issue at struct definition level"
  - "Selection operators stay at U: ChromosomeT; all selection/*.rs unchanged"
  - "Fitness-only survivor/extension functions stay at ChromosomeT: age.rs, fitness.rs, mu_plus_lambda.rs, mu_comma_lambda.rs, mass_genesis.rs, mass_extinction.rs"
  - "aga_probability and compute_cardinality in mutation.rs stay at ChromosomeT (fitness/cardinality only)"
  - "linear_chromosome.rs needed GeneT import for default new_gene() impl"

metrics:
  duration: "~30 minutes execution"
  completed_date: "2026-05-20"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 53
---

# Phase 47 Plan 02: Operator Layer Migration to LinearChromosome — Summary

**One-liner:** Migrated all three built-in chromosome types to two-impl-block form (ChromosomeT + LinearChromosome) and updated 50+ operator/engine files to use `U: LinearChromosome` where they access DNA, making Wave 0 tests GREEN and restoring clean library compilation.

## What Was Built

### Task 1: Split built-in chromosome implementors

Split four chromosome types from a single `impl ChromosomeT` block (with all methods) into two separated blocks:

- `impl ChromosomeT for X` — fitness, set_fitness, calculate_fitness, age, set_age (evaluation contract)
- `impl LinearChromosome for X` — dna, dna_mut, set_dna, set_fitness_fn (flat-slice contract)

Applied to: `Binary`, `Range<T>`, `ListChromosome<T>`, and `tests/structures.rs::Chromosome`.

The `fn default(mut self) -> Self` instance helper was absent from all implementors (it was never present — the old ChromosomeT trait had it as a method, but the implementors used `Default::default()`). No `.default()` call sites needed migration.

Added `use crate::traits::LinearChromosome` to all four files' imports.

**Post-split state:** Wave 0 tests (`test_chromosomet_core`, `test_linear_chromosome`) could structurally compile but needed the operator layer fixed too.

### Task 2: Mechanical bound change across operator layer + ValueMutable supertrait upgrade

**Step A: Crossover operator files (12 files with LinearChromosome)**

All 9 crossover function files with `U: ChromosomeT` bound updated to `U: LinearChromosome` (single_point, multipoint, uniform, cycle, order, pmx, edge_recombination, clone, rejuvenate). Additionally, arithmetic.rs, blend_alpha.rs, sbx.rs had their `ChromosomeT` import changed (they use `RangeChromosome<T>` directly but needed the `LinearChromosome` trait in scope for the type to work).

`crossover.rs` dispatch module and `CrossoverOperator` trait method updated to `U: LinearChromosome`.

**Step B: Mutation operator files (15 files)**

- Files with `U: ChromosomeT` bound: swap, inversion, scramble, insertion, bit_flip, differential — all updated to `U: LinearChromosome`
- Files with only ChromosomeT import (using concrete RangeChromosome<T>/ListChromosome<T>): cauchy, levy_flight, gaussian, polynomial, uniform, non_uniform, creep, value, list_value — imports updated to LinearChromosome

`src/operations/mutation.rs`: `ValueMutable` trait supertrait changed from `ChromosomeT` to `LinearChromosome`. Factory functions updated to `U: LinearChromosome + ValueMutable + 'static`. `MutationOperator` trait updated. `aga_probability` and `compute_cardinality` stay at `ChromosomeT`.

**Step C: Survivor and extension dispatch**

- `deterministic_crowding.rs`: `U: LinearChromosome` (calls `dna()` for Hamming distance)
- `mass_deduplication.rs`: `U: LinearChromosome` (calls `dna()` for gene ID hash)
- `mass_degeneration.rs`: `U: LinearChromosome` (calls `swap()` which needs `dna_mut()`)
- `survivor.rs` dispatch and `SurvivorOperator` trait: `U: LinearChromosome`
- `extension/mod.rs` dispatch and `ExtensionOperator` trait: `U: LinearChromosome`

Files that STAY at ChromosomeT: `age.rs`, `fitness.rs`, `mu_plus_lambda.rs`, `mu_comma_lambda.rs`, `mass_genesis.rs`, `mass_extinction.rs`, all `selection/*.rs`.

**Step D: Supporting layer**

- `traits/operators.rs`: CrossoverOperator, SurvivorOperator, ExtensionOperator, LocalSearchOperator traits updated
- `traits/common.rs`: initialize_chromosomes functions updated (call `set_dna`, `set_fitness_fn`)
- `traits/linear_chromosome.rs`: added `GeneT` import for `new_gene()` default impl
- `validators/generic_validator.rs`, `validators/validator_factory.rs`: LinearChromosome (call `dna()`)
- `hall_of_fame.rs`: LinearChromosome struct bound (calls `dna()` for genotypic deduplication)
- Engine files (ga.rs, island, nsga2, de, ibea, scatter, sms_emoa): LinearChromosome struct/impl bounds

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical import] linear_chromosome.rs missing GeneT import**
- **Found during:** Task 2, Step A — cargo check revealed E0599 `new` not found in `<Self as ChromosomeT>::Gene`
- **Issue:** `LinearChromosome::new_gene()` default impl calls `Self::Gene::new()`. The `GeneT` trait (which defines `new()`) was not imported.
- **Fix:** Added `GeneT` to `use crate::traits::{ChromosomeT, GeneT}` in `linear_chromosome.rs`
- **Files modified:** `src/traits/linear_chromosome.rs`

**2. [Rule 3 - Blocking] Engine struct bounds needed LinearChromosome upgrade (partial scope expansion)**
- **Found during:** Task 2, Step D — `HallOfFame<U: LinearChromosome>` was used in `Ga<U: ChromosomeT>` struct definition
- **Issue:** The plan deferred engine orchestrator updates to 47-03, but `HallOfFame` struct bound required `LinearChromosome` at struct definition level in ga.rs, island, nsga2, etc.
- **Fix:** Applied `U: ChromosomeT -> U: LinearChromosome` to all 8 engine files' struct and impl bounds. This is a superset of what 47-03 would have done anyway — the mechanical change is correct and necessary.
- **Files modified:** `src/engines/ga.rs`, `src/engines/island/mod.rs`, `src/engines/island/nsga2.rs`, `src/engines/de/engine.rs`, `src/engines/de/mutation.rs`, `src/engines/ibea/mod.rs`, `src/engines/scatter/engine.rs`, `src/engines/sms_emoa/mod.rs`

**3. [Rule 2 - Missing functionality] SurvivorOperator and ExtensionOperator trait methods needed LinearChromosome**
- **Found during:** Task 2, Step C — dispatch functions required `LinearChromosome` but trait methods said `ChromosomeT`
- **Issue:** Plan text said "fitness-only survivors stay at ChromosomeT" but the enum dispatch includes DeterministicCrowding (needs dna()) and MassDeduplication (needs dna()), which forces the enum's `impl SurvivorOperator` to require `LinearChromosome`.
- **Fix:** Updated `SurvivorOperator::select_survivors` and `ExtensionOperator::apply_extension` trait method bounds to `U: LinearChromosome`. Individual fitness-only functions (age.rs, fitness.rs, etc.) correctly remain at `U: ChromosomeT`.
- **Files modified:** `src/traits/operators.rs`, `src/operations/survivor.rs`, `src/operations/extension/mod.rs`

**4. [Rule 1 - Bug] de/mutation.rs had `impl ChromosomeT<Gene = G>` missed by sed**
- **Found during:** Task 2, final cargo check — E0405 ChromosomeT not in scope
- **Issue:** The file used `archive: Option<&[impl ChromosomeT<Gene = G>]>` syntax (not `U: ChromosomeT`) — missed by the `s/U: ChromosomeT/` sed pattern. The function calls `.dna()` on archive items.
- **Fix:** Changed to `impl LinearChromosome<Gene = G>`
- **Files modified:** `src/engines/de/mutation.rs`

**5. [Rule 2 - Missing import] local_search.rs test module missing trait imports**
- **Found during:** `cargo test --lib` — `set_fitness` and `fitness()` not found in test scope
- **Issue:** The inline `#[cfg(test)]` module used `use super::*` which brought in `LinearChromosome` but not `ChromosomeT`. Test code called `set_fitness` and `fitness()` (on ChromosomeT) without importing it.
- **Fix:** Added `use crate::traits::{ChromosomeT, LinearChromosome};` to the test module.
- **Files modified:** `src/operations/local_search.rs`

## Known Stubs

None. All implementations are complete.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes were introduced. All changes are pure trait bound refactoring.

## Self-Check: PASSED

- `src/types/chromosomes/binary.rs` — FOUND with `impl ChromosomeT for Binary` and `impl LinearChromosome for Binary`
- `src/types/chromosomes/range.rs` — FOUND with both impl blocks
- `src/types/chromosomes/list.rs` — FOUND with both impl blocks
- `tests/structures.rs` — FOUND with both impl blocks
- `src/operations/mutation.rs` — FOUND `pub trait ValueMutable: LinearChromosome`
- Commit `9c96446` (Task 1 — implementor split) — FOUND
- Commit `fdfefb4` (Task 2 — operator bound changes) — FOUND
- `cargo check --lib` — CONFIRMED GREEN (0 errors, 0 warnings)
- `cargo check --target wasm32-unknown-unknown --lib` — CONFIRMED GREEN
- `cargo test --test test_chromosomet_core` — CONFIRMED 2 passed
- `cargo test --test test_linear_chromosome` — CONFIRMED 4 passed
- `cargo test --lib` — CONFIRMED 56 passed
- All selection/*.rs contain `U: ChromosomeT` and NO `U: LinearChromosome` — CONFIRMED
- `age.rs`, `fitness.rs`, `mu_plus_lambda.rs`, `mass_genesis.rs` stay at ChromosomeT — CONFIRMED
- `deterministic_crowding.rs`, `mass_deduplication.rs` contain `U: LinearChromosome` — CONFIRMED
