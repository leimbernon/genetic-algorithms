# Phase 61: Performance — Clone Reduction & Parallel Survivor - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 61-performance-clone-reduction-parallel-survivor
**Areas discussed:** Clone scope & priority, Parallel survivor design, Benchmark harness

---

## Clone Scope & Priority

| Option | Description | Selected |
|--------|-------------|----------|
| Crossover fallback path | Lines 2916-2917: eliminates one full chromosome clone per couple in the rayon inner loop. High impact at large populations. | ✓ |
| Selection output collect | Line 3091: currently clones chromosomes to build the output Vec. Could use indices instead. | |
| Both — systematic pass | Audit both hot-path sites plus any other per-generation-per-chromosome clones. | |

**User's choice:** Crossover fallback path

---

| Option | Description | Selected |
|--------|-------------|----------|
| Mutate parents in-place | Take ownership of parent chromosomes from the couple and mutate directly. Eliminates both clones. | ✓ |
| Skip the couple, produce 0 offspring | If crossover fails, don't inject fallback children. Behavioral change. | |
| You decide | Claude picks safest approach preserving current behavior. | |

**User's choice:** Mutate parents in-place

---

| Option | Description | Selected |
|--------|-------------|----------|
| Defer — not hot path | Once-per-generation observer clones are negligible. Keep observer API unchanged. | |
| Include — change observer to &U | Modify GaObserver trait to accept &U (reference). Breaking change under v3.0.0. | ✓ |

**User's choice:** Include — change observer to &U

---

| Option | Description | Selected |
|--------|-------------|----------|
| Only clone-triggered callbacks | Change on_new_best and on_generation_complete to &U. Less disruption. | |
| All observer callbacks uniformly | Consistent API: all U params become &U. Cleaner trait, wider breaking change. | ✓ |

**User's choice:** All observer callbacks uniformly

---

## Parallel Survivor Design

| Option | Description | Selected |
|--------|-------------|----------|
| par_sort_unstable_by | One-liner: replace sort_by with par_sort_unstable_by. Minimal code change, rayon owns parallelism. | ✓ |
| Score-precompute path | par_iter().map(score).collect() then sequential sort. More cache-friendly for large chromosomes. | |
| You decide | Claude picks whichever is measurably better at pop=500. | |

**User's choice:** par_sort_unstable_by

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — parallelize the sort step | Apply par_sort_unstable_by to the survivors sub-vec in mu_comma_lambda. | ✓ |
| No — skip mu_comma_lambda | Keep sequential. Rarely used in practice. | |

**User's choice:** Yes — parallelize the sort step

---

## Benchmark Harness

| Option | Description | Selected |
|--------|-------------|----------|
| Add to benches/ga_run.rs | Add rastrigin_pop500 bench to existing file. Keeps GA benchmarks consolidated. | |
| New benches/rastrigin.rs | Dedicated file for Rastrigin-specific scenarios. Easier to expand later. | ✓ |

**User's choice:** New benches/rastrigin.rs

---

| Option | Description | Selected |
|--------|-------------|----------|
| RangeChromosome<f64> (fixed 10 dims) | Rastrigin is real-valued. Bounds [-5.12, 5.12], 10 dimensions. | |
| RangeChromosome<f64> with configurable dims | Parameterize over [10, 20, 50] dimensions. | ✓ |

**User's choice:** RangeChromosome<f64> with configurable dims (10, 20, 50)

---

## Claude's Discretion

- Whether `use rayon::prelude::*` is added to each survivor file or imported at the call site
- Internal variable name for captured `fitness_target` in parallel sort comparator closure
- Exact `max_generations` for rastrigin bench (balance warmup time vs measurement signal)
- Whether benchmark uses `BatchSize::SmallInput` or `BatchSize::LargeInput`

## Deferred Ideas

- Selection output collect (line 3091) — explicitly descoped; future performance phase
- DeterministicCrowding parallelism — order-dependent pairing; future restructure needed
- Observer async support — `&U` signatures open the door but async trait is deferred
