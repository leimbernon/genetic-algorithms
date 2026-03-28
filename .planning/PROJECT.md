# genetic_algorithms

## What This Is

A modular, concurrent Genetic Algorithms library for Rust. Provides composable operators (selection, crossover, mutation, survivor), multi-threaded execution via `rayon`, Island Model GA, NSGA-II multi-objective optimization, adaptive GA mode, elitism/stopping criteria, population diversity tracking, a `Reporter<U>` lifecycle trait, an optional visualization feature (PNG/SVG charts), and a `List<T>` genotype for finite symbolic alphabets. Published on crates.io as `genetic_algorithms` with six runnable examples covering every major GA mode.

## Core Value

Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ Core GA engine with configurable operators — v1.0
- ✓ Binary and Range<T> genotypes — v1.0
- ✓ Selection: Random, RouletteWheel, SUS, Tournament, Rank, Boltzmann, Truncation — v2.0
- ✓ Crossover: Cycle, MultiPoint, Uniform, SinglePoint, Order, SBX, BlendAlpha, PMX, Arithmetic — v2.0
- ✓ Mutation: Swap, Inversion, Scramble, Value, BitFlip, Creep, Gaussian, Polynomial, NonUniform, Insertion — v2.0
- ✓ Survivor: Fitness, Age, μ+λ, μ,λ — v2.0
- ✓ Island Model GA with migration topologies — v2.0
- ✓ NSGA-II multi-objective optimization — v2.0
- ✓ Island + NSGA-II hybrid — v2.0
- ✓ Fitness sharing / niching — v2.0
- ✓ Elitism support — v2.0
- ✓ Compound stopping criteria (stagnation, convergence, time limit) — v2.0
- ✓ Structured error handling (GaError) — v2.0
- ✓ Serde support (feature flag) — v2.0
- ✓ Checkpoint save/load — v2.0
- ✓ Seedable RNG — v2.0
- ✓ Adaptive GA (dynamic crossover/mutation probabilities) — v2.0
- ✓ Per-generation statistics (GenerationStats) — v2.0
- ✓ Rayon-based parallelism — v2.0
- ✓ Population diversity metric (`GenerationStats.diversity`) wired into extension trigger and dynamic mutation — v2.1.0
- ✓ `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets, compatible with all operators — v2.1.0
- ✓ `Reporter<U>` trait with `on_start`, `on_generation_complete`, `on_new_best`, `on_finish` hooks; zero overhead when unset — v2.1.0
- ✓ `visualization` feature flag: `plot_fitness`, `plot_diversity`, `plot_histogram` (PNG/SVG via plotters) — v2.1.0
- ✓ Six runnable examples: `rastrigin`, `feature_selection`, `niching`, `nsga2_zdt1`, `island_model`, `job_scheduling` — v2.1.0
- ✓ README `## Examples` table documenting all 10 examples with `cargo run` commands — v2.1.0

### Active

<!-- Current scope. Building toward these. -->

- ✓ GaObserver trait with lifecycle, operator, and special event hooks — Validated in Phase 13: GaObserver Base Trait
- ✓ LogObserver replacing hardcoded log!() calls — Validated in Phase 14: LogObserver + Log Migration
- ✓ TracingObserver behind `observer-tracing` feature flag (#184) — Validated in Phase 15: TracingObserver
- ✓ Island GA and NSGA-II specialized observer sub-traits (#185) — Validated in Phase 16: Sub-Traits
- ✓ CompositeObserver for combining multiple observers (#186) — Validated in Phase 17: CompositeObserver + MetricsObserver
- ✓ MetricsObserver behind `observer-metrics` feature flag (#186) — Validated in Phase 17: CompositeObserver + MetricsObserver
- ✓ Observer API polish: TracingObserver composability, hook ordering, real operator timing, crate-root re-exports — Validated in Phase 18: Observer API Polish

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- GUI/interactive visualization — library generates static PNG/SVG charts; interactive dashboards are users' concern
- Specific telemetry backends (Prometheus, Jaeger) — facade pattern lets users pick
- Per-gene hooks in observer — too granular, unacceptable overhead in hot loops

## Current Milestone: v2.2.0 Observability & Traceability

**Goal:** Implement a generic, telemetry-agnostic observability system enabling full metrics, tracing, and alerting without coupling to any specific tool.

**Target features:**
- GaObserver trait with zero-overhead opt-in
- LogObserver (backward-compatible migration of existing logging)
- TracingObserver (structured spans via `tracing` crate)
- Island GA & NSGA-II observer extensions
- CompositeObserver + MetricsObserver

## Context

- Library is published on crates.io, backward compatibility matters
- v2.1.0 shipped: ~14,600 LOC Rust, 10 runnable examples, `visualization` feature, `Reporter<U>` trait, `List<T>` genotype
- Current logging uses hardcoded `log!()` macros with 8 log targets in ga.rs — migration to observer is the v2.2.0 goal
- Observer pattern must have zero overhead when no observer is set
- Feature flags keep optional dependencies (tracing, metrics) out of default builds
- All observer traits use default no-op methods for forward compatibility
- `Reporter<U>` (v2.1.0) is a simpler precursor to `GaObserver<U>` (v2.2.0) — both will coexist

## Constraints

- **Backward compatibility**: LogObserver must reproduce identical log output to current behavior
- **Zero overhead**: `Option::None` branch when no observer — no allocations, no measurements
- **Feature flags**: `observer-tracing` and `observer-metrics` off by default
- **Rust edition**: 2021, MSRV 1.81.0
- **Thread safety**: All observers must be `Send + Sync` (used across rayon threads)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| fitness_std_dev as diversity metric | Simple, allocation-free, one pass over fitness values | ✓ Good — wired cleanly into extension and dynamic mutation |
| Stats computed once per generation, then passed to subsystems | Eliminates duplicate computation; diversity is authoritative | ✓ Good — removed compute_cardinality call from ga.rs |
| `Reporter<U>` as `Option<Box<dyn Reporter<U> + Send>>` | Simpler than Arc for single-owner; Box is fine for non-shared | ✓ Good — zero overhead confirmed via if-let guard |
| plotters 0.3.7 for visualization | Widely used, pure Rust, no C deps, good PNG/SVG support | ✓ Good — compiles cleanly behind feature flag |
| RangeChromosome<i32> for job_scheduling permutation | `list_random_initialization(..., Some(false))` achieves unique IDs | ⚠ Revisit — ListChromosome would be more semantic |
| Facade pattern (log, tracing, metrics crates) | Users choose their own backends; library stays agnostic | — Pending |
| Observer via `Option<Arc<dyn GaObserver<U>>>` | Zero cost when unused, shared across threads | — Pending |
| Default no-op methods on traits | Forward-compatible: new events don't break existing observers | — Pending |

---
*Last updated: 2026-03-28 — Phase 18 complete: Observer API Polish shipped; all v2.2.0 observer gaps closed*
