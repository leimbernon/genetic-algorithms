# Feature Research

**Domain:** Observability system for a Rust genetic algorithms library
**Researched:** 2026-03-25
**Confidence:** HIGH (direct codebase inspection + verified tracing 0.1.44 and metrics 0.24.3 via docs.rs)

---

## Context: What Already Exists

The observer system is additive. Understanding the existing surface prevents duplication.

**Existing `Reporter<U>` trait (v2.1.0 — stays, does NOT get replaced):**

| Hook | Signature | Notes |
|------|-----------|-------|
| `on_start` | `&mut self` | Before generation loop |
| `on_generation_complete` | `&mut self, &GenerationStats` | After each generation |
| `on_new_best` | `&mut self, generation: usize, best: U` | When best fitness improves (takes ownership of clone) |
| `on_finish` | `&mut self, TerminationCause, &[GenerationStats]` | After loop exits |

`Reporter<U>` uses `&mut self` and is stored as `Option<Box<dyn Reporter<U> + Send>>` — single-threaded ownership. It cannot be shared across island threads. `GaObserver<U>` must use `&self` and `Arc` to satisfy the island model's `rayon` parallelism.

**Existing `GenerationStats` fields available to observers:**
`generation`, `best_fitness`, `worst_fitness`, `avg_fitness`, `fitness_std_dev`, `population_size`, `diversity`

**Existing `TerminationCause` variants:**
`GenerationLimitReached`, `FitnessTargetReached`, `StagnationReached`, `ConvergenceReached`, `TimeLimitReached`, `CallbackRequested`, `NotTerminated`

**Existing hardcoded `log!()` call-site inventory:**

| Log Target | Location | Level | Count |
|---|---|---|---|
| `ga_events` | `ga.rs` main loop | info/debug/trace | ~12 |
| `population_events` | `population.rs` | debug/trace | ~3 |
| `chromosome_events` | `population.rs` | debug | ~1 |
| `selection_events` | All selection operators | debug/trace | ~25 |
| `crossover_events` | All crossover operators | debug/trace | ~20 |
| `mutation_events` | All mutation operators | debug/trace | ~18 |
| `survivor_events` | All survivor operators | debug/trace | ~9 |
| `island_events` | `island/mod.rs` | info/debug | ~4 |
| `nsga2_events` | `nsga2/mod.rs` | info/debug | ~2 |

Total: approximately 94 call sites across 9 targets. `LogObserver` must reproduce all of them to satisfy the backward-compatibility constraint.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist when they see "observability system". Missing these makes the feature feel incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `GaObserver<U>` trait with default no-op methods | Foundation — all other observer types implement this; default no-ops enable forward compatibility without breaking existing observers | LOW | Uses `&self` (not `&mut self`) so it can be stored as `Arc<dyn GaObserver<U>>` for thread-safety |
| `on_run_start` lifecycle hook | Users need to open spans, start timers, or print headers at the start of a run | LOW | Receives `&GaConfiguration` |
| `on_generation_complete` lifecycle hook | Primary telemetry point — most users only care about per-generation fitness stats | LOW | Receives `generation: usize`, `&GenerationStats` |
| `on_new_best` lifecycle hook | Most important optimization signal — tells users when the algorithm improves | LOW | Receives `generation: usize`, best fitness `f64` (no chromosome clone — avoids allocation in hot path) |
| `on_run_complete` lifecycle hook | Required for flushing metrics, closing spans, printing final summary | LOW | Receives `&TerminationCause`, `total_generations: usize`, `&[GenerationStats]` |
| `with_observer()` builder method on `Ga<U>` | Ergonomic attachment — consistent with `with_reporter()` already on `Ga<U>` | LOW | Accepts `Arc<dyn GaObserver<U> + Send + Sync>` |
| Observer stored as `Option<Arc<dyn GaObserver<U> + Send + Sync>>` | Zero overhead when `None`; `Arc` required for sharing across island rayon threads | LOW | Pattern mirrors `Option<Arc<FitnessFn>>` already used in `Ga<U>` |
| `LogObserver` behind no feature flag | Drop-in replacement for current hardcoded `log!()` calls; backward-compatible migration for all 9 log targets | MEDIUM | Depends on `log` crate already in `[dependencies]`; must reproduce identical log output for all 94 call sites |
| `LogObserver` must reproduce identical log output | Backward compatibility — existing users using `env_logger` must see same messages with same targets and levels | MEDIUM | Requires careful audit of all 9 log targets; any deviation is a regression |
| `with_observer()` on `IslandGa<U>` and `Nsga2Ga<U>` | Consistency — users expect all three GA modes to support observers | LOW | Each orchestrator adds one field and one builder method |
| All `GaObserver` methods have default no-op bodies | Forward compatibility — new event hooks added in later versions do not break existing `GaObserver` implementations | LOW | Rust trait default methods; same design as existing `Reporter<U>` |

