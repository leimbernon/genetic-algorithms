---
phase: 52-variable-length-chromosomes
plan: "03"
subsystem: operations/crossover+survivor+engines
tags: [wave2, chr-01, chr-02, variable-length-crossover, alignment-strategy, parsimony-pressure, survivor-config]
dependency_graph:
  requires: [52-02]
  provides:
    - src/operations.rs (AlignmentStrategy enum, Crossover::VariableLength variant)
    - src/operations/crossover/variable_length.rs
    - src/operations/survivor/parsimony.rs
    - src/configuration.rs (GaConfiguration.length_penalty)
    - src/traits/configuration.rs (SurvivorConfig trait)
    - src/traits.rs (SurvivorConfig re-export)
    - src/engines/ga.rs (variable-length init + parsimony wiring)
  affects:
    - src/operations/crossover.rs (VariableLength dispatch)
    - src/operations/survivor.rs (parsimony module)
    - tests/observe/test_serde.rs (AlignmentStrategy + VariableLength serde coverage)
tech_stack:
  added: []
  patterns: [enum-factory-dispatch, temporary-fitness-adjustment, wasm-gated-parallel-init]
key_files:
  created:
    - src/operations/crossover/variable_length.rs
    - src/operations/survivor/parsimony.rs
  modified:
    - src/operations.rs
    - src/operations/crossover.rs
    - src/operations/survivor.rs
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/traits.rs
    - src/engines/ga.rs
    - tests/observe/test_serde.rs
decisions:
  - "AlignmentStrategy::Trim aligns both parents to min(len_a, len_b) before single-point crossover — offspring length equals the shorter parent"
  - "AlignmentStrategy::Pad extends shorter parent by cloning random genes from its own DNA (consistent with Insertion mutation allele sampling)"
  - "Parsimony pressure adjusts fitness temporarily in apply_parsimony_pressure: adjust → sort → truncate → restore; stored fitness() never mutated"
  - "SurvivorConfig trait added with with_length_penalty(f64) builder; wired into ConfigurationT supertrait"
  - "Variable-length initialization samples random lengths from [min, max] and passes each as genes_per_chromosome to init_fn — zero changes to init_fn signature"
  - "WASM-compatible: variable-length initialization has cfg-gated par_iter / iter paths"
metrics:
  duration: "45m"
  completed: "2026-05-24"
  tasks_completed: 1
  files_changed: 10
---

# Phase 52 Plan 03: Wave 2 — CHR-01 VariableLength Crossover + CHR-02 Parsimony Pressure Summary

## One-liner

AlignmentStrategy enum, Crossover::VariableLength(AlignmentStrategy) with Trim/Pad alignment, parsimony pressure survivor config, and variable-length initialization sampling in the GA engine.

## What Was Built

### AlignmentStrategy enum (`src/operations.rs`)

New public enum `AlignmentStrategy` exported as `genetic_algorithms::operations::AlignmentStrategy`:

```rust
pub enum AlignmentStrategy {
    Trim,  // both parents truncated to min(len_a, len_b)
    Pad,   // shorter parent padded to max(len_a, len_b)
}
```

Derives `Copy`, `Clone`, `Debug`, `PartialEq`, and serde round-trip support.

### Crossover::VariableLength variant (`src/operations.rs`)

New variant added to the `Crossover` enum:

```rust
Crossover::VariableLength(AlignmentStrategy)
```

Dispatches to `variable_length_crossover` in the factory. Satisfies the Wave 0 stub import `use genetic_algorithms::operations::{AlignmentStrategy, Crossover}`.

### VariableLength crossover (`src/operations/crossover/variable_length.rs`)

`variable_length_crossover(parent_1, parent_2, AlignmentStrategy)`:
- `Trim`: both parents sliced to `min(len_a, len_b)` — no padding needed
- `Pad`: shorter parent extended by cloning random genes from its own DNA until length equals `max(len_a, len_b)`
- After alignment: single-point crossover at a random point in `[1, aligned_len)`
- Returns `GaError::CrossoverError` if both parents are empty after alignment

This is the implementation selected in the Phase 52 discussion log: "Fixed single-point within aligned region" with "Random from alleles" padding (implemented as random clone from existing DNA, consistent with `Mutation::Insertion`).

### Parsimony pressure (`src/operations/survivor/parsimony.rs`)

