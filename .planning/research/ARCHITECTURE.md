# Architecture Patterns

**Domain:** Observability integration for Rust genetic algorithms library
**Researched:** 2026-03-23

---

## Scope

This document answers a single question: how do the observability features
(GaObserver trait, Island/NSGA-II sub-traits, CompositeObserver) integrate
with the existing `Ga<U>` / `IslandGa<U>` / `Nsga2Ga<U>` architecture?

---

## Existing Architecture — What We're Integrating Into

### Lifecycle touchpoints (from code inspection)

`Ga<U>::run_with_callback` in `src/ga.rs` is the canonical GA loop. Every
meaningful lifecycle event already happens there:

| Step | Code location | Current instrumentation |
|------|--------------|------------------------|
| Run start | top of `run_with_callback` | `info!("Initialization started")` |
| Generation start | `for i in 0..max_generations` | `info!(target="ga_events", ...)` |
| Selection complete | after `selection::factory` | `debug!(target="ga_events", ...)` |
| Crossover complete | after `parent_crossover` | `debug!(target="ga_events", ...)` |
| Survivors chosen | after `survivor::factory` | `debug!(target="ga_events", ...)` |
| Best chromosome updated | after best-chromosome scan | `debug!(target="ga_events", ...)` |
| `GenerationStats` computed | `GenerationStats::from_fitness_values` | passed to callback |
| Stopping criteria fired | each stopping branch | `info!` / break |
| Run complete | after the generation loop | implicit (return) |

`IslandGa<U>::run` in `src/island/mod.rs` has its own discrete events:

| Event | Current instrumentation |
|-------|------------------------|
| Island initialized | `debug!(target="island_events", ...)` |
| Islands run start | `info!(target="island_events", ...)` |
| Fitness target reached | `info!(target="island_events", ...)` |
| Migration executed | `debug!(target="island_events", ...)` |

`Nsga2Ga<U>::run` in `src/nsga2/mod.rs`:

| Event | Current instrumentation |
|-------|------------------------|
| Run start | `info!(target="nsga2_events", ...)` |
| Generation complete | `debug!(target="nsga2_events", ...)` |

### Rayon usage pattern

All three structs use `rayon` for parallel fitness evaluation and, in the
island model, `par_iter_mut` over the islands slice. The observer is held
at the orchestrator level — it is never called from inside a rayon closure.
This means `Arc<dyn GaObserver<U>>` is cloned once before the parallel
region and can be used safely without additional locking.

The `Ga<U>` parallel region is `population.fitness_calculation` (internal,
not user-visible). The main loop body in `run_with_callback` is sequential.
The island model parallelises `evolve_islands_one_generation` via
`par_iter_mut`, but the migration and observer calls live *outside* that
parallel section.

**Conclusion:** `Arc<dyn GaObserver<U> + Send + Sync>` is sufficient.
No `Mutex` is needed at the call sites we control because observer methods
are called only from the sequential driver loop. Feature flags for tracing
and metrics can add their own internal synchronisation if the upstream crates
require it.

---

## Recommended Architecture

### New trait hierarchy

```
src/observer/
├── mod.rs          — re-exports, GaObserver trait
├── composite.rs    — CompositeObserver
├── log_observer.rs — LogObserver (replaces hardcoded log!() calls)
└── tracing_observer.rs   — TracingObserver (feature = "observer-tracing")
```

`GaObserver<U>` is the base trait. Island and NSGA-II extensions are
sub-traits that add their own lifecycle hooks.

