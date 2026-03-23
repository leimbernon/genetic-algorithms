# genetic_algorithms

## What This Is

A modular, concurrent Genetic Algorithms library for Rust. Provides composable operators (selection, crossover, mutation, survivor), multi-threaded execution via `rayon`, Island Model GA, NSGA-II multi-objective optimization, adaptive GA mode, and elitism/stopping criteria. Published on crates.io as `genetic_algorithms`.

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

### Active

<!-- Current scope. Building toward these. -->

- [ ] GaObserver trait with lifecycle, operator, and special event hooks (#182)
- [ ] LogObserver replacing hardcoded log!() calls (#183)
- [ ] TracingObserver behind `observer-tracing` feature flag (#184)
- [ ] Island GA and NSGA-II specialized observer sub-traits (#185)
- [ ] CompositeObserver for combining multiple observers (#186)
- [ ] MetricsObserver behind `observer-metrics` feature flag (#186)

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- GUI/visualization — library focus, users choose their own frontend
- Specific telemetry backends (Prometheus, Jaeger) — facade pattern lets users pick

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
- Current logging uses hardcoded `log!()` macros with 8 log targets in ga.rs
- Observer pattern must have zero overhead when no observer is set
- Feature flags keep optional dependencies (tracing, metrics) out of default builds
- All observer traits use default no-op methods for forward compatibility

## Constraints

- **Backward compatibility**: LogObserver must reproduce identical log output to current behavior
- **Zero overhead**: `Option::None` branch when no observer — no allocations, no measurements
- **Feature flags**: `observer-tracing` and `observer-metrics` off by default
- **Rust edition**: 2021, MSRV 1.81.0
- **Thread safety**: All observers must be `Send + Sync` (used across rayon threads)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Facade pattern (log, tracing, metrics crates) | Users choose their own backends; library stays agnostic | — Pending |
| Observer via `Option<Arc<dyn GaObserver<U>>>` | Zero cost when unused, shared across threads | — Pending |
| Default no-op methods on traits | Forward-compatible: new events don't break existing observers | — Pending |

---
*Last updated: 2026-03-23 after milestone v2.2.0 initialization*
