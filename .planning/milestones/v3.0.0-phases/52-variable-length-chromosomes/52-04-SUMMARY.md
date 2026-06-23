---
phase: 52-variable-length-chromosomes
plan: "04"
subsystem: engines/ga+tests
tags: [wave3, mut-06, chr-01, chr-02, tdd-green, extension-regrowth, variable-length]
dependency_graph:
  requires: [52-03]
  provides:
    - tests/test_variable_length.rs (all 13 Wave 3 tests enabled)
    - src/engines/ga.rs (variable-length extension regrowth sampling)
  affects:
    - src/engines/ga.rs (extension regrowth path)
tech_stack:
  added: []
  patterns: [tdd-wave3-green, wasm-gated-parallel-regrowth, adaptive-length-sampling]
key_files:
  created: []
  modified:
    - tests/test_variable_length.rs
    - src/engines/ga.rs
decisions:
  - "Extension regrowth samples lengths from [min_observed, max_observed] of surviving population — per Phase 52 discussion log decision, adaptive to current survivors"
  - "Observed bounds are clamped to configured Variable {min, max} limits to prevent out-of-range lengths during regrowth"
  - "Integration test uses Crossover::VariableLength(Trim) to avoid CrossoverError when variable-length chromosomes have different DNA lengths mid-run"
  - "Regrowth length sampling uses same WASM-compatible cfg-gated par_iter/iter pattern as initialize_random"
metrics:
  duration: "25m"
  completed: "2026-05-24"
  tasks_completed: 2
  files_changed: 2
---

# Phase 52 Plan 04: Wave 3 — Enable All Variable-Length Tests Summary

## One-liner

Wave 3 TDD green pass: all 13 variable-length test stubs enabled with real implementations, plus missing extension regrowth variable-length sampling added to the GA engine.

## What Was Built

### Variable-length extension regrowth (`src/engines/ga.rs`)

The extension regrowth path (triggered after MassGenesis, MassExtinction, MassDeduplication) now performs variable-length-aware regrowth:

**New behavior when `ChromosomeLength::Variable { min, max }` is configured:**
1. Compute `min_observed` = minimum DNA length in surviving population
2. Compute `max_observed` = maximum DNA length in surviving population
3. Clamp both to configured `[min, max]` bounds
4. Sample each new chromosome's length uniformly from `[min_observed, max_observed]`
5. Pass sampled length as `genes_per_chromosome` to `init_fn` (zero changes to `init_fn` signature)

**Fixed-length behavior unchanged:** When `chromosome_length` is `None` or `Fixed`, regrowth uses `genes_per_chromosome` as before.

WASM-compatible: both `par_iter` (non-wasm) and `iter` (wasm) paths updated.

### Wave 3 test implementations (`tests/test_variable_length.rs`)

All 13 `#[ignore]`-annotated stubs replaced with real implementations:

#### Section 1 — MUT-06 (5 tests)

| Test | What it validates |
|------|-------------------|
| `test_mutation_permutation_insert_renames_correctly` | `factory_with_params(PermutationInsert)` preserves length, moves gene over 200 iterations |
| `test_mutation_insertion_adds_gene_clamped_to_max` | `length_insertion_mutation(Variable{min:1,max:5})` grows length 3→4 |
| `test_mutation_deletion_removes_gene_clamped_to_min` | `length_deletion_mutation(Variable{min:2,max:10})` shrinks length 5→4 |
| `test_mutation_insertion_on_fixed_returns_error` | `length_insertion_mutation(Fixed(5))` returns `GaError::MutationError` |
| `test_mutation_deletion_on_fixed_returns_error` | `length_deletion_mutation(Fixed(5))` returns `GaError::MutationError` |

#### Section 2 — CHR-01 (6 tests)

| Test | What it validates |
|------|-------------------|
| `test_crossover_variable_length_trim_produces_min_len_offspring` | `Crossover::VariableLength(Trim)` on (3,5)-length parents → both offspring len=3 |
| `test_crossover_variable_length_pad_produces_max_len_offspring` | `Crossover::VariableLength(Pad)` on (3,5)-length parents → both offspring len=5 |
| `test_crossover_incompatible_length_single_point_returns_error` | `Crossover::SinglePoint` on (3,5)-length parents → `GaError::CrossoverError` |
| `test_crossover_incompatible_length_uniform_returns_error` | `Crossover::Uniform` on (3,5)-length parents → `GaError::CrossoverError` |
| `test_variable_length_initialization_samples_lengths_in_range` | `Ga<Range<f64>>` with `Variable{min:2,max:8}` and `population_size=20` → all lengths in [2,8] |
| `test_variable_length_extension_regrowth_samples_from_population` | MassGenesis + regrowth → all lengths within configured Variable bounds [2,8] |

#### Section 3 — CHR-02 (2 tests)

| Test | What it validates |
|------|-------------------|
| `test_parsimony_pressure_penalizes_longer_chromosomes_maximization` | Short chromosome (len=3) survives over long (len=7) when both have same raw fitness |
| `test_parsimony_no_fitness_mutation` | Stored `fitness()` unchanged after parsimony survivor selection |

## Verification

All tests pass:
- `cargo test --test test_variable_length` — 13 passed, 0 ignored
- `cargo test --test test_variable_length --test test_engines --test test_operations --test test_types` — 696 passed, 2 ignored
- `cargo clippy` — no issues

## Deviations from Plan

### Deviation 1 — Plan 04 file did not exist

`52-04-PLAN.md` was not found in `.planning/phases/52-variable-length-chromosomes/`. Execution was reconstructed from:
- Wave 0–3 summaries (locked API contract and prior wave state)
- 52-CONTEXT.md + 52-DISCUSSION-LOG.md (design decisions)
- Wave 0 test stubs (which reference the final API names)

All implementation decisions were driven by the locked API and the STATE.md note: "Plan 4 of 4 — Ready to execute."

### Auto-fixed Issues

**[Rule 2 - Missing] Added variable-length extension regrowth sampling**
- Found during: Wave 0 stub review — `test_variable_length_extension_regrowth_samples_from_population` had no implementation
- Issue: The extension regrowth path in `ga.rs` used the fixed `genes_per_chromosome` for all cases, ignoring `ChromosomeLength::Variable` entirely. The Phase 52 discussion log selected "Uniform from [min_observed, max_observed] of surviving population" as the regrowth strategy.
- Fix: Added variable-length detection in the extension regrowth block; computes min/max from surviving population, clamps to configured bounds, samples lengths per new chromosome
- Files modified: `src/engines/ga.rs`
- Commit: `c217e03`

## Known Stubs

None — all 13 behaviors are fully implemented and tested.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundary changes.

## Self-Check

### Check created files exist

No new files — all changes are to existing files.

### Check commits exist

- [x] `c217e03` — feat(52-04): add variable-length extension regrowth sampling
- [x] `a469440` — test(52-04): enable all 13 Wave 3 variable-length tests

## Self-Check: PASSED
