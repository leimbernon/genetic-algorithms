---
gsd_state_version: 1.0
milestone: v3.0.0
milestone_name: — Advanced Representations, Alternative Strategies & Architecture Simplification
status: executing
stopped_at: context exhaustion at 80% (2026-05-25)
last_updated: "2026-05-25T12:51:42.809Z"
last_activity: 2026-05-25
progress:
  total_phases: 24
  completed_phases: 6
  total_plans: 46
  completed_plans: 49
  percent: 25
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-18)

**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Current focus:** Phase 53 — tree-chromosome-gpga-engine

## Current Position

Phase: 53 (tree-chromosome-gpga-engine) — EXECUTING
Plan: 4 of 4
Status: Ready to execute
Last activity: 2026-05-25

Progress bar: [░░░░░░░] 0/7 phases complete

## Accumulated Context

### Decisions

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
- v3.0.0: Phase 52 Wave 0 stubs lock API contract — PermutationInsert rename, Insertion/Deletion length operators, VariableLength crossover with AlignmentStrategy, ChromosomeLength::Variable, length_penalty parsimony pressure
- v3.0.0: ChromosomeLength::Fixed(n)/Variable{min,max} lives in chromosomes module; Mutation::Insertion(length-grow)/Deletion(length-shrink) require ChromosomeLength::Variable in MutationConfiguration; new gene for Insertion sampled by cloning a random existing gene (generic, works with all ChromosomeT)
- v3.0.0: AlignmentStrategy::Trim/Pad and Crossover::VariableLength use single-point crossover within aligned region; Pad fills shorter parent by cloning random genes from its own DNA (consistent with Mutation::Insertion allele sampling)
- v3.0.0: Parsimony pressure uses temporary fitness adjustment (adjust → select → restore) so stored fitness() is never mutated; SurvivorConfig trait with with_length_penalty() builder wired into ConfigurationT supertrait
- v3.0.0: Extension regrowth for variable-length chromosomes samples lengths from [min_observed, max_observed] of surviving population, clamped to configured Variable {min, max} bounds

### Roadmap Evolution

- 2026-05-19: Roadmap created — Phases 47-53 defined for v3.0.0

### Blockers/Concerns

- Phase 47 is the highest-risk change (~30 files touched mechanically) — it must merge and pass CI before any feature branch touching `ChromosomeT` opens a PR
- `serde_stacker` wasm32 compatibility is unverified — must be checked in Phase 53 before committing to it; if it fails, an iterative serde approach is needed
- Fitness function removal from chromosomes (Phase 47 decision) may break users calling `chromosome.calculate_fitness()` directly — scope must be validated and a migration path documented in MIGRATION.md

## Session Continuity

Last session: 2026-05-25T12:51:42.799Z
Stopped at: context exhaustion at 80% (2026-05-25)
Resume file: None
