# genetic_algorithms

## What This Is

A modular, concurrent Genetic Algorithms library for Rust. Provides composable operators (selection, crossover, mutation, survivor), multi-threaded execution via `rayon`, Island Model GA, NSGA-II multi-objective optimization, adaptive GA mode, elitism/stopping criteria, population diversity tracking, a `Reporter<U>` lifecycle trait, an optional visualization feature (PNG/SVG charts), a `List<T>` genotype for finite symbolic alphabets, and a full `GaObserver<U>` trait system with `LogObserver`, `TracingObserver`, `CompositeObserver`, and `MetricsObserver`. Published on crates.io as `genetic_algorithms` with ten runnable examples covering every major GA mode.

## Core Value

Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library.

## Last Milestone: v2.2.1 — Performance Optimizations (Shipped 2026-04-23)

Eliminated unnecessary heap allocations, reduced algorithmic complexity, and improved concurrency across the GA engine. All 24 requirements closed. No public API changes.

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
- ✓ Zero-allocation crossover and mutation: parent clones deferred/eliminated; PMX and OX upgraded to O(n) position maps — v2.2.1
- ✓ Rank/Boltzmann selection upgraded to O(log n) binary search; fitness values collected once per generation — v2.2.1
- ✓ Fitness sharing O(n) on-the-fly (eliminates O(n²) distance matrix); elite reinsertion and mass genesis use `select_nth_unstable_by()` — v2.2.1
- ✓ RNG atomics relaxed to Acquire/Relaxed; extension regrow parallelized with rayon — v2.2.1
- ✓ `Range` gene uses `Arc<[(T,T)]>` shared slice; `Copy`-specialized `value()`; `MassDeduplication` uses incremental `DefaultHasher` — v2.2.1
- ✓ `GenerationStats` moved (not cloned); truncation and island migration use O(n) partitioning; `Arc` migrant sharing — v2.2.1

### Active

<!-- Current scope. Building toward these. -->

(Planning next milestone — see /gsd:new-milestone)

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- GUI/interactive visualization — library generates static PNG/SVG charts; interactive dashboards are users' concern
- Specific telemetry backends (Prometheus, Jaeger) — facade pattern lets users pick
- Per-gene hooks in observer — too granular, unacceptable overhead in hot loops
- Public API changes in v2.2.1 — this is a pure internal optimization patch

## Context

- Library is published on crates.io; backward compatibility matters
- v2.2.0 shipped: ~15,000 LOC Rust, 10 runnable examples, full observer system, `observer-tracing` and `observer-metrics` feature flags
- `Reporter<U>` (v2.1.0) coexists with `GaObserver<U>` (v2.2.0) — soft-deprecated but not removed
- All observer traits use default no-op methods for forward compatibility
- Feature flags keep optional dependencies (`tracing`, `metrics`) out of default builds
- GitHub milestone #6 tracks all 6 performance issues (#187–#192)

## Constraints

- **Backward compatibility**: No breaking changes without major version bump
- **Zero overhead**: `Option::None` branch when no observer — no allocations, no measurements
- **Feature flags**: `observer-tracing` and `observer-metrics` off by default
- **Rust edition**: 2021, MSRV 1.81.0
- **Thread safety**: All observers must be `Send + Sync` (used across rayon threads)
- **No API changes**: v2.2.1 is a patch release — all optimizations must be purely internal

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
| `Arc<[(T,T)]>` for Range gene ranges | Constructed once, read-many — Arc eliminates per-gene heap allocation; serde rc feature enables deserialization | ✓ Good — zero-copy sharing across crossover/mutation |
| `Copy` bound on Range gene impls | All concrete Range users (f64, i32) satisfy Copy; enables value-return without clone in hot accessor | ✓ Good — backward compatible (Copy implies Clone) |
| On-the-fly niching (apply_fitness_sharing_with_dna) | Old O(n²) matrix left intact (no breaking change); new function replaces it in ga.rs loop only | ✓ Good — safe incremental opt, old API preserved |
| `Acquire`/`Relaxed` RNG atomics | Acquire on SEED.load pairs with Release on set_seed — correct visibility; COUNTER only needs monotonic uniqueness | ✓ Good — minimal ordering, correct under Rust memory model |
| `Arc<Vec<U>>` for island migrant sharing | Neighbors auto-deref through Arc; single allocation shared across topology — no per-neighbor clone | ✓ Good — measurable reduction for high-connectivity topologies |

## Context

- Library is published on crates.io; backward compatibility matters
- v2.2.1 shipped: ~13,882 LOC Rust (src/), 10 runnable examples, full observer system + performance optimizations
- `Reporter<U>` (v2.1.0) coexists with `GaObserver<U>` (v2.2.0) — soft-deprecated but not removed
- All observer traits use default no-op methods for forward compatibility
- Feature flags keep optional dependencies (`tracing`, `metrics`) out of default builds
- GitHub milestones: #1–#5 shipped; next milestone candidates: New Operators (#196–#202), Advanced Multi-Objective (#203–#207)

---
*Last updated: 2026-04-23 — v2.2.1 milestone shipped: 6 phases, 13 plans, 24/24 requirements closed. All performance optimizations complete.*