### Differentiators (Competitive Advantage)

Features that meaningfully raise the value above baseline.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| `TracingObserver` behind `observer-tracing` feature flag | Structured spans via `tracing` 0.1.44 — integrates with OpenTelemetry, Jaeger, Honeycomb, and any `tracing` subscriber; each generation becomes a named span with fitness fields | MEDIUM | Depends on `tracing = "0.1"` (MSRV 1.65, compatible with project MSRV 1.81.0); users bring their own subscriber |
| `TracingObserver` maps generation lifecycle to spans | `span!("generation", gen=%i, best=%fitness)` enables waterfall views, per-generation duration, and distributed trace integration | MEDIUM | `on_generation_complete` opens and closes the span; fields set from `GenerationStats` |
| `MetricsObserver` behind `observer-metrics` feature flag | Emits counters/gauges/histograms via `metrics` 0.24.3 facade — integrates with Prometheus, StatsD, any metrics backend; users install their own recorder | MEDIUM | Depends on `metrics = "0.24"` (MSRV ~1.70, compatible with project MSRV 1.81.0); users bring their own recorder |
| `MetricsObserver` emits standard GA metric names | `ga.generation.best_fitness`, `ga.generation.avg_fitness`, `ga.population.size`, etc. — standard names let users graph convergence, compare runs, set alerts | LOW | Metric names follow OpenMetrics/Prometheus conventions |
| `CompositeObserver` for combining multiple observers | Users want logging AND metrics AND tracing simultaneously — one `with_observer()` call handles all; fan-out with no additional overhead | LOW | Pure Rust: `Vec<Arc<dyn GaObserver<U>>>`, no new deps |
| `IslandGaObserver` sub-trait with `on_migration` and `on_island_generation_end` hooks | Island model produces events (migration, per-island stats) the base trait cannot expose; sub-trait pattern preserves the single observer attachment point | MEDIUM | `IslandGaObserver: GaObserver` — a single `Arc<dyn IslandGaObserver<U>>` satisfies both traits |
| `Nsga2Observer` sub-trait with `on_pareto_front_assigned` hook | NSGA-II has no scalar fitness — Pareto front sizes, front count, and crowding distance are the meaningful signals; base trait is insufficient | MEDIUM | `Nsga2Observer: GaObserver` — same sub-trait pattern as island; receives front size counts per generation |
| Operator-level hooks (`on_selection_complete`, `on_crossover_complete`, `on_survivor_selection_complete`) on `GaObserver` | Enables profiling which operator phase is slowest; useful for benchmarking operator configurations; called from the sequential driver loop so overhead is bounded | MEDIUM | These hooks are in the base trait with default no-ops; called once per generation, not per chromosome |
| `GaObserver::on_extension_triggered` hook | Extension events (mass extinction, mass degeneration, mass genesis) are already logged with `info!(target="extension_events", ...)` — surfacing them in the observer enables alerting on diversity collapse | LOW | Triggered once per invocation of the extension strategy; receives `diversity: f64` and extension method |

