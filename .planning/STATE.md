---
gsd_state_version: 1.0
milestone: v3.0.0
milestone_name: — Advanced Representations, Alternative Strategies & Architecture Simplification
status: completed
stopped_at: Phase 52 verified, passing
last_updated: "2026-05-29T10:27:29.332Z"
last_activity: 2026-05-29 -- Phase 54 marked complete
progress:
  total_phases: 36
  completed_phases: 14
  total_plans: 56
  completed_plans: 72
  percent: 39
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-23)

**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Current focus:** Phase 54 — n-ary-selection-per-operator-mutation-params

## Current Position

Phase: 54 — COMPLETE
Plan: 2 of 2 (both complete)
Plans: 2 plans in 2 waves (all complete)
Status: Phase 54 complete
Last activity: 2026-05-29 -- Phase 54 marked complete

Progress bar: [█░░░░░░] 1/7 phases complete

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
- v3.0.0 Phase 50: `factory_lexicase<U: ChromosomeT + MultiCaseFitness>()` is the dispatch path for lexicase — standard `factory()` returns ConfigurationError for Lexicase/EpsilonLexicase variants
- v3.0.0 Phase 50: `epsilon = 0.0` in SelectionConfiguration is the sentinel for dynamic MAD mode; any positive value is fixed epsilon — SelectionConfiguration remains Copy
- v3.0.0 Phase 50: `Ga<U: MultiCaseFitness>` adds `select_parents_lexicase()` in a separate impl block — Rust cannot overload `run()`; users call this explicitly for the lexicase path
- v3.0.0 Phase 52: `ChromosomeLength::Variable { min, max }` lives in `src/types/chromosomes/mod.rs` — re-exported as `genetic_algorithms::chromosomes::ChromosomeLength`
- v3.0.0 Phase 52: `Mutation::Insertion` (old permutation-insert operator) renamed to `Mutation::PermutationInsert` — breaking change, documented in MIGRATION.md
- v3.0.0 Phase 52: Variable init uses per-individual `rng.random_range(min..=max)` for length sampling — zero changes to `init_fn` signature
- v3.0.0 Phase 52: Parsimony pressure adjusts fitness temporarily in `apply_parsimony_pressure` — stored `fitness()` never permanently mutated
- v3.0.0 Phase 52: `initialize_with_seeds()` does NOT yet support `ChromosomeLength::Variable` — returns ConfigurationError; only random init supports Variable

### Roadmap Evolution

- 2026-05-19: Roadmap created — Phases 47-53 defined for v3.0.0
- 2026-05-28: Phases 54-65 added from v3.0.0-EXECUTION-ORDER.md (N-ary Selection, RFC Multi-Valued Fitness, CMA-ES/PSO/EDA engines, Restart Strategies, Batch Fitness, Performance, Surrogate, Visualization, Test+Doc Quality, Migration Guide)

### Blockers/Concerns

- Phase 47 is the highest-risk change (~30 files touched mechanically) — it must merge and pass CI before any feature branch touching `ChromosomeT` opens a PR
- `serde_stacker` wasm32 compatibility is unverified — must be checked in Phase 53 before committing to it; if it fails, an iterative serde approach is needed
- Fitness function removal from chromosomes (Phase 47 decision) may break users calling `chromosome.calculate_fitness()` directly — scope must be validated and a migration path documented in MIGRATION.md
- Phase 52 gap: `initialize_with_seeds()` returns ConfigurationError for `ChromosomeLength::Variable` — needs follow-up in a future phase or bugfix

## Session Continuity

Last session: 2026-05-28T00:00:00Z
Stopped at: Phase 52 verified, passing
Resume file: .planning/phases/52-variable-length-chromosomes/52-VERIFICATION.md
