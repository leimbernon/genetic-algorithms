# Project Research Summary

**Project:** genetic_algorithms — v2.1.0 Observability & Traceability
**Domain:** Observability layer for a high-performance Rust genetic algorithms library
**Researched:** 2026-03-23
**Confidence:** HIGH

## Executive Summary

This milestone adds a structured observability system to an existing, production-quality Rust genetic algorithms library. The library already has hardcoded `log!()` calls scattered across ~88 call sites in 8 targets; the task is to replace these with a pluggable `GaObserver<U>` trait while preserving identical backward-compatible log output via a `LogObserver`. Two optional concrete observers (`TracingObserver`, `MetricsObserver`) are added behind feature flags, following the same facade pattern that `tracing` and `metrics` crates use — the library emits, users route. No breaking API changes are required; the existing `run_with_callback` mechanism is preserved alongside the new observer system.

The recommended approach is a layered trait hierarchy rooted at `GaObserver<U: ChromosomeT + Send + Sync>` with `IslandGaObserver` and `Nsga2Observer` sub-traits for engine-specific events. The observer is stored as `Option<Arc<dyn GaObserver<U>>>` on each orchestrator (`Ga`, `IslandGa`, `Nsga2Ga`), which yields zero overhead when `None`. `CompositeObserver` enables users to stack multiple observers with a single `with_observer()` call. The feature flag pattern mirrors the existing `serde` feature: `observer-tracing` pulls in the `tracing` crate; `observer-metrics` pulls in the `metrics` crate. Neither is a default dependency.

The primary risks are all concurrency-related: observer hooks must never be called from inside rayon `par_iter` closures because that causes `Arc` atomic contention (Pitfall 1), tracing `Span::enter()` guard violations (Pitfall 2), and metrics registry contention (Pitfall 3). These are avoidable by design — all observer hooks must be placed in the sequential outer generation loop, not in the inner parallel fitness/crossover regions. A secondary risk is that `LogObserver` must not own logger initialization; that responsibility stays with `ga.rs`, preserving the `LogLevel` backward-compatibility contract.

## Key Findings

### Recommended Stack

The existing stack (`rand`, `rayon`, `log`, `env_logger`, `serde`) is unchanged. Two optional crates are added, both behind feature flags. `tracing 0.1` is the de facto structured tracing facade for the Rust ecosystem — it is stable since 2021, `Send + Sync` safe, and zero-cost when no subscriber is installed. `metrics 0.24` plays the same role for metrics: a pure facade with no-op behavior when no recorder is installed. Neither crate bundles a backend; backend choice belongs to the library's users, not to the library itself.

**Core technologies:**
- `tracing 0.1` (optional): structured spans and events — zero-cost facade, integrates with any OTel/Jaeger/fmt backend via subscriber
- `metrics 0.24` (optional): counter/gauge/histogram recording — zero-cost facade, integrates with Prometheus/StatsD via recorder
- `Arc<dyn GaObserver<U> + Send + Sync>`: observer storage — `Arc` required (not `Box`) because `IslandGa` shares one observer across rayon island threads
- Existing `log` + `env_logger`: kept as-is; `LogObserver` is a bridge that calls `log::*!()` macros, not a logger owner

**Critical version note:** `tracing 0.1` is HIGH confidence. `metrics 0.24` is MEDIUM confidence — verify with `cargo search metrics` before committing to Cargo.toml.

### Expected Features

The `GaObserver` trait defines 8 lifecycle hooks. All have default no-op bodies for forward compatibility. Engine-specific sub-traits extend the base with 2 additional hooks each.

**Must have (table stakes):**
- `GaObserver<U>` trait with `on_run_start`, `on_generation_end`, `on_best_chromosome_updated`, `on_termination`, `on_run_end` hooks — foundational contract
- `LogObserver` (no feature flag) — drop-in replacement for all 88 hardcoded `log!()` call sites; must produce identical output
- `with_observer()` builder method on `Ga`, `IslandGa`, `Nsga2Ga` — ergonomic attachment point
- `CompositeObserver` — enables stacking multiple observers via a single `with_observer()` call
- `Send + Sync` supertrait bounds on `GaObserver` from day one — cannot be added later without a semver break

**Should have (differentiators):**
- `TracingObserver` (feature `observer-tracing`) — structured spans integrating with OpenTelemetry, Jaeger, Honeycomb
- `MetricsObserver` (feature `observer-metrics`) — standard metric names (`ga.generation.best_fitness`, etc.) for Prometheus/StatsD dashboards
- `IslandGaObserver` sub-trait with `on_island_generation_end` and `on_migration` hooks
- `Nsga2Observer` sub-trait with `on_pareto_front_updated` and `on_generation_end_nsga2` hooks

**Defer (v2.2+):**
- `on_operator_event` per-operator hooks — requires threading observer through every operator factory; high complexity, low MVP value
- Per-gene hooks (`on_crossover`, `on_mutation` per chromosome) — explicitly anti-feature; would be called millions of times per run

### Architecture Approach

