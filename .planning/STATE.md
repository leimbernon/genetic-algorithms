---
gsd_state_version: 1.0
milestone: v3.0.0
milestone_name: — Advanced Representations, Alternative Strategies & Architecture Simplification
status: executing
stopped_at: Phase 60 context gathered
last_updated: "2026-06-07T16:47:49.371Z"
last_activity: 2026-06-07 -- Phase 60 execution started
progress:
  total_phases: 35
  completed_phases: 17
  total_plans: 71
  completed_plans: 84
  percent: 49
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-18)

**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Current focus:** Phase 60 — batch-fitness-fitness-cache-extension

## Current Position

Phase: 60 (batch-fitness-fitness-cache-extension) — EXECUTING
Plan: 1 of 3
Status: Executing Phase 60
Last activity: 2026-06-07 -- Phase 60 execution started

Progress bar: [███░░░░░░░░░░░░░░░░] phases 47-56 complete of 47-65

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

### Decisions (phase 59)

- v3.0.0: IPOP-CMA-ES example uses DIMENSIONS=10, stagnation_threshold=50, max_restarts=3 — bounds example to at most 800 generations while demonstrating restart benefit on multimodal Rastrigin landscape

### Roadmap Evolution

- 2026-05-19: Roadmap created — Phases 47-53 defined for v3.0.0
- 2026-05-31: Phases 54 (N-ary selection) and 55 (VectorFitness) completed
- 2026-06-01: Phase 56 (CMA-ES) complete; phases 57-65 added to ROADMAP with goals and success criteria; `DeGene` renamed to `RealGene`

### Decisions (phase 56)

- v3.0.0: `DeGene` hard-renamed to `RealGene` in phase 56 — relocated to `src/traits/real_gene.rs`; `CmaEngine` and future real-valued engines use `U::Gene: RealGene` bound
- v3.0.0: `CmaEngine` uses Jacobi eigendecomposition (no lapack) + Box-Muller sampling; WASM-compatible (no par_iter, no Instant)

### Blockers/Concerns

- PR #273 (feat/252-cma-es-engine → milestone/v3.0.0) awaiting CI + merge
- Phases 57–65 have ROADMAP entries but no plans yet — next step is plan-phase for whichever comes first

## Session Continuity

Last session: 2026-06-07T16:13:42.082Z
Stopped at: Phase 60 context gathered
Resume file: .planning/phases/60-batch-fitness-fitness-cache-extension/60-CONTEXT.md
