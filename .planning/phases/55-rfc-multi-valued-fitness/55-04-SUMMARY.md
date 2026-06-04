---
phase: 55-rfc-multi-valued-fitness
plan: 04
subsystem: multi-objective-engines
tags:
  - rust
  - multi-objective
  - nsga2
  - nsga3
  - moead
  - vector-fitness
  - breaking-change
requires:
  - 55-01
  - 55-02
provides:
  - nsga2-vector-fitness
  - nsga3-vector-fitness
  - moead-vector-fitness
affects:
  - src/engines/nsga2/mod.rs
  - src/engines/nsga3/mod.rs
  - src/engines/moead/mod.rs
tech-stack:
  added: []
  patterns:
    - VectorFitness trait bound on all three MO engine impl blocks
    - Runtime objective-count validation via fitness_values().len()
    - calculate_fitness() + fitness_values().to_vec() replaces Arc<ObjectiveFn> closures
key-files:
  created: []
  modified:
    - src/engines/nsga2/mod.rs
    - src/engines/nsga3/mod.rs
    - src/engines/moead/mod.rs
key-decisions:
  - Add VectorFitness to struct where clause (not just run() impl block) to make the bound visible at construction time
  - Remove ObjectiveFn import from nsga3/moead entirely since neither has constraint_fns
  - In MOEA/D sub-problem loop, re-bind offspring_chrom as mut for calculate_fitness() call rather than using WASM-split (inline, not parallel)
requirements-completed:
  - TRAITS-01
duration: ~15 min
completed: 2026-05-30
---

# Phase 55 Plan 04: MO Engine VectorFitness Migration Summary

Migrated NSGA-II, NSGA-III, and MOEA/D to remove `objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>` and adopt `U: VectorFitness`, populating `ParetoIndividual.objectives` from `chrom.fitness_values().to_vec()` after `chrom.calculate_fitness()`.

## Duration

- Start: 2026-05-30T08:44:30Z
- End: 2026-05-30T09:00Z (approx)
- Total: ~15 min
- Tasks completed: 2/2
- Files modified: 3

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Migrate src/engines/nsga2/mod.rs | 2d5b12d |
| 2 | Migrate src/engines/nsga3/mod.rs and src/engines/moead/mod.rs | 4535182 |

## What Was Built

**NSGA-II (`src/engines/nsga2/mod.rs`):**
- Removed `pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>` struct field
- Removed `objective_fns: Vec::new()` from constructor
- Deleted `with_objective_fns(...)` builder method
- Removed `objective_fns.len() != num_objectives` check from `validate()`
- Added `VectorFitness` to `use crate::traits::{...}` import
- Added `VectorFitness` bound to struct, first impl block, and `run()` impl block
- Inserted runtime objective-count guard in `run()` after `initialize_population()`
- Replaced closure-driven objectives in `initialize_population()` and `create_offspring()` with `chrom.calculate_fitness(); chrom.fitness_values().to_vec()`
- Preserved WASM split (`par_iter`/`iter`) in both evaluation sites
- Kept `constraint_fns` field and `ObjectiveFn` re-export unchanged

**NSGA-III (`src/engines/nsga3/mod.rs`):**
- Same 10-step migration applied
- Removed `ObjectiveFn` import entirely (no `constraint_fns` field)
- Removed `objective_fns.len()` check from both `validate()` and `validate_and_get_ref_points()`
- VectorFitness bound on struct + 2 impl blocks
- Runtime guard uses `GaError::InvalidNsga3Configuration`

**MOEA/D (`src/engines/moead/mod.rs`):**
- Same 10-step migration applied
- Removed `ObjectiveFn` import entirely (no `constraint_fns` field)
- Removed `objective_fns.len()` check from both `validate()` and `validate_and_get_weight_vectors()`
- Sub-problem loop (step 5c): replaced `self.objective_fns.iter().map(|f| f(...)).collect()` with inline `offspring_chrom.calculate_fitness(); offspring_chrom.fitness_values().to_vec()` (no WASM split needed — single-threaded inner loop)
- Runtime guard uses `GaError::InvalidMoeaDConfiguration`

