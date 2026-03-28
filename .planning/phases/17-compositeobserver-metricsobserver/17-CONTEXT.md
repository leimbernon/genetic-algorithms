# Phase 17: CompositeObserver + MetricsObserver - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement `CompositeObserver<U>` — a fan-out observer that fans all three observer traits to multiple inner observers simultaneously — and `MetricsObserver` behind the `observer-metrics` feature flag, recording per-generation counters, gauges, and histograms via the `metrics` facade crate. Scope: `src/observer/composite.rs`, `src/observer/metrics_observer.rs`, `Cargo.toml` feature wiring, `src/observer/mod.rs` re-exports, `src/lib.rs` re-exports. No changes to engine files (`ga.rs`, `island/mod.rs`, `nsga2/mod.rs`).

</domain>

<decisions>
## Implementation Decisions

### CompositeObserver sub-observer contract

- Each observer added to `CompositeObserver` must implement **all 3 traits**: `GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>` (plus `Send + Sync`)
- This is captured via a combined **`AllObserver<U>`** supertrait:

```rust
pub trait AllObserver<U: ChromosomeT>:
    GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U>
    + Send + Sync {}

// Blanket impl — all types satisfying the bounds automatically implement AllObserver
impl<U, T> AllObserver<U> for T
where
    U: ChromosomeT,
    T: GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U> + Send + Sync
{}
```

- `AllObserver<U>` is **publicly exported** from `src/lib.rs` alongside `GaObserver`, `IslandGaObserver`, `Nsga2Observer`
- `LogObserver` already satisfies `AllObserver<U>` (implements all 3 traits from Phases 13–16)

### CompositeObserver builder API

- **Chained `.add()` builder** — consistent with existing `with_observer()` pattern:

```rust
let composite = CompositeObserver::new()
    .add(Arc::new(LogObserver))
    .add(Arc::new(MetricsObserver::new("experiment_42")));

ga.with_observer(Arc::new(composite));
```

- `CompositeObserver` itself implements all 3 observer traits (i.e., satisfies `AllObserver<U>`), so it can be attached to `Ga<U>`, `IslandGa<U>`, or `Nsga2Ga<U>` interchangeably
- Each hook fans out to all inner observers in insertion order

### MetricsObserver metric catalog

Three categories of metrics, all emitted via the `metrics` facade:

**Per-generation fitness gauges** (from `on_generation_end` `GenerationStats`):
```
gauge!("ga.generation.best_fitness", best_fitness, "run_id" => self.run_id);
gauge!("ga.generation.mean_fitness", mean_fitness, "run_id" => self.run_id);
gauge!("ga.generation.diversity", diversity, "run_id" => self.run_id);
```

**Operator timing histograms** (from operator hooks):
```
histogram!("ga.operator.selection_ms", dur_ms, "run_id" => self.run_id);
histogram!("ga.operator.crossover_ms", dur_ms, "run_id" => self.run_id);
histogram!("ga.operator.mutation_ms", dur_ms, "run_id" => self.run_id);
histogram!("ga.operator.fitness_eval_ms", dur_ms, "run_id" => self.run_id);
histogram!("ga.operator.survivor_ms", dur_ms, "run_id" => self.run_id);
```

**Event counters** (from special event hooks):
```
counter!("ga.event.new_best", 1, "run_id" => self.run_id);
counter!("ga.event.stagnation", 1, "run_id" => self.run_id);
counter!("ga.event.extension_triggered", 1, "run_id" => self.run_id);
```

### Metric key naming convention

- **Dot-delimited**: `ga.generation.best_fitness`, `ga.operator.selection_ms`, `ga.event.new_best`
- Consistent with TracingObserver's `ga_run` / `ga_generation` span name prefix and the `target="ga_events"` convention
- Dots are natively supported by Grafana Tempo, Datadog, and StatsD; the `metrics` crate passes them through unchanged

### MetricsObserver struct design

```rust
pub struct MetricsObserver {
    run_id: &'static str,
}

impl MetricsObserver {
    pub fn new(run_id: &'static str) -> Self { Self { run_id } }
}
```

- `run_id` is a `&'static str` label attached to every emitted metric — disambiguates parallel or sequential runs in shared dashboards
- No mutable state — all metrics emitted directly via `metrics` macros (facade handles thread safety)
- `MetricsObserver` implements `GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>`, and therefore `AllObserver<U>`
- Behind `observer-metrics` feature flag — default builds unaffected (COMP-02)
- COMP-03: metric calls are restricted to sequential per-generation hooks only; no `metrics::*` calls inside `par_iter()` closures

### Feature flag

- Flag name: `observer-metrics` (consistent with `observer-tracing`)
- Adds `metrics` crate as optional dependency: `metrics = { version = "0.24", optional = true }`
- Entire `src/observer/metrics_observer.rs` is `#[cfg(feature = "observer-metrics")]`

