---
gsd_state_version: 1.0
milestone: v3.0.0
milestone_name: — Advanced Representations, Alternative Strategies & Architecture Simplification
status: planning
stopped_at: Phase 51 context gathered
last_updated: "2026-05-23T09:07:11.798Z"
last_activity: 2026-05-23
progress:
  total_phases: 24
  completed_phases: 10
  total_plans: 42
  completed_plans: 58
  percent: 42
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-23)

**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Current focus:** Phase 51 — multi parent crossover self adaptive mutation

## Current Position

Phase: 51
Plan: Not started
Status: Ready to plan
Last activity: 2026-05-23

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
- v3.0.0 Phase 50: `factory_lexicase<U: ChromosomeT + MultiCaseFitness>()` is the dispatch path for lexicase — standard `factory()` returns ConfigurationError for Lexicase/EpsilonLexicase variants
- v3.0.0 Phase 50: `epsilon = 0.0` in SelectionConfiguration is the sentinel for dynamic MAD mode; any positive value is fixed epsilon — SelectionConfiguration remains Copy
- v3.0.0 Phase 50: `Ga<U: MultiCaseFitness>` adds `select_parents_lexicase()` in a separate impl block — Rust cannot overload `run()`; users call this explicitly for the lexicase path

### Roadmap Evolution

- 2026-05-19: Roadmap created — Phases 47-53 defined for v3.0.0

### Blockers/Concerns

- Phase 47 is the highest-risk change (~30 files touched mechanically) — it must merge and pass CI before any feature branch touching `ChromosomeT` opens a PR
- `serde_stacker` wasm32 compatibility is unverified — must be checked in Phase 53 before committing to it; if it fails, an iterative serde approach is needed
- Fitness function removal from chromosomes (Phase 47 decision) may break users calling `chromosome.calculate_fitness()` directly — scope must be validated and a migration path documented in MIGRATION.md

## Session Continuity

Last session: 2026-05-23T09:07:11.793Z
Stopped at: Phase 51 context gathered
Resume file: .planning/phases/51-multi-parent-crossover-self-adaptive-mutation/51-CONTEXT.md
