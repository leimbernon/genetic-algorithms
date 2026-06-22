---
gsd_state_version: 1.0
milestone: v3.0.0
milestone_name: — Advanced Representations, Alternative Strategies & Architecture Simplification
status: ready_to_plan
stopped_at: Phase 81 complete (2/2) — ready to discuss Phase 82
last_updated: 2026-06-22T20:49:01.800Z
progress:
  total_phases: 51
  completed_phases: 31
  total_plans: 107
  completed_plans: 166
  percent: 61
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-18)

**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Current focus:** Phase 82 — per engine convergence integration tests (issue #284)

## Current Position

Phase: 82
Plan: Not started
Plans: 138/138 complete (new phases have no plans yet)
Status: Ready to plan

Progress bar: [██████████████████░░] 45/50 phases complete

## Accumulated Context

### Decisions

- v3.0.0: GaError::InternalError(String) is the canonical variant for violated internal invariants (poisoned mutexes); mutex callers use map_err propagation, never unwrap/expect
- v2.3.0: `#[path]` re-exports in lib.rs are the canonical non-breaking restructure pattern — no semver bump needed
- v2.3.0: `mod.rs` directory form required when restructured modules have nested submodules
- v2.3.0: New engines land in `src/engines/` with their own subdirectory; `src/lib.rs` adds the re-export
- v2.3.0: `DeGene` trait extension pattern for engines requiring type-specific arithmetic (f64)
- v2.4.0: Observer wiring uses same `Option<Arc<dyn GaObserver<U>>>` pattern as `ga.rs` — zero overhead when None, no per-engine sub-traits
- v2.4.0: Observer import path is `use crate::observer::GaObserver` (not `crate::observe::observer::GaObserver`) — lib.rs re-exports via `#[path]` alias
- v2.4.0: normalize_st uses ASF-based intercepts with degenerate-nadir fallback + epsilon clamp for DTLZ2 and sparse-population safety
- v2.4.0: SMS-EMOA uses steady-state (mu+1) with hypervolume contribution removal; IBEA uses pairwise I_eps+ indicator with exponential scaling
- v3.0.0: Architecture audit goes first — its decisions shape how new types (Strategy trait, advanced genotypes, variable-length chromosomes) are designed
- v3.0.0: `ChromosomeT` splits into `ChromosomeT` (minimal core) and `LinearChromosome: ChromosomeT` (flat-slice contract) — `TreeChromosome: ChromosomeT` is a parallel branch, never a subtrait of `LinearChromosome`
- v3.0.0: `MultiCaseFitness: ChromosomeT` locked in Phase 50 before Phase 53 — reused by `GpChromosome` for GP program synthesis
- v3.0.0: No new external crates required except conditional `serde_stacker` (gated behind existing `serde` feature flag) — verify wasm32 compatibility before committing
- v3.0.0: `GpGa<U: TreeChromosome>` is a separate engine from `Ga<U: LinearChromosome>` — GP loop differences (ramped init, bloat control, depth limits) do not belong in the standard GA hot path
- v3.0.0: `Box<N>` recursive enum for tree nodes (rejected arena crates) — subtree clone is O(subtree), not O(arena); arena index-remapping across arenas is too complex
- [Phase ?]: .planning/phases/71-per-operator-mutation-params/71-01-SUMMARY.md
- [Phase ?]: v3.0.0: Mutation and all *Params structs derive Copy (D-01); MutationConfiguration derives Copy (D-02) — zero-runtime-cost prerequisite for Plan 02 clone elimination
- [Phase ?]: v3.0.0: offspring_buf allocated once before generation loop (Vec::with_capacity(population_size * 2)) and reused each generation via parent_crossover out: &mut Vec<U> (D-07/D-08/D-09)
- [Phase ?]: v3.0.0: Uncrossed pairs produce no offspring — return Ok(Vec::new()) when crossover probability roll fails (D-04/D-05); offspring = crossed_pairs * 2 per generation
- [Phase ?]: v3.0.0: 1-child multi-parent crossover fallback uses parent_2 not parent_1 (D-06)
- [Phase ?]: v3.0.0: extract_elite returns Vec<usize> indices (D-10) — allocation-free extract phase; caller clones from pre-survivor-selection snapshot
- [Phase ?]: v3.0.0: Discretionary local-search clone retained — >=10 elimination target met exactly (10 of 19); parallel-path clone architecturally required for rayon

### Decisions (phase 59)

- v3.0.0: IPOP-CMA-ES example uses DIMENSIONS=10, stagnation_threshold=50, max_restarts=3 — bounds example to at most 800 generations while demonstrating restart benefit on multimodal Rastrigin landscape

### Roadmap Evolution

- 2026-05-19: Roadmap created — Phases 47-53 defined for v3.0.0
- 2026-05-31: Phases 54 (N-ary selection) and 55 (VectorFitness) completed
- 2026-06-01: Phase 56 (CMA-ES) complete; phases 57-65 added to ROADMAP with goals and success criteria; `DeGene` renamed to `RealGene`

### Decisions (phase 56)

- v3.0.0: `DeGene` hard-renamed to `RealGene` in phase 56 — relocated to `src/traits/real_gene.rs`; `CmaEngine` and future real-valued engines use `U::Gene: RealGene` bound
- v3.0.0: `CmaEngine` uses Jacobi eigendecomposition (no lapack) + Box-Muller sampling; WASM-compatible (no par_iter, no Instant)

### Decisions (phase 60)

- v3.0.0: `BatchFitnessEvaluator<U>` placed in `src/fitness/batch.rs`, re-exported via `src/lib.rs`
- v3.0.0: CMA mutual-exclusivity for batch vs scalar is last-writer-wins (batch silently overrides scalar) — Ga uses stricter ConfigurationError because its `fitness_fn` is Optional
- v3.0.0: `batch_evaluate_pop` structurally replicated on CmaEngine (not shared utility) — bounded footprint; extraction deferred to refactor phase
- v3.0.0: batch+cache partition (D-06) releases Mutex before `evaluate_batch` call to avoid blocking during expensive GPU/remote evaluations (Pitfall 2 / T-60-05)

### Decisions (phase 76)

- v3.0.0: Module deduplication via `pub use` re-export: delete duplicate file, add `pub use` in mod.rs — eliminates code duplication so parallel improvements in shared module apply to all engines
- v3.0.0: Parallel NDS threshold of n >= 100 chosen to balance parallelization overhead against speedup for typical multi-objective workloads
- v3.0.0: `domination_count` derived by inverting `dominated_set` rather than from per-thread results — the parallel split means per-thread results only capture dominators j > i
- v3.0.0: Cross-thread merge deduplication via `sort_unstable + dedup` prevents front extraction underflow from duplicate entries

### Blockers/Concerns

- Phases 70-74 have ROADMAP entries but no plans yet — next step is plan-phase for each
- Phase 70 (#247) and 71 (#249) are architecture refactors with breaking-change potential
- Phase 72 (#265), 73 (#266), 74 (#267) are non-breaking quality improvements

## Session Continuity

Last session: 2026-06-22T19:19:43.560Z
Stopped at: Phase 81 context gathered
Resume file: .planning/phases/81-add-a-prelude-module-for-ergonomic-imports-issue-283/81-CONTEXT.md

## Performance Metrics

| Phase | Plan | Duration | Notes |
|-------|------|----------|-------|
| Phase 70-replace-operator-downcasting P01 | 2min | 2 tasks | 3 files |
| Phase 71 P01 | 50min | - tasks | - files |
| Phase 75 P01 | 5min | 2 tasks | 2 files |
| Phase 75 P02 | 7min | 3 tasks | 2 files |
| Phase 75 P03 | 30min | 4 tasks | 14 files |
| Phase 76 P01 | 1min | 1 task | 2 files |
| Phase 76 P02 | 8min | 2 tasks | 3 files |
| Phase 78 P01 | 5min | 3 tasks | 6 files |
| Phase 80 P01 | 5min | 2 tasks | 2 files |
| Phase Phase 80 PP02 | 2min | 1 task tasks | 1 file files |
| Phase 80 P03 | 5min | 2 tasks | 2 files |