### Anti-Features (Commonly Requested, Often Problematic)

| Anti-Feature | Why Avoid | What to Do Instead |
|---|---|---|
| Async observer methods (`async fn on_generation_complete`) | `rayon` is sync; `async` traits require `async-trait` crate, add `Pin<Box<Future>>` overhead, and have no value since GA runs are blocking | Keep all hooks synchronous; users can bridge to async via `tokio::task::spawn_blocking` or `std::sync::mpsc` channels |
| Bundled metrics backend (Prometheus exporter, StatsD emitter) | Couples the library to a specific ops stack; violates the facade principle that `metrics` 0.24 is built on | `metrics` crate facade — users install their own recorder (e.g. `metrics-exporter-prometheus`) |
| Bundled tracing subscriber (e.g. `tracing_subscriber::fmt`) | Same vendor coupling problem; library should emit but not route | `tracing` crate facade — users call `tracing_subscriber::fmt().init()` independently |
| Observer receiving mutable population access | Passing `&mut Population<U>` to observer methods violates separation of concerns; an observer mutating the population would cause undefined GA behavior | Pass only shared references: `&GenerationStats`, `&TerminationCause`; never pass `&mut Population<U>` |
| Per-gene observer hooks (`on_gene_mutated`, `on_gene_selected`) | Called millions of times per run; any non-zero-cost hook at gene level makes the observer unusable for non-trivial populations | Aggregate at generation level via `GenerationStats`; per-operator hooks aggregate over all genes in one call |
| `Box<dyn GaObserver>` (single ownership) | `IslandGa<U>` needs to share one observer across parallel island threads; `Box` cannot be shared without `Arc` wrapping | `Arc<dyn GaObserver<U> + Send + Sync>` — same cost as `Box` when not shared; enables safe cross-thread sharing |
| Observer as a generic type parameter `Ga<U, O: GaObserver<U>>` | Monomorphizes every `Ga<U>` type; users cannot swap observers at runtime; breaks existing `Ga<U>` public API | `dyn GaObserver` via `Arc` — dynamic dispatch overhead is negligible compared to fitness evaluation (called once per generation) |
| Replacing `Reporter<U>` with `GaObserver<U>` | `Reporter<U>` shipped in v2.1.0 and is public API; removing it is a breaking change; existing users have implemented it | Both coexist; `Reporter<U>` fires from `Ga<U>` only; `GaObserver<U>` fires from all three engines; users can migrate at their own pace |
| Removing `with_logs(LogLevel)` config method | Breaking change for existing users | Keep working; `LogObserver` attaches under the hood when log level is non-Off, or let users attach it manually alongside the new observer |

---

## Feature Dependencies

```
Reporter<U> (v2.1.0 — existing, unchanged)
  └── fires from Ga<U> only (on_start, on_generation_complete, on_new_best, on_finish)

GaObserver<U> trait (new — v2.2.0 foundation)
  ├── requires: GenerationStats (exists), TerminationCause (exists), GaConfiguration (exists)
  ├── LogObserver
  │     └── requires: log crate (already dep), GaObserver trait
  ├── CompositeObserver
  │     └── requires: GaObserver trait only
  ├── TracingObserver  [feature: observer-tracing]
  │     └── requires: tracing 0.1.44 (new optional dep), GaObserver trait
  └── MetricsObserver  [feature: observer-metrics]
        └── requires: metrics 0.24.3 (new optional dep), GaObserver trait

IslandGaObserver sub-trait
  └── requires: GaObserver trait, IslandGa run loop (exists)
  └── observers implementing IslandGaObserver automatically satisfy GaObserver

Nsga2Observer sub-trait
  └── requires: GaObserver trait, Nsga2Ga run loop (exists), ParetoFront type (exists)
  └── observers implementing Nsga2Observer automatically satisfy GaObserver

with_observer() on Ga<U>
  └── requires: GaObserver trait to be defined first

with_observer() on IslandGa<U>
  └── requires: IslandGaObserver sub-trait (accepts IslandGaObserver or plain GaObserver)

with_observer() on Nsga2Ga<U>
  └── requires: Nsga2Observer sub-trait (accepts Nsga2Observer or plain GaObserver)
```

