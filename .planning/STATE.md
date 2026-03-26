---
gsd_state_version: 1.0
milestone: v2.1
milestone_name: milestone
status: unknown
stopped_at: Phase 16 context gathered
last_updated: "2026-03-26T10:09:00.497Z"
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 6
  completed_plans: 6
---

# State

## Current Position

Phase: 15 (tracingobserver) — COMPLETE
Plan: 2 of 2 (DONE)

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-25)

**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Current focus:** Phase 15 — tracingobserver

## Accumulated Context

### Decisions

- v2.1.0 shipped: diversity metric, List genotype, Reporter trait, visualization feature flag, 6 examples
- Observer stored as `Option<Arc<dyn GaObserver<U> + Send + Sync>>` — Arc for island thread sharing, Option for zero-cost when absent (contrasts with Reporter's Box)
- `Reporter<U>` and `GaObserver<U>` coexist as separate fields; removing Reporter is a breaking change
- All hooks use `&self` (not `&mut self`) — required for rayon parallel regions
- Feature flags: `observer-tracing` and `observer-metrics` off by default; naming follows existing `serde` precedent
- [Phase 13]: GaObserver<U> uses &self and Send+Sync supertraits for Arc-based island sharing
- [Phase 13]: Reporter<U> and with_reporter() soft-deprecated since 2.2.0, removed in v3.0.0
- [Phase 13]: on_mutation_complete and on_fitness_evaluation_complete fire with Duration::ZERO since parent_crossover is opaque — timing separation requires future refactor
- [Phase 13]: Instant::now() gated behind observer.is_some() — zero overhead when no observer attached
- [Phase 14]: LogObserver reorders dynamic mutation block before on_generation_end so stats carry dynamic_mutation_probability
- [Phase 14]: log::warn! kept with EXT-02 comment — on_checkpoint_failed hook deferred, serde-gated, fires only on I/O errors
- [Phase 15-tracingobserver]: TracingObserver stores Mutex<Option<Span>> not EnteredSpan — EnteredSpan is !Send, breaking GaObserver: Send+Sync
- [Phase 15-tracingobserver]: Zero log::* calls in tracing_observer.rs — prevents LogTracer infinite recursion when LogObserver and TracingObserver both active (TRAC-03)
- [Phase 15-tracingobserver]: observer-tracing feature flag off by default — default builds do not pull in tracing crate (TRAC-02)
- [Phase 15-tracingobserver]: Integration tests use #![cfg(feature = "observer-tracing")] at file top (single gate) — entire file skipped in default cargo test (TRAC-02 verification)
- [Phase 15-tracingobserver]: LogTracer coexistence test uses tracing::subscriber::with_default (scoped) not set_global_default — avoids test suite subscriber state poisoning

### Blockers/Concerns

- Phase 14: Log migration must be atomic per module — never leave both direct log!() call and observer dispatch active simultaneously (~94 call sites across 9 targets)
- Phase 15: `tracing::Span::enter()` must never be called inside rayon closures; use `in_scope()` or `event!()` only
- Phase 15: `TracingObserver` must emit only via `tracing::event!()`, never `log::*` — prevents LogTracer infinite recursion
- Phase 16: Island `par_iter_mut()` requires same clone-once-before-parallel pattern as Phase 13
- Warn users: attaching `LogObserver` alongside an existing `SimpleReporter` produces duplicate per-generation log output

## Session Continuity

Last session: 2026-03-26T10:09:00.495Z
Stopped at: Phase 16 context gathered
Resume file: .planning/phases/16-sub-traits/16-CONTEXT.md
