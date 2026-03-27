# Phase 17: CompositeObserver + MetricsObserver - Research

**Researched:** 2026-03-27
**Domain:** Rust composite/fan-out pattern + `metrics` facade crate integration behind a feature flag
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Each observer added to `CompositeObserver` must implement all 3 traits: `GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>` (plus `Send + Sync`)
- Combined bound captured via a `pub trait AllObserver<U: ChromosomeT>: GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U> + Send + Sync {}` supertrait with a blanket impl
- `AllObserver<U>` is publicly exported from `src/lib.rs`
- `CompositeObserver` builder: chained `.add(Arc::new(...))` method, consistent with existing `with_observer()` pattern
- `CompositeObserver` itself implements all 3 traits (satisfies `AllObserver<U>`)
- Fan-out is in insertion order
- MetricsObserver metric catalog: fitness gauges from `on_generation_end`, operator timing histograms from operator hooks, event counters from special-event hooks (see CONTEXT.md for exact metric names)
- Metric key naming: dot-delimited (`ga.generation.best_fitness`, etc.)
- `MetricsObserver` struct: `pub struct MetricsObserver { run_id: &'static str }` with `pub fn new(run_id: &'static str) -> Self`
- `run_id` is `&'static str` — attached as label to every metric
- No mutable state in `MetricsObserver` — all emission via `metrics` macros
- Feature flag name: `observer-metrics`; Cargo.toml entry: `metrics = { version = "0.24", optional = true }` + `observer-metrics = ["dep:metrics"]`
- Entire `src/observer/metrics_observer.rs` is `#[cfg(feature = "observer-metrics")]`
- COMP-03: metric calls restricted to sequential per-generation hooks only — no `metrics::*` inside `par_iter()` closures
- Scope: `src/observer/composite.rs`, `src/observer/metrics_observer.rs`, `Cargo.toml`, `src/observer/mod.rs`, `src/lib.rs` — no changes to `ga.rs`, `island/mod.rs`, `nsga2/mod.rs`

### Claude's Discretion

- Whether `CompositeObserver` derives `Clone`
- Whether `CompositeObserver::new()` pre-allocates the internal Vec
- Whether `MetricsObserver` implements `Default` in addition to `new()`
- Exact `metrics` crate version (verify against crates.io)
- Whether an `is_some()` guard is needed before emitting metrics (metrics facade is zero-cost when no recorder — guard likely unnecessary)
- Integration test strategy for COMP-01 through COMP-03

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| COMP-01 | User can combine multiple observers via `CompositeObserver`; all three trait interfaces fan out to every attached observer | `AllObserver<U>` supertrait + blanket impl; composite `Vec<Arc<dyn AllObserver<U>>>` inner storage; per-hook iteration pattern |
| COMP-02 | User can attach `MetricsObserver` (behind `observer-metrics` feature flag); per-generation counters, gauges, histograms via `metrics` facade | `metrics 0.24.3` macro syntax verified; feature-gate pattern from `observer-tracing` precedent; zero-cost noop when no recorder installed |
| COMP-03 | `MetricsObserver` safe inside island parallel execution — metric calls restricted to sequential hooks, never inside `par_iter()` closures | `metrics` macros are thread-safe by design; COMP-03 enforced by hook-placement policy not by runtime locking |
</phase_requirements>

---

## Summary

Phase 17 adds two orthogonal observer types: `CompositeObserver<U>` (fan-out aggregator) and `MetricsObserver` (metrics-facade emitter). Both are pure library additions in `src/observer/`; no engine files change.

`CompositeObserver` introduces the `AllObserver<U>` supertrait so the planner can represent a homogeneous `Vec<Arc<dyn AllObserver<U> + Send + Sync>>` inner store. The composite itself satisfies `AllObserver<U>`, so it plugs into any of the three GA engines via their existing `with_observer()` methods. The fan-out pattern is already demonstrated by `ga.rs::notify()` — the composite just does the same iteration over its inner Vec.

