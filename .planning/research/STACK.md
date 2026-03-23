# Technology Stack

**Project:** genetic_algorithms — v2.1.0 Observability & Traceability
**Researched:** 2026-03-23
**Scope:** NEW dependencies only. Existing stack (rand, rayon, log, env_logger, serde) is unchanged.

---

## Dependency Changes Summary

This milestone adds two optional crates behind feature flags and removes nothing. The existing
`log` + `env_logger` setup is NOT replaced — it stays as the default logging path via LogObserver.

---

## New Dependencies

### observer-tracing Feature

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `tracing` | `0.1` | Structured spans and events emitted by TracingObserver | De facto standard structured tracing facade in the Rust async/sync ecosystem; zero-cost when no subscriber is installed; `Send + Sync` safe; spans compose naturally with generation lifecycle events |

**Confidence:** HIGH — `tracing 0.1` has been the stable, shipping version since 2021 and the crate follows a slow semver cadence. No 0.2 has been released as of August 2025.

**NOT added:** `tracing-subscriber`. That is a backend concern. Users who enable `observer-tracing` bring their own subscriber (fmt, OpenTelemetry, Jaeger exporter, etc.). The library emits; users route.

### observer-metrics Feature

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `metrics` | `0.24` | Counter/gauge/histogram recording emitted by MetricsObserver | The `metrics` crate is a pure facade: zero-cost macros expand to nothing when no recorder is installed. Exact same architectural role as `log` for logging. Users install a recorder (metrics-exporter-prometheus, metrics-exporter-statsd, etc.) independently. `Send + Sync` compatible. |

**Confidence:** MEDIUM — `metrics` 0.24 was the current release as of mid-2025. Verify exact minor version with `cargo search metrics` before pinning; the `0.24` minor series is used here because the API stabilized around the `Recorder` trait in 0.22+ and no breaking 1.0 had shipped as of August 2025.

**NOT added:** `metrics-exporter-prometheus`, `metrics-exporter-statsd`, or any concrete recorder. Same facade rationale as `tracing`.

---

## No New Dependencies (Confirmed)

| Concern | Decision | Rationale |
|---------|----------|-----------|
| LogObserver | No new dep | Uses existing `log` crate already in `[dependencies]`. LogObserver is a zero-cost migration of existing `log!()` calls — all 8 targets preserved. |
| CompositeObserver | No new dep | Pure Rust: `Vec<Arc<dyn GaObserver<U>>>`, dispatch via iteration. |
| Island/NSGA-II observer sub-traits | No new dep | Trait definitions only; extend existing module structure. |
| `env_logger` | Keep as-is | Still needed for LogObserver to initialize the global logger in `run_with_callback`. Not removed. |

---

## Cargo.toml Changes

```toml
[features]
default = []
serde = ["dep:serde", "dep:serde_json"]
observer-tracing = ["dep:tracing"]
observer-metrics = ["dep:metrics"]

[dependencies]
# ... existing deps unchanged ...
tracing = { version = "0.1", optional = true }
metrics = { version = "0.24", optional = true }
```

**Feature naming rationale:** `observer-tracing` and `observer-metrics` (not `tracing`/`metrics` alone)
because:
1. Avoids collision with the crate names if users also depend on those crates directly.
2. Communicates intent — these flags enable the *observer implementations*, not raw access to the crates.
3. Consistent with the `serde` precedent in this codebase (feature name matches what it unlocks, not just the dep name).

---

## Observer Pattern: Thread Safety Requirements

All observers must be `Send + Sync` because:
- `Ga<U>` uses `rayon` for parallel fitness evaluation
- `IslandGa<U>` runs islands in parallel `rayon` threads
- The observer is stored as `Option<Arc<dyn GaObserver<U> + Send + Sync>>`

Both `tracing` and `metrics` macros are safe to call from multiple threads. The `Arc<dyn ...>` wrapper
provides shared ownership without mutation.

---

## Zero-Overhead Contract

The zero-overhead requirement is met at two levels:

**Level 1 — No observer set (most users):**
`Option::None` branch in the generated code. The `if let Some(obs) = &self.observer` check is a
single branch prediction miss at worst. No allocation, no measurement.

