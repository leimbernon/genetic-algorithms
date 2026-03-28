# Technology Stack

**Project:** genetic_algorithms — v2.2.0 Observability & Traceability
**Researched:** 2026-03-25
**Confidence:** HIGH (all versions verified via `cargo info` against crates.io)
**Scope:** NEW dependencies only. Existing stack (rand, rayon, log, env_logger, serde, plotters) is unchanged.

---

## Dependency Changes Summary

This milestone adds two optional crates behind feature flags and removes nothing. The existing
`log` + `env_logger` setup is NOT replaced — it stays as the default logging path via LogObserver.
No existing feature flags change. Two new flags are added: `observer-tracing` and `observer-metrics`.

---

## New Dependencies

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `tracing` | `0.1.44` | Structured spans and events emitted by TracingObserver | De facto standard structured tracing facade in the Rust async/sync ecosystem. Zero-cost when no subscriber is installed (atomic load early-exit). `Send + Sync` safe. Spans compose naturally with GA lifecycle events (per-generation, per-run). MSRV 1.65.0 — compatible with project MSRV 1.81.0. |
| `metrics` | `0.24.3` | Counter/gauge/histogram recording emitted by MetricsObserver | Pure facade: macros are no-ops when no recorder is installed, identical architectural role to `log` for logging. Users install a recorder (metrics-exporter-prometheus, metrics-exporter-statsd, etc.) independently — the library stays backend-agnostic. MSRV 1.71.1 — compatible with project MSRV 1.81.0. |

Both crates are **optional** — gated behind feature flags, not in the default build.

**NOT added: `tracing-subscriber`.** That is a backend concern. Users who enable `observer-tracing`
bring their own subscriber (fmt, OpenTelemetry exporter, Jaeger, etc.). The library emits; users route.

**NOT added: `metrics-exporter-prometheus`, `metrics-exporter-statsd`, or any concrete recorder.**
Same facade rationale as `tracing`.

---

## No New Dependencies (Confirmed)

| Concern | Decision | Rationale |
|---------|----------|-----------|
| LogObserver | No new dep | Uses existing `log 0.4.22` already in `[dependencies]`. LogObserver migrates all 8 log targets — zero new cost. |
| CompositeObserver | No new dep | Pure Rust: `Vec<Arc<dyn GaObserver<U> + Send + Sync>>`, dispatch via iteration. |
| Island/NSGA-II observer sub-traits | No new dep | Trait definitions only; extend existing module structure. |
| `env_logger` | Keep as-is | Still initializes the global log subscriber in `run_with_callback`. Not removed. LogObserver becomes a wrapper over it. |
| `GaObserver` trait itself | No new dep | Always compiled, no feature gate. It is the public API surface. |

---

## Cargo.toml Changes

```toml
[features]
default = []
serde = ["dep:serde", "dep:serde_json"]
visualization = ["dep:plotters"]
observer-tracing = ["dep:tracing"]
observer-metrics = ["dep:metrics"]

[dependencies]
# ... all existing deps unchanged ...
tracing = { version = "0.1", optional = true }
metrics = { version = "0.24", optional = true }
```

**Pin to minor, not patch** (`"0.1"` not `"0.1.44"`) — standard Rust crate convention. Cargo will
resolve to the latest compatible patch automatically.

**Feature naming rationale:** `observer-tracing` and `observer-metrics` (not bare `tracing`/`metrics`):
1. Avoids shadowing the crate names if users also depend on those crates directly.
2. Communicates intent — these flags enable the *observer implementations*, not raw access to the crates.
3. Consistent with `serde` precedent: feature name maps to what it unlocks, not just the dep name.

---

## Supporting Libraries (No Code Changes, Referenced for Awareness)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing-subscriber` | user-side | Routes tracing spans to stdout/file/OTEL | Users who enable `observer-tracing` and want to see output |
| `metrics-exporter-prometheus` | user-side | Prometheus scrape endpoint for metrics | Users who enable `observer-metrics` and export to Prometheus |
| `opentelemetry` | user-side | Full OTel pipeline (traces + metrics) | Users wanting vendor-neutral telemetry; bridges via `tracing-opentelemetry` |

These are never added to the library's `Cargo.toml` — they go in user application code only.

---

## Observer Pattern: Thread Safety Requirements

All observers must be `Send + Sync` because:
- `Ga<U>` uses rayon for parallel fitness evaluation
- `IslandGa<U>` runs islands in parallel rayon threads
- The observer is stored as `Option<Arc<dyn GaObserver<U> + Send + Sync>>`

Both `tracing` and `metrics` macros are safe to call from multiple threads. The `Arc<dyn ...>` wrapper
provides shared ownership without mutation. `Arc` (not `Box`) is the correct storage because island
threads need shared access without requiring `Clone` on the observer.

---

## Zero-Overhead Contract

The zero-overhead requirement is met at three levels:

**Level 1 — No observer set (most users):**
`Option::None` branch in generated code. The `if let Some(obs) = &self.observer` check is a
single branch-prediction miss at worst. No allocation, no vtable dispatch, no measurement.

**Level 2 — Observer set but feature not compiled in:**
`#[cfg(feature = "observer-tracing")]` gates ensure no `tracing` code is compiled into default builds.
Users who do not enable `observer-tracing` pay zero binary size cost.

