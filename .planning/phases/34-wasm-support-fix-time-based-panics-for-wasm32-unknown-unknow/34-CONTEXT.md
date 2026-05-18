# Phase 34: WASM support — fix time-based panics for wasm32-unknown-unknown - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Eliminate `Instant::now()` panics and rayon link failures that prevent the library from running on `wasm32-unknown-unknown` targets. After this phase the library must compile and run a standard GA/NSGA-II loop in a WASM/browser environment (as shown in issue #236).

**In scope:**
- Wrap all 7 `Instant::now()` call sites with `#[cfg(not(target_arch = "wasm32"))]`; return `Duration::ZERO` on WASM
- Wrap rayon parallel iterators in all 6 engines with cfg-gated sequential fallbacks on WASM
- Keep `DurationReporter` compiling on WASM (reports `Duration::ZERO`) — no API surface removal
- Emit `log::warn!` when `max_duration_secs` is configured but the target has no clock

**Out of scope:**
- wasm-bindgen / wasm-pack integration, JS bindings, or WASM-specific examples
- `web-time` / `performance.now()` for actual browser timing (deferred)
- Island model WASM support (migration uses threads — separate problem)
- Explicit `wasm` feature flag in Cargo.toml

</domain>

<decisions>
## Implementation Decisions

### Detection Mechanism

- **D-01:** Use `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(target_arch = "wasm32"))]` for all conditional compilation. **No feature flag** — detection is automatic when cross-compiling to wasm32. Zero friction for library consumers; follows Rust ecosystem conventions (same pattern as `getrandom`, `rand`).

### Time Resolution on WASM

- **D-02:** All `Instant::now()` call sites are wrapped with `#[cfg(not(target_arch = "wasm32"))]`. On WASM, all elapsed times are `Duration::ZERO`. Applies to: `src/engines/ga.rs` (4 sites), `src/engines/nsga2/mod.rs` (2 sites), `src/observe/reporter/duration.rs` (1 site).
- **D-03:** `DurationReporter` **compiles on WASM** and returns `Duration::ZERO` for all measurements. Non-breaking — the struct remains in the public API on all targets.
- **D-04:** Observer callbacks that accept `Duration` parameters receive `Duration::ZERO` on WASM. No signature changes.

### Rayon Sequential Fallback

- **D-05:** All 6 engines (GA, NSGA-II, DE, Scatter, Cellular, ALPS) get sequential fallback on WASM. Rayon parallel iterators (`.par_iter()`, `par_iter_mut()`, `into_par_iter()`) are wrapped with `#[cfg]`:
  - `#[cfg(not(target_arch = "wasm32"))]` → rayon parallel version
  - `#[cfg(target_arch = "wasm32")]` → standard sequential iterator version
- **D-06:** No new trait abstractions for the parallel/sequential toggle — direct `cfg` at each call site is sufficient and follows the no-premature-abstraction policy.

### max_duration_secs Behavior on WASM

- **D-07:** When `max_duration_secs` is set and the target is WASM (where `start_time.elapsed()` always returns `Duration::ZERO`), emit a one-time `log::warn!` at engine start:
  ```
  log::warn!(target: "ga_events", "max_duration_secs is not supported on wasm32 — time limit will be ignored");
  ```
  The limit is then silently ignored for the rest of the run. No panic, no compile error.

### Claude's Discretion

- Exact placement of `cfg` guards (module-level `use` vs. inline at each call site) — choose whichever keeps the diff minimal and readable.
- Whether to add a `#[allow(unused_imports)]` or restructure `use std::time::Instant` for WASM targets to avoid dead-code warnings.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Engine Time Usage (all Instant::now() sites)
- `src/engines/ga.rs` — 4 `Instant::now()` sites: line 750 (`start_time`), lines 766/785/825 (observer timing)
- `src/engines/nsga2/mod.rs` — 2 `Instant::now()` sites: lines 234/254 (observer timing)
- `src/observe/reporter/duration.rs` — 1 `Instant::now()` site: line 55 (DurationReporter::on_start)

### Rayon Usage per Engine
- `src/engines/ga.rs` — parallel fitness eval and crossover/mutation loops
- `src/engines/nsga2/mod.rs` — parallel fitness eval
- `src/engines/de/` — parallel trial vector evaluation
- `src/engines/cellular/` — parallel neighbor updates
- `src/engines/alps/` — parallel layer processing
- `src/engines/scatter/` — parallel improvement phase

### Observer Interface
- `src/observe/observer/mod.rs` — `GaObserver` trait; `Duration` parameter on all `on_*_complete` methods (line 34+)
- `src/observe/observer/composite.rs` — passes `Duration` through to all observers

### Configuration
- `src/configuration.rs:252` — `StoppingCriteria.max_duration_secs: Option<f64>`
- `src/engines/ga.rs:1191` — max_duration_secs check location

### GitHub Issue
- Issue #236 — original WASM panic report and user's Cargo.toml setup (reference for reproduction)

No external ADRs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/observe/reporter/duration.rs` — the entire struct only needs its `Instant` field and `Instant::now()` calls wrapped; the rest of the implementation is unchanged
- `log::warn!` macro already used throughout the codebase with `target: "ga_events"` — same pattern applies for the max_duration warning

### Established Patterns
- `#[cfg(feature = "serde")]` / `#[cfg(feature = "visualization")]` — existing conditional-compilation pattern in `lib.rs` and throughout; `cfg(target_arch)` follows the same structure
- Operator factory pattern (enum + factory fn) is unaffected — WASM changes are purely in engine execution and timing code

### Integration Points
- `GaObserver` trait methods all accept `Duration` — these must continue to compile on WASM; passing `Duration::ZERO` is the chosen approach (no signature change)
- `StoppingCriteria` struct stays unchanged — the `max_duration_secs` field is kept; only the engine's runtime check is cfg-gated

</code_context>

<specifics>
## Specific Ideas

- The issue reporter's exact stack trace originates in `std/src/sys/time/unsupported.rs:13:9` — this confirms the panic is from `Instant::now()`, not from any other platform-unsupported call
- User's setup: `wasm-pack`, `wasm-bindgen = "0.2"`, `getrandom = { version = "0.3", features = ["wasm_js"] }` — the getrandom wasm_js feature is already in their tree, so `rand`'s WASM support is likely already handled; only `Instant` and `rayon` need fixing in this library

</specifics>

<deferred>
## Deferred Ideas

- `web-time` crate integration for real `performance.now()` timing in WASM (user chose stub zeros; can be a future `wasm` feature flag phase)
- Island model WASM support — migration topology relies on thread-based channels; separate, larger problem
- wasm-bindgen JS bindings / `#[wasm_bindgen]` public API surface — out of scope for this fix phase
- WASM-specific example (`examples/wasm_onemax.rs`) — could demonstrate usage once the fix is in

</deferred>

---

*Phase: 34-wasm-support-fix-time-based-panics-for-wasm32-unknown-unknow*
*Context gathered: 2026-05-07*
