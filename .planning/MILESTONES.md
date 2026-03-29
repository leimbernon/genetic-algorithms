# Milestones

## v2.2.0 Observability & Traceability (Shipped: 2026-03-28)

**Phases:** 13–18 (6 phases, 14 plans) | **Timeline:** 2026-03-25 → 2026-03-28 | **Files changed:** 102 | **LOC delta:** +13,235 / -858

**Key accomplishments:**

- Introduced `GaObserver<U>` trait with 12 `Send+Sync` hooks (lifecycle, operator, special events) and zero-overhead `Option<Arc<dyn GaObserver>>` field on `Ga<U>`
- Added `LogObserver` reproducing all pre-v2.2.0 `log!()` output; migrated all 16 hardcoded `info!/debug!/trace!` calls out of `ga.rs` — grep regression test prevents regressions
- Added `TracingObserver` (behind `observer-tracing` feature flag) emitting structured `tracing` spans per generation; zero compile-time cost when disabled
- Added `IslandGaObserver<U>` and `Nsga2Observer<U>` sub-traits with engine-specific hooks; wired into `IslandGa<U>` and `Nsga2Ga<U>` run loops
- Added `CompositeObserver<U>` for N-observer fan-out via `AllObserver` blanket impl, and `MetricsObserver` (behind `observer-metrics`) recording 11 per-generation metrics via `metrics` facade
- Closed v2.2.0 audit gaps: `TracingObserver` composable, hook ordering fixed, `Duration::ZERO` replaced with real elapsed time, `NoopObserver`/`ExtensionEvent`/`TerminationCause` re-exported from crate root

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