```rust
// src/observer/mod.rs
pub trait GaObserver<U: ChromosomeT>: Send + Sync {
    // All methods have default no-op bodies.
    fn on_run_start(&self, _config: &GaConfiguration) {}
    fn on_generation_start(&self, _generation: usize) {}
    fn on_selection_complete(&self, _generation: usize, _parent_count: usize) {}
    fn on_crossover_complete(&self, _generation: usize, _offspring_count: usize) {}
    fn on_mutation_complete(&self, _generation: usize) {}
    fn on_survivor_selection_complete(&self, _generation: usize, _population_size: usize) {}
    fn on_generation_complete(&self, _generation: usize, _stats: &GenerationStats) {}
    fn on_new_best(&self, _generation: usize, _best: &U) {}
    fn on_run_complete(&self, _cause: &TerminationCause, _stats: &[GenerationStats]) {}
}

pub trait IslandGaObserver<U: ChromosomeT>: GaObserver<U> {
    fn on_island_initialized(&self, _island_index: usize, _population_size: usize) {}
    fn on_migration(&self, _generation: usize, _migrant_count: usize) {}
}

pub trait Nsga2Observer<U: ChromosomeT>: GaObserver<U> {
    fn on_front_assigned(&self, _generation: usize, _front_sizes: &[usize]) {}
}
```

### Observer held on the orchestrators

Add a single field to each orchestrator. No other fields change.

```rust
// Ga<U>
pub struct Ga<U: ChromosomeT> {
    // ... existing fields unchanged ...
    pub observer: Option<Arc<dyn GaObserver<U>>>,
}

// IslandGa<U>
pub struct IslandGa<U: ChromosomeT> {
    // ... existing fields unchanged ...
    pub observer: Option<Arc<dyn IslandGaObserver<U>>>,
}

// Nsga2Ga<U>
pub struct Nsga2Ga<U: ChromosomeT> {
    // ... existing fields unchanged ...
    pub observer: Option<Arc<dyn Nsga2Observer<U>>>,
}
```

Because `IslandGaObserver` and `Nsga2Observer` are sub-traits of `GaObserver`,
a single `Arc<dyn IslandGaObserver<U>>` satisfies both the base lifecycle
hooks and the island-specific hooks. The same principle holds for NSGA-II.

### Builder method (same pattern for all three structs)

```rust
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U>>) -> Self {
    self.observer = Some(observer);
    self
}
```

### Call sites in the run loops

Replace each `log!()` call with an observer dispatch wrapped in the
`Option::as_ref()` pattern. No heap allocation when the observer is `None`.

```rust
// Before
info!(target="ga_events", method="run"; "Generation number: {}", i+1);

// After
if let Some(obs) = &self.observer {
    obs.on_generation_start(i);
}
```

The `log!()` calls are **not deleted** during the transition. `LogObserver`
reproduces them identically, so the replacement is: remove the hardcoded
`log!()` call and rely on `LogObserver` to emit the same message. This
satisfies the backward-compatibility constraint from `PROJECT.md`.

---

## Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `GaObserver<U>` trait | Define lifecycle contract with default no-ops | `Ga<U>`, `GenerationStats`, `TerminationCause` |
| `IslandGaObserver<U>` sub-trait | Island-specific events on top of base trait | `IslandGa<U>` |
| `Nsga2Observer<U>` sub-trait | NSGA-II-specific events on top of base trait | `Nsga2Ga<U>` |
| `LogObserver` | Emit `log!()` macros exactly as current hardcoded calls do | `log` crate (always present) |
| `TracingObserver` | Emit `tracing` spans and events | `tracing` crate (feature-gated) |
| `MetricsObserver` | Record counters/histograms | `metrics` crate (feature-gated) |
| `CompositeObserver` | Fan-out calls to a `Vec<Arc<dyn GaObserver<U>>>` | All observer impls |
| `Ga<U>`, `IslandGa<U>`, `Nsga2Ga<U>` | Call observer methods at lifecycle points | `GaObserver<U>` |

---

## Data Flow Changes

### Before (current)

```
run_with_callback()
  → log::info! / log::debug! / log::trace!  (directly into log crate)
  → user callback (only via run_with_callback API)
```

### After (observability milestone)

```
run_with_callback()
  → Option<Arc<dyn GaObserver<U>>>::on_*()
        → LogObserver::on_*()      → log::info! / log::debug! (unchanged output)
        → TracingObserver::on_*()  → tracing::span / tracing::event
        → MetricsObserver::on_*()  → metrics::counter / metrics::histogram
        → CompositeObserver        → fans out to N inner observers
  → user callback (unchanged, still supported in parallel with observer)
```