### Dependency Notes

- **`GaObserver` trait must be defined before any other observer work:** All concrete implementations (LogObserver, TracingObserver, MetricsObserver, CompositeObserver) and all engine integrations (Ga, IslandGa, Nsga2Ga) depend on the base trait. Design the full hook surface up front — adding hooks later is safe (default no-ops), but removing or renaming hooks is a breaking change.
- **`LogObserver` has no new dependencies:** It uses the `log` crate which is already in `[dependencies]`. This makes it the safest first concrete implementation.
- **Sub-traits (`IslandGaObserver`, `Nsga2Observer`) are supersets of `GaObserver`:** A user implementing `IslandGaObserver` gets all base lifecycle hooks for free via the sub-trait relationship. The orchestrators each store a typed observer: `Option<Arc<dyn IslandGaObserver<U>>>` (not the base `GaObserver<U>`), so island-specific hooks fire naturally.
- **`CompositeObserver` requires all inner observers to implement `GaObserver<U>`:** It cannot compose `IslandGaObserver` and `Nsga2Observer` instances in the same composite — they are separate hierarchies for separate engines.
- **`Reporter<U>` and `GaObserver<U>` coexist without conflict:** Both are stored as separate `Option` fields on `Ga<U>`. They fire independently. `Reporter<U>` is not replaced.

---

## MVP Definition

### Launch With (v2.2.0)

Minimum viable product — what is needed to deliver "Observability & Traceability" milestone.

- [ ] `GaObserver<U>` trait with complete hook surface and default no-ops — everything else is blocked on this
- [ ] `LogObserver` — backward-compatible migration of all 9 log targets; unblocks removing hardcoded `log!()` calls; no new deps
- [ ] `with_observer()` on `Ga<U>` — makes `GaObserver` usable immediately on the primary engine
- [ ] `with_observer()` on `IslandGa<U>` and `Nsga2Ga<U>` — consistency across all three GA modes
- [ ] `CompositeObserver` — simple fan-out; enables combining LogObserver + user-defined observer in one `with_observer()` call
- [ ] `IslandGaObserver` sub-trait with `on_migration` and `on_island_generation_end` — island model has unique events that the base trait cannot surface
- [ ] `Nsga2Observer` sub-trait with `on_pareto_front_assigned` — NSGA-II has no scalar fitness; Pareto front signal is the meaningful hook
- [ ] `TracingObserver` behind `observer-tracing` feature flag — highest-value differentiator; structured span integration with the `tracing` ecosystem

### Add After Validation (v2.2.x)

Features to add once the core observer system is working and validated.

- [ ] `MetricsObserver` behind `observer-metrics` feature flag — similar complexity to TracingObserver; add when TracingObserver pattern is proven
- [ ] `on_extension_triggered` hook — surfacing extension events is low complexity; add when LogObserver migration is complete and the hook surface is stable

### Future Consideration (v2.3+)

- [ ] Per-operator timing hooks (`on_selection_start` / `on_selection_complete` with elapsed `Duration`) — useful for benchmarking operator configurations; deferred because operator call-sites are deep in factory functions and would require threading `Arc<dyn GaObserver>` through every operator invocation
- [ ] `on_checkpoint_saved` hook — wraps the serde checkpoint event; low priority since checkpointing is already working and the event is not critical for observability

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| `GaObserver<U>` trait definition | HIGH | LOW | P1 |
| `LogObserver` (backward compat) | HIGH | MEDIUM | P1 |
| `with_observer()` on all three engines | HIGH | LOW | P1 |
| `CompositeObserver` | MEDIUM | LOW | P1 |
| `IslandGaObserver` sub-trait | HIGH | MEDIUM | P1 |
| `Nsga2Observer` sub-trait | HIGH | MEDIUM | P1 |
| `TracingObserver` (`observer-tracing`) | HIGH | MEDIUM | P1 |
| `MetricsObserver` (`observer-metrics`) | MEDIUM | MEDIUM | P2 |
| `on_extension_triggered` hook | LOW | LOW | P2 |
| Per-operator start/complete hooks with timing | MEDIUM | HIGH | P3 |
| `on_checkpoint_saved` hook | LOW | LOW | P3 |