### Claude's Discretion

- Whether `CompositeObserver` derives `Clone` (useful for attaching to multiple engines)
- Whether `CompositeObserver::new()` pre-allocates the internal Vec
- Whether `MetricsObserver` implements `Default` in addition to `new()`
- Exact `metrics` crate version to use (check crates.io for latest stable)
- Whether `is_some()` guard is needed before emitting metrics (metrics facade is always zero-cost when no recorder is installed, so guard may be unnecessary)
- Integration test strategy for COMP-01 through COMP-03

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Composite + Metrics (#186) — COMP-01 through COMP-03 (the three acceptance criteria)

### Observer infrastructure (already in place)
- `src/observer/mod.rs` — all three trait definitions (`GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>`), `LogObserver` re-export, `TracingObserver` re-export; `AllObserver<U>` and composite/metrics added here
- `src/observer/log.rs` — structural template for a multi-trait impl block; `LogObserver` already satisfies all 3 traits

### Prior observer patterns (must follow)
- `src/observer/tracing_observer.rs` — structural template for a feature-gated observer module: `#[cfg(feature = "...")]`, module file layout, `impl<U: ChromosomeT> GaObserver<U>` block, re-export pattern
- `.planning/phases/15-tracingobserver/15-CONTEXT.md` — feature flag, struct design, and Send+Sync decisions that MetricsObserver must follow
- `.planning/phases/16-sub-traits/16-CONTEXT.md` — AllObserver/composite storage pattern, IslandGaObserver + Nsga2Observer impl conventions

### Cargo.toml
- `Cargo.toml` `[features]` section — existing `observer-tracing = ["dep:tracing"]` pattern; `observer-metrics` follows the same structure

### Types used in hook signatures
- `src/stats.rs` — `GenerationStats` struct (`best_fitness`, `mean_fitness`, `diversity` fields used in gauges)
- `src/ga.rs` `TerminationCause` — payload for `on_run_end` (MetricsObserver ignores this hook, but must implement it with default no-op)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/observer/tracing_observer.rs` — direct structural template for `MetricsObserver`: file layout, `#[cfg(feature = "...")]`, `impl<U: ChromosomeT> GaObserver<U>` block, `pub use` in `mod.rs`, `src/lib.rs` re-export
- `src/observer/log.rs` — `LogObserver` already implements all 3 traits; its multi-`impl` block structure is the template for any `AllObserver<U>`-compliant type
- `src/ga.rs` `notify()` helper — fan-out pattern to study for `CompositeObserver`'s inner loop

### Established Patterns
- Feature-gated optional dependencies: `tracing = { version = "0.1", optional = true }` + `observer-tracing = ["dep:tracing"]` — apply identically for `metrics`
- `#[cfg(feature = "observer-metrics")]` gate on both the module declaration in `mod.rs` and the re-export in `lib.rs`
- Observer module re-exports: `src/observer/mod.rs` already has `mod log; pub use log::LogObserver` and the tracing equivalent — add composite and metrics following the same pattern

### Integration Points
- `Cargo.toml` — add `metrics = { version = "0.24", optional = true }` + `observer-metrics = ["dep:metrics"]` feature
- `src/observer/mod.rs` — add `AllObserver<U>` trait + blanket impl; add `mod composite; pub use composite::CompositeObserver`; add `#[cfg(feature = "observer-metrics")] mod metrics_observer; pub use metrics_observer::MetricsObserver`
- `src/lib.rs` — add `AllObserver`, `CompositeObserver` to observer re-exports; add `MetricsObserver` under `#[cfg(feature = "observer-metrics")]`
- No changes to `ga.rs`, `island/mod.rs`, or `nsga2/mod.rs` — observer infrastructure already in place

</code_context>

<specifics>
## Specific Ideas

- `CompositeObserver` satisfying `AllObserver<U>` means a user can attach the same composite to `Ga<U>`, `IslandGa<U>`, and `Nsga2Ga<U>` in the same program — a key use case for users running multiple engine types
- `run_id: &'static str` on `MetricsObserver` is the label that separates runs in dashboards (e.g., `"experiment_42"` or `"production_run"`); using `&'static str` avoids heap allocation and satisfies `Send + Sync` trivially
- The `metrics` crate's recorder registration is the user's responsibility (same as `tracing`'s subscriber) — `MetricsObserver` only emits, never sets up a recorder
- Operator timing uses `Duration` from the hook parameters, converted to `f64` milliseconds: `dur.as_secs_f64() * 1000.0` — consistent with TracingObserver's `duration_ms` field naming

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 17-compositeobserver-metricsobserver*
*Context gathered: 2026-03-27*