The user callback (`run_with_callback`) is **not replaced**. It remains a
separate mechanism for control flow (it can return `ControlFlow::Break`).
Observers are fire-and-forget; they cannot interrupt the run.

---

## Integration Points: New vs Modified

### New files (no existing code changed)

| File | What it contains |
|------|-----------------|
| `src/observer/mod.rs` | `GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>` |
| `src/observer/composite.rs` | `CompositeObserver<U>` |
| `src/observer/log_observer.rs` | `LogObserver` |
| `src/observer/tracing_observer.rs` | `TracingObserver` (feature-gated) |
| `src/observer/metrics_observer.rs` | `MetricsObserver` (feature-gated) |

### Modified files (minimal, targeted)

| File | Change |
|------|--------|
| `src/lib.rs` | Add `pub mod observer;` |
| `src/ga.rs` | Add `observer` field to `Ga<U>`, add `with_observer()` builder method, replace hardcoded `log!()` calls with `if let Some(obs) = &self.observer { obs.on_*(...) }` |
| `src/island/mod.rs` | Same pattern: field, builder method, replace `log!()` calls |
| `src/nsga2/mod.rs` | Same pattern: field, builder method, replace `log!()` calls |
| `Cargo.toml` | Add `observer-tracing` and `observer-metrics` feature flags with optional `tracing` and `metrics` deps |

---

## Patterns to Follow

### Pattern: Option-guarded observer dispatch (zero overhead when None)

The `Option<Arc<dyn GaObserver<U>>>` field costs nothing when `None` —
the branch is a single pointer comparison and is trivially branch-predicted
away. This matches the `Option<Arc<FitnessFn>>` pattern already in use on
all three orchestrators.

```rust
// Canonical call-site pattern — use this everywhere
if let Some(obs) = &self.observer {
    obs.on_generation_complete(i, &gen_stats);
}
```

Do not use `observer.as_ref().map(|o| o.on_*(…))` — it is equivalent but
less readable.

### Pattern: Sub-trait upcast via Arc coercion

`IslandGaObserver<U>` and `Nsga2Observer<U>` extend `GaObserver<U>`. Users
who implement both need to provide a single concrete type; Arc coercion
handles the upcast transparently:

```rust
let obs: Arc<dyn IslandGaObserver<MyChrom>> = Arc::new(MyObserver::new());
let island_ga = IslandGa::new(config, ga_config)
    .with_observer(obs);
```

### Pattern: CompositeObserver fan-out

```rust
pub struct CompositeObserver<U: ChromosomeT> {
    observers: Vec<Arc<dyn GaObserver<U>>>,
}

impl<U: ChromosomeT> GaObserver<U> for CompositeObserver<U> {
    fn on_generation_complete(&self, generation: usize, stats: &GenerationStats) {
        for obs in &self.observers {
            obs.on_generation_complete(generation, stats);
        }
    }
    // ... same pattern for all hooks
}
```

### Pattern: Feature-gated concrete impls

```toml
# Cargo.toml
[features]
observer-tracing = ["dep:tracing"]
observer-metrics = ["dep:metrics"]
```

```rust
// src/observer/tracing_observer.rs
#[cfg(feature = "observer-tracing")]
pub struct TracingObserver { /* ... */ }
```

`LogObserver` does not need a feature flag because `log` is already a
non-optional dependency.

---

## Anti-Patterns to Avoid

### Anti-Pattern: Calling observer inside rayon closures

**What:** Passing `Arc<dyn GaObserver<U>>` into `par_iter_mut` closures and
calling observer methods per-chromosome.

**Why bad:** Observer implementations (tracing, metrics) may not be
optimised for high-frequency concurrent calls. It also defeats the design
intent — per-generation hooks, not per-chromosome hooks.

**Instead:** Call observer methods only from the sequential driver loop
(before/after the parallel region), passing aggregated data like
`GenerationStats`.

### Anti-Pattern: Removing `run_with_callback` or merging it into observer

**What:** Replacing the existing callback API with the observer.

**Why bad:** The callback has control-flow semantics (`ControlFlow::Break`)
that the observer intentionally does not have. Merging breaks the public API
(`run_with_callback` is `pub`).