**Priority key:**
- P1: Must have for v2.2.0 launch
- P2: Should have, add in v2.2.x
- P3: Nice to have, v2.3+ consideration

---

## Hook Surface Reference

Complete event surface for `GaObserver<U>`, derived from existing `log!()` call sites in `ga.rs`, `island/mod.rs`, and `nsga2/mod.rs`:

**Base `GaObserver<U>` hooks:**

| Hook | Trigger Point | Data Available | Complexity |
|------|---------------|----------------|------------|
| `on_run_start` | Before generation loop | `&GaConfiguration` | LOW |
| `on_generation_start` | Start of each generation | `generation: usize` | LOW |
| `on_selection_complete` | After `selection::factory` | `generation: usize`, parent pairs count | LOW |
| `on_crossover_complete` | After `parent_crossover` | `generation: usize`, offspring count | LOW |
| `on_survivor_selection_complete` | After `survivor::factory` | `generation: usize`, surviving population size | LOW |
| `on_generation_complete` | After stats are collected | `generation: usize`, `&GenerationStats` | LOW |
| `on_new_best` | When best fitness improves | `generation: usize`, best fitness `f64` | LOW |
| `on_extension_triggered` | When diversity threshold triggers extension | `generation: usize`, `diversity: f64`, extension method name | LOW |
| `on_run_complete` | After loop exits | `&TerminationCause`, `total_generations: usize`, `&[GenerationStats]` | LOW |

**`IslandGaObserver<U>` additional hooks (sub-trait of `GaObserver<U>`):**

| Hook | Trigger Point | Data Available |
|------|---------------|----------------|
| `on_island_initialized` | After each island is initialized | `island_index: usize`, `population_size: usize` |
| `on_island_generation_end` | End of each island's generation | `island_index: usize`, `generation: usize` |
| `on_migration` | After each migration step | `generation: usize`, `migrant_count: usize` |

**`Nsga2Observer<U>` additional hooks (sub-trait of `GaObserver<U>`):**

| Hook | Trigger Point | Data Available |
|------|---------------|----------------|
| `on_generation_complete_nsga2` | End of each NSGA-II generation | `generation: usize`, `pareto_front_size: usize`, `front_count: usize` |
| `on_pareto_front_assigned` | After non-dominated sorting assigns fronts | `generation: usize`, `front_sizes: &[usize]` |

---

## TracingObserver: How GA Events Map to Spans and Fields

The `tracing` crate models instrumentation as spans (bounded time ranges) and events (point-in-time). The mapping:

| Observer Hook | Tracing Primitive | Span/Event Fields |
|---|---|---|
| `on_run_start` | `span!(Level::INFO, "ga_run")` opened | `max_generations`, `population_size`, `problem_solving` |
| `on_generation_start` | `span!(Level::DEBUG, "ga_generation", gen=%generation)` opened | `generation` |
| `on_generation_complete` | Span closed; `event!` with stats | `best_fitness`, `avg_fitness`, `diversity`, `population_size` |
| `on_new_best` | `event!(Level::INFO, "new_best", gen=%generation, fitness=%best)` | `generation`, `best_fitness` |
| `on_run_complete` | `ga_run` span closed; event with cause | `termination_cause`, `total_generations` |
| `on_migration` (island) | `event!(Level::DEBUG, "migration", gen=%generation, migrants=%count)` | `generation`, `migrant_count` |
| `on_pareto_front_assigned` (nsga2) | `event!(Level::DEBUG, "pareto_front", gen=%generation, size=%size)` | `generation`, `front_sizes` |