**Level 3 — Feature compiled in but no subscriber/recorder installed:**
- `tracing 0.1.44`: if no `Subscriber` is set globally, span creation is a nanosecond-scale overhead
  (atomic load + early return). Acceptable for per-generation lifecycle events (not per-gene hot paths).
- `metrics 0.24.3`: if no `Recorder` is installed, `metrics::counter!()` etc. are no-ops (single
  atomic load check).

**Observer hooks are never placed in per-gene hot loops.** Generation-level callbacks (once per
generation, not once per fitness evaluation) are the correct hook granularity.

---

## Feature Flag Pattern: Integration with Existing Codebase

The existing `serde` feature is the model. Applied to observers:

```rust
// In lib.rs — GaObserver trait is always compiled (no feature gate needed)
pub mod observer;  // always present

// In observer/mod.rs — trait definition always compiled
pub trait GaObserver<U: ChromosomeT>: Send + Sync { ... }

// In observer/log_observer.rs — always compiled (uses existing log dep)
pub struct LogObserver { ... }

// In observer/tracing_observer.rs — gated
#[cfg(feature = "observer-tracing")]
pub mod tracing_observer;

// In observer/metrics_observer.rs — gated
#[cfg(feature = "observer-metrics")]
pub mod metrics_observer;
```

**Default no-op methods** on `GaObserver` are the forward-compatibility mechanism. New event hooks
added in v2.3+ do not break existing observer implementations.

---

## MSRV Compatibility

| Crate | Version | MSRV | Compatible with 1.81.0 |
|-------|---------|------|------------------------|
| `tracing` | 0.1.44 | 1.65.0 | YES |
| `metrics` | 0.24.3 | 1.71.1 | YES |

Both verified via `cargo info` against live crates.io index (2026-03-25).

---

## Log Target Inventory (for LogObserver Migration)

Existing `log!()` call sites that LogObserver must reproduce identically:

| Target | Approx Sites | Level | Location |
|--------|-------------|-------|----------|
| `ga_events` | ~12 | info/debug/trace | ga.rs, population.rs |
| `population_events` | ~3 | debug/trace | population.rs |
| `chromosome_events` | ~1 | debug | population.rs |
| `selection_events` | ~25 | debug/trace | selection operators |
| `crossover_events` | ~20 | debug/trace | crossover operators |
| `mutation_events` | ~18 | debug/trace | mutation operators |
| `survivor_events` | ~9 | debug/trace | survivor operators |

**Total:** ~88 call sites across 8 targets. LogObserver maps observer events to the matching
target+level. Backward compatibility means identical log output when LogObserver is active.

---

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| `tracing` facade | `opentelemetry` direct | OTel is a backend, not a facade; couples library to a telemetry vendor; `tracing-opentelemetry` bridge gives users this path without library coupling |
| `metrics` facade | `prometheus` crate direct | Same vendor coupling; `metrics` crate provides the identical facade pattern as `log`/`tracing`; users bring their own exporter |
| `metrics` facade | `statsd` crate direct | Same issue; backend choice belongs to the user application |
| `Arc<dyn GaObserver + Send + Sync>` | `Box<dyn GaObserver + Send + Sync>` | `Arc` enables shared ownership across island threads without requiring `Clone`; `Box` would force single-owner constraint incompatible with island parallelism |
| `Option<Arc<...>>` | Always-present no-op observer struct | `Option::None` is provably zero-cost; a struct-based no-op still allocates the Arc and has vtable dispatch overhead |
| Keep `env_logger` | Remove `env_logger` | Initializes the global log subscriber in `run_with_callback`; removing it breaks existing LogLevel-based initialization |

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `tracing 0.1` | `tracing-subscriber 0.3` | Both in tokio-rs org; 0.1/0.3 are the stable long-running versions |
| `metrics 0.24` | `metrics-exporter-prometheus 0.15+` | Exporter version must match metrics facade version; users verify their own exporter compatibility |
| `tracing 0.1` | `tracing-opentelemetry 0.27+` | Bridges tracing spans to OTel; user-side only |

---

## Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo info <crate>` | Verify exact crate version + MSRV before pinning | Used to verify both tracing and metrics for this research |
| `cargo test --features observer-tracing,observer-metrics` | CI gate for feature-flagged code | Add to CI matrix alongside existing `--features serde` |

---

## Sources

- `cargo info tracing` (live crates.io, 2026-03-25) — version 0.1.44, MSRV 1.65.0. HIGH confidence.
- `cargo info metrics` (live crates.io, 2026-03-25) — version 0.24.3, MSRV 1.71.1. HIGH confidence.
- `cargo search tracing` (live crates.io, 2026-03-25) — confirmed 0.1.44 is current stable.
- `cargo search metrics` (live crates.io, 2026-03-25) — confirmed 0.24.3 is current stable.
- Existing `Cargo.toml`: `/Users/luis/RustroverProjects/genetic-algorithms/Cargo.toml` — feature flag pattern, existing deps.
- `PROJECT.md`: `/Users/luis/RustroverProjects/genetic-algorithms/.planning/PROJECT.md` — constraints, decisions, out-of-scope list.
- `CLAUDE.md`: project MSRV 1.81.0, edition 2021, feature flag conventions, observer-tracing/observer-metrics flag names.

---
*Stack research for: GaObserver observability system (v2.2.0 Observability & Traceability)*
*Researched: 2026-03-25*
