# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- ✅ **v2.2 — Improve Usability (completion)** — Phases 6-9 (shipped 2026-03-21)
- ✅ **v2.1.0 — New Examples** — Phases 10-12 (shipped 2026-03-22)
- 🚧 **v2.2.0 — Observability & Traceability** — Phases 13-17 (in progress)

## Phases

<details>
<summary>✅ v2.1 — Improve Usability, partial (Phases 1-5) — SHIPPED 2026-03-20</summary>

Phases 1-5 predate GSD tracking. Issues closed: #165, #166, #167, #168, #169.

- [x] Extension strategies (MassExtinction, MassGenesis, MassDegeneration, MassDeduplication)
- [x] Dynamic mutation probability based on population cardinality
- [x] Clone crossover strategy
- [x] Rejuvenate crossover operator
- [x] LRU fitness cache

</details>

<details>
<summary>✅ v2.2 — Improve Usability, completion (Phases 6-9) — SHIPPED 2026-03-21</summary>

Issues closed: #170, #171, #178, #179.

- [x] **Phase 6: Diversity Estimation** — `GenerationStats.diversity` wired into extension trigger and dynamic mutation (completed 2026-03-20)
- [x] **Phase 7: List Genotype** — `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets (completed 2026-03-21)
- [x] **Phase 8: Reporter Trait** — `Reporter<U>` with 4 lifecycle hooks, `SimpleReporter`, `DurationReporter` (completed 2026-03-21)
- [x] **Phase 9: Visualization** — `visualization` feature flag, `plot_fitness`, `plot_diversity`, `plot_histogram` (completed 2026-03-21)

</details>

<details>
<summary>✅ v2.1.0 — New Examples (Phases 10-12) — SHIPPED 2026-03-22</summary>

- [x] **Phase 10: Single-population Examples** — `rastrigin`, `feature_selection`, `niching` (completed 2026-03-22)
- [x] **Phase 11: Advanced Mode Examples** — `nsga2_zdt1`, `island_model`, `job_scheduling` (completed 2026-03-22)
- [x] **Phase 12: Documentation** — README `## Examples` table with all 10 examples and `cargo run` commands (completed 2026-03-22)

Full archive: `.planning/milestones/v2.1.0-ROADMAP.md`

</details>

### 🚧 v2.2.0 — Observability & Traceability (In Progress)

**Milestone goal:** Implement a generic, telemetry-agnostic observability system — `GaObserver` trait, `LogObserver`, `TracingObserver`, Island/NSGA-II sub-traits, `CompositeObserver`, `MetricsObserver`.

Issues: #182, #183, #184, #185, #186

- [x] **Phase 13: GaObserver Base Trait** — Core trait + `Ga<U>` integration; foundation all other phases depend on (completed 2026-03-25)
- [x] **Phase 14: LogObserver + Log Migration** — Backward-compatible log migration; validates Phase 13 end-to-end (completed 2026-03-25)
- [x] **Phase 15: TracingObserver** — Structured tracing spans behind `observer-tracing` feature flag (completed 2026-03-26)
- [x] **Phase 16: Sub-Traits** — `IslandGaObserver` and `Nsga2Observer` for engine-specific events (completed 2026-03-27)
- [ ] **Phase 17: CompositeObserver + MetricsObserver** — Fan-out composition and metrics facade behind `observer-metrics` flag

## Phase Details

### Phase 13: GaObserver Base Trait
**Goal**: Users can attach a structured observer to `Ga<U>` and receive lifecycle notifications with zero overhead when no observer is attached
**Depends on**: Nothing (first phase of this milestone)
**Requirements**: OBS-01, OBS-02, OBS-03, OBS-04
**Success Criteria** (what must be TRUE):
  1. User can call `ga.with_observer(arc_observer)` and have `on_run_start`, `on_generation_end`, `on_new_best`, `on_run_end`, `on_stagnation`, and `on_extension_triggered` fire at the correct points in the GA loop
  2. A custom observer that implements only one hook compiles without error — all other hooks have default no-op bodies
  3. Running `Ga<U>` with no observer attached produces identical output and timing to pre-v2.2.0 (zero-overhead branch confirmed by benchmarks)
  4. A custom observer type that is not `Send + Sync` is rejected at compile time when passed to `with_observer()`
**Plans:** 2/2 plans complete

Plans:
- [ ] 13-01-PLAN.md — GaObserver trait definition, ExtensionEvent, NoopObserver, Extension::as_str(), Reporter deprecation
- [ ] 13-02-PLAN.md — Ga<U> integration (observer field, builder, notify helper, 12 call sites) + integration tests

### Phase 14: LogObserver + Log Migration
**Goal**: Users can reproduce all pre-v2.2.0 log output by attaching `LogObserver`, and no hardcoded `log!()` calls remain in the GA execution paths
**Depends on**: Phase 13
**Requirements**: LOG-01, LOG-02, LOG-03
**Success Criteria** (what must be TRUE):
  1. User can attach `LogObserver` to `Ga<U>` and observe log output at the same targets, levels, and message formats as produced by v2.1.0
  2. A `grep` for `info!\|debug!\|trace!\|warn!` in `src/ga.rs`, `src/island/`, and `src/nsga2/` returns results only inside `log_observer.rs` itself — no call sites remain in the execution loops
  3. `cargo build` (default features) and `cargo build --features serde` both succeed with zero new dependencies added
