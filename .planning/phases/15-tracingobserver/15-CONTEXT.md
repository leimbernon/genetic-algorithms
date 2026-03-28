# Phase 15: TracingObserver - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement `TracingObserver` behind the `observer-tracing` feature flag — a `GaObserver<U>` implementation that emits structured tracing spans and events per generation, enabling integration with OpenTelemetry, Jaeger, or any `tracing`-compatible subscriber. Scope is strictly `src/observer/tracing.rs` + feature flag wiring in `Cargo.toml`. Island and NSGA-II sub-traits are Phase 16.

</domain>

<decisions>
## Implementation Decisions

### Span architecture

- **Two-level span hierarchy**: root span `"ga_run"` (INFO) for the entire run + child span `"ga_generation"` (DEBUG) per generation
- `on_run_start` enters the `ga_run` span; `on_run_end` drops it
- `on_generation_start` enters the `ga_generation` span; `on_generation_end` exits it (and emits a final event with stats fields before dropping)
- All operator events and special events appear nested under the active `ga_generation` span
- Interior mutability: `Mutex<Option<tracing::span::EnteredSpan>>` for both `run_span` and `gen_span` fields

### Hook coverage and levels

All 12 hooks emit — users filter verbosity via their subscriber's filter layer (e.g., `EnvFilter`):

| Hook | Level | Notes |
|------|-------|-------|
| `on_run_start` | INFO | fires once |
| `on_run_end` | INFO | includes `cause` and `total_generations` |
| `on_generation_start` | DEBUG | span entry (no event emitted separately) |
| `on_generation_end` | DEBUG | event with full `GenerationStats` fields before dropping span |
| `on_new_best` | INFO | includes `generation` and `fitness` |
| `on_stagnation` | WARN | includes `generation` and `stagnation_count` |
| `on_extension_triggered` | INFO | includes all `ExtensionEvent` fields |
| `on_selection_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |
| `on_crossover_complete` | TRACE | includes `generation`, `duration_ms`, `offspring_count` |
| `on_mutation_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |
| `on_fitness_evaluation_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |
| `on_survivor_selection_complete` | TRACE | includes `generation`, `duration_ms`, `population_size` |

### Field naming

- **Duration**: `duration_ms = dur.as_secs_f64() * 1000.0` (f64) — human-readable, queryable in Jaeger/Grafana Tempo
- **GenerationStats fields**: match struct field names directly — `generation`, `best_fitness`, `mean_fitness`, `diversity` — no translation layer
- **Span names**: `"ga_run"` and `"ga_generation"` (ga_ prefix to namespace from user code, consistent with existing `target="ga_events"` convention)
- **ExtensionEvent fields**: `generation`, `diversity`, `extension_type`, `threshold` — match struct fields directly

### Struct design

- **Not a unit struct** (requires Mutex fields for span storage), but no configuration options:
  ```rust
  pub struct TracingObserver {
      run_span: Mutex<Option<tracing::span::EnteredSpan>>,
      gen_span: Mutex<Option<tracing::span::EnteredSpan>>,
  }

  impl TracingObserver {
      pub fn new() -> Self { ... }
  }

  impl Default for TracingObserver { ... }
  ```
- Users attach with: `ga.with_observer(Arc::new(TracingObserver::new()))`
- No builder pattern — all behavior is fixed; verbosity controlled via subscriber filter layers

### Feature flag

- Flag name: `observer-tracing` (consistent with planned `observer-metrics`)
- Adds `tracing` crate as an optional dependency gated on this flag
- Entire `src/observer/tracing.rs` module is `#[cfg(feature = "observer-tracing")]`
- Default builds are entirely unaffected (TRAC-02)

### Critical constraint: no log::* calls

`TracingObserver` must emit **exclusively** via `tracing::event!()` and `tracing::span!()` — never `log::*`. This prevents infinite recursion when `LogTracer` bridges `log` → `tracing` and the user has both active (TRAC-03).