`MetricsObserver` follows the `TracingObserver` structural template exactly: a `#[cfg(feature = "observer-metrics")]`-gated file, optional dep in `Cargo.toml`, `impl<U: ChromosomeT> GaObserver<U>` block, `IslandGaObserver<U>` block, and `Nsga2Observer<U>` block. The `metrics 0.24.3` crate confirmed current. Its macros return handles (`gauge!(...).set(v)`, `counter!(...).increment()`, `histogram!(...).record(v)`) — they are thread-safe and zero-cost when no recorder is installed, so no guard is needed.

**Primary recommendation:** Implement in three tasks — (1) `AllObserver<U>` supertrait + `CompositeObserver`, (2) `MetricsObserver` + feature-flag wiring, (3) integration tests and benchmark.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `metrics` | 0.24.3 | Metrics facade: gauges, counters, histograms | Verified current via `cargo search`; chosen by user decision |

### Supporting (dev / test only)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `criterion` | 0.8.2 (already in dev-deps) | Benchmark harness for COMP-03 island parallel safety check | Already present; add bench target for `metrics_observer` |
| `metrics-util` | (latest compatible with 0.24) | In-memory recorder for test assertions | Optional; CountingObserver pattern (already established) may be sufficient for COMP-01/COMP-02 |

**Installation (Cargo.toml additions):**
```toml
metrics = { version = "0.24", optional = true }

[features]
observer-metrics = ["dep:metrics"]
```

**Version verification:** `cargo search metrics` confirmed `metrics = "0.24.3"` as current stable (2026-03-27). The CONTEXT.md specifies `"0.24"` — the `^0.24` semver range covers 0.24.3 so this is correct.

---

## Architecture Patterns

### Recommended File Layout
```
src/observer/
├── mod.rs              # Add: AllObserver trait + blanket impl, mod composite, mod metrics_observer (cfg-gated)
├── log.rs              # Unchanged — already AllObserver-compliant
├── tracing_observer.rs # Unchanged
├── composite.rs        # NEW: CompositeObserver<U>
└── metrics_observer.rs # NEW: MetricsObserver (cfg observer-metrics)

tests/
├── test_composite_observer.rs    # NEW: COMP-01 fan-out tests
└── test_metrics_observer.rs      # NEW: COMP-02/COMP-03 (cfg-gated)

benches/
└── metrics_observer.rs           # NEW: COMP-03 island parallel benchmark
```

### Pattern 1: AllObserver Supertrait + Blanket Impl

`GaObserver<U>`, `IslandGaObserver<U>`, and `Nsga2Observer<U>` are already defined in `src/observer/mod.rs`. Adding `AllObserver<U>` there alongside a blanket impl lets any type that satisfies all three bounds automatically become an `AllObserver<U>` — no extra `impl` needed for `LogObserver` or `MetricsObserver`.

```rust
// src/observer/mod.rs — append after the three trait definitions
pub trait AllObserver<U: ChromosomeT>:
    GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U>
    + Send + Sync {}

impl<U, T> AllObserver<U> for T
where
    U: ChromosomeT,
    T: GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U> + Send + Sync,
{}
```

**Why this works:** Blanket impls over foreign traits are allowed when the supertrait is defined in the same crate. `LogObserver` and `MetricsObserver` automatically satisfy `AllObserver<U>` without extra boilerplate.

### Pattern 2: CompositeObserver Inner Storage

The `Vec<Arc<dyn AllObserver<U> + Send + Sync>>` storage is straightforward. `AllObserver<U>` already requires `Send + Sync`, so the redundant bounds in the `dyn` position are technically a no-op but make the type unambiguous.

```rust
// src/observer/composite.rs
use std::sync::Arc;
use crate::observer::{AllObserver, GaObserver, IslandGaObserver, Nsga2Observer, ExtensionEvent};
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
use crate::ga::TerminationCause;
use std::time::Duration;

pub struct CompositeObserver<U: ChromosomeT> {
    observers: Vec<Arc<dyn AllObserver<U> + Send + Sync>>,
}

impl<U: ChromosomeT> CompositeObserver<U> {
    pub fn new() -> Self {
        Self { observers: Vec::new() }
    }

    pub fn add(mut self, obs: Arc<dyn AllObserver<U> + Send + Sync>) -> Self {
        self.observers.push(obs);
        self
    }
}
```