**Plans:** 2/2 plans complete

Plans:
- [ ] 14-01-PLAN.md — LogObserver struct, ExtensionEvent/GenerationStats extensions, module registration, tests
- [ ] 14-02-PLAN.md — Remove all 17 log!() calls from ga.rs, grep regression test

### Phase 15: TracingObserver
**Goal**: Users can attach `TracingObserver` to emit structured tracing spans and events per generation, enabling integration with OpenTelemetry, Jaeger, or any `tracing`-compatible subscriber
**Depends on**: Phase 14
**Requirements**: TRAC-01, TRAC-02, TRAC-03
**Success Criteria** (what must be TRUE):
  1. User can add `features = ["observer-tracing"]` to their `Cargo.toml`, attach `TracingObserver`, and observe `tracing::event!()` emissions per generation in their subscriber
  2. `cargo build` (default features, no `observer-tracing`) succeeds without pulling in the `tracing` crate
  3. A CI test running 10 generations with `LogTracer::init()` and `TracingObserver` both active completes without stack overflow or infinite recursion
**Plans:** 2/2 plans complete

Plans:
- [ ] 15-01-PLAN.md — Feature flag wiring, TracingObserver implementation (all 12 hooks), module re-exports
- [ ] 15-02-PLAN.md — Integration tests (TRAC-01 attach/run/Send+Sync, TRAC-02 feature gate, TRAC-03 LogTracer coexistence)

### Phase 16: Sub-Traits
**Goal**: Users can attach engine-specific observers to `IslandGa<U>` and `Nsga2Ga<U>` and receive events unique to each engine's execution model
**Depends on**: Phase 13
**Requirements**: SUB-01, SUB-02, SUB-03
**Success Criteria** (what must be TRUE):
  1. User can call `island_ga.with_observer(arc_observer)` with an `IslandGaObserver` implementation and receive `on_migration_triggered`, `on_island_run_start`, `on_island_run_end`, and `on_island_generation_end` events
  2. User can call `nsga2_ga.with_observer(arc_observer)` with a `Nsga2Observer` implementation and receive `on_pareto_front_assigned`, `on_non_dominated_sort_complete`, and `on_crowding_distance_calculated` events
  3. A single `LogObserver` instance implements all three observer traits (`GaObserver`, `IslandGaObserver`, `Nsga2Observer`) and can be passed to any of the three GA engines
**Plans:** 3 plans complete

Plans:
- [ ] 16-01-PLAN.md — IslandGaObserver + Nsga2Observer trait definitions, LogObserver multi-trait impl, module re-exports
- [ ] 16-02-PLAN.md — IslandGa<U> integration (observer field, hooks, migration dispatch)
- [ ] 16-03-PLAN.md — Nsga2Ga<U> integration + integration tests for all three sub-traits

### Phase 17: CompositeObserver + MetricsObserver
**Goal**: Users can combine multiple observers in a single run and optionally record per-generation metrics counters, gauges, and histograms via the `metrics` facade
**Depends on**: Phases 13, 14, 15, 16
**Requirements**: COMP-01, COMP-02, COMP-03
**Success Criteria** (what must be TRUE):
  1. User can build a `CompositeObserver` with two or more observers and all three trait interfaces (`GaObserver`, `IslandGaObserver`, `Nsga2Observer`) fan out to every attached observer
  2. User can add `features = ["observer-metrics"]` and attach `MetricsObserver`; per-generation counters and gauges are recorded via the `metrics` facade without installing any backend in the library
  3. `cargo build` (default features, no `observer-metrics`) succeeds without pulling in the `metrics` crate
  4. A criterion benchmark shows `MetricsObserver` used inside island parallel execution produces no data races or panics (metric calls are sequential-only)
**Plans:** 1/3 plans executed

Plans:
- [ ] 17-01-PLAN.md — AllObserver<U> supertrait + CompositeObserver<U> with fan-out for all 19 hooks
- [ ] 17-02-PLAN.md — MetricsObserver behind observer-metrics feature flag (11 metric calls)
- [ ] 17-03-PLAN.md — Integration tests (COMP-01/02/03) + criterion benchmark

## Progress

**Execution Order:**
Phases execute in numeric order: 13 → 14 → 15 → 16 → 17

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 6. Diversity Estimation | v2.2 | 2/2 | Complete | 2026-03-20 |
| 7. List Genotype | v2.2 | 2/2 | Complete | 2026-03-21 |
| 8. Reporter Trait | v2.2 | 2/2 | Complete | 2026-03-21 |
| 9. Visualization | v2.2 | 2/2 | Complete | 2026-03-21 |
| 10. Single-population Examples | v2.1.0 | 3/3 | Complete | 2026-03-22 |
| 11. Advanced Mode Examples | v2.1.0 | 3/3 | Complete | 2026-03-22 |
| 12. Documentation | v2.1.0 | 1/1 | Complete | 2026-03-22 |
| 13. GaObserver Base Trait | 2/2 | Complete    | 2026-03-25 | - |
| 14. LogObserver + Log Migration | 2/2 | Complete    | 2026-03-25 | - |
| 15. TracingObserver | 2/2 | Complete    | 2026-03-26 | - |
| 16. Sub-Traits | 3/3 | Complete    | 2026-03-27 | - |
| 17. CompositeObserver + MetricsObserver | 1/3 | In Progress|  | - |
