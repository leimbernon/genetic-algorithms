# Project Research Summary

**Project:** genetic_algorithms — v2.2.0 Observability & Traceability
**Domain:** Adding a structured observer system to a concurrent Rust GA library
**Researched:** 2026-03-25
**Confidence:** HIGH

## Executive Summary

This milestone adds a `GaObserver<U>` trait system to the `genetic_algorithms` crate, enabling users to attach structured observability hooks to all three GA execution engines (`Ga<U>`, `IslandGa<U>`, `Nsga2Ga<U>`). Research confirms this is a well-understood problem domain with a clear in-codebase precedent: the `Reporter<U>` trait and `src/reporter/` module shipped in v2.1.0. The recommended approach is a layered trait hierarchy — base trait plus engine-specific sub-traits — stored as `Option<Arc<dyn GaObserver<U> + Send + Sync>>` to satisfy rayon's cross-thread sharing requirements. Two new optional crates (`tracing 0.1.44`, `metrics 0.24.3`) are added behind feature flags; the default build adds zero new dependencies. Both crates are verified against live crates.io and are compatible with the project MSRV 1.81.0.

The key design constraint is thread safety. The island model uses `par_iter_mut()`, which means the observer reference can be accessed from multiple rayon worker threads simultaneously. Every design decision flows from this: `&self` (not `&mut self`) on all hook methods, `Arc` (not `Box`) for storage, and `Send + Sync` as supertraits on `GaObserver<U>`. The existing `Reporter<U>` trait (v2.1.0 public API) coexists unchanged — it and `GaObserver<U>` are separate fields on `Ga<U>` with separate responsibilities. No breaking changes are introduced.

The top risk is correctness during the log migration in Phase 2: approximately 94 `log!()` call sites across 9 targets must be replaced atomically per module to avoid duplicate output. The second major risk is threading errors in Phase 3 (`TracingObserver`): `tracing`'s `Span::enter()` must never be called inside rayon parallel closures because the guard must be dropped on the same thread it was created on — rayon work-stealing violates this invariant. Both risks are well-understood and have clear prevention strategies.

## Key Findings

### Recommended Stack

The milestone adds exactly two optional crates, both gated behind named feature flags. No existing dependencies change and the default build is unaffected. The facade pattern — library emits, users route — is the deliberate architectural choice: `tracing` and `metrics` are facades analogous to the existing `log` crate, not backend libraries.

**Core technologies:**
- `tracing 0.1.44` (optional, `observer-tracing` flag) — structured spans and events for `TracingObserver`; zero-cost when no subscriber is installed; MSRV 1.65.0, compatible with project MSRV 1.81.0; verified via `cargo info`
- `metrics 0.24.3` (optional, `observer-metrics` flag) — counter/gauge/histogram facade for `MetricsObserver`; no-ops when no recorder is installed; MSRV 1.71.1, compatible with project MSRV 1.81.0; verified via `cargo info`
- `log 0.4.22` (existing, unchanged) — `LogObserver` wraps this; no new dependency needed for the most critical concrete observer

**Feature flag naming:** `observer-tracing` and `observer-metrics` (not bare crate names) to avoid shadowing user-side dependencies and to communicate intent. Feature names mirror the existing `serde` precedent — they name what they unlock, not just the dep.

**Storage pattern:** `Option<Arc<dyn GaObserver<U> + Send + Sync>>` — `Arc` for island thread sharing, `Option::None` for provably zero-cost when no observer is attached. This contrasts with `Reporter<U>` which uses `Option<Box<dyn Reporter<U> + Send>>` — valid for single-threaded `Ga<U>` but incompatible with island parallelism.

### Expected Features

Research distinguishes clearly between what is needed for v2.2.0 launch and what should follow.