**Level 2 — Observer set but feature not compiled in:**
`#[cfg(feature = "observer-tracing")]` gates ensure no `tracing` code is compiled into default builds.
Users who do not enable `observer-tracing` pay zero binary cost.

**Level 3 — Feature compiled in but no subscriber/recorder installed:**
- `tracing`: if no `Subscriber` is set globally, span creation is a few nanoseconds of overhead (atomic
  load + early return). Acceptable for non-hot-path lifecycle events (per-generation, not per-gene).
- `metrics`: if no `Recorder` is installed, `metrics::counter!()` etc. are no-ops (atomic load check).

Generation-level observer hooks (once per generation) are not hot-path operations. The per-gene operator
calls that are hot-path remain ungated and are not candidate sites for observer hooks.

---

## Feature Flag Pattern: How It Works in This Codebase

The existing `serde` feature is the model. Applied to observers:

```rust
// In lib.rs — gate the observer module itself
#[cfg(any(feature = "observer-tracing", feature = "observer-metrics"))]
pub mod observer;

// In observer/mod.rs — always compile GaObserver trait (no feature needed)
// In observer/tracing_observer.rs
#[cfg(feature = "observer-tracing")]
pub mod tracing_observer;

// In observer/metrics_observer.rs
#[cfg(feature = "observer-metrics")]
pub mod metrics_observer;
```

The `GaObserver` trait itself does NOT need a feature flag — it is always compiled and is the
public API surface users implement. Feature flags gate only the concrete observer implementations
that pull in optional deps.

**Default no-op methods** on `GaObserver` are the forward-compatibility mechanism. New event hooks
added in future versions do not break existing observer implementations.

---

## Log Target Inventory (for LogObserver Migration)

Existing `log!()` call sites that LogObserver must reproduce identically:

| Target | Sites | Level | Location |
|--------|-------|-------|----------|
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

## MSRV Compatibility

- Project MSRV: Rust 1.81.0 (edition 2021)
- `tracing 0.1`: MSRV is 1.56 — compatible
- `metrics 0.24`: MSRV is approximately 1.70 — compatible with 1.81.0

No MSRV breakage from either new dep.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Tracing facade | `tracing` | `opentelemetry` direct | OTel is a backend, not a facade; would couple library to a specific telemetry vendor; tracing-opentelemetry bridge exists for users who need OTel |
| Metrics facade | `metrics` | `prometheus` direct | Same vendor coupling problem; `metrics` crate provides the same facade-pattern abstraction as `log`/`tracing` |
| Metrics facade | `metrics` | `statsd` direct | Same issue; backend choice belongs to user |
| Observer storage | `Arc<dyn GaObserver + Send + Sync>` | `Box<dyn GaObserver + Send + Sync>` | `Arc` enables shared ownership across island threads without cloning; `Box` would require `Clone` on observer or single-owner constraint |
| Observer storage | `Option<Arc<...>>` | Always-present no-op observer | `Option::None` is provably zero-cost; a struct-based no-op observer still allocates and has vtable dispatch overhead |
| LogObserver integration | Keep `env_logger` | Remove `env_logger` | `env_logger` initializes the global log subscriber; removing it breaks existing LogLevel-based initialization in `run_with_callback`; keep it, LogObserver becomes a wrapper |

---

## Sources

- `tracing` crate: https://docs.rs/tracing (training data, HIGH confidence for 0.1 stability)
- `metrics` crate: https://docs.rs/metrics (training data, MEDIUM confidence — verify 0.24 exact version)
- Cargo feature flag docs: https://doc.rust-lang.org/cargo/reference/features.html
- Existing Cargo.toml: `/Users/luis/RustroverProjects/genetic-algorithms/Cargo.toml`
- Existing log call sites: grep across `src/` (direct inspection, HIGH confidence)
- MSRV for tracing: https://github.com/tokio-rs/tracing (training data)

**Note:** WebSearch, WebFetch, and Brave Search were unavailable during this research session.
The `tracing 0.1` version claim is HIGH confidence (stable for 4+ years, slow semver cadence).
The `metrics 0.24` version claim is MEDIUM confidence — verify with `cargo search metrics` before
committing to Cargo.toml.