**Fan-out pattern** — copy of `ga.rs::notify()` applied to Vec:
```rust
impl<U: ChromosomeT> GaObserver<U> for CompositeObserver<U> {
    fn on_run_start(&self) {
        for obs in &self.observers { obs.on_run_start(); }
    }
    fn on_generation_end(&self, stats: &GenerationStats) {
        for obs in &self.observers { obs.on_generation_end(stats); }
    }
    // ... all 12 hooks follow same pattern
}
```

### Pattern 3: MetricsObserver — metrics macro call syntax (verified 0.24.3)

The `metrics` 0.24 macros return handles; you call methods on the handle:

```rust
// gauge — set an absolute value
metrics::gauge!("ga.generation.best_fitness", "run_id" => self.run_id)
    .set(stats.best_fitness);

// counter — increment by 1
metrics::counter!("ga.event.new_best", "run_id" => self.run_id)
    .increment(1);

// histogram — record a duration value
metrics::histogram!("ga.operator.selection_ms", "run_id" => self.run_id)
    .record(duration.as_secs_f64() * 1000.0);
```

**No `is_some()` guard needed.** Official docs confirm: when no recorder is installed, a noop recorder is active. Atomic load + comparison — negligible overhead. Zero heap allocation. This is the same guarantee `tracing` events have when no subscriber is active.

### Pattern 4: Feature-Gate Structure (mirrors TracingObserver exactly)

```toml
# Cargo.toml
metrics = { version = "0.24", optional = true }
observer-metrics = ["dep:metrics"]
```

```rust
// src/observer/mod.rs — append at end
#[cfg(feature = "observer-metrics")]
mod metrics_observer;
#[cfg(feature = "observer-metrics")]
pub use metrics_observer::MetricsObserver;
```

```rust
// src/lib.rs — append to observer re-exports
#[cfg(feature = "observer-metrics")]
pub use observer::MetricsObserver;
```

### Anti-Patterns to Avoid

- **Calling `metrics::*` inside `par_iter()` closures:** While `metrics` macros are thread-safe, COMP-03 explicitly restricts metric emission to sequential hooks. The hooks themselves (`on_generation_end`, `on_island_generation_end`, etc.) are called sequentially by the engine. Never add metric calls inside the `par_iter_mut()` closure that runs island worker threads.
- **Using `&mut self` hooks in `AllObserver`:** All three observer traits use `&self` — this is a hard requirement for `Arc`-sharing. Never change hook signatures.
- **Storing `EnteredSpan` or scoped guards in MetricsObserver:** Unlike `TracingObserver`, `MetricsObserver` has no span state. All metric calls are fire-and-forget via macros. No `Mutex` needed.
- **Adding `Default` impl for `MetricsObserver` without a sensible `run_id`:** If `Default` is implemented, `run_id` must be a meaningful static string (e.g., `"default"`). Discretion item — evaluate whether it adds user value.
- **`dyn AllObserver<U>` object safety:** `AllObserver<U>` is a supertrait of three object-safe traits. Confirm no generic methods are added to `AllObserver<U>` itself — it must remain object-safe for `dyn` usage.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Thread-safe metric counters/gauges | Custom `AtomicF64` wrappers or `Mutex<HashMap>` | `metrics 0.24` facade | Thread safety, recorder pluggability, zero-cost noop are all solved |
| Metrics backend (Prometheus, StatsD) | Any bundled exporter | User's responsibility via recorder | Library emits, user routes — same contract as `tracing` subscribers |
| Fan-out dispatch | Custom enum dispatch | `Vec<Arc<dyn AllObserver<U>>>` iteration | Zero extra machinery; already used by `ga.rs::notify()` |

**Key insight:** The `metrics` crate noop recorder means `MetricsObserver` can safely be constructed and attached with zero overhead even when the user hasn't installed a recorder backend.

---

## Common Pitfalls

### Pitfall 1: Object Safety of AllObserver