**Must have (table stakes) — v2.2.0:**
- `GaObserver<U>` trait with complete hook surface (9 base hooks) and default no-op bodies — everything else is blocked on this; hooks use `&self` throughout
- `LogObserver` — backward-compatible migration of all 94 `log!()` call sites across 9 targets; uses existing `log` dep, no new dependencies
- `with_observer()` builder on `Ga<U>`, `IslandGa<U>`, and `Nsga2Ga<U>` — consistency across all three GA modes
- `CompositeObserver` — pure Rust fan-out over `Vec<Arc<dyn GaObserver<U>>>`, no new dependencies
- `IslandGaObserver` sub-trait with `on_migration`, `on_island_generation_end`, `on_island_run_start/end` hooks — island model has unique events the base trait cannot expose
- `Nsga2Observer` sub-trait with `on_pareto_front_assigned`, `on_non_dominated_sort_complete`, `on_crowding_distance_calculated` hooks — NSGA-II has no scalar fitness; Pareto front signals are the meaningful hooks
- `TracingObserver` behind `observer-tracing` feature flag — highest-value differentiator; structured span integration with OpenTelemetry, Jaeger, Honeycomb

**Should have (v2.2.x, after validation):**
- `MetricsObserver` behind `observer-metrics` feature flag — similar complexity to `TracingObserver`; add once tracing pattern is proven
- `on_extension_triggered` hook — low complexity; add when hook surface is stable

**Defer (v2.3+):**
- Per-operator timing hooks with `Duration` parameters — useful for benchmarking operator configurations; deferred because threading `Arc<dyn GaObserver>` through every operator factory invocation requires significant refactoring
- `on_checkpoint_saved` hook — low priority; checkpoint already works and the event is not critical for observability

**Anti-features to avoid:**
- Async observer methods — `rayon` is sync; `async` traits add `Pin<Box<Future>>` overhead with no benefit since GA runs are blocking
- Bundled metrics backends or tracing subscribers — violates the facade principle; backend choice belongs to the user application
- Per-gene or per-chromosome observer hooks — called millions of times per run; unacceptable overhead regardless of no-op cost
- `Box<dyn GaObserver>` instead of `Arc` — incompatible with island rayon parallelism
- Replacing `Reporter<U>` — breaking change; both systems must coexist

### Architecture Approach

The architecture mirrors the existing `src/reporter/` module from v2.1.0. A new `src/observer/` module contains the base trait, sub-traits, and all concrete observers. The three GA orchestrators are modified to add one `observer` field and one `with_observer()` builder each. Notification happens via a `notify()` helper that checks `Option::None` before vtable dispatch — generation-level granularity only, never per-gene.

**Major components:**
1. `src/observer/mod.rs` — `GaObserver<U>` base trait (lifecycle, operator, and special event hooks), `ExtensionEvent` typed payload struct, `NoopObserver`, re-exports; always compiled
2. `src/observer/island.rs` and `src/observer/nsga2.rs` — sub-traits extending `GaObserver<U>` with engine-specific hooks; single `LogObserver` can implement all three
3. `src/observer/log_observer.rs` — `LogObserver` implementing all three traits; maps each hook to the matching `log!()` target+level; always compiled
4. `src/observer/composite.rs` — `CompositeObserver<U>` with builder pattern; implements all three observer traits by iteration
5. `src/observer/tracing_observer.rs` and `src/observer/metrics_observer.rs` — concrete observers behind feature flags; whole files gated with `#[cfg(feature = "...")]`
6. Modified `src/ga.rs`, `src/island/mod.rs`, `src/nsga2/mod.rs` — each gains `observer` field, `with_observer()`, `notify()`, and `Instant` measurements at operator phase boundaries

**Key patterns:**
- `Arc<dyn T + Send + Sync>` for observer storage (not `Box`) to enable island rayon sharing
- Default no-op method bodies on all traits for forward-compatibility — new hooks added in later versions do not break existing observer implementations
- Typed `ExtensionEvent` struct (stack-allocated) for multi-field payloads when an event carries more than ~3 parameters
- Sub-trait extension (`IslandObserver<U>: GaObserver<U>`) allows a single `LogObserver` to implement all three traits; trait upcasting available at MSRV 1.81.0 (stabilized in 1.76)

### Critical Pitfalls

1. **Arc clone inside `par_iter()` closures** — Clone `Arc<dyn GaObserver<U>>` exactly once before entering the parallel region; pass `Option<&dyn GaObserver<U>>` (a shared borrow) into the closure. Observer methods must be `&self`. This must be decided in Phase 1 before any hook call sites are written — all downstream implementations inherit this constraint.

2. **`tracing` `Span::enter()` guard in rayon closures** — `Entered<'_>` must be dropped on the same thread it was created on; rayon work-stealing violates this. Use `tracing::Span::in_scope(|| { ... })` inside parallel regions; emit `event!()` only, never enter spans inside rayon closures. The bug only manifests under actual task-stealing and not in single-threaded tests.

