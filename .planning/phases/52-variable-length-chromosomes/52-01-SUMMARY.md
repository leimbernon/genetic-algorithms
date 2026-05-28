---
phase: 52-variable-length-chromosomes
plan: "01"
subsystem: tests
tags: [wave0, tdd, stubs, variable-length, mutation, crossover, parsimony]
dependency_graph:
  requires: []
  provides: [tests/test_variable_length.rs]
  affects: []
tech_stack:
  added: []
  patterns: [nyquist-compliance, wave0-stubs, ignore-annotated-stubs]
key_files:
  created:
    - tests/test_variable_length.rs
  modified: []
decisions:
  - "Wave 0 stubs reference post-Wave 1-3 API names (PermutationInsert, Insertion, Deletion, VariableLength, AlignmentStrategy, ChromosomeLength, length_penalty) to lock the interface contract before implementation"
  - "All imports kept even though they generate unused-import warnings — they are necessary for enabling stubs after implementation"
  - "File intentionally fails to compile on missing AlignmentStrategy and ChromosomeLength — this is expected Wave 0 behavior"
metrics:
  duration: "5m"
  completed: "2026-05-24"
  tasks_completed: 1
  files_changed: 1
---

# Phase 52 Plan 01: Wave 0 Test Stubs Summary

## One-liner

Wave 0 Nyquist stubs for all 13 Phase 52 behaviors covering MUT-06 (mutation rename + length operators), CHR-01 (VariableLength crossover + guards + init), and CHR-02 (parsimony pressure).

## What Was Built

Created `tests/test_variable_length.rs` with 13 `#[ignore]`-annotated test stubs covering all Phase 52 requirements. The stubs reference the final post-Wave 1-3 API names, ensuring the interface contract is locked before implementation begins.

### Section 1 — MUT-06 (5 stubs)

| Stub | Validates |
|------|-----------|
| `test_mutation_permutation_insert_renames_correctly` | `Mutation::PermutationInsert` moves gene without changing length |
| `test_mutation_insertion_adds_gene_clamped_to_max` | `Mutation::Insertion` grows DNA length, clamped to max |
| `test_mutation_deletion_removes_gene_clamped_to_min` | `Mutation::Deletion` shrinks DNA length, clamped to min |
| `test_mutation_insertion_on_fixed_returns_error` | `Mutation::Insertion` returns `GaError::MutationError` for Fixed config |
| `test_mutation_deletion_on_fixed_returns_error` | `Mutation::Deletion` returns `GaError::MutationError` for Fixed config |

### Section 2 — CHR-01 (6 stubs)

| Stub | Validates |
|------|-----------|
| `test_crossover_variable_length_trim_produces_min_len_offspring` | Trim alignment yields offspring of min(len_a, len_b) |
| `test_crossover_variable_length_pad_produces_max_len_offspring` | Pad alignment yields offspring of max(len_a, len_b) |
| `test_crossover_incompatible_length_single_point_returns_error` | `Crossover::SinglePoint` rejects unequal-length parents |
| `test_crossover_incompatible_length_uniform_returns_error` | `Crossover::Uniform` rejects unequal-length parents |
| `test_variable_length_initialization_samples_lengths_in_range` | Init samples lengths in [min, max] for Variable config |
| `test_variable_length_extension_regrowth_samples_from_population` | Regrowth lengths within [min_obs, max_obs] of survivors |

### Section 3 — CHR-02 (2 stubs)

| Stub | Validates |
|------|-----------|
| `test_parsimony_pressure_penalizes_longer_chromosomes_maximization` | Shorter chromosome survives when length_penalty is set |
| `test_parsimony_no_fitness_mutation` | Stored `fitness()` value unchanged after parsimony survivor selection |

## Deviations from Plan

None — plan executed exactly as written.

## Compilation State

The file fails to compile on two missing symbols:

- `genetic_algorithms::chromosomes::ChromosomeLength` — added in Wave 1 (Plan 52-02)
- `genetic_algorithms::operations::AlignmentStrategy` — added in Wave 2 (Plan 52-03)

All other imports (`GaError`, `ProblemSolving`, `Ga`, `ConfigurationT`, `Crossover`, `Mutation`, `Survivor`) resolve correctly. The compilation failures are expected Wave 0 behavior per the plan's Nyquist compliance goal.

## Self-Check

### Check created files exist

- [x] `tests/test_variable_length.rs` — confirmed exists

### Check commits exist

- [x] `df95c7e` — test(52-01): add Wave 0 test stubs for variable-length chromosomes

## Self-Check: PASSED
