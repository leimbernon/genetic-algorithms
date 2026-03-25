# Phase 14: LogObserver + Log Migration - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement `LogObserver` — a `GaObserver<U>` implementation that reproduces all pre-v2.2.0 log output — and remove every hardcoded `log!()` call from `src/ga.rs`. Island GA and NSGA-II log calls are deferred to Phase 16 (where `IslandGaObserver` / `Nsga2Observer` sub-traits land). Phase 14 scope is strictly `src/ga.rs` + `src/observer/log.rs`.

</domain>

<decisions>
## Implementation Decisions

### Island/nsga2 migration strategy

- Phase 14 removes **only** the 17 `log!()` calls in `src/ga.rs` — not `island/` or `nsga2/`
- Island GA and NSGA-II log output **goes silent** between Phase 14 and Phase 16 (acceptable clean break)
- The final grep check (`grep returns results only inside log_observer.rs for ga.rs, island/, nsga2/`) is Phase 16's acceptance criterion, not Phase 14's
- Phase 14's grep check: `grep -n "info!\|debug!\|trace!\|warn!" src/ga.rs` returns zero results

### Log message fidelity

- **Exact strings required** — byte-for-byte match: same `target=`, same KV fields (e.g., `method="run"`), same message format string, same log level
- `LogObserver` must reproduce the `target="ga_events"` target on all relevant calls
- KV fields like `method="run"` are preserved (not dropped)
- Claude's discretion: if exact fidelity requires extending hook parameters or mapping values, do whatever achieves fidelity — LogObserver is the priority

### ga.rs removal scope

- **All 17** `log!()` call sites in `src/ga.rs` are removed (run loop + helpers `limit_reached()` + `parent_crossover()`)
- Debug/trace calls inside `limit_reached()` and `parent_crossover()` that don't map directly to a hook: **absorb into the nearest lifecycle hook** (e.g., parent_crossover debug → `on_crossover_complete`, limit_reached debug → `on_generation_end`)
- The `warn!()` for checkpoint save failures **is reproduced** by `LogObserver` at the same warn level

### LogObserver public API

- **Module**: `src/observer/log.rs`, re-exported from `src/observer/mod.rs` as `pub use log::LogObserver`
- **Path**: `genetic_algorithms::observer::LogObserver`
- **Also re-exported at crate root**: `pub use observer::LogObserver` in `src/lib.rs`
- **Non-generic unit struct**: `pub struct LogObserver;` with `impl<U: ChromosomeT> GaObserver<U> for LogObserver`
- Users attach with `ga.with_observer(Arc::new(LogObserver))`

### Claude's Discretion

- Internal mapping strategy for helper-method log calls (absorb into lifecycle hooks)
- Whether `LogObserver` uses a helper `emit()` method or inline `log::log!()` calls per hook
- Test strategy for fidelity verification (capturing log output in tests)
- Whether to add any convenience constructors beyond `Arc::new(LogObserver)`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Log Observer (#183) — LOG-01 through LOG-03
- `.planning/REQUIREMENTS.md` §Observer Trait (#182) — OBS-01 through OBS-04 (hook surface is locked from Phase 13)

### Existing log call sites (must all be removed)
- `src/ga.rs` — 17 `log!()` call sites to remove; exact targets, levels, KV fields, and message strings must be reproduced by `LogObserver`
- `src/island/mod.rs`, `src/island/nsga2.rs`, `src/island/migration.rs`, `src/nsga2/mod.rs` — log sites deferred to Phase 16; DO NOT modify these files in Phase 14

### Observer infrastructure (Phase 13 output)
- `src/observer/mod.rs` — `GaObserver<U>` trait definition with all 12 hooks; `LogObserver` implements this trait
- `src/ga.rs` — `observer` field, `with_observer()` builder, `notify()` helper already in place

### Structural precedents
- `src/reporter/mod.rs` — `Reporter<U>` trait and its built-in implementations (NoopReporter, SimpleReporter, DurationReporter) — structural template for observer module layout
- `src/stats.rs` — `GenerationStats` struct fields (available to hooks like `on_generation_end`)
- `src/ga.rs` `TerminationCause` enum (available to `on_run_end` hook)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `log` crate v0.4 — already a dependency; `log::log!(target, level, ...)` macro is the right API for LogObserver's implementation
- `GaObserver<U>` hooks now in place in `src/observer/mod.rs` — LogObserver implements the 12 hooks
- `notify()` helper in `src/ga.rs` — already wired at 12 call sites; LogObserver receives all notifications automatically once attached

### Established Patterns
- Non-generic unit structs implementing generic traits: see `NoopReporter` in `src/reporter/mod.rs` as the direct template
- `src/observer/mod.rs` module layout: follow same pattern for `log.rs` submodule and `pub use log::LogObserver` re-export
- Crate root re-exports: `src/lib.rs` already exports `GaObserver`, `ExtensionEvent`, `NoopObserver` — add `LogObserver` alongside

### Integration Points
- `src/observer/mod.rs` — add `mod log;` and `pub use log::LogObserver`
- `src/lib.rs` — add `LogObserver` to the observer re-export block
- `src/ga.rs` — remove the 17 `log!()` calls (the `notify()` infrastructure is already there)

</code_context>

<specifics>
## Specific Ideas

- `LogObserver` is a unit struct — `pub struct LogObserver;` — with no fields. Users do `ga.with_observer(Arc::new(LogObserver))`. No builder pattern needed.
- The exact log output contract is defined by the existing `log!()` calls in `src/ga.rs` — treat those as the "golden output" specification. LogObserver must match them.
- KV fields in the structured log macros (e.g., `method="run"`) must be preserved in `LogObserver` — these are used by log filters and formatters in production.
- The `target="ga_events"` target must be preserved on all existing calls — users may have configured log targets to filter on this value.

</specifics>

<deferred>
## Deferred Ideas

- Island/nsga2 log migration — Phase 16 (when IslandGaObserver/Nsga2Observer sub-traits land)
- Any additional LogObserver configuration (verbosity levels, format customization) — out of scope; LogObserver is a faithful migration, not a new feature

</deferred>

---

*Phase: 14-logobserver-log-migration*
*Context gathered: 2026-03-25*