**What goes wrong:** Adding any generic associated function or generic method to `AllObserver<U>` breaks `dyn AllObserver<U>` usage. `CompositeObserver` stores `dyn AllObserver<U>`, which requires object safety.
**Why it happens:** Rust's object safety rules prohibit `dyn Trait` when the trait has associated generics beyond those from the containing trait's own parameters.
**How to avoid:** `AllObserver<U>` must have zero methods of its own — it is a pure supertrait marker. All methods come from `GaObserver<U>`, `IslandGaObserver<U>`, and `Nsga2Observer<U>`, which are already object-safe.
**Warning signs:** `error[E0038]: the trait AllObserver cannot be made into an object`

### Pitfall 2: metrics 0.24 macro call syntax changed from 0.22/0.23

**What goes wrong:** Older `metrics` docs (pre-0.23) show direct value-passing syntax like `gauge!("name", value)`. In 0.24 the macros return handles and you chain `.set()` / `.increment()` / `.record()`.
**Why it happens:** Breaking API change in the metrics crate's major refactor.
**How to avoid:** Use the handle-chained syntax confirmed from official 0.24.3 docs. The CONTEXT.md's example snippets use the old format — adapt them to the current API:
  - `gauge!("ga.generation.best_fitness", best_fitness, "run_id" => ...)` → `gauge!("ga.generation.best_fitness", "run_id" => ...).set(best_fitness)`
  - `counter!("ga.event.new_best", 1, "run_id" => ...)` → `counter!("ga.event.new_best", "run_id" => ...).increment(1)`
  - `histogram!("ga.operator.selection_ms", dur_ms, "run_id" => ...)` → `histogram!("ga.operator.selection_ms", "run_id" => ...).record(dur_ms)`
**Warning signs:** Compile error referencing wrong number of arguments to macro.

### Pitfall 3: CompositeObserver and GaObserver<U> type alias clash

**What goes wrong:** `ga.rs` stores `Option<Arc<dyn GaObserver<U> + Send + Sync>>`. When a user attaches `Arc::new(composite)`, `CompositeObserver<U>` must implement `GaObserver<U>` — and it does — but the `AllObserver<U>` bound on `.add()` must not prevent attaching the composite to a `Ga<U>` (which only holds `GaObserver<U>`).
**Why it happens:** `CompositeObserver<U>` implements `GaObserver<U>` directly, so `Arc::new(composite)` coerces to `Arc<dyn GaObserver<U>>` without any `AllObserver<U>` involvement.
**How to avoid:** Keep the engine's `with_observer()` signatures unchanged — they accept `Arc<dyn SpecificObserver<U>>`, not `Arc<dyn AllObserver<U>>`. The composite coerces to whichever observer the engine needs.

### Pitfall 4: `GaObserver<U>` not in scope for IslandGaObserver impl block

**What goes wrong:** `src/observer/composite.rs` must implement all three traits, which requires importing all three plus their dependent types (`GenerationStats`, `TerminationCause`, `ExtensionEvent`, `Duration`).
**Why it happens:** Each trait uses types from different modules.
**How to avoid:** Add a full import block at top of `composite.rs` mirroring `log.rs` — it already imports all three traits and their dependencies.

### Pitfall 5: Benchmark cfg gates

**What goes wrong:** A `benches/metrics_observer.rs` benchmark with `#[cfg(feature = "observer-metrics")]`-gated code inside a non-feature-gated bench binary can produce compile errors when built without the feature.
**Why it happens:** `[[bench]]` entries in `Cargo.toml` are always compiled, unlike test files which can use `#![cfg(...)]`.
**How to avoid:** Either gate the entire bench binary behind `required-features = ["observer-metrics"]` in `Cargo.toml`, or restructure the benchmark to use `#[cfg(feature = "observer-metrics")]` blocks and fall back to a trivial benchmark when the feature is off.

---

## Code Examples

Verified patterns from official sources and project codebase:

### AllObserver Supertrait + Blanket Impl
```rust
// src/observer/mod.rs — Source: CONTEXT.md (locked decision)
pub trait AllObserver<U: ChromosomeT>:
    GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U>
    + Send + Sync {}

impl<U, T> AllObserver<U> for T
where
    U: ChromosomeT,
    T: GaObserver<U> + IslandGaObserver<U> + Nsga2Observer<U> + Send + Sync,
{}
```