The `ga_run` span wraps the entire run. The `ga_generation` span wraps each generation. This nesting enables waterfall views in Jaeger/Honeycomb when a compatible subscriber is installed. No tracing code compiles into default builds (behind `#[cfg(feature = "observer-tracing")]`).

---

## MetricsObserver: Standard Metric Names

Naming follows OpenMetrics / Prometheus conventions (dot-separated hierarchy):

| Metric Name | Type | Emitted At | Description |
|---|---|---|---|
| `ga.generation.best_fitness` | Gauge | `on_generation_complete` | Best fitness in current generation |
| `ga.generation.worst_fitness` | Gauge | `on_generation_complete` | Worst fitness in current generation |
| `ga.generation.avg_fitness` | Gauge | `on_generation_complete` | Average fitness across population |
| `ga.generation.fitness_std_dev` | Gauge | `on_generation_complete` | Fitness standard deviation (= diversity) |
| `ga.generation.population_size` | Gauge | `on_generation_complete` | Population size at generation end |
| `ga.run.total_generations` | Counter | `on_run_complete` | Total generations completed in this run |
| `ga.island.migration_count` | Counter | `on_migration` (island) | Number of migrants at this migration step |
| `ga.pareto.front_size` | Gauge | `on_pareto_front_assigned` (nsga2) | First Pareto front size |

---

## Reporter<U> vs GaObserver<U>: Coexistence Contract

Both traits fire from `Ga<U>`. They are separate fields and separate concerns:

| Aspect | `Reporter<U>` (v2.1.0) | `GaObserver<U>` (v2.2.0) |
|--------|------------------------|--------------------------|
| Self mutability | `&mut self` — can accumulate state | `&self` — immutable; state via `Arc<Mutex<>>` if needed |
| Storage | `Option<Box<dyn Reporter<U> + Send>>` | `Option<Arc<dyn GaObserver<U> + Send + Sync>>` |
| Thread sharing | Single-owner only | Shareable across rayon threads |
| Available on | `Ga<U>` only | `Ga<U>`, `IslandGa<U>`, `Nsga2Ga<U>` |
| Migration intent | Keep as-is indefinitely | New preferred API going forward |
| Breaking change risk | None — unchanged | None — additive |

Users who implemented `Reporter<U>` do not need to migrate. New users should prefer `GaObserver<U>`.

---

## Sources

- `src/ga.rs` — `run_with_callback` full implementation, `Reporter<U>` call sites, `TerminationCause` definition (direct inspection, HIGH confidence)
- `src/reporter/mod.rs` — `Reporter<U>` trait contract and signature (direct inspection, HIGH confidence)
- `src/reporter/simple.rs`, `src/reporter/noop.rs`, `src/reporter/duration.rs` — existing built-in reporters (direct inspection, HIGH confidence)
- `src/island/mod.rs` — `IslandGa::run`, migration events, log targets (direct inspection, HIGH confidence)
- `src/nsga2/mod.rs` — `Nsga2Ga::run`, Pareto front generation events (direct inspection, HIGH confidence)
- `src/stats.rs` — `GenerationStats` struct, all available fields (direct inspection, HIGH confidence)
- `Cargo.toml` — existing dependencies, MSRV 1.81.0, existing feature flag pattern (direct inspection, HIGH confidence)
- `.planning/PROJECT.md` — zero-overhead constraint, backward compat, feature flag names, observer coexistence decision (direct read, HIGH confidence)
- `tracing` crate v0.1.44: https://docs.rs/tracing/latest/tracing/ (verified via WebFetch, HIGH confidence)
- `metrics` crate v0.24.3: https://docs.rs/metrics/latest/metrics/ (verified via WebFetch, HIGH confidence)

---
*Feature research for: Observability & Traceability system (v2.2.0)*
*Researched: 2026-03-25*