The observer module lives at `src/observer/` and is purely additive. Three orchestrator structs (`Ga`, `IslandGa`, `Nsga2Ga`) each gain a single `observer` field and a `with_observer()` builder method. All existing `log!()` call sites in the sequential driver loops are replaced with `if let Some(obs) = &self.observer { obs.on_*(...) }` dispatch; `LogObserver` emits the identical `log!()` output. The user callback (`run_with_callback`) is not replaced — observers are fire-and-forget; the callback retains `ControlFlow::Break` semantics.

**Major components:**
1. `src/observer/mod.rs` — `GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>` trait definitions; always compiled
2. `src/observer/log_observer.rs` — `LogObserver`; bridges observer events to `log!()` macros; no feature flag needed
3. `src/observer/composite.rs` — `CompositeObserver<U>`; fans out to `Vec<Arc<dyn GaObserver<U>>>`; no feature flag needed
4. `src/observer/tracing_observer.rs` — `TracingObserver`; behind `observer-tracing` feature flag
5. `src/observer/metrics_observer.rs` — `MetricsObserver`; behind `observer-metrics` feature flag
6. Modified run loops in `ga.rs`, `island/mod.rs`, `nsga2/mod.rs` — replace hardcoded `log!()` with observer dispatch

### Critical Pitfalls

1. **Arc clone inside rayon par_iter closures** — clone `Arc` once before the parallel region; pass `&Arc` or `Option<&dyn GaObserver<U>>` into closures; never call observer hooks from inside `par_iter`
2. **`tracing::Span::enter()` guard in rayon closures** — guards must be dropped on the thread they were created on; use `span.in_scope(|| ...)` or emit `tracing::event!()` only (no span operations) inside parallel regions
3. **`metrics` counter contention in par_iter** — only record metrics from `on_generation_end` (sequential outer loop); never call `metrics::counter!()` from inside `par_iter` closures
4. **LogObserver breaks env_logger initialization** — `LogObserver` must NOT call `try_init()` or own the logger; it is a bridge that calls `log::*!()` macros; logger initialization stays in `ga.rs`
5. **Missing `Send + Sync` supertrait on `GaObserver`** — adding these bounds after publishing is a semver break; they must be part of the initial trait definition

## Implications for Roadmap

Based on research, the build order is strictly dependency-driven. The trait hierarchy is the keystone; nothing else can be built until it is defined. Observer integration into run loops must happen engine by engine so tests can validate backward compatibility at each step. Feature-flagged observers are independent of each other and can be deferred or parallelized.

### Phase 1: GaObserver Trait Foundation

**Rationale:** Every other component depends on the trait definition. `Send + Sync` supertrait bounds and default no-op methods must be locked in here — both are semver-breaking if added later. Hook granularity decisions (per-generation vs per-individual) made here determine whether Pitfalls 1, 3, and 5 are avoided by design.

**Delivers:** `GaObserver<U>`, `IslandGaObserver<U>`, `Nsga2Observer<U>` trait definitions; `src/observer/mod.rs`; `Cargo.toml` feature flag stubs

**Addresses:** Table stakes — foundational trait contract; all five hook lifecycle events

**Avoids:** Pitfall 11 (`Send + Sync` missing), Pitfall 5 (zero-overhead contract), Pitfall 1 (arc contention — access pattern decided here)

### Phase 2: LogObserver and Backward Compatibility

**Rationale:** `LogObserver` is the trust-building phase — it proves the observer system replicates existing behavior exactly. No new dependencies. Completing this phase means all 88 log call sites are migrated and the backward-compatibility guarantee is validated by tests. This is the riskiest migration step and must be done before optional features are added.

**Delivers:** `LogObserver` implementation; migration of all `log!()` call sites in `ga.rs`; integration of observer field and `with_observer()` into `Ga<U>`; backward-compat test suite

**Addresses:** Table stakes — identical log output requirement; `with_observer()` builder method

**Avoids:** Pitfall 4 (`LogObserver` must not own `env_logger`), Pitfall 12 (serde-gated log call sites missed in migration), Pitfall 10 (env_logger re-init in tests)

### Phase 3: Engine Integration (IslandGa and Nsga2Ga)

**Rationale:** Once `Ga<U>` integration is validated, the same pattern is applied to `IslandGa<U>` and `Nsga2Ga<U>`. These engines have unique lifecycle events (`on_migration`, `on_pareto_front_updated`) that require their sub-traits. This phase also covers `CompositeObserver`, which depends on all three trait definitions being stable.

**Delivers:** Observer fields + `with_observer()` on `IslandGa` and `Nsga2Ga`; migration of island and NSGA-II log call sites; `CompositeObserver`

**Addresses:** `IslandGaObserver` and `Nsga2Observer` sub-traits; `CompositeObserver`

**Avoids:** Pitfall 8 (island observer Arc contention — same rules as base observer)

### Phase 4: TracingObserver (observer-tracing feature)