### Claude's Discretion

- Whether to use `tracing::span!()` macro or `tracing::Span::new()` API for span creation
- Exact handling of Mutex poison (can `unwrap()` — Mutex poisoning means the observer itself panicked, not a recoverable state)
- Whether `TracingObserver` implements `Default` in addition to `new()`
- Test strategy for TRAC-03 (LogTracer integration test)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Tracing Observer (#184) — TRAC-01 through TRAC-03 (the three success criteria are the acceptance gate)

### Observer infrastructure (already in place)
- `src/observer/mod.rs` — `GaObserver<U>` trait with all 12 hooks; `TracingObserver` implements this trait; hook signatures define the available parameters
- `src/observer/log.rs` — `LogObserver` structural template: module layout, `impl<U: ChromosomeT> GaObserver<U>` pattern, re-export from `mod.rs`

### Types used in hook signatures
- `src/stats.rs` — `GenerationStats` struct fields (`generation`, `best_fitness`, `mean_fitness`, `diversity`)
- `src/ga.rs` `TerminationCause` enum — payload for `on_run_end`
- `src/observer/mod.rs` `ExtensionEvent` struct — payload for `on_extension_triggered` (`generation`, `diversity`, `extension_type`, `threshold`)

### Feature flag pattern
- `Cargo.toml` `[features]` section — existing `serde` and `visualization` optional dependency patterns; `observer-tracing` follows the same structure

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/observer/log.rs` — direct structural template: module file layout, `impl<U: ChromosomeT> GaObserver<U>` impl block, `pub use` re-export from `mod.rs`, crate-root re-export in `src/lib.rs`
- `src/observer/mod.rs` — `GaObserver<U>` trait already wired in `ga.rs` via `notify()` helper; `TracingObserver` receives all notifications automatically once attached
- `tracing` crate macros: `tracing::span!()`, `tracing::event!()`, `tracing::Span::enter()` — standard API, no custom wrappers needed

### Established Patterns
- Feature-gated modules: `src/checkpoint.rs` (`#[cfg(feature = "serde")]` pattern), `src/visualization/` (`#[cfg(feature = "visualization")]`) — same pattern applies to `src/observer/tracing.rs`
- Optional dependencies in `Cargo.toml`: `plotters = { version = "...", optional = true }` — `tracing` follows the same `optional = true` + `dep:tracing` syntax
- Observer module re-exports: `src/observer/mod.rs` already has `mod log; pub use log::LogObserver` — add `mod tracing; pub use tracing::TracingObserver` (behind cfg)
- Crate root re-exports: `src/lib.rs` exports `GaObserver`, `LogObserver` — add `TracingObserver` to the same block (behind cfg)

### Integration Points
- `Cargo.toml` — add `tracing = { version = "0.1", optional = true }` + `observer-tracing = ["dep:tracing"]` feature
- `src/observer/mod.rs` — add `#[cfg(feature = "observer-tracing")] mod tracing;` + `pub use` re-export
- `src/lib.rs` — add `TracingObserver` to observer re-exports under `#[cfg(feature = "observer-tracing")]`
- No changes to `src/ga.rs` — observer infrastructure already in place

</code_context>

<specifics>
## Specific Ideas

- The `ga_` span name prefix is consistent with the existing `target="ga_events"` naming convention already used by `LogObserver` — keeps all GA telemetry identifiable in shared backends
- Operator hooks at TRACE level means users get operator profiling data with `RUST_LOG=genetic_algorithms=trace` without requiring code changes
- The `duration_ms` field as `f64` enables direct numeric queries in Grafana Tempo / Jaeger (e.g., filter generations where `duration_ms > 100`)
- `Mutex::unwrap()` is appropriate for Mutex access in hooks — if the Mutex is poisoned, the observer itself has already panicked, and recovery is not meaningful

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 15-tracingobserver*
*Context gathered: 2026-03-26*