3. **`tracing-log` bridge infinite recursion** — If users enable `LogTracer` (routes `log::*` into tracing) and `TracingObserver` internally calls `log::*`, an infinite loop results in a stack overflow. `TracingObserver` must emit only via `tracing::event!()`, never via `log::*`. Document this in the `TracingObserver` rustdoc. Add a CI test with `LogTracer::init()` + `TracingObserver` for 10 generations.

4. **Duplicate log output during `LogObserver` migration** — The 94 existing `log!()` call sites must be removed in the same commit they are replaced by observer hooks. Migration must be atomic per module — never leave both the direct `log!()` call and the observer dispatch active simultaneously.

5. **Missing `Send + Sync` supertrait discovered post-release** — Adding `Send + Sync` as a supertrait on `GaObserver<U>` after the first release is a breaking change for any user whose custom observer type is not `Send + Sync`. Lock this in during Phase 1 trait design; verify by compiling `CompositeObserver` assigned to an `IslandGa<U>` field.

## Implications for Roadmap

Research strongly supports a 5-phase build order driven by trait dependencies. The base trait must exist before any concrete observer; `LogObserver` must exist before hardcoded log calls are removed; sub-traits must be stable before `CompositeObserver` can implement all three. The phases below correspond directly to GitHub issues #182–#186.

### Phase 1: GaObserver Base Trait (Issue #182)

**Rationale:** Everything else is blocked on the `GaObserver<U>` trait definition. This phase locks in the `&self` hook signatures, `Send + Sync` supertraits, and `Arc` storage pattern — decisions that cannot be changed after Phase 2 without breaking changes. The full hook surface should be designed up front; adding hooks later is safe (default no-ops), but removing or renaming them is a breaking change.
**Delivers:** `GaObserver<U>` trait with all 9 lifecycle hooks and default no-ops, `ExtensionEvent` struct, `NoopObserver`, `src/observer/mod.rs` module, `observer` field + `with_observer()` + `notify()` on `Ga<U>`, `Instant` measurements at 6 operator phase boundaries in `ga.rs`
**Addresses:** Foundation for all table-stakes features; locks in concurrency model before any concrete observer exists
**Avoids:** Pitfall 1 (Arc contention — access pattern decided here), Pitfall 5 (missing Send+Sync supertrait), CompositeObserver compile failure in Phase 5

### Phase 2: LogObserver and Log Migration (Issue #183)

**Rationale:** `LogObserver` uses only the existing `log` dep — no new dependencies, lowest risk. This phase validates that the Phase 1 notification points work end-to-end with a real implementation. The log migration (removing ~94 hardcoded `log!()` calls) must happen atomically per module before any other concrete observer is built.
**Delivers:** `LogObserver` implementing `GaObserver<U>`, automatic `LogObserver` default when `log_level != Off`, all hardcoded `log!()` calls removed from `ga.rs`, backward-compatible log output preserved for all 9 targets
**Uses:** Existing `log 0.4.22` dep — no new dependencies
**Avoids:** Pitfall 4 (LogObserver breaking env_logger init — it must be a bridge, not a logger owner), Pitfall 5 (duplicate output during migration — atomic per-module removal), Pitfall 12 (missed serde-gated `log::warn!` in checkpoint code — requires `cargo build --features serde` in CI)

### Phase 3: TracingObserver (Issue #184)

**Rationale:** Feature-gated behind `observer-tracing`, so zero impact on default builds. Highest-value differentiator — structured span integration with OpenTelemetry, Jaeger, and the broader tracing ecosystem. Must be built after Phase 2 ensures all hardcoded `log!()` calls are removed, eliminating the `tracing-log` bridge recursion risk.
**Delivers:** `TracingObserver` behind `observer-tracing` feature flag, `Cargo.toml` updated, usage example, CI matrix entry for `--features observer-tracing`
**Uses:** `tracing 0.1.44` (new optional dep, MSRV 1.65.0)
**Avoids:** Pitfall 2 (`Span::enter()` in rayon closures — use `in_scope()` or `event!()` only), Pitfall 6 (`tracing-log` infinite recursion — `TracingObserver` emits only via `tracing::event!()`), Pitfall 10 (feature flag re-export without cfg guard — caught by CI `--no-default-features` job)