**Rationale:** First optional feature. Medium complexity but high user value for anyone using distributed tracing or OpenTelemetry. Must add CI matrix for feature flag combinations at this point (`--no-default-features`, `--features observer-tracing`, `--all-features`).

**Delivers:** `TracingObserver` behind `observer-tracing` feature flag; `tracing = "0.1"` optional dep; CI matrix for feature combinations

**Addresses:** `TracingObserver` differentiator feature

**Avoids:** Pitfall 2 (`Span::enter()` in rayon closures — `in_scope` or events only), Pitfall 9 (feature flag re-export errors — CI matrix catches this)

### Phase 5: MetricsObserver (observer-metrics feature)

**Rationale:** Second optional feature, independent of Phase 4. Parallel build if bandwidth allows. Slightly lower confidence on `metrics` crate version — verify `0.24` before this phase begins.

**Delivers:** `MetricsObserver` behind `observer-metrics` feature flag; `metrics = "0.24"` optional dep; standard metric name set (`ga.generation.best_fitness`, etc.)

**Addresses:** `MetricsObserver` differentiator feature

**Avoids:** Pitfall 3 (metrics counter contention — only record from `on_generation_end`)

### Phase Ordering Rationale

- Phases 1 and 2 are strictly sequential: the trait must exist before `LogObserver` can implement it, and `Ga<U>` integration must be validated before other engines are touched
- Phase 3 depends on Phase 2 (the pattern is proven); `CompositeObserver` is placed here because it depends on all three trait definitions
- Phases 4 and 5 are independent of each other and can be built in either order; both depend on Phase 1 (trait definitions) and Phase 2 (feature flag infrastructure pattern)
- This ordering ensures tests can catch regressions at each step before more complexity is added

### Research Flags

Phases needing deeper research during planning:

- **Phase 4:** Verify that `tracing` span semantics behave correctly when `IslandGa` runs islands in rayon threads — specifically whether `Span::follows_from` is the right model for parallel island traces or whether a flat event model is safer
- **Phase 5:** Verify exact `metrics` crate version (`0.24`) via `cargo search metrics` before implementation begins; confirm recorder API has not changed in recent releases

Phases with standard, well-documented patterns (skip research-phase):

- **Phase 1:** Rust trait design with default methods and `Send + Sync` supertraits is fully documented
- **Phase 2:** `log!()` macro bridge pattern is established; all call sites are known from codebase analysis
- **Phase 3:** Same integration pattern as Phase 2; `CompositeObserver` fan-out is straightforward Rust

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH (tracing) / MEDIUM (metrics) | `tracing 0.1` stable 4+ years; `metrics 0.24` version needs verification at build time |
| Features | HIGH | Based on direct codebase analysis; all 88 log call sites inventoried; hook surface derived from existing log targets |
| Architecture | HIGH | Integration points identified from direct code inspection of all three run loops; rayon safety model confirmed |
| Pitfalls | HIGH | Concurrency pitfalls are well-documented in rayon and tracing crate documentation; env_logger behavior is confirmed |

**Overall confidence:** HIGH

### Gaps to Address

- **`metrics` crate version:** Run `cargo search metrics` before Phase 5 to confirm `0.24` is current; the facade API has been stable since `0.21` but exact minor version should not be assumed
- **`on_generation_start` vs `on_generation_end` hook naming:** ARCHITECTURE.md uses `on_generation_complete` while FEATURES.md uses `on_generation_end` — standardize naming before Phase 1 ships the public trait
- **`with_observer()` type signature for sub-trait engines:** `IslandGa::with_observer` takes `Arc<dyn IslandGaObserver<U>>` while `Ga::with_observer` takes `Arc<dyn GaObserver<U>>`; the type mismatch is intentional but must be documented clearly in the API so users know which observer type to implement for each engine
- **Checkpoint log warning (`log::warn!` in serde-gated block):** Must be audited and migrated during Phase 2; easy to miss without `cargo build --features serde` in the CI matrix

## Sources

### Primary (HIGH confidence)
- `/src/ga.rs`, `/src/island/mod.rs`, `/src/nsga2/mod.rs`, `/src/stats.rs` — direct code inspection; all lifecycle touchpoints and log call sites inventoried
- `Cargo.toml` — existing feature flag patterns and dependency versions
- `.planning/PROJECT.md` — backward compat, zero-overhead, `Send + Sync`, MSRV 1.81.0 constraints
- `tracing` crate (docs.rs/tracing) — `Send + Sync` model, span threading contract, `in_scope` vs `enter()` semantics
- Rust API compatibility guidelines — semver implications of adding default methods to published traits

### Secondary (MEDIUM confidence)
- `metrics 0.24` crate (docs.rs/metrics) — recorder-separation facade pattern confirmed; exact version unverified via live web
- rayon `par_iter` closure capture semantics — Arc atomic contention pattern under parallel writes

### Tertiary (LOW confidence)
- `metrics` crate internal use of `RwLock`/`DashMap` for global registry — inferred from common patterns; validate with benchmark before shipping Phase 5

---
*Research completed: 2026-03-23*
*Ready for roadmap: yes*
