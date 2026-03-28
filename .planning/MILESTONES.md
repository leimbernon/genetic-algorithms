# Milestones

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