### Phase 4: IslandGaObserver and Nsga2Observer Sub-Traits (Issue #185)

**Rationale:** Sub-traits depend on the base trait (Phase 1). `IslandGa` and `Nsga2Ga` integrations can be built in parallel within this phase. `LogObserver` and `TracingObserver` must be extended to implement the new sub-traits. MSRV 1.81.0 guarantees trait upcasting is available (stabilized in 1.76).
**Delivers:** `IslandObserver<U>` and `Nsga2Observer<U>` sub-traits, `observer` field + `with_observer()` + `notify()` on `IslandGa<U>` and `Nsga2Ga<U>`, extended `LogObserver` and `TracingObserver` implementations covering all migration log targets in `island/mod.rs` and `nsga2/mod.rs`
**Addresses:** `IslandGaObserver` and `Nsga2Observer` (P1 table stakes), consistency across all three GA modes
**Avoids:** Pitfall 1 repeated in island's `par_iter_mut()` path (same clone-once-before-parallel pattern), Pitfall 9 (island sub-trait Arc contention)

### Phase 5: CompositeObserver and MetricsObserver (Issue #186)

**Rationale:** `CompositeObserver` requires all three observer traits to exist (Phases 1 and 4). `MetricsObserver` is the highest-risk feature flag (new dependency, potential contention risk in parallel paths) and should come last, after the tracing pattern from Phase 3 is proven.
**Delivers:** `CompositeObserver<U>` with builder pattern implementing all three observer traits, `MetricsObserver` behind `observer-metrics` feature flag, `Cargo.toml` updated, usage example, CI matrix entry for `--features observer-metrics`
**Uses:** `metrics 0.24.3` (new optional dep, MSRV 1.71.1)
**Avoids:** Pitfall 3 (metrics counter contention — only record in sequential `on_generation_end` hooks, never inside `par_iter()`)

### Phase Ordering Rationale

- Trait before implementations: `GaObserver<U>` (Phase 1) must exist before `LogObserver` (Phase 2), `TracingObserver` (Phase 3), or `CompositeObserver` (Phase 5) can implement it.
- Validation before risk: Phase 2 (`LogObserver`, no new deps) validates the full notification architecture before Phase 3 introduces an optional dependency.
- Log migration before tracing: All hardcoded `log!()` calls must be removed in Phase 2, before `TracingObserver` is built, to eliminate the `tracing-log` bridge recursion risk from having both systems active simultaneously.
- Base before sub-traits: Phases 1-2 establish `GaObserver<U>` before Phases 3-4 add sub-traits and feature-gated observers that extend it.
- Composition last: `CompositeObserver` (Phase 5) logically comes last because it composes all other observers; building it before they exist produces incomplete tests.

### Research Flags

Phases requiring careful implementation review:
- **Phase 2:** Log migration audit is the most error-prone step. Pre-audit all 94 call sites before starting. After completion, verify with `grep -rn 'info!\|debug!\|trace!\|warn!' src/` — only `LogObserver` itself and unmigrated operator files should remain. Also requires explicit `cargo build --features serde` to catch the serde-gated `log::warn!` in checkpoint code.
- **Phase 3:** Threading correctness for `TracingObserver` requires deliberate per-hook review. Every hook that fires inside a rayon parallel region must use `tracing::event!()` only — never `span.enter()`. Add a CI test: `LogTracer::init()` + `TracingObserver` running for 10 generations without stack overflow.
- **Phase 4:** Island `par_iter_mut()` is the same Arc contention risk as Phase 1 — apply clone-once-before-parallel. Verify with a criterion benchmark showing observer=None vs observer=no-op with <1% overhead difference.

