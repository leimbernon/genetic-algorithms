# Feature Landscape

**Domain:** Observability system for a Rust genetic algorithms library
**Researched:** 2026-03-23
**Confidence:** HIGH (based on direct codebase analysis + strong training knowledge of tracing/metrics/log crate APIs)

---

## Context: What Already Exists

Before mapping features, it is critical to understand what the codebase already provides, because the observer system must replace or wrap these — not duplicate them.

**Existing logging surface (hardcoded `log!()` macros):**

| Log Target | Where Called | Level Used |
|---|---|---|
| `ga_events` | `ga.rs` main loop | `info`, `debug`, `trace` |
| `population_events` | `population.rs` | `debug`, `trace` |
| `selection_events` | All selection operators | `debug`, `trace` |
| `crossover_events` | All crossover operators | `debug` |
| `mutation_events` | All mutation operators | `debug`, `warn` |
| `survivor_events` | All survivor operators | `debug`, `trace` |
| `chromosome_events` | `population.rs` | `debug` |
| `island_events` | `island/mod.rs`, `island/nsga2.rs` | `info`, `debug` |
| `nsga2_events` | `nsga2/mod.rs` | `info`, `debug` |
| `extension_events` | (implicit, for niching/adaptive) | `debug` |

**Existing data the observer can observe:**
- `GenerationStats` (best/worst/avg fitness, std dev, population size, generation number)
- `TerminationCause` (7 variants)
- `Population<U>` (chromosomes, best chromosome)
- Island index, migration events, Pareto front rank/crowding distance

---

## Table Stakes

Features users expect when they see "observability system". Missing these makes the feature feel incomplete.

| Feature | Why Expected | Complexity | Depends On |
|---|---|---|---|
| `GaObserver` trait with default no-op methods | Foundational — all other observers implement this | Low | `GenerationStats`, `TerminationCause`, `ChromosomeT` |
| `on_generation_end` hook | Fundamental telemetry point — every generation emits stats | Low | `GaObserver` |
| `on_run_start` hook | Baseline lifecycle: users need to open spans/timers at start | Low | `GaObserver` |
| `on_run_end` hook | Lifecycle: flush metrics, close spans, log summary | Low | `GaObserver` |
| `on_best_chromosome_updated` hook | Most important observability signal for optimization problems | Low-Med | `GaObserver`, `ChromosomeT` |
| `on_termination` hook | Required to log/record why the run stopped | Low | `GaObserver`, `TerminationCause` |
| `LogObserver` (behind no feature flag) | Drop-in replacement for current hardcoded `log!()` calls | Med | `GaObserver`, `log` crate (already dep) |
| `LogObserver` must reproduce identical log output | Backward compatibility — existing users using `env_logger` must see same messages | Med | Requires audit of all 8 log targets |
| `with_observer()` builder method on `Ga`, `IslandGa`, `Nsga2Ga` | Ergonomic attachment point | Low | `Option<Arc<dyn GaObserver<U>>>` |
| Observer stored as `Option<Arc<dyn GaObserver<U>>>` | Zero overhead when `None`; `Arc` required for `Send + Sync` across rayon | Low | Existing `Arc` usage pattern in codebase |
| All observer trait methods have default no-op impls | Forward compatibility — new events added later don't break existing observers | Low | Trait design |

---

## Differentiators

Features that meaningfully raise the value above baseline. Not expected, but distinguishing.

