---
phase: 52-variable-length-chromosomes
plan: "02"
subsystem: operations/mutation
tags: [wave1, mut-06, chromosome-length, permutation-insert, insertion, deletion, variable-length]
dependency_graph:
  requires: [52-01]
  provides:
    - src/types/chromosomes/mod.rs (ChromosomeLength enum)
    - src/operations/mutation/length_mutation.rs
    - src/operations.rs (Mutation::PermutationInsert, Mutation::Insertion, Mutation::Deletion)
    - src/configuration.rs (MutationConfiguration::chromosome_length)
    - src/traits/configuration.rs (MutationConfig::with_chromosome_length)
  affects:
    - src/engines/ga.rs (factory_with_chromosome_length dispatch)
    - tests/observe/test_serde.rs (PermutationInsert rename)
    - examples/job_scheduling.rs (PermutationInsert rename)
tech_stack:
  added: []
  patterns: [enum-factory-dispatch, cow-zero-copy-mutation, optional-config-fields]
key_files:
  created:
    - src/operations/mutation/length_mutation.rs
  modified:
    - src/types/chromosomes/mod.rs
    - src/operations.rs
    - src/operations/mutation.rs
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/engines/ga.rs
    - tests/observe/test_serde.rs
    - examples/job_scheduling.rs
decisions:
  - "ChromosomeLength lives in src/types/chromosomes/mod.rs and is re-exported as genetic_algorithms::chromosomes::ChromosomeLength to match Wave 0 test stub imports"
  - "MutationConfiguration gains chromosome_length: Option<ChromosomeLength> with None default — fully backward-compatible, no breaking changes"
  - "MutationOperator::mutate signature unchanged; Insertion/Deletion dispatch happens in ga.rs via new factory_with_chromosome_length function"
  - "New gene for Insertion is sampled by cloning a random existing gene — generic approach that works with any ChromosomeT without requiring allele-set access"
  - "Mutation::Insertion called via factory_with_params (without config) returns a descriptive GaError::MutationError pointing to the correct API"
metrics:
  duration: "32m"
  completed: "2026-05-24"
  tasks_completed: 1
  files_changed: 9
---

# Phase 52 Plan 02: Wave 1 — ChromosomeLength + MUT-06 Summary

## One-liner

ChromosomeLength::Variable { min, max } enum and MUT-06 operators: Mutation::PermutationInsert (rename), Mutation::Insertion (length-grow), Mutation::Deletion (length-shrink) with full GA engine dispatch.

## What Was Built

### ChromosomeLength enum (`src/types/chromosomes/mod.rs`)

New public enum exported as `genetic_algorithms::chromosomes::ChromosomeLength`:

```rust
pub enum ChromosomeLength {
    Fixed(usize),
    Variable { min: usize, max: usize },
}
```

Satisfies the Wave 0 stub import `use genetic_algorithms::chromosomes::ChromosomeLength`. Derives `Copy`, `Clone`, `Debug`, `PartialEq`, and `serde` round-trip support.

### MUT-06: Mutation operator changes (`src/operations.rs`)

| Old variant | New variant | Behavior |
|-------------|-------------|----------|
| `Mutation::Insertion` | `Mutation::PermutationInsert` | Permutation-move: removes gene and reinserts elsewhere (length unchanged) |
| (new) | `Mutation::Insertion` | Length-grow: inserts cloned gene at random position, clamped to `max` |
| (new) | `Mutation::Deletion` | Length-shrink: removes gene at random position, clamped to `min` |

### Length mutation operators (`src/operations/mutation/length_mutation.rs`)

`length_insertion_mutation(individual, ChromosomeLength)` — grows DNA by 1:
- Returns `GaError::MutationError` for `ChromosomeLength::Fixed`
- No-op if already at `max`
- New gene cloned from a random existing gene

`length_deletion_mutation(individual, ChromosomeLength)` — shrinks DNA by 1:
- Returns `GaError::MutationError` for `ChromosomeLength::Fixed`
- No-op if already at `min`
- Removes a gene at a random position

### MutationConfiguration extension (`src/configuration.rs`)

Added `chromosome_length: Option<ChromosomeLength>` field with `None` default. Fully backward-compatible — no existing configurations break.

### MutationConfig trait + builder (`src/traits/configuration.rs`, `src/engines/ga.rs`)

`with_chromosome_length(ChromosomeLength)` added to `MutationConfig` trait and implemented for both `Ga<U>` and `GaConfiguration`.

### GA engine dispatch (`src/engines/ga.rs`)

New `factory_with_chromosome_length` function in `mutation.rs` dispatches `Insertion`/`Deletion` to the appropriate length operator. GA engine calls this function for these two variants, passing `configuration.mutation_configuration.chromosome_length`.

### Downstream updates

- `tests/observe/test_serde.rs` — renamed `Mutation::Insertion` to `Mutation::PermutationInsert`, added `Mutation::Insertion`/`Deletion` serde round-trip tests, added `chromosome_length: None` to struct literal
- `examples/job_scheduling.rs` — renamed `Mutation::Insertion` → `Mutation::PermutationInsert` in comment and method call

## Verification

All tests pass:
- `cargo test --test test_operations --test test_observe --test test_engines --test test_types` (738 passed, 2 ignored)
- `cargo test --test test_operations --test test_observe --test test_engines --test test_types --test test_constraints --test test_error --test test_population --test test_stats --test test_validators` (809 passed, 2 ignored)
- `cargo test --features serde --test test_observe` (82 passed)
- `cargo clippy` — no issues
- `cargo check --target wasm32-unknown-unknown` — passes

Wave 0 test file (`tests/test_variable_length.rs`) still fails to compile on `AlignmentStrategy` — expected behavior until Plan 52-03.

## Deviations from Plan

### Deviation 1 — Plan 02 file did not exist

The plan file `52-02-PLAN.md` was not found in `.planning/phases/52-variable-length-chromosomes/`. The execution was reconstructed from Wave 0 summary, CONTEXT.md, DISCUSSION-LOG.md, and the Wave 0 test stubs (which locked the API contract). All implementation decisions were driven by the existing locked API.

### Auto-fixed Issues

**[Rule 2 - Missing] Added serde round-trip for Insertion/Deletion in test_serde.rs**
- Found during: post-commit verification
- Issue: serde test only covered old `Insertion` (now `PermutationInsert`) — new variants not covered
- Fix: Added `Mutation::Insertion` and `Mutation::Deletion` to the serde round-trip test
- Files modified: `tests/observe/test_serde.rs`

**[Rule 3 - Blocking] Fixed missing `chromosome_length` field in MutationConfiguration struct literal**
- Found during: `cargo test --features serde`
- Issue: `test_serde.rs` creates `MutationConfiguration` with all fields explicitly — compilation error after adding new field
- Fix: Added `chromosome_length: None` to the struct literal
- Files modified: `tests/observe/test_serde.rs`

## Compilation State

The test file `tests/test_variable_length.rs` still fails to compile on:
- `genetic_algorithms::operations::AlignmentStrategy` — to be added in Wave 2 (Plan 52-03)

`ChromosomeLength` is now resolved. After Plan 03, the full Wave 0 stubs will compile and the ignored tests can be enabled.

## Self-Check

### Check created files exist

- [x] `src/operations/mutation/length_mutation.rs` — confirmed exists
- [x] `src/types/chromosomes/mod.rs` — updated with ChromosomeLength

### Check commits exist

- [x] `413f5bc` — feat(52-02): add ChromosomeLength enum and MUT-06 length mutation operators

## Self-Check: PASSED