### CompositeObserver Fan-Out (GaObserver hooks)
```rust
// src/observer/composite.rs — pattern from ga.rs::notify()
impl<U: ChromosomeT> GaObserver<U> for CompositeObserver<U> {
    fn on_run_start(&self) {
        for obs in &self.observers { obs.on_run_start(); }
    }
    fn on_generation_start(&self, generation: usize) {
        for obs in &self.observers { obs.on_generation_start(generation); }
    }
    fn on_selection_complete(&self, generation: usize, duration: Duration, population_size: usize) {
        for obs in &self.observers { obs.on_selection_complete(generation, duration, population_size); }
    }
    // ... all 12 hooks follow the same single-line for loop pattern
}
```

### MetricsObserver on_generation_end (verified 0.24.3 syntax)
```rust
// src/observer/metrics_observer.rs — Source: docs.rs/metrics/0.24.3
fn on_generation_end(&self, stats: &GenerationStats) {
    metrics::gauge!("ga.generation.best_fitness", "run_id" => self.run_id)
        .set(stats.best_fitness);
    metrics::gauge!("ga.generation.mean_fitness", "run_id" => self.run_id)
        .set(stats.avg_fitness);
    metrics::gauge!("ga.generation.diversity", "run_id" => self.run_id)
        .set(stats.diversity);
}
```

Note: The CONTEXT.md uses `mean_fitness` as the metric key but `GenerationStats` has `avg_fitness` as the field name. The metric key `ga.generation.mean_fitness` is correct (dashboard-facing name); the value source is `stats.avg_fitness`.

### MetricsObserver operator timing hooks (verified 0.24.3 syntax)
```rust
fn on_selection_complete(&self, _generation: usize, duration: Duration, _population_size: usize) {
    metrics::histogram!("ga.operator.selection_ms", "run_id" => self.run_id)
        .record(duration.as_secs_f64() * 1000.0);
}
```

### MetricsObserver event counter hooks (verified 0.24.3 syntax)
```rust
fn on_new_best(&self, _generation: usize, _best: U) {
    metrics::counter!("ga.event.new_best", "run_id" => self.run_id)
        .increment(1);
}

fn on_stagnation(&self, _generation: usize, _stagnation_count: usize) {
    metrics::counter!("ga.event.stagnation", "run_id" => self.run_id)
        .increment(1);
}

fn on_extension_triggered(&self, _event: ExtensionEvent) {
    metrics::counter!("ga.event.extension_triggered", "run_id" => self.run_id)
        .increment(1);
}
```

### Cargo.toml feature-gate (mirrors observer-tracing pattern)
```toml
# [dependencies]
metrics = { version = "0.24", optional = true }

# [features]
observer-metrics = ["dep:metrics"]
```

### Bench binary with required-features guard
```toml
# Cargo.toml — [[bench]] entry
[[bench]]
name = "metrics_observer"
harness = false
required-features = ["observer-metrics"]
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `metrics` direct value macro: `gauge!("name", value)` | Handle-chaining: `gauge!("name").set(value)` | metrics 0.23 → 0.24 | All metric macro calls must use the chained form |
| Separate observer per engine type | Single `AllObserver<U>` satisfies all three engines | Phase 17 (this phase) | One composite attaches to Ga, IslandGa, Nsga2Ga interchangeably |

---

## Open Questions

1. **`CompositeObserver: Clone`**
   - What we know: `Arc<dyn AllObserver<U>>` is `Clone` (cloning the Arc). The Vec itself is cloneable.
   - What's unclear: Whether users need to clone a composite (e.g., attach to two engines simultaneously)
   - Recommendation: Derive `Clone` — minimal cost, aids the multi-engine use case from CONTEXT.md. Discretion item per CONTEXT.md.

2. **`MetricsObserver: Default`**
   - What we know: `run_id: &'static str` has no obvious default. `"default"` is a reasonable static string.
   - What's unclear: Whether users would ever construct `MetricsObserver::default()` vs `new("run_id")`
   - Recommendation: Implement `Default` with `run_id: "default"` — provides symmetry with `TracingObserver::default()`. Discretion item per CONTEXT.md.

