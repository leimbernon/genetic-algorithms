# Architecture Research

**Domain:** GaObserver observability system for genetic_algorithms Rust library
**Researched:** 2026-03-25
**Confidence:** HIGH (based on direct source reading + GitHub issues #182-#186)

## Standard Architecture

### System Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                        User / Application                           │
│  Ga<U>::with_observer(Arc<dyn GaObserver<U>>)                       │
│  IslandGa<U>::with_observer(...)                                    │
│  Nsga2Ga<U>::with_observer(...)                                     │
└──────────────────────────────┬─────────────────────────────────────┘
                               │  Option<Arc<dyn GaObserver<U>>>
                               ▼
┌────────────────────────────────────────────────────────────────────┐
│                    Observer Trait Hierarchy                          │
│                                                                     │
│  GaObserver<U>  (base trait — src/observer/mod.rs)                  │
│  ├── lifecycle:  on_run_start, on_run_end                           │
│  ├── per-gen:    on_generation_start, on_generation_end             │
│  ├── operators:  on_selection_complete, on_crossover_complete,      │
│  │               on_mutation_complete, on_fitness_evaluation_complete│
│  │               on_survivor_selection_complete                     │
│  └── special:    on_best_chromosome_updated, on_extension_triggered │
│                  on_niching_applied, on_elitism_applied              │
│                  on_stagnation_detected, on_convergence_detected     │
│                  on_adaptive_parameters_updated                      │
│                  on_checkpoint_saved, on_checkpoint_failed           │
│                                                                     │
│  IslandObserver<U>  : GaObserver<U>  (src/observer/island.rs)       │
│  ├── on_migration_start, on_migration_complete                      │
│  ├── on_island_generation_end                                       │
│  └── on_island_run_start, on_island_run_end                         │
│                                                                     │
│  Nsga2Observer<U>  : GaObserver<U>  (src/observer/nsga2.rs)         │
│  ├── on_non_dominated_sort_complete                                 │
│  ├── on_crowding_distance_calculated                                │
│  └── on_pareto_front_updated                                        │
└──────────────────────────────┬─────────────────────────────────────┘
                               │  implemented by
                               ▼
┌────────────────────────────────────────────────────────────────────┐
│                     Concrete Observers                               │
│                                                                     │
│  LogObserver       — log crate, replicates existing 8 targets       │
│  TracingObserver   — tracing crate spans (feature: observer-tracing)│
│  MetricsObserver   — metrics crate gauges/histograms (feature:      │
│                       observer-metrics)                             │
│  NoopObserver      — zero-cost default, all methods compile away    │
│  CompositeObserver — Vec<Arc<dyn GaObserver<U>>>, fan-out           │
└──────────────────────────────┬─────────────────────────────────────┘
                               │  notified by
                               ▼
┌────────────────────────────────────────────────────────────────────┐
│                 Instrumented Execution Loops                         │
│                                                                     │
│  src/ga.rs          (Ga<U>::run_with_callback)                      │
│  src/island/mod.rs  (IslandGa<U>::run, evolve_islands_one_gen)      │
│  src/nsga2/mod.rs   (Nsga2Ga<U>::run)                               │
└────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Implementation File |
|-----------|---------------|---------------------|
| `GaObserver<U>` trait | Base contract for all observability hooks; all methods have default no-op bodies | `src/observer/mod.rs` (new) |
| `IslandObserver<U>` trait | Sub-trait extending `GaObserver<U>` with migration and per-island events | `src/observer/island.rs` (new) |
| `Nsga2Observer<U>` trait | Sub-trait extending `GaObserver<U>` with Pareto-sorting and crowding events | `src/observer/nsga2.rs` (new) |
| `ExtensionEvent` struct | Typed payload for `on_extension_triggered` | `src/observer/mod.rs` (new) |
| `LogObserver` | Maps each hook to the matching `log!()` call with the correct target string | `src/observer/log_observer.rs` (new) |
| `TracingObserver` | Maps each hook to a `tracing` span or event; gated behind `observer-tracing` | `src/observer/tracing_observer.rs` (new) |
| `MetricsObserver` | Emits counters/gauges/histograms via `metrics` facade; gated behind `observer-metrics` | `src/observer/metrics_observer.rs` (new) |
| `NoopObserver` | Empty struct; used as compile-away default type | `src/observer/mod.rs` (new) |
| `CompositeObserver<U>` | Holds `Vec<Arc<dyn GaObserver<U>>>`, fans-out all method calls | `src/observer/composite.rs` (new) |
| `Ga<U>` (modified) | Gains `observer: Option<Arc<dyn GaObserver<U>>>` field and `with_observer()` builder | `src/ga.rs` (modified) |
| `IslandGa<U>` (modified) | Gains `observer: Option<Arc<dyn IslandObserver<U>>>` field and `with_observer()` builder | `src/island/mod.rs` (modified) |
| `Nsga2Ga<U>` (modified) | Gains `observer: Option<Arc<dyn Nsga2Observer<U>>>` field and `with_observer()` builder | `src/nsga2/mod.rs` (modified) |

## Recommended Project Structure

```
src/
├── observer/                   # new module — all observer concerns
│   ├── mod.rs                  # GaObserver<U> trait, ExtensionEvent, NoopObserver, re-exports
│   ├── island.rs               # IslandObserver<U> sub-trait
│   ├── nsga2.rs                # Nsga2Observer<U> sub-trait
│   ├── log_observer.rs         # LogObserver — always compiled
│   ├── composite.rs            # CompositeObserver<U>
│   ├── tracing_observer.rs     # TracingObserver — cfg(feature="observer-tracing")
│   └── metrics_observer.rs     # MetricsObserver — cfg(feature="observer-metrics")
├── ga.rs                       # modified: +observer field, +with_observer(), +notify()
├── island/mod.rs               # modified: +observer field, +with_observer(), +notify()
├── nsga2/mod.rs                # modified: +observer field, +with_observer(), +notify()
└── lib.rs                      # modified: pub mod observer; re-export types
```

### Structure Rationale

- **`src/observer/`**: Single dedicated module keeps observer logic isolated from execution logic. Mirrors the existing `src/reporter/` module design pattern established in v2.1.0.
- **Separation of sub-traits**: `island.rs` and `nsga2.rs` are separate files so callers only import what they need. They extend `GaObserver<U>` via `trait IslandObserver<U>: GaObserver<U>`, so a single `LogObserver` can implement all three.
- **Feature-gated files**: `tracing_observer.rs` and `metrics_observer.rs` exist as full files rather than inline cfg blocks — cleaner than sprinkling `#[cfg]` everywhere.

## Architectural Patterns

### Pattern 1: Arc-based Shared Observer

**What:** Store the observer as `Option<Arc<dyn GaObserver<U> + Send + Sync>>` rather than `Box`. Use a `notify()` helper that checks the `Option` before calling.

**When to use:** Required because `IslandGa` uses `par_iter_mut()` (rayon), meaning the observer reference may be accessed from multiple threads simultaneously. `Arc` is clone-cheap and `Send + Sync` can be enforced at the bound.

**Trade-offs:** Slightly heavier than `Box` (reference count), but avoids requiring `&mut self` on observer methods (which would preclude shared access). All hooks take `&self` — implementations use interior mutability (`Mutex`, `AtomicU64`) when they need mutable state.

**Example:**
```rust
// In Ga<U>
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}

// Usage in run loop
let t_selection = Instant::now();
let parents = selection::factory(...)?;
self.notify(|obs| obs.on_selection_complete(
    generation,
    parents.len(),
    t_selection.elapsed(),
));
```

**Contrast with `Reporter<U>`:** The existing `reporter: Option<Box<dyn Reporter<U> + Send>>` uses `Box` and `&mut self` hooks — fine for single-threaded `Ga<U>`, but not usable from the island rayon loop. `GaObserver` must use `Arc` + `&self` throughout.

### Pattern 2: Default No-Op Method Bodies

**What:** Every method on `GaObserver<U>`, `IslandObserver<U>`, and `Nsga2Observer<U>` has a default empty body `{}`. Implementors only override what they need.

**When to use:** Always. This is the forward-compatibility guarantee: adding a new event in a future version does not break existing observer implementations.

**Trade-offs:** None. The Rust compiler inlines empty default bodies to zero instructions. `Option::None` guard in `notify()` eliminates even the trait dispatch when no observer is set.

### Pattern 3: Typed Event Structs for Complex Payloads

**What:** For events with many fields, pass a dedicated struct rather than a long parameter list. Example: `ExtensionEvent` for `on_extension_triggered`.

**When to use:** When an event carries more than ~3 parameters or when the set of fields is expected to grow.

**Trade-offs:** Slight extra allocation if the struct is heap-allocated; use stack structs (no `Box`) to keep it zero-alloc.

**Example:**
```rust
// Defined in src/observer/mod.rs
pub struct ExtensionEvent {
    pub method: Extension,
    pub diversity: f64,
    pub threshold: f64,
    pub population_before: usize,
    pub population_after: usize,
    pub elite_preserved: usize,
}
```

### Pattern 4: Sub-Trait Extension for Specialized Orchestrators

**What:** `IslandObserver<U>: GaObserver<U>` and `Nsga2Observer<U>: GaObserver<U>` add new methods. A single concrete type (e.g., `LogObserver`) can implement all three traits.

**When to use:** When a sub-system (Island, NSGA-II) has events that do not belong on the base trait but should be observable with the same abstraction.

**Trade-offs:** The field type in `IslandGa` must be `Option<Arc<dyn IslandObserver<U>>>`. Rust's trait upcasting (`dyn IslandObserver` to `dyn GaObserver`) is stable since 1.76. The project MSRV is 1.81.0, so upcasting is available — `IslandGa` can safely store `Arc<dyn IslandObserver<U>>` and pass it as `&dyn GaObserver<U>` where the base trait is expected.

## Data Flow

### Observer Notification Flow in ga.rs

```
run_with_callback()
  │
  ├── notify(on_initialization_end)     [after initialization()]
  ├── notify(on_run_start)
  │
  └── for generation in 0..max_generations:
       │
       ├── t_gen = Instant::now()
       ├── notify(on_generation_start)
       │
       ├── t = Instant::now()
       ├── selection::factory()
       ├── notify(on_selection_complete(gen, couples, t.elapsed()))
       │
       ├── t = Instant::now()
       ├── parent_crossover()           [crossover + mutation + fitness eval]
       ├── notify(on_crossover_complete(gen, offspring_count, crossovers, t.elapsed()))
       ├── notify(on_mutation_complete(gen, mutations, t.elapsed()))
       ├── notify(on_fitness_evaluation_complete(gen, evaluations, t.elapsed()))
       │
       ├── [elitism: extract_elite()]
       ├── notify(on_elitism_applied(gen, elite_count))           [if elitism_count > 0]
       │
       ├── t = Instant::now()
       ├── survivor::factory()
       ├── notify(on_survivor_selection_complete(gen, survivors, t.elapsed()))
       │
       ├── [adaptive_ga recalculate_aga()]
       ├── notify(on_adaptive_parameters_updated(...))            [if adaptive_ga]
       │
       ├── [niching apply_fitness_sharing()]
       ├── notify(on_niching_applied(gen, adjustments, t.elapsed()))  [if niching]
       │
       ├── [best chromosome update]
       ├── notify(on_best_chromosome_updated(gen, old, new))     [if improved]
       │
       ├── GenerationStats::from_fitness_values()
       ├── notify(on_generation_end(gen, &stats, t_gen.elapsed()))
       │
       ├── [dynamic mutation update]
       ├── notify(on_dynamic_mutation_updated(...))               [if dynamic_mutation]
       │
       ├── [extension if diversity < threshold]
       ├── notify(on_extension_triggered(gen, &ExtensionEvent)) [if triggered]
       │
       ├── [checkpoint save]
       ├── notify(on_checkpoint_saved(gen, path))                [if saved]
       ├── notify(on_checkpoint_failed(gen, error))              [if failed]
       │
       ├── [stopping criteria checks]
       ├── notify(on_stagnation_detected(gen, stagnant_gens))   [if stagnation]
       └── notify(on_convergence_detected(gen, std_dev, threshold)) [if convergence]
       │
  └── notify(on_run_end(cause, total_gens, total_duration))
```

### Observer Notification Flow in island/mod.rs

```
IslandGa::run()
  │
  ├── notify(on_island_run_start(island_id, config))      [per island, in initialize()]
  │
  └── for gen in 0..max_generations:
       │
       ├── evolve_islands_one_generation()
       ├──   per island (parallel): notify(on_island_generation_end(island_id, gen, stats))
       │
       └── [if migration interval hit]
            ├── notify(on_migration_start(from, to, migrants))
            ├── migrate()
            └── notify(on_migration_complete(from, to, duration))
```

### Observer Notification Flow in nsga2/mod.rs

```
Nsga2Ga::run()
  │
  └── for gen in 0..max_gens:
       │
       ├── t = Instant::now()
       ├── non_dominated_sort()
       ├── notify(on_non_dominated_sort_complete(gen, fronts.len(), t.elapsed()))
       │
       ├── t = Instant::now()
       ├── assign_crowding_distance()
       ├── notify(on_crowding_distance_calculated(gen, front_size, t.elapsed()))
       │
       ├── create_offspring()
       ├── [combine + re-sort + truncate]
       └── notify(on_pareto_front_updated(gen, front0.len()))
```

### LogObserver Migration: 8 Log Targets Mapping

The current 8 log targets in `ga.rs` map to observer methods as follows:

| Current log target | Becomes observer call |
|-------------------|----------------------|
| `ga_events` (info: generation N) | `on_generation_start` |
| `ga_events` (debug: parents selected) | `on_selection_complete` |
| `ga_events` (debug: offspring created) | `on_crossover_complete` |
| `ga_events` (debug: survivors selected) | `on_survivor_selection_complete` |
| `ga_events` (debug: best chromosome calculated) | `on_best_chromosome_updated` |
| `ga_events` (debug: limit_reached methods) | `on_run_end` (cause field) |
| `extension_events` (info: extension triggered) | `on_extension_triggered` |
| `ga_events` (debug: dynamic mutation) | `on_dynamic_mutation_updated` |

Island log calls (`island_events` target in `island/mod.rs`):
- "Initialized island N" maps to `on_island_run_start` (in `LogObserver` impl of `IslandObserver`)
- "Starting island model GA" maps to `on_run_start`
- "Fitness target reached at generation N" maps to `on_run_end`
- "Migration performed at generation N" maps to `on_migration_complete`

NSGA-II log calls (`nsga2_events` target in `nsga2/mod.rs`):
- "Starting NSGA-II: ..." maps to `on_run_start`
- "Generation N complete" maps to `on_non_dominated_sort_complete` (most natural mapping)

**Backward compatibility rule for `LogObserver`:** When `with_logs(LogLevel::X)` is set and no observer is explicitly provided via `with_observer()`, `Ga<U>` automatically installs a `LogObserver` configured to that level before entering the run loop. This preserves existing logging behavior for all current users without requiring any code changes.

## Scaling Considerations

This is a library, not a service — scaling means "overhead per generation" not user counts.

| Scale | Architecture Adjustments |
|-------|--------------------------|
| No observer | `Option::None` branch: zero allocations, zero trait dispatch, zero overhead |
| Single observer | One `if let Some` check + one `&dyn` dispatch per notification point — nanosecond range |
| CompositeObserver (N observers) | N dispatch calls per event; still nanoseconds unless observers do I/O |
| TracingObserver with subscriber | Observer is fast; subscriber (e.g., OpenTelemetry exporter) controls throughput |
| MetricsObserver | Atomic operations in `metrics` facade — thread-safe, microsecond range |

### Scaling Priorities

1. **First concern — per-generation overhead:** Timing calls (`Instant::now()`) add approximately 20ns per call. Place them only around the four main phases (selection, crossover+mutation+fitness, survivor, generation total). Do not time per-chromosome operations.
2. **Second concern — Island parallelism:** `par_iter_mut()` means the observer can be called from multiple rayon threads simultaneously. `Arc<dyn GaObserver + Send + Sync>` handles this correctly. `LogObserver` and `TracingObserver` are already thread-safe. `MetricsObserver` uses atomics. Custom observers must document their thread-safety.

## Anti-Patterns

### Anti-Pattern 1: Box Instead of Arc for Observer Storage

**What people do:** Store observer as `Option<Box<dyn GaObserver<U> + Send>>` (mirroring the existing `Reporter<U>` pattern).

**Why it's wrong:** `IslandGa::evolve_islands_one_generation` uses `par_iter_mut()` which calls into a closure on rayon worker threads. A `Box` requires exclusive ownership or a `Sync` bound to share across threads. `Box<dyn T + Send>` is not `Sync` unless T is. `Arc<dyn T + Send + Sync>` is `Clone` and `Sync` by construction.

**Do this instead:** `Option<Arc<dyn GaObserver<U> + Send + Sync>>` throughout all three orchestrators.

### Anti-Pattern 2: &mut self on Observer Hook Methods

**What people do:** Define hooks as `fn on_generation_end(&mut self, ...)` — natural for sequential code.

**Why it's wrong:** Requires `&mut` access to call, which is incompatible with sharing across rayon threads via `Arc`. You cannot hold `&mut` through an `Arc` without adding a `Mutex` on every call (erasing the zero-overhead goal).

**Do this instead:** All hook methods take `&self`. Implementations that need mutable state (e.g., a counter in `MetricsObserver`) use `AtomicU64` or `Mutex<inner>` internally.

### Anti-Pattern 3: Replacing Reporter<U> With GaObserver<U>

**What people do:** Remove `Reporter<U>` and migrate existing users to `GaObserver<U>`.

**Why it's wrong:** `Reporter<U>` is a public trait in a published crate (v2.1.0). Removing it is a breaking change. The project constraint is explicit: no breaking changes.

**Do this instead:** Keep `Reporter<U>` as-is. Both systems coexist in `Ga<U>` — the `reporter` field stays alongside the new `observer` field. They serve different granularities: `Reporter` (4 hooks) is simple and mutation-friendly; `GaObserver` (20+ hooks) is granular and thread-safe.

### Anti-Pattern 4: Per-Gene or Per-Chromosome Observer Calls

**What people do:** Notify the observer for every chromosome fitness evaluation or every gene mutation.

**Why it's wrong:** Populations of 1000+ chromosomes x 500 generations = 500,000+ observer calls per run. Even a no-op dispatch at that granularity adds measurable overhead.

**Do this instead:** Aggregate — notify once per operator phase with a count (e.g., `on_mutation_complete(gen, mutations_applied, duration)` rather than a per-gene event). This matches the `out_of_scope` constraint in PROJECT.md: "Per-gene hooks in observer — too granular, unacceptable overhead in hot loops."

### Anti-Pattern 5: Hardcoding the Default Observer Type

**What people do:** Change `observer: Option<Arc<dyn GaObserver<U>>>` to a concrete type to avoid trait object overhead.

**Why it's wrong:** Couples the GA engine to a specific observer implementation. Users cannot inject their own observer without modifying the library.

**Do this instead:** Use trait objects (`dyn GaObserver<U>`) for the stored field. The `Option::None` path eliminates overhead when unused. When overhead truly matters (benchmarks), the user can implement `GaObserver<U>` as a zero-sized struct with all no-op bodies — identical to `Option::None` in generated code.

## Integration Points

### New vs Modified Files

| File | Status | Change |
|------|--------|--------|
| `src/observer/mod.rs` | NEW | `GaObserver<U>` trait, `ExtensionEvent`, `NoopObserver`, module re-exports |
| `src/observer/island.rs` | NEW | `IslandObserver<U>: GaObserver<U>` sub-trait |
| `src/observer/nsga2.rs` | NEW | `Nsga2Observer<U>: GaObserver<U>` sub-trait |
| `src/observer/log_observer.rs` | NEW | `LogObserver` implementing all three traits |
| `src/observer/composite.rs` | NEW | `CompositeObserver<U>` |
| `src/observer/tracing_observer.rs` | NEW | `TracingObserver` (`#[cfg(feature="observer-tracing")]`) |
| `src/observer/metrics_observer.rs` | NEW | `MetricsObserver` (`#[cfg(feature="observer-metrics")]`) |
| `src/ga.rs` | MODIFIED | Add `observer` field, `with_observer()`, `notify()`, `Instant` measurements at 6 phase boundaries, automatic `LogObserver` default when `log_level != Off` |
| `src/island/mod.rs` | MODIFIED | Add `observer` field, `with_observer()`, `notify()` calls at migration and per-island generation boundaries |
| `src/nsga2/mod.rs` | MODIFIED | Add `observer` field, `with_observer()`, `notify()` calls after sort, crowding distance, Pareto front update |
| `src/lib.rs` | MODIFIED | `pub mod observer;` plus prelude re-exports for `GaObserver`, `LogObserver`, `CompositeObserver` |
| `Cargo.toml` | MODIFIED | Add `observer-tracing = ["tracing"]` and `observer-metrics = ["metrics"]` feature flags |
| `src/reporter/mod.rs` | UNCHANGED | `Reporter<U>` stays as-is for backward compatibility |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `ga.rs` to `observer/mod.rs` | Direct method call on `&dyn GaObserver<U>` via `notify()` helper | One indirection through `Option` check |
| `island/mod.rs` to `observer/island.rs` | Direct method call on `&dyn IslandObserver<U>` via `notify()` helper | `par_iter_mut` requires `Arc + Send + Sync` |
| `nsga2/mod.rs` to `observer/nsga2.rs` | Direct method call on `&dyn Nsga2Observer<U>` via `notify()` | |
| `CompositeObserver` to inner observers | Fan-out: iterate `Vec<Arc<dyn GaObserver<U>>>`, call each | `CompositeObserver` itself implements all three observer traits |
| `LogObserver` to `log` crate | Calls `info!`, `debug!`, `trace!` with same targets as current hardcoded calls | Preserves backward compat with `env_logger` filter config |
| `TracingObserver` to `tracing` crate | `info_span!`, `tracing::info!` — feature-gated | Users provide subscriber |
| `MetricsObserver` to `metrics` crate | `metrics::histogram!`, `metrics::gauge!`, `metrics::counter!` — feature-gated | Users provide recorder |

## Suggested Build Order

This order respects trait dependencies (base before implementors) and allows each step to be independently tested and merged.

**Phase 1 — Base Trait (Issue #182)**

Build order within the phase:
1. `src/observer/mod.rs` — define `GaObserver<U>` trait with all methods and default no-op bodies; define `ExtensionEvent` struct; define `NoopObserver`
2. `src/ga.rs` — add `observer` field, `with_observer()`, and `notify()` helper; instrument the main loop with `Instant::now()` and `notify()` calls at each of the 12 notification points; keep `reporter` field untouched
3. `src/lib.rs` — expose `pub mod observer` and re-export `GaObserver`

Rationale: `GaObserver` trait must exist before any concrete observer can implement it. Instrumenting `ga.rs` in this phase means Phase 2 can immediately test `LogObserver` against the live loop.

**Phase 2 — LogObserver + Logging Migration (Issue #183)**

Build order within the phase:
1. `src/observer/log_observer.rs` — implement `GaObserver<U>` for `LogObserver` using `log!()` with the same 8 targets
2. `src/ga.rs` — add automatic `LogObserver` default: when `log_level != Off` and no observer is set, install `LogObserver` before the run loop
3. Remove the hardcoded `log!()` macro calls from `ga.rs` that are now covered by `LogObserver`

Rationale: This phase validates that the observer notification points work end-to-end with a real implementation. Must be done before #184 to confirm the trait surface is complete.

**Phase 3 — TracingObserver (Issue #184)**

Build order within the phase:
1. `Cargo.toml` — add `observer-tracing = ["tracing"]` feature
2. `src/observer/tracing_observer.rs` — implement all hooks using `tracing` spans per the hierarchy in issue #184
3. Example in `examples/` demonstrating `TracingObserver` with `tracing-subscriber`

Rationale: Feature-gated, so zero impact on default builds. Can be built and reviewed independently of sub-trait work.

**Phase 4 — Island and NSGA-II Sub-Traits (Issue #185)**

Build order within the phase:
1. `src/observer/island.rs` — define `IslandObserver<U>: GaObserver<U>` sub-trait
2. `src/observer/nsga2.rs` — define `Nsga2Observer<U>: GaObserver<U>` sub-trait
3. `src/island/mod.rs` — add `observer: Option<Arc<dyn IslandObserver<U>>>`, `with_observer()`, `notify()`, and notification calls at migration boundaries
4. `src/nsga2/mod.rs` — add `observer: Option<Arc<dyn Nsga2Observer<U>>>`, `with_observer()`, `notify()`, and notification calls after sort/crowding/Pareto phases
5. Extend `LogObserver` to implement `IslandObserver<U>` and `Nsga2Observer<U>`
6. Extend `TracingObserver` (if feature-enabled) with sub-trait implementations

Rationale: Sub-traits depend on the base trait (Phase 1). `IslandObserver` and `Nsga2Observer` can be built in parallel within this phase.

**Phase 5 — CompositeObserver + MetricsObserver (Issue #186)**

Build order within the phase:
1. `src/observer/composite.rs` — implement `CompositeObserver<U>` with builder pattern; implement all three observer traits by fan-out
2. `Cargo.toml` — add `observer-metrics = ["metrics"]` feature
3. `src/observer/metrics_observer.rs` — implement all hooks with `metrics::histogram!`, `metrics::gauge!`, `metrics::counter!`
4. Example demonstrating `CompositeObserver::new().with(LogObserver).with(MetricsObserver).build()`

Rationale: `CompositeObserver` requires all three observer traits to exist (Phases 1 and 4). `MetricsObserver` is the last concrete observer and the highest-risk feature flag (new dependency).

## Sources

- Source code: `src/ga.rs` (direct read — all notification points identified from the run loop, lines 716-1115)
- Source code: `src/island/mod.rs` (direct read — migration and per-island log calls)
- Source code: `src/nsga2/mod.rs` (direct read — non-dominated sort and crowding loop)
- Source code: `src/reporter/mod.rs` (direct read — `Reporter<U>` trait design as precedent)
- Source code: `src/stats.rs` (direct read — `GenerationStats` fields available to hooks)
- GitHub issue #182: GaObserver trait definition and notification point inventory
- GitHub issue #183: LogObserver scope and 8-target backward compat requirement
- GitHub issue #184: TracingObserver span hierarchy
- GitHub issue #185: IslandObserver and Nsga2Observer sub-trait method signatures
- GitHub issue #186: CompositeObserver builder pattern and MetricsObserver metric list
- `CLAUDE.md`: MSRV 1.81.0 (trait upcasting stabilized in 1.76), `Send + Sync` threading constraint

---
*Architecture research for: GaObserver observability system in genetic_algorithms Rust library*
*Researched: 2026-03-25*
