# genetic_algorithms

## What This Is

A modular, concurrent Genetic Algorithms library for Rust. Provides composable operators (selection, crossover, mutation, survivor), multi-threaded execution via `rayon`, Island Model GA, NSGA-II multi-objective optimization, adaptive GA mode, elitism/stopping criteria, population diversity tracking, a `Reporter<U>` lifecycle trait, an optional visualization feature (PNG/SVG charts), a `List<T>` genotype for finite symbolic alphabets, and a full `GaObserver<U>` trait system with `LogObserver`, `TracingObserver`, `CompositeObserver`, and `MetricsObserver`. Published on crates.io as `genetic_algorithms` with ten runnable examples covering every major GA mode.

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
- ✓ `GaObserver<U>` trait with 12 lifecycle/operator/event hooks, zero overhead via `Option<Arc<dyn GaObserver>>` — v2.2.0
- ✓ `LogObserver` reproducing all pre-v2.2.0 `log!()` output; all hardcoded log calls removed from `ga.rs` — v2.2.0
- ✓ `TracingObserver` behind `observer-tracing` feature flag — structured `tracing` spans per generation — v2.2.0
- ✓ `IslandGaObserver<U>` and `Nsga2Observer<U>` sub-traits wired into Island GA and NSGA-II run loops — v2.2.0
- ✓ `CompositeObserver<U>` fan-out + `AllObserver` blanket impl — v2.2.0
- ✓ `MetricsObserver` behind `observer-metrics` feature flag — 11 per-generation metrics via `metrics` facade — v2.2.0

### Active

<!-- Current scope. Building toward these. Next milestone TBD. -->

(none — planning next milestone)

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- GUI/interactive visualization — library generates static PNG/SVG charts; interactive dashboards are users' concern
- Specific telemetry backends (Prometheus, Jaeger) — facade pattern lets users pick
- Per-gene hooks in observer — too granular, unacceptable overhead in hot loops

## Context

- Library is published on crates.io; backward compatibility matters
- v2.2.0 shipped: ~15,000 LOC Rust, 10 runnable examples, full observer system, `observer-tracing` and `observer-metrics` feature flags
- `Reporter<U>` (v2.1.0) coexists with `GaObserver<U>` (v2.2.0) — soft-deprecated but not removed
- All observer traits use default no-op methods for forward compatibility
- Feature flags keep optional dependencies (`tracing`, `metrics`) out of default builds

## Constraints

- **Backward compatibility**: No breaking changes without major version bump
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
| Facade pattern (log, tracing, metrics crates) | Users choose their own backends; library stays agnostic | ✓ Good — confirmed working in v2.2.0 |
| Observer via `Option<Arc<dyn GaObserver<U>>>` | Zero cost when unused, shared across rayon threads | ✓ Good — zero overhead confirmed, `Send+Sync` enforced at compile time |
| Default no-op methods on GaObserver hooks | Forward-compatible: new events don't break existing observers | ✓ Good — all observers implement only what they need |
| `AllObserver<U>` blanket impl | Any type implementing all three observer traits gets AllObserver automatically | ✓ Good — TracingObserver composability gap closed via empty impls |
| Extension block before on_generation_end | Extension is part of the generation, not a post-generation side effect | ✓ Good — restores pre-v2.2.0 hook ordering semantics |
| Combined elapsed for operator timing (EXT-01 deferred) | Single timing block covers crossover+mutation+fitness; per-operator separation is v2.3+ | — Pending (EXT-01) |

---
*Last updated: 2026-03-28 — v2.2.0 complete: Observability & Traceability milestone shipped*