3. **`metrics-util` for test assertions**
   - What we know: `metrics-util` provides an in-memory recorder for asserting metric values in tests. Adds a dev-dependency.
   - What's unclear: Whether the overhead is justified vs a simple `CountingObserver` pattern (already established in test_observer.rs) that doesn't require a recorder at all
   - Recommendation: Prefer `CountingObserver` approach for COMP-01 (no recorder needed). For COMP-02 tests verifying metric values are actually emitted, add `metrics-util` to dev-dependencies if needed. The noop-recorder approach is sufficient to prove COMP-03 (no panic/data race).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + criterion 0.8.2 |
| Config file | none (cargo-native) |
| Quick run command | `cargo test test_composite_observer` |
| Full suite command | `cargo test && cargo test --features observer-metrics` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-01 | CompositeObserver fans out GaObserver hooks to all inner observers | unit | `cargo test test_composite_observer_ga_hooks` | Wave 0 |
| COMP-01 | CompositeObserver fans out IslandGaObserver hooks | unit | `cargo test test_composite_observer_island_hooks` | Wave 0 |
| COMP-01 | CompositeObserver fans out Nsga2Observer hooks | unit | `cargo test test_composite_observer_nsga2_hooks` | Wave 0 |
| COMP-01 | AllObserver supertrait compile-time bounds check | compile | `cargo test test_all_observer_bounds` | Wave 0 |
| COMP-02 | MetricsObserver attaches and runs 10 generations without panic | integration | `cargo test --features observer-metrics test_metrics_observer_runs` | Wave 0 |
| COMP-02 | MetricsObserver is Send + Sync | compile | `cargo test --features observer-metrics test_metrics_observer_send_sync` | Wave 0 |
| COMP-02 | Default build (`cargo build`) succeeds without metrics crate | build | `cargo build` (no features) | existing |
| COMP-03 | MetricsObserver in island parallel execution — no panic/data race | benchmark | `cargo bench --features observer-metrics --bench metrics_observer` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test` (default features)
- **Per wave merge:** `cargo test && cargo test --features observer-metrics && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/test_composite_observer.rs` — covers COMP-01 (fan-out for all three traits)
- [ ] `tests/test_metrics_observer.rs` — covers COMP-02/COMP-03 (`#![cfg(feature = "observer-metrics")]` at file top)
- [ ] `benches/metrics_observer.rs` — covers COMP-03 island parallel benchmark (requires `required-features = ["observer-metrics"]` in `Cargo.toml`)

---

## Sources

### Primary (HIGH confidence)
- `src/observer/mod.rs` — existing trait definitions (`GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>`)
- `src/observer/log.rs` — multi-trait impl template for `AllObserver<U>`-compliant types
- `src/observer/tracing_observer.rs` — direct structural template for `MetricsObserver` file layout and feature-gate pattern
- `src/ga.rs` — `notify()` fan-out helper (line 556), confirmed pattern for composite iteration
- `Cargo.toml` — existing `observer-tracing = ["dep:tracing"]` feature-flag pattern
- `src/stats.rs` — `GenerationStats` fields (`best_fitness`, `avg_fitness`, `diversity`) used in gauges
- `https://docs.rs/metrics/0.24.3/metrics/` — official metrics 0.24.3 macro syntax verified
- `https://docs.rs/metrics/0.24.3/metrics/macro.gauge.html` — handle-chained `.set()` syntax confirmed

### Secondary (MEDIUM confidence)
- `cargo search metrics` output (2026-03-27) confirming `metrics = "0.24.3"` as current stable

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — metrics 0.24.3 confirmed via `cargo search` and official docs
- Architecture: HIGH — composite pattern from existing `ga.rs::notify()`, feature-gate from `tracing_observer.rs`, all confirmed from source
- Pitfalls: HIGH — metrics macro API change (0.22→0.24) verified from official docs; others from code inspection

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (metrics facade is stable; composite pattern is internal — both stable for 30 days)