`apply_parsimony_pressure(survivor, chromosomes, population_size, limit_config, length_penalty)`:

1. Adjusts each chromosome's fitness temporarily: `effective = fitness ∓ (penalty × len)`
   - Maximization: subtract (longer chromosomes appear less fit)
   - Minimization: add (longer chromosomes appear less fit)
2. Calls standard `survivor::factory` with adjusted fitness
3. Restores original fitness for all survivors by reversing the adjustment

The stored `fitness()` value is **never** permanently mutated.

### GaConfiguration.length_penalty (`src/configuration.rs`)

Added `length_penalty: Option<f64>` field to `GaConfiguration`:
- Default: `None` (parsimony disabled, zero overhead)
- Fully backward-compatible — no existing configurations break

### SurvivorConfig trait + builder (`src/traits/configuration.rs`, `src/traits.rs`)

`SurvivorConfig` trait with `with_length_penalty(f64)` builder:
- Implemented on `GaConfiguration` and `Ga<U>`
- Added to `ConfigurationT` supertrait

### Variable-length initialization (`src/engines/ga.rs`)

`initialize_random` now checks `mutation_configuration.chromosome_length`:
- `ChromosomeLength::Variable { min, max }` → samples random length per chromosome from `[min, max]`
- All other cases → unchanged (fixed `genes_per_chromosome` as before)
- WASM-compatible: `par_iter` path gated with `#[cfg(not(target_arch = "wasm32"))]`

### GA engine parsimony wiring (`src/engines/ga.rs`)

Survivor call in the generation loop:
```rust
if let Some(penalty) = self.configuration.length_penalty {
    survivor::apply_parsimony_pressure(..., penalty)?;
} else {
    survivor::factory(...)?;
}
```

### Downstream updates

`tests/observe/test_serde.rs`:
- Added `Crossover::VariableLength(AlignmentStrategy::Trim/Pad)` to `serde_crossover_enum` test
- Added new `serde_alignment_strategy_enum` test
- Added `length_penalty: None` to `GaConfiguration` struct literal in `serde_ga_configuration_with_values`

## Verification

All tests pass:
- `cargo test --test test_operations` — 320 passed
- `cargo test --test test_engines` — 325 passed, 2 ignored
- `cargo test --test test_types` — 38 passed
- `cargo test --test test_variable_length` — 0 passed, 13 ignored (all stubs compile)
- `cargo test --features serde --test test_observe -- serde_crossover_enum serde_alignment_strategy_enum serde_ga_configuration` — 4 passed
- `cargo clippy` — no issues
- `cargo check --target wasm32-unknown-unknown` — passes

Wave 0 test file (`tests/test_variable_length.rs`) now compiles fully with no missing symbols. All 13 stubs are ready to enable in Wave 3 (Plan 52-04).

## Deviations from Plan

### Deviation 1 — Plan 03 file did not exist

`52-03-PLAN.md` was not found. Execution was reconstructed from:
- Wave 0 test stubs (API contract)
- 52-CONTEXT.md + 52-DISCUSSION-LOG.md (design decisions)
- 52-01-SUMMARY.md + 52-02-SUMMARY.md (prior wave state)

All implementation decisions were driven by the locked API and user choices in the discussion log.

### Auto-fixed Issues

**[Rule 2 - Missing] Added serde round-trip coverage for AlignmentStrategy and Crossover::VariableLength**
- Found during: post-implementation review
- Issue: serde test `serde_crossover_enum` did not cover the new `VariableLength` variant; `AlignmentStrategy` had no serde test
- Fix: Added `VariableLength(AlignmentStrategy::Trim/Pad)` to existing test; added new `serde_alignment_strategy_enum` test; added `length_penalty: None` to `GaConfiguration` struct literal
- Files modified: `tests/observe/test_serde.rs`

## Known Stubs

None — all behaviors are fully wired.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundary changes.

## Self-Check

### Check created files exist

- [x] `src/operations/crossover/variable_length.rs` — confirmed exists
- [x] `src/operations/survivor/parsimony.rs` — confirmed exists

### Check commits exist

- [x] `e629b50` — feat(52-03): add VariableLength crossover, AlignmentStrategy, and parsimony pressure (CHR-01, CHR-02)

## Self-Check: PASSED