## Acceptance Criteria Verification

### NSGA-II
- `grep -c "self.objective_fns"` → 0 (PASS)
- `grep -c "with_objective_fns"` → 0 (PASS)
- `grep -cE "pub\s+objective_fns"` → 0 (PASS)
- `grep -c "VectorFitness"` → 4 (PASS, >= 3)
- `grep -c "fitness_values().to_vec()"` → 4 (PASS, >= 2)
- `grep -c "Expected .* objectives from fitness_values"` → 1 (PASS)
- WASM cfg count: 5 (unchanged, PASS)
- `constraint_fns` count: 15 (unchanged, PASS)

### NSGA-III
- `grep -c "self.objective_fns"` → 0 (PASS)
- `grep -c "with_objective_fns"` → 0 (PASS)
- `grep -cE "pub\s+objective_fns"` → 0 (PASS)
- `grep -c "VectorFitness"` → 4 (PASS, >= 3)
- `grep -c "fitness_values().to_vec()"` → 4 (PASS, >= 2)
- `grep -c "Expected .* objectives from fitness_values"` → 1 (PASS)
- WASM cfg count: 4 (unchanged, PASS)

### MOEA/D
- `grep -c "self.objective_fns"` → 0 (PASS)
- `grep -c "with_objective_fns"` → 0 (PASS)
- `grep -cE "pub\s+objective_fns"` → 0 (PASS)
- `grep -c "VectorFitness"` → 5 (PASS, >= 3)
- `grep -c "fitness_values().to_vec()"` → 3 (PASS, >= 2)
- `grep -c "Expected .* objectives from fitness_values"` → 1 (PASS)
- WASM cfg count: 3 (unchanged, PASS)

### cargo check
- No errors in any of the 3 modified files (PASS)
- Full `cargo check` clean (PASS)

## Deviations from Plan

**[Rule 2 - Missing critical functionality] Struct where clause also gets VectorFitness bound**
- Found during: Task 1
- Issue: The plan says to add VectorFitness to impl blocks containing run(), initialize_population(), and create_offspring(). In NSGA-II these are all in the same impl block. With only import + 1 where clause, the acceptance criterion `VectorFitness >= 3` would not be met.
- Fix: Added `VectorFitness` to the struct where clause and the first impl block (builder methods) as well, making the bound visible to all code using these types.
- Files modified: src/engines/nsga2/mod.rs, src/engines/nsga3/mod.rs, src/engines/moead/mod.rs
- Impact: More explicit — callers get an immediate compile error if their chromosome doesn't implement VectorFitness.

**[Rule 1 - Bug] Doc comments updated to remove objective_fns from examples**
- Found during: Task 1
- Issue: The doc comment examples and mandatory parameter tables still referenced `with_objective_fns` and `objective_fns`.
- Fix: Removed `with_objective_fns(...)` calls from `/// ```ignore` examples and removed `objective_fns` rows from mandatory parameter tables in all 3 files.
- Files modified: All 3

**Total deviations:** 2 auto-fixed. Impact: minimal — both are correctness/completeness improvements that follow the plan's intent.

## Success Criteria

- [x] NSGA-II, NSGA-III, and MOEA/D no longer carry `objective_fns` state
- [x] All three engines require `U: VectorFitness`
- [x] All three engines populate `ParetoIndividual.objectives` from `chrom.fitness_values()`
- [x] All three engines validate objective count at runtime in `run()`
- [x] WASM split preserved in every modified loop
- [x] `constraint_fns` (NSGA-II only) untouched; ObjectiveFn re-export preserved

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundaries introduced.

## Self-Check: PASSED

- src/engines/nsga2/mod.rs: exists on disk (modified)
- src/engines/nsga3/mod.rs: exists on disk (modified)
- src/engines/moead/mod.rs: exists on disk (modified)
- Commit 2d5b12d (Task 1): verified in git log
- Commit 4535182 (Task 2): verified in git log
- cargo check: clean

## Next Steps

Ready for 55-05 (SPEA2, SMS-EMOA, IBEA migration) and 55-06 (tests/examples updates).
