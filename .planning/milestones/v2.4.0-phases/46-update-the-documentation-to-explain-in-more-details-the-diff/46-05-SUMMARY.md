---
phase: 46-update-the-documentation-to-explain-in-more-details-the-diff
plan: 05
subsystem: documentation
tags: [docs, examples, engines, operators, stale-api-refs]
requires: [46-01, 46-04]
affects: [docs/examples.md, docs/engines.md, docs/operators/selection.md, docs/operators/crossover.md, docs/operators/mutation.md, docs/operators/survivor.md, docs/operators/extension.md]
key-files:
  - docs/examples.md: complete rewrite (473 lines, +473 -205)
  - docs/engines.md: expanded from 7 to 11 engines (+404 -97)
  - docs/operators/selection.md: added Clearing selection
  - docs/operators/crossover.md: added EdgeRecombination crossover
  - docs/operators/mutation.md: added Cauchy, LevyFlight, Uniform, Differential operators
  - docs/operators/survivor.md: added DeterministicCrowding survivor
  - docs/operators/extension.md: verified accurate (no changes needed)
decisions: []
metrics:
  duration: ~15 min
  completed_date: 2026-05-14
---

# Phase 46 Plan 05: Update Existing docs/ to Current API

## One-liner

Rewrote `docs/examples.md` with current API (all 19 examples), expanded `docs/engines.md` from 7 to 11 engines with per-engine guide links, and updated 5 operator guide files with new operators from phases 31-33 (Clearing selection, EdgeRecombination crossover, Cauchy/LevyFlight/Uniform/Differential mutation, DeterministicCrowding survivor).

## Tasks

### Task 1: Rewrite docs/examples.md with current API and all 19 examples

**Commit:** `46a714a`

- Complete rewrite removing all stale API references (`ga_lib` → `genetic_algorithms::`, `BinaryChromosome`/`RangeChromosome` as standalone type names → `chromosomes::Binary`/`chromosomes::Range` type aliases, `generation_limit` → `max_generations`)
- New examples catalog table covering all 19 examples with engine/feature, problem type, and key concepts
- 4 detailed walkthroughs: rastrigin, nsga2_zdt1, constrained_g1, aos_demo — each with problem description, configuration walkthrough, key lines explained, and expected output
- Running an Example section with cargo run commands and feature flags
- 473 lines of comprehensive documentation

### Task 2: Expand docs/engines.md to cover all 11 engines

**Commit:** `c8e6e21`

- Updated overview table from 7 to 11 engines (added Nsga3Ga, MoeaDGa, Spea2Ga, SmsEmoaGa, IbeaGa)
- Added 5 new engine sections following the existing format: description, when-to-use, configuration code examples, key parameters tables, see-also links to per-engine guide files
- All existing engine sections (Ga, IslandGa, Nsga2Ga, AlpsEngine, CellularEngine, DeEngine, ScatterEngine) preserved with current API references
- Updated Related section with links to all 5 new per-engine guide files

### Task 3: Update 5 operator guide files with new operators from phases 31-33

**Commit:** `609dbfa`

- `docs/operators/selection.md`: Added Clearing selection with niche_radius parameter, niche preservation description, enum variant documentation
- `docs/operators/crossover.md`: Added EdgeRecombination crossover with adjacency preservation algorithm documentation
- `docs/operators/mutation.md`: Added Cauchy (heavy-tailed perturbation), LevyFlight (Mantegna algorithm), Uniform (random gene reset), Differential (DE-style) mutation operators with configuration details
- `docs/operators/survivor.md`: Added DeterministicCrowding survivor with Hamming distance pairing algorithm
- `docs/operators/extension.md`: Verified accurate — all 5 Extension variants (Noop, MassExtinction, MassGenesis, MassDegeneration, MassDeduplication) already documented, no stale API references found
- All files: Fixed stale `ga_lib` references, updated to `genetic_algorithms::` crate prefix and current type names

## Deviations from Plan

None — plan executed exactly as written.

## Pre-existing Issues (Deferred)

The following stale API references exist in docs/ files NOT modified by this plan. These are pre-existing from Phase 12 documentation and are out of scope for this plan:

- `docs/fitness.md` — uses `ga_lib::` crate prefix, `BinaryChromosome` standalone type
- `docs/traits.md` — uses `my_ga_lib::` prefix (demonstration code), standalone type references
- `docs/population.md` — uses `ga_lib::` crate prefix, `BinaryChromosome`/`RangeChromosome` standalone types
- `docs/validators.md` — uses `my_ga_lib::` prefix, `BinaryChromosome`/`RangeChromosome` standalone types

These should be addressed in a follow-up documentation cleanup plan.

## Verification

All plan-level verification criteria pass within scope:

1. **docs/examples.md uses current API:** No `ga_lib`, no `BinaryChromosome`/`RangeChromosome` as standalone types (only as `chromosomes::Binary as ...` aliases), no `generation_limit`
2. **docs/examples.md covers all 19 examples:** Catalog table includes all examples including 9 new ones
3. **docs/engines.md covers all 11 engines:** Overview table and individual sections for all engines with links
4. **All 5 operator files updated:** Clearing, EdgeRecombination, Cauchy, LevyFlight, Uniform, Differential, DeterministicCrowding added; all stale `ga_lib` references in modified files removed
5. **Zero stale references in updated files:** No `ga_lib`, no `BinaryChromosome`/`RangeChromosome`, no `generation_limit` in any of the 7 modified files

## Success Criteria

- [x] docs/examples.md completely rewritten with current API, all 19 examples, and detailed walkthroughs
- [x] docs/engines.md expanded from 7 to 11 engines with per-engine guide links
- [x] All 5 operator guide files updated with new operators from phases 31-33
- [x] Zero stale API references across all updated files

## Self-Check

PASSED — All claims verified against file content and git history.
