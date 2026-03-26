# Requirements: genetic_algorithms

**Defined:** 2026-03-25
**Milestone:** v2.2.0 — Observability & Traceability
**Core Value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library.

## v2.2.0 Requirements

### Observer Trait (GaObserver base — #182)

- [x] **OBS-01**: User can attach a `GaObserver<U>` to `Ga<U>` via `with_observer()` and receive notifications for run start/end, each generation, new best, and special events (stagnation, extension triggered)
- [x] **OBS-02**: `GaObserver<U>` has default no-op implementations for all hooks — users implement only the hooks they care about
- [x] **OBS-03**: No overhead when no observer is attached (`Option::None` branch eliminates all vtable dispatch and measurement)
- [x] **OBS-04**: `GaObserver<U>` is safely shareable across rayon threads (`Arc<dyn GaObserver<U> + Send + Sync>`)

### Log Observer (#183)

- [x] **LOG-01**: User can attach `LogObserver` to reproduce identical log output to pre-v2.2.0 behavior (fully backward-compatible migration)
- [x] **LOG-02**: All hardcoded `log!()` call sites in `ga.rs`, `island/`, and `nsga2/` are replaced by observer notifications — duplicate output is structurally impossible
- [x] **LOG-03**: `LogObserver` compiles and works with zero new dependencies (uses existing `log 0.4` crate)

### Tracing Observer (#184)

- [x] **TRAC-01**: User can attach `TracingObserver` (behind `observer-tracing` feature flag) to emit structured tracing spans and events per generation
- [x] **TRAC-02**: `TracingObserver` compiles only when `--features observer-tracing` is enabled; default builds are entirely unaffected
- [x] **TRAC-03**: `TracingObserver` is safe to use alongside `LogTracer` — emits exclusively via `tracing::event!()`, no infinite recursion possible

### Sub-Traits (#185)

- [ ] **SUB-01**: User can attach an `IslandGaObserver` to `IslandGa<U>` via `with_observer()` and receive island-specific events (migration triggered, per-island run start/end, per-island generation complete)
- [ ] **SUB-02**: User can attach a `Nsga2Observer` to `Nsga2Ga<U>` via `with_observer()` and receive NSGA-II-specific events (Pareto front assigned, non-dominated sort complete, crowding distance calculated)
- [ ] **SUB-03**: `LogObserver` implements all three observer traits (`GaObserver`, `IslandGaObserver`, `Nsga2Observer`) providing complete log migration coverage across all GA modes

### Composite + Metrics (#186)

- [ ] **COMP-01**: User can combine multiple observers simultaneously via `CompositeObserver`, with all three observer traits fanning out to every attached observer
- [ ] **COMP-02**: User can attach `MetricsObserver` (behind `observer-metrics` feature flag) to record per-generation counters, gauges, and histograms via the `metrics` facade crate
- [ ] **COMP-03**: `MetricsObserver` is safe inside island parallel execution — metric calls are restricted to sequential per-generation hooks, never inside `par_iter()` closures

## Future Requirements (v2.3+)

### Observer Extensions

- **EXT-01**: Per-operator timing hooks with `Duration` parameters — deferred; requires threading observer reference through operator factory invocations (significant refactor)
- **EXT-02**: `on_checkpoint_saved` hook — low priority; checkpoint already works independently

## Out of Scope

| Feature | Reason |
|---------|--------|
| Async observer methods | `rayon` is sync; `async fn` in traits adds `Pin<Box<Future>>` overhead with no benefit for CPU-bound GA workloads |
| Bundled telemetry backends (Prometheus, Jaeger, OTLP exporters) | Facade pattern — library emits, users route. Backend choice belongs to the user's application. |
| Per-gene or per-chromosome observer hooks | Called millions of times per run; unacceptable hot-path overhead regardless of no-op optimization |
| Removing `Reporter<U>` | Public API from v2.1.0; removing is a breaking change in a published crate. Both systems coexist. |
| GUI/interactive dashboards | Library generates structured events; visualization is the user's concern |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| OBS-01 | Phase 13 | Complete |
| OBS-02 | Phase 13 | Complete |
| OBS-03 | Phase 13 | Complete |
| OBS-04 | Phase 13 | Complete |
| LOG-01 | Phase 14 | Complete |
| LOG-02 | Phase 14 | Complete |
| LOG-03 | Phase 14 | Complete |
| TRAC-01 | Phase 15 | Complete |
| TRAC-02 | Phase 15 | Complete |
| TRAC-03 | Phase 15 | Complete |
| SUB-01 | Phase 16 | Pending |
| SUB-02 | Phase 16 | Pending |
| SUB-03 | Phase 16 | Pending |
| COMP-01 | Phase 17 | Pending |
| COMP-02 | Phase 17 | Pending |
| COMP-03 | Phase 17 | Pending |

**Coverage:**
- v2.2.0 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-25*
*Last updated: 2026-03-25 — traceability filled after roadmap creation*
