---
phase: 47
plan: 06
subsystem: caller-migration
tags:
  - rust
  - caller-migration
  - examples
  - tests
  - phase-gate
dependency_graph:
  requires:
    - 47-05
  provides:
    - pr2-gate-green
    - arch04-satisfied
    - arch05-satisfied
    - arch06-satisfied
  affects:
    - 47-07
tech_stack:
  added: []
  patterns:
    - ChromosomeLength::Fixed(n) builder pattern (callers)
    - flat stopping builders (with_stagnation_limit/convergence_threshold/max_duration_secs)
    - 2-arg InitializationFn at all call sites
key_files:
  modified:
    - src/engines/ga.rs
    - src/engines/nsga2/mod.rs
    - src/engines/nsga3/mod.rs
    - src/engines/moead/mod.rs
    - src/engines/spea2/mod.rs
    - src/engines/sms_emoa/mod.rs
    - src/engines/ibea/mod.rs
    - src/engines/island/mod.rs
    - src/engines/island/nsga2.rs
    - src/validators/generic_validator.rs
    - examples/ (all 19 examples)
    - tests/ (all affected test files)
decisions:
  - D-06 (alleles_can_be_repeated/needs_unique_ids removed without replacement — engine + caller migration complete)
  - D-08 (StoppingCriteria flat fields — all callers migrated)
  - D-09 (GaConfiguration pub(crate) — external callers now use builders/accessors)
  - T-47-16 (Variable ChromosomeLength → explicit error in all engines)
  - job_scheduling permutation initializer: Phase 47 compile-fix using manual shuffle; Phase 48 migrates to UniqueChromosome<i32>
metrics:
  completed_date: "2026-05-21"
  tasks_completed: 2
  files_changed: ~45
---

# Phase 47 Plan 06: Caller Migration + PR 2 Gate Summary

Migrate every remaining caller in src/engines/ (multi-objective + alt-metaheuristic engines), every example, and every test that previously read removed fields. Run the PR 2 phase verification gate.

## Tasks Completed

| Task | Commit | Description |
|------|--------|-------------|
| Task 1: Engine migration | 6786307 | Migrate all 8 engine files to ChromosomeLength + drop removed fields |
| Task 2: Examples + tests + PR gate | 39531aa | Migrate ~30 example/test files; all 5 gates green |
| Cleanup: unused imports | HEAD | Remove Range/TypeId from generic_validator |

## What Was Built

### Task 1 — Engine Code (src/engines/*)

Replaced `genes_per_chromosome` + `alleles_can_be_repeated` / `needs_unique_ids` reads in:
- `src/engines/ga.rs` — `initialize_random`, `initialize_with_seeds`, extension regrow block
- `src/engines/nsga2/mod.rs`, `nsga3/mod.rs`, `moead/mod.rs`, `spea2/mod.rs`, `sms_emoa/mod.rs`, `ibea/mod.rs` — `initialize_population`
- `src/engines/island/mod.rs`, `island/nsga2.rs` — `initialize_islands`

Pattern used everywhere:
```rust
let length = match self.ga_config.limit_configuration.chromosome_length {
    crate::chromosomes::ChromosomeLength::Fixed(n) => n,
    crate::chromosomes::ChromosomeLength::Variable { .. } => {
        return Err(GaError::InvalidXConfiguration(
            "ChromosomeLength::Variable is not yet supported (Phase 52). Use ChromosomeLength::Fixed.".into(),
        ));
    }
};
```

Updated all `with_initialization_fn` bounds from `Fn(usize, Option<&[G]>, Option<bool>) -> Vec<G>` to `Fn(usize, Option<&[G]>) -> Vec<G>`.

Removed `alleles_can_be_repeated` guard block from `generic_validator.rs` (section 2.4). Cleaned unused `Range` and `TypeId` imports.

`cargo check --lib` and `cargo check --target wasm32-unknown-unknown --lib` both GREEN.

### Task 2 — Examples + Tests

All 19 examples migrated:
- `with_genes_per_chromosome(n)` → `with_chromosome_length(ChromosomeLength::Fixed(n))`
- Direct `limit_configuration.genes_per_chromosome = N` → builder chain
- Direct `limit_configuration.alleles_can_be_repeated = true` → removed
- 3-arg closures `|n, _, _|` → 2-arg `|n, _|`
- `range_random_initialization(n, alleles, Some(bool))` → `range_random_initialization(n, alleles)`
- Multi-obj examples (nsga2, nsga3, moead, spea2, sms_emoa, ibea) direct field access → builder chain
- `island_model.rs` full builder chain conversion (limit/mutation/crossover/selection/survivor)

`examples/job_scheduling.rs` — Phase 47 compile-fix: replaced 3-arg closure with explicit permutation shuffle (Phase 48 migrates to `UniqueChromosome<i32>`).

All test files migrated:
- `tests/engines/test_ga.rs` — with_genes_per_chromosome, with_stopping_criteria, with_needs_unique_ids, with_alleles_can_be_repeated
- `tests/wasm_smoke.rs` — with_genes_per_chromosome, with_stopping_criteria
- `tests/validators/test_generic_validator.rs` — private field access replaced with builders; alleles_can_be_repeated tests removed
- `tests/operations/test_mutation_cauchy_levy_uniform.rs` — `ga.configuration().mutation().cauchy_scale` accessor pattern
- `tests/initializers/test_initializers.rs` — 2-arg initializer calls
- ~20 other test files — with_genes_per_chromosome + import additions

## PR 2 Phase Verification Gate

All 5 gates GREEN:
- `cargo test` ✓
- `cargo test --features serde` ✓
- `cargo clippy --all-features -- -D warnings` ✓
- `cargo check --target wasm32-unknown-unknown` ✓
- `cargo doc --no-deps --all-features` ✓

## Deferred to 47-07 (PR 3)

- Reporter-related cleanup (Reporter trait, SimpleReporter, DurationReporter, NoopReporter removal)
- MIGRATION.md creation for Cargo.toml include array

## PR 2 Boundary Contents

PR 2 = plans 47-04 + 47-05 + 47-06:
- ARCH-04: LimitConfiguration cleanup (ChromosomeLength, removed genes_per_chromosome/alleles_can_be_repeated/needs_unique_ids)
- ARCH-05: StoppingCriteria dissolution (flat fields on GaConfiguration)
- ARCH-06: GaConfiguration encapsulation (pub(crate) fields + sub-struct accessors)

## Self-Check: PASSED

- No engine file contains `alleles_can_be_repeated`, `needs_unique_ids`, or `genes_per_chromosome` field-access syntax
- All 19 examples contain `ChromosomeLength::Fixed` usage
- PR 2 verification gate GREEN
- WASM gate GREEN
