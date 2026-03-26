---
gsd_state_version: 1.0
milestone: v2.1
milestone_name: milestone
status: unknown
stopped_at: Phase 15 context gathered
last_updated: "2026-03-26T09:19:29.651Z"
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 4
  completed_plans: 4
---

# State

## Current Position

Phase: 14 (logobserver-log-migration) — EXECUTING
Plan: 1 of 2

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-25)

**Core value:** Users can solve complex optimization problems with composable, performant genetic algorithms — without fighting the library
**Current focus:** Phase 14 — logobserver-log-migration

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

### Blockers/Concerns

- Phase 14: Log migration must be atomic per module — never leave both direct log!() call and observer dispatch active simultaneously (~94 call sites across 9 targets)
- Phase 15: `tracing::Span::enter()` must never be called inside rayon closures; use `in_scope()` or `event!()` only
- Phase 15: `TracingObserver` must emit only via `tracing::event!()`, never `log::*` — prevents LogTracer infinite recursion
- Phase 16: Island `par_iter_mut()` requires same clone-once-before-parallel pattern as Phase 13
- Warn users: attaching `LogObserver` alongside an existing `SimpleReporter` produces duplicate per-generation log output

## Session Continuity

Last session: 2026-03-26T09:19:29.643Z
Stopped at: Phase 15 context gathered
Resume file: .planning/phases/15-tracingobserver/15-CONTEXT.md