| Feature | Value Proposition | Complexity | Depends On |
|---|---|---|---|
| `TracingObserver` (behind `observer-tracing` feature flag) | Structured spans via the `tracing` crate — integrates with OpenTelemetry, Jaeger, Honeycomb; users bring their own subscriber | Med | `GaObserver`, `tracing` crate ≥ 0.1 |
| `TracingObserver` uses spans for generation lifecycle | Each generation becomes a `tracing::span!` with fitness fields — enables waterfall views, distributed traces | Med | `TracingObserver` |
| `MetricsObserver` (behind `observer-metrics` feature flag) | Emits counters/gauges/histograms via the `metrics` crate facade — integrates with Prometheus, StatsD, any metrics backend | Med | `GaObserver`, `metrics` crate ≥ 0.21 |
| `MetricsObserver` emits `ga.generation.best_fitness`, `ga.generation.avg_fitness`, `ga.population.size` | Standard metric names let users graph convergence, compare runs, set alerts | Low-Med | `MetricsObserver`, `GenerationStats` |
| `CompositeObserver` for combining multiple observers | Users want both logging and metrics simultaneously — single `with_observer()` call handles it | Low | `GaObserver`, `Vec<Arc<dyn GaObserver<U>>>` |
| `IslandGaObserver` sub-trait with `on_migration` and `on_island_generation_end` hooks | Island model produces island-specific events (migration, per-island stats) that the base trait cannot expose | Med | `GaObserver`, `IslandGa` |
| `Nsga2Observer` sub-trait with `on_pareto_front_updated` and `on_generation_end_nsga2` hooks | NSGA-II has no scalar fitness — needs Pareto front size, hypervolume signal, front rank counts | Med | `GaObserver`, `ParetoFront<U>` |
| `on_operator_event` hook for operator-level tracing | Enables profiling which operators are slow — selection, crossover, mutation timing | High | `GaObserver`, requires operator call-site wrapping |

---

## Anti-Features

Features to explicitly NOT build in this milestone.

| Anti-Feature | Why Avoid | What to Do Instead |
|---|---|---|
| Async observer methods (`async fn on_generation_end`) | `rayon` is sync; `async` traits require `async-trait` crate, leak into user code, and have no value here since GA runs are blocking | Keep all hooks synchronous; users can use channels to bridge to async if needed |
| Bundled metrics backend (Prometheus exporter, StatsD emitter) | Couples the library to a specific ops stack; violates the facade principle that `tracing`/`metrics` crates are built on | `metrics` crate facade pattern — users install their own recorder |
| Bundled tracing subscriber (e.g. `fmt::Subscriber`) | Same reason — library should not own the subscriber pipeline | `tracing` crate facade — users call `tracing_subscriber::fmt().init()` |
| Observer receiving mutable population access | Mutations inside an observer callback violate separation of concerns and are thread-unsafe | Pass only shared references: `&Population<U>`, `&GenerationStats` |
| `on_crossover` / `on_mutation` per-gene hooks | Called millions of times per run; cannot have non-zero-cost hooks at gene level | Aggregate at generation end via `GenerationStats`; per-operator hooks only for tracing opt-in |
| `Box<dyn GaObserver>` (non-`Arc`) | Island model uses rayon across threads; `Box` is not `Sync` | `Arc<dyn GaObserver<U> + Send + Sync>` |
| Removing `with_logs(LogLevel)` config method | That would be a breaking change for existing users | Keep it working; `LogObserver` attaches under the hood when log level is non-Off, or make it opt-in alongside the new observer |
| Observer as a generic type parameter on `Ga<U, O>` | Monomorphizes every Ga type; users cannot swap observers at runtime; breaks existing API | `dyn GaObserver` via `Arc` — dynamic dispatch cost is negligible compared to fitness evaluation |

---

## Feature Dependencies

```
log crate (already dep)
  └── LogObserver (implements GaObserver, no new dep)

tracing crate (new optional dep, observer-tracing feature)
  └── TracingObserver (implements GaObserver)

metrics crate (new optional dep, observer-metrics feature)
  └── MetricsObserver (implements GaObserver)

GaObserver trait
  ├── LogObserver
  ├── TracingObserver (optional)
  ├── MetricsObserver (optional)
  ├── CompositeObserver (wraps Vec<Arc<dyn GaObserver<U>>>)
  └── User-defined observers (downstream)

IslandGaObserver sub-trait
  └── depends on GaObserver + IslandGa run loop
      (on_migration, on_island_generation_end)

Nsga2Observer sub-trait
  └── depends on GaObserver + Nsga2Ga run loop
      (on_pareto_front_updated, on_generation_end_nsga2)

with_observer() builder method
  └── requires GaObserver trait to be defined first
  └── must be added to Ga, IslandGa, Nsga2Ga
```