**Instead:** Keep both. Observer fires for all events; callback remains an
optional interrupt mechanism.

### Anti-Pattern: Storing observer as `Box<dyn GaObserver<U>>`

**What:** Using `Box` instead of `Arc`.

**Why bad:** `IslandGa<U>` needs to share one observer reference across all
islands. `Arc` enables cheap cloning across threads. `Box` cannot be shared.

**Instead:** Always `Arc<dyn GaObserver<U> + Send + Sync>`.

### Anti-Pattern: Adding observer methods to GaConfiguration

**What:** Storing the observer inside `GaConfiguration` as a serde-serialised
field.

**Why bad:** `GaConfiguration` is `Serialize`/`Deserialize` (serde feature).
Trait objects cannot implement `Serialize`. This would break the checkpoint
system.

**Instead:** Keep observer as a separate field directly on `Ga<U>` /
`IslandGa<U>` / `Nsga2Ga<U>`, outside of configuration.

---

## Build Order (Feature Dependencies)

1. **`src/observer/mod.rs`** — Define `GaObserver<U>`, `IslandGaObserver<U>`,
   `Nsga2Observer<U>` traits. No deps on new code.

2. **`src/observer/log_observer.rs`** — Implement `LogObserver`. Depends on
   trait definition. Mirrors every existing `log!()` call 1:1.

3. **Integrate into `Ga<U>`** — Add field and call sites in `ga.rs`. Replace
   all hardcoded `log!()` calls with observer dispatch + delegate to
   `LogObserver` by default (or leave default as None, let the user wire it).
   Verify existing tests still pass.

4. **Integrate into `IslandGa<U>`** — Same as above for `island/mod.rs`.

5. **Integrate into `Nsga2Ga<U>`** — Same as above for `nsga2/mod.rs`.

6. **`src/observer/composite.rs`** — Implement `CompositeObserver`. Depends on
   all three trait definitions. Pure delegation; no new logic.

7. **`src/observer/tracing_observer.rs`** — Implement `TracingObserver` behind
   `observer-tracing` feature flag. Depends on base trait and `tracing` crate.

8. **`src/observer/metrics_observer.rs`** — Implement `MetricsObserver` behind
   `observer-metrics` feature flag. Depends on base trait and `metrics` crate.

Steps 3–5 can proceed in any order; they are independent modules. Steps 7–8
are independent of each other and can be built or deferred independently.

---

## Cargo.toml Changes Required

```toml
[features]
default = []
serde = ["dep:serde", "dep:serde_json"]
observer-tracing = ["dep:tracing"]
observer-metrics = ["dep:metrics"]

[dependencies]
# ... existing deps ...
tracing  = { version = "0.1", optional = true }
metrics  = { version = "0.24", optional = true }
```

Confidence on version numbers: MEDIUM. `tracing = "0.1"` has been stable for
several years. `metrics = "0.24"` — verify against crates.io at build time.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Call-site integration pattern | HIGH | Based on direct code inspection of all three run loops |
| Rayon safety of Arc<dyn Trait> | HIGH | Rayon requires Send+Sync; Arc satisfies both |
| Sub-trait design | HIGH | Standard Rust; pattern used throughout codebase (ChromosomeT hierarchy) |
| Feature flag approach | HIGH | Mirrors existing `serde` feature flag pattern in Cargo.toml |
| Tracing/metrics crate versions | MEDIUM | Requires verification at build time |

---

## Sources

- `src/ga.rs` — `run_with_callback` implementation (direct inspection)
- `src/island/mod.rs` — `IslandGa::run` and `evolve_islands_one_generation` (direct inspection)
- `src/nsga2/mod.rs` — `Nsga2Ga::run` (direct inspection)
- `src/stats.rs` — `GenerationStats` struct (direct inspection)
- `src/traits/chromosome.rs` — `ChromosomeT` trait bounds (direct inspection)
- `Cargo.toml` — existing features and dependency versions (direct inspection)
- `.planning/PROJECT.md` — backward compat, zero-overhead, and feature-flag constraints
