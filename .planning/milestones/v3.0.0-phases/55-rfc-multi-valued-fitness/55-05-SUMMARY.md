---
phase: 55-rfc-multi-valued-fitness
plan: "05"
subsystem: multi-objective-engines
tags:
  - rust
  - multi-objective
  - spea2
  - sms-emoa
  - ibea
  - island
  - vector-fitness
  - breaking-change

dependency_graph:
  requires:
    - 55-01  # VectorFitness trait defined
    - 55-02  # Chromosome types implement VectorFitness
  provides:
    - spea2-migrated
    - sms-emoa-migrated
    - ibea-migrated
    - island-nsga2-migrated
  affects:
    - src/engines/spea2/mod.rs
    - src/engines/ibea/mod.rs
    - src/engines/sms_emoa/mod.rs
    - src/engines/island/nsga2.rs

tech_stack:
  added: []
  patterns:
    - VectorFitness bound on MO engine impl blocks
    - Runtime objective-count guard after initialize_population()
    - chrom.calculate_fitness() + fitness_values().to_vec() replaces objective_fns closures

key_files:
  created: []
  modified:
    - src/engines/spea2/mod.rs
    - src/engines/ibea/mod.rs
    - src/engines/sms_emoa/mod.rs
    - src/engines/island/nsga2.rs
    - src/engines/ga.rs
    - src/operations/selection.rs

decisions:
  - "initialize_population() moved to VectorFitness-bounded impl block in IBEA and SMS-EMOA to avoid split bounds"
  - "Island NSGA-II: VectorFitness added to both LinearChromosome-only and mutation::ValueMutable impl blocks"
  - "Island NSGA-II retains unconditional rayon::par_iter (pre-existing pattern, no WASM gate was present before)"
  - "ga.rs and selection.rs MultiCaseFitness rename auto-fixed as Rule 3 blocker from plan 55-03"

metrics:
  duration_minutes: 25
  completed: "2026-05-30T08:56:32Z"
  tasks_completed: 2
  files_modified: 6
---

# Phase 55 Plan 05: SPEA2, SMS-EMOA, IBEA, Island NSGA-II VectorFitness Migration Summary

Migrated four remaining MO engine locations to `VectorFitness`, completing the v3.0.0 breaking change recipe across all 7 MO engine sites (3 in Plan 04 + 4 here).

## What Was Built

- **Spea2Ga**: `objective_fns` field removed, `with_objective_fns()` builder removed, `VectorFitness` bound added to run/create_offspring/initialize_population impl block. Runtime objective-count guard installed.
- **IbeaGa**: Same migration. `initialize_population()` moved from `LinearChromosome`-only block to `LinearChromosome + mutation::ValueMutable + VectorFitness` block to resolve method resolution.
- **SmsEmoaGa**: Same migration. `initialize_population()` similarly moved to VectorFitness-bounded block. Inner-lambda direct `.objective_fns` access in `run()` steady-state loop replaced with `calculate_fitness() + fitness_values().to_vec()` (RESEARCH.md A3 anti-pattern closed).
- **IslandNsga2Ga**: `objective_fns` field + `with_objective_fns()` + validate check removed. `VectorFitness` added to both `impl` blocks. `initialize_islands()` and `evolve_islands_one_generation()` now use `chrom.calculate_fitness() + chrom.fitness_values().to_vec()`. Runtime objective-count guard reads first island's first individual.

## Task Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 0d3e321 | feat(55-05): migrate Spea2Ga and IbeaGa to VectorFitness |
| 2 | 30f8093 | feat(55-05): migrate SmsEmoaGa and Island Nsga2Ga to VectorFitness |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] MultiCaseFitness rename leftover from plan 55-03**
- **Found during:** Task 1 — first `cargo check` after SPEA2/IBEA migration
- **Issue:** `src/engines/ga.rs` and `src/operations/selection.rs` still imported `crate::traits::MultiCaseFitness` (renamed to `VectorFitness` in Plan 55-03 but these call sites were missed)
- **Fix:** Replaced `MultiCaseFitness` with `VectorFitness` in both files; also renamed `case_fitness()` calls in `selection.rs` to `fitness_values()`
- **Files modified:** `src/engines/ga.rs`, `src/operations/selection.rs`
- **Commit:** 0d3e321 (included in Task 1 commit)

**2. [Rule 1 - Struct Design] initialize_population() impl block mismatch in IBEA and SMS-EMOA**
- **Found during:** Tasks 1 & 2 — `cargo check` showed `method fitness_values() not found for type parameter U`
- **Issue:** `initialize_population()` was in the `U: LinearChromosome` impl block, which doesn't have `VectorFitness`. The call to `chrom.fitness_values()` requires the bound to be visible.
- **Fix:** Moved `initialize_population()` to the `U: LinearChromosome + mutation::ValueMutable + VectorFitness` impl block in both IBEA and SMS-EMOA.
- **Files modified:** `src/engines/ibea/mod.rs`, `src/engines/sms_emoa/mod.rs`
- **Commit:** included in respective task commits

## Verification Results

```
# All 4 target files
grep -c "self.objective_fns" spea2/mod.rs  → 0
grep -c "self.objective_fns" ibea/mod.rs   → 0
grep -c "self.objective_fns" sms_emoa/mod.rs → 0
grep -c "self.objective_fns" island/nsga2.rs → 0

grep -c "with_objective_fns" (all 4)       → 0 (doc comment `//!` in spea2 doesn't count as code)
grep -c "pub objective_fns" (all 4)        → 0

cargo check → 0 errors
```

## Known Stubs

None. All 4 engines now populate `ParetoIndividual.objectives` from `chrom.fitness_values().to_vec()` after `calculate_fitness()`.

## Threat Flags

No new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- `0d3e321` exists: feat(55-05): migrate Spea2Ga and IbeaGa to VectorFitness
- `30f8093` exists: feat(55-05): migrate SmsEmoaGa and Island Nsga2Ga to VectorFitness
- `src/engines/spea2/mod.rs` — modified (confirmed by grep)
- `src/engines/ibea/mod.rs` — modified (confirmed by grep)
- `src/engines/sms_emoa/mod.rs` — modified (confirmed by grep)
- `src/engines/island/nsga2.rs` — modified (confirmed by grep)
- `cargo check` — clean (0 errors)
