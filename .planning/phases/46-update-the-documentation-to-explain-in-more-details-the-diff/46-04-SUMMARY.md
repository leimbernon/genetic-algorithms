---
phase: 46
plan: 04
subsystem: documentation
tags: [docs, per-engine-guides, concept-guides, multi-objective, framework-extensions]
requires: []
provides: [17-new-docs-files]
affects: docs/index.md
tech-stack:
  added: []
  patterns: [D-04 ficha tecnica template, concept guide template]
key-files:
  created:
    - docs/nsga3.md
    - docs/moead.md
    - docs/spea2.md
    - docs/sms_emoa.md
    - docs/ibea.md
    - docs/multi_objective.md
    - docs/observer.md
    - docs/constraints.md
    - docs/hall_of_fame.md
    - docs/aos.md
    - docs/benchmarks.md
    - docs/memetic.md
    - docs/operations.md
    - docs/niching.md
    - docs/extension.md
    - docs/error.md
    - docs/initializers.md
  modified:
    - docs/index.md
decisions: []
metrics:
  duration: ~25 min
  completed: "2026-05-14"
---

# Phase 46 Plan 04: Create 17 missing docs/ guide files

**One-liner:** Created all 17 missing docs/ guide files — 6 per-engine multi-objective guides following the D-04 ficha tecnica template and 11 concept/framework guides following the concept guide template, with verified docs/index.md links.

## Summary

This plan completed the docs/ guide directory by creating 17 missing guide files spanning all phases 35-45 features. The work was split into 3 tasks:

**Task 1 (6 files):** Per-engine guides for NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA using the D-04 ficha tecnica template — each with Description, When to Use (problem type/objectives/variable/key strength/weakness), Mandatory and Optional parameter tables, Complete Example (with actual API paths like `genetic_algorithms::nsga3::Nsga3Ga`), Configuration Tips, When to Choose This vs comparison table, References, and See Also section.

Plus `multi_objective.md` covering shared Pareto dominance, non-dominated sorting, quality indicators (hypervolume, GD, IGD, spread), and usage examples.

**Task 2 (6 files):** Concept guides for framework extensions and observability:
- `observer.md` — 12 lifecycle hooks, built-in observers (LogObserver, CompositeObserver, NoopObserver, MetricsObserver, TracingObserver), engine-specific sub-traits
- `constraints.md` — Static/Dynamic/Adaptive penalty strategies, Deb's feasibility rules, violation computation
- `hall_of_fame.md` — Archive admission criteria, DistanceMetric (Fitness/Genotypic), limit-based insertion
- `aos.md` — Probability Matching, Adaptive Pursuit, Multi-Armed Bandit (UCB1) strategies
- `benchmarks.md` — BenchmarkFn trait, Sphere/Rastrigin/Ackley, ZDT1-6, DTLZ1-7 tables, feature flag documentation
- `memetic.md` — LocalSearch trait, HillClimbing, Lamarckian/Baldwinian modes, application strategies

**Task 3 (5 + 1 files):** Core subsystem guides and index update:
- `operations.md` — All 8 selection, 12 crossover, 15 mutation, 5 survivor, 5 extension operators with decision table
- `niching.md` — Fitness sharing mechanics, sigma_share tuning, alpha parameter
- `extension.md` — ExtensionConfiguration builder, all 4 diversity-rescue strategies
- `error.md` — Complete 20-variant GaError table with mitigations
- `initializers.md` — binary/range/list/generic initialization functions, warm start patterns
- `docs/index.md` — Verified all 17 links resolve; added missing `multi_objective.md` link

## Task Status

| Task | Name | Status | Commit | Key Files |
|------|------|--------|--------|-----------|
| 1 | Create 6 per-engine guide files | Done | `b023a1f` | nsga3.md, moead.md, spea2.md, sms_emoa.md, ibea.md, multi_objective.md |
| 2 | Create 6 concept guides (extensions) | Done | `018673f` | observer.md, constraints.md, hall_of_fame.md, aos.md, benchmarks.md, memetic.md |
| 3 | Create 5 core subsystem guides + index | Done | `fcde186` | operations.md, niching.md, extension.md, error.md, initializers.md, docs/index.md |

## Verification

All acceptance criteria verified:

- **17 new files exist:** All confirmed via Read tool (non-empty, with correct heading)
- **Per-engine guides follow D-04 template:** Each has Description, When to Use, Quick Reference (mandatory/optional params), Complete Example, Configuration Tips, cross-comparison table, References, See Also
- **Concept guides follow template:** Each has Overview, Key Concepts, API/Usage Example, Configuration, See Also
- **All files have 40+ lines:** Confirmed by commit contents (777+819+688 = 2284 lines across 17 files)
- **docs/index.md links to all 17 files:** Verified by Read — all links present; missing multi_objective.md link added in Task 3
- **API paths reference actual crate paths:** All examples use `genetic_algorithms::nsga3::Nsga3Ga`, etc. (verified against source module paths)
- **Complete Examples are substantive:** DTLZ2 for NSGA-III, ZDT1 for SPEA2/SMS-EMOA/IBEA, ZDT with Tchebycheff for MOEA/D

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all guides contain complete, substantive content with real code examples referencing actual API paths. No "coming soon" or "TODO" placeholders.

## Threat Flags

None — all guides document existing public API. No new attack surface introduced (documentation-only changes).

## Self-Check: PASSED

All 17 created files verified to exist via Read tool. docs/index.md updated and verified. All 3 commits confirmed.