Phases with standard patterns (safe to implement without deeper research):
- **Phase 1:** Rust trait design with default methods and `Send + Sync` supertraits is well-documented. The `src/reporter/` module from v2.1.0 is the direct structural precedent.
- **Phase 5:** `CompositeObserver` is a straightforward fan-out. `MetricsObserver` follows the same hook-per-generation pattern as `TracingObserver`. Ensure metrics facade calls are restricted to sequential per-generation hooks only.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Both new crates verified via `cargo info` against live crates.io (2026-03-25); MSRV compatibility confirmed; feature flag naming follows existing `serde` precedent |
| Features | HIGH | Based on direct codebase inspection of `src/ga.rs`, `src/island/mod.rs`, `src/nsga2/mod.rs`, `src/reporter/`, `src/stats.rs`; all hook triggers verified against live source; 94 call sites counted; anti-features grounded in actual concurrency constraints |
| Architecture | HIGH | Build order derived from direct source reading plus GitHub issues #182–#186; trait upcasting availability confirmed against MSRV 1.81.0 (stabilized 1.76); `src/reporter/` module is a verified structural precedent |
| Pitfalls | HIGH | Threading pitfalls verified against rayon, tracing, and metrics official documentation; log migration pitfall grounded in actual call-site count; one MEDIUM item: `metrics` internal `RwLock/DashMap` contention assertion based on source inspection, not yet benchmarked |

**Overall confidence:** HIGH

### Gaps to Address

- **`MetricsObserver` contention claim needs benchmarking:** The assertion that `metrics::counter!` inside `par_iter()` causes measurable contention is based on `metrics-rs` source inspection, not an actual benchmark. Validate with a criterion bench before including the claim in public documentation (Phase 5).
- **`Reporter<U>` + `GaObserver<U>` coexistence warning:** A user attaching `LogObserver` (as `GaObserver`) alongside an existing `SimpleReporter` (as `Reporter`) will see duplicate per-generation log output. The `with_reporter()` and `with_observer()` API docs must warn about this. Draft the warning during Phase 1 or Phase 2.
- **Operator-level hook threading (future):** Per-operator hooks (`on_selection_complete`, `on_crossover_complete`) fire from the sequential driver loop — safe now. If operator parallelism is introduced in a future milestone, hook placement would need to be reviewed to stay outside those parallel regions.
- **Hook naming consistency:** ARCHITECTURE.md and FEATURES.md use slightly different hook names in places (`on_generation_end` vs `on_generation_complete`, `on_run_end` vs `on_run_complete`). Standardize naming in Phase 1 before any public API is committed.

## Sources

### Primary (HIGH confidence)
- `src/ga.rs` (direct inspection, lines 554–1115) — run loop structure, all notification points, `Reporter<U>` call sites, `TerminationCause` definition
- `src/island/mod.rs` (direct inspection) — migration events, per-island log targets, `par_iter_mut()` usage
- `src/nsga2/mod.rs` (direct inspection) — non-dominated sort and crowding loop, log targets
- `src/reporter/mod.rs` (direct inspection) — `Reporter<U>` trait as architectural precedent; `Option<Box<dyn Reporter<U> + Send>>` storage pattern
- `src/stats.rs` (direct inspection) — `GenerationStats` fields available to all hooks
- `Cargo.toml` (direct inspection) — existing feature flag pattern, MSRV 1.81.0, existing dependencies
- `.planning/PROJECT.md` (direct read) — zero-overhead constraint, backward-compat mandate, feature flag names, out-of-scope list
- GitHub issues #182–#186 (direct read) — issue scope, method signatures, metric names
- `cargo info tracing` (live crates.io, 2026-03-25) — version 0.1.44, MSRV 1.65.0
- `cargo info metrics` (live crates.io, 2026-03-25) — version 0.24.3, MSRV 1.71.1

### Secondary (HIGH confidence, external documentation)
- `tracing` crate v0.1.44 docs: https://docs.rs/tracing/latest/tracing/ — span threading contract, `Span::enter()` same-thread requirement, `in_scope()` as rayon-safe alternative
- `tracing-log` crate docs: https://docs.rs/tracing-log/latest/tracing_log/ — `LogTracer` infinite recursion warning (documented by the crate itself)
- `metrics` crate v0.24.3 docs: https://docs.rs/metrics/latest/metrics/ — facade pattern, recorder installation model
- Rayon `ParallelIterator` docs: https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html — `Send + Sync` closure requirements, work-stealing model
- Rust SemVer guidelines: https://doc.rust-lang.org/cargo/reference/semver.html — adding default trait methods is a minor (not breaking) change

### Tertiary (MEDIUM confidence)
- `metrics-rs` source (source inspection, not benchmarked) — internal `RwLock/DashMap` registry structure; contention under parallel write workload is inferred, not measured; validate with criterion bench in Phase 5

---
*Research completed: 2026-03-25*
*Ready for roadmap: yes*
