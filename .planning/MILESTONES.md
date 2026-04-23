# Milestones

## v2.2.1 — Performance Optimizations (Shipped: 2026-04-23)

Eliminated unnecessary heap allocations, reduced algorithmic complexity, and improved concurrency across the GA engine — all internal changes with no public API impact.

**Phases:** 19–24 (6 phases, 13 plans) | **Timeline:** 2026-03-30 → 2026-04-05 | **Commits:** 21 perf/refactor

**Key accomplishments:**

- Eliminated redundant parent clones in crossover hot path; five numeric mutation operators use `set_gene()` instead of `dna().to_vec()` — zero Vec allocation per mutation call
- PMX crossover replaced O(n²) linear position scan with O(n) `HashMap` position map; OX similarly uses O(n) `HashSet` membership
- Rank and Boltzmann selection use `partition_point()` binary search (O(log n)); fitness values collected once per generation and shared across extension, niching, and stats
- Fitness sharing computes distance on-the-fly — eliminates O(n²) distance matrix allocation per generation
- Elite reinsertion and mass genesis both use `select_nth_unstable_by()` O(n) instead of O(n log n) sort
- RNG atomic ordering relaxed from `SeqCst` to `Acquire`/`Relaxed`; extension population regrow parallelized via rayon
- `Range` genes share `Arc<[(T,T)]>` slice per chromosome; `value()` for `Copy` types returns by value; `MassDeduplication` uses incremental `DefaultHasher`
- `GenerationStats` moved (not cloned) into stats history; island migration uses `select_nth_unstable_by()` and `Arc`-shared migrant vectors

---

## v2.1.0 — New Examples (Shipped: 2026-03-22)

Added `GenerationStats.diversity`, `ListChromosome<T>` genotype, `Reporter<U>` lifecycle trait, and a `visualization` feature flag — then demonstrated the whole library with six runnable examples covering every major GA mode.

**Phases:** 6–12 (7 phases, 15 plans) | **Timeline:** 2026-03-20 → 2026-03-22 | **Commits:** ~103

**Key accomplishments:**

- Added `diversity: f64` to `GenerationStats` (fitness std-dev); wired into extension trigger and dynamic mutation
- Introduced `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets, integrating with all existing operators
- Shipped `Reporter<U>` trait with `on_start`, `on_generation_complete`, `on_new_best`, `on_finish` hooks; zero overhead when unset
- Added `visualization` feature flag with `plot_fitness`, `plot_diversity`, and `plot_histogram` (PNG/SVG via plotters)
- Added six self-contained examples: `rastrigin`, `feature_selection`, `niching`, `nsga2_zdt1`, `island_model`, `job_scheduling`
- Updated README with `## Examples` table documenting all 10 examples with exact `cargo run` commands

**Known gaps (deferred):** Reporter/Visualization not demonstrated in examples; ListChromosome has no dedicated example. See `.planning/milestones/v2.1.0-MILESTONE-AUDIT.md`.

---

## v2.0.0 — Restructuring & Optimisation (Completed 2026-03-01)

Major rewrite: Island GA, NSGA-II, structured errors, rayon parallelism, serde support, new operators, elitism, stopping criteria, adaptive GA, checkpoint support.

**Phases:** Pre-GSD (no phase tracking)

---