---

## MVP Recommendation

Prioritize in order:

1. **`GaObserver` trait definition** — all other features are blocked on this; design the full event surface (lifecycle + operator-level) up front even if most hooks start as no-ops
2. **`LogObserver`** — backward-compatible migration of all 8 log targets; no new deps; unblocks removing hardcoded `log!()` calls from operators
3. **`with_observer()` on `Ga`** — makes the trait usable immediately; `IslandGa`/`Nsga2Ga` follow same pattern
4. **`CompositeObserver`** — simple; enables users to compose observers even before tracing/metrics land
5. **`IslandGaObserver` + `Nsga2Observer` sub-traits** — migration and pareto-front hooks are unique to those engines; they cannot reuse the base lifecycle hooks
6. **`TracingObserver`** — medium complexity; high user value for distributed tracing; feature-flagged
7. **`MetricsObserver`** — similar complexity to tracing; feature-flagged

Defer:
- **`on_operator_event` per-operator hooks**: Very high complexity (requires threading observer through every operator factory call), low MVP value. Good candidate for v2.2.
- **Any per-gene hooks**: Explicitly out of scope (anti-feature), see above.

---

## Hook Surface Reference

The following is the minimum complete event surface for `GaObserver`, derived from the existing log call sites:

| Hook | Trigger Point | Data Available |
|---|---|---|
| `on_run_start` | Before generation loop | `&GaConfiguration` |
| `on_generation_end` | End of each generation | `generation: usize`, `&GenerationStats`, `&Population<U>` |
| `on_best_chromosome_updated` | When best chromosome improves | `generation: usize`, best chromosome fitness `f64` |
| `on_termination` | After loop exits | `&TerminationCause`, `total_generations: usize` |
| `on_run_end` | After termination | `&Population<U>`, `&[GenerationStats]` (full history) |

**`IslandGaObserver` additional hooks:**

| Hook | Trigger Point | Data Available |
|---|---|---|
| `on_island_generation_end` | End of each island's generation | `island_index: usize`, `generation: usize`, island population size |
| `on_migration` | After each migration step | `generation: usize`, migrant count |

**`Nsga2Observer` additional hooks:**

| Hook | Trigger Point | Data Available |
|---|---|---|
| `on_generation_end_nsga2` | End of each NSGA-II generation | `generation: usize`, pareto front size, front count |
| `on_pareto_front_updated` | When first front changes | `generation: usize`, `&ParetoFront<U>` |

---

## Metric Names for MetricsObserver

Standard naming convention (following OpenMetrics / Prometheus conventions):

| Metric | Type | Description |
|---|---|---|
| `ga.generation.best_fitness` | Gauge | Best fitness in current generation |
| `ga.generation.worst_fitness` | Gauge | Worst fitness in current generation |
| `ga.generation.avg_fitness` | Gauge | Average fitness in current generation |
| `ga.generation.fitness_std_dev` | Gauge | Fitness standard deviation |
| `ga.generation.population_size` | Gauge | Population size at generation end |
| `ga.run.total_generations` | Counter | Total generations completed on run end |
| `ga.island.migration_count` | Counter | Number of migrants (IslandGA only) |

---

## Sources

- Codebase analysis: `/src/ga.rs`, `/src/island/mod.rs`, `/src/nsga2/mod.rs`, `/src/stats.rs`, all operator files (direct read, HIGH confidence)
- `log` crate 0.4.x API: already a project dependency, well-understood (HIGH confidence)
- `tracing` crate 0.1.x facade pattern: subscriber-separation architecture is a core documented design (HIGH confidence via training data; could not verify current version via web tools)
- `metrics` crate 0.21.x+ facade pattern: recorder-separation mirrors tracing's subscriber pattern (MEDIUM confidence via training data; exact current version unverified)
- PROJECT.md constraints (zero-overhead, `Send + Sync`, MSRV 1.81.0, feature flags): direct read (HIGH confidence)
