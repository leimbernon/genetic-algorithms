# Phase 59: Restart Strategies — IPOP / BIPOP - Context

**Gathered:** 2026-06-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Extend the existing `CmaEngine` (Phase 56) with automatic restart strategies: IPOP (increasing population on each restart) and BIPOP (alternating large and small restarts). When the engine stagnates (no fitness improvement for N consecutive generations), it resets its internal state and resumes the search with an adjusted population size according to the configured strategy. A new `GaObserver::on_restart` hook notifies observers on each restart event. This phase does NOT introduce a new engine — it extends `CmaEngine` in-place via a new `RestartStrategy` enum field on `CmaConfiguration`.

**Out of scope:** Restart strategies for any engine other than `CmaEngine`. Sigma-collapse-based stagnation detection. Budget-based BIPOP alternation (Hansen 2009 original algorithm). Per-restart fitness history in `CmaResult`.

</domain>

<decisions>
## Implementation Decisions

### Stagnation Detection

- **D-01:** A restart is triggered when best fitness has not improved for `stagnation_threshold` consecutive generations. No sigma-collapse detection in this phase — improvement-based stagnation is sufficient and easier for users to reason about.

### RestartStrategy Enum

- **D-02:** `RestartStrategy` is a public enum with two variants, each carrying its own knobs:
  ```
  pub enum RestartStrategy {
      Ipop {
          population_scale: f64,      // multiply population_size by this on each restart (e.g. 2.0)
          stagnation_threshold: usize, // generations without improvement before restart
          max_restarts: usize,         // engine exits after this many restarts
      },
      Bipop {
          population_scale: f64,       // large restart: multiply by this (same as IPOP)
          small_population_size: usize,// small restart: fixed size (or 0 = 1/5 of default)
          stagnation_threshold: usize,
          max_restarts: usize,
      },
  }
  ```
  BIPOP alternates strictly: odd restarts use large population (IPOP-style scaled), even restarts use small population (`small_population_size`). No budget-tracking — strict alternation.

- **D-03:** `CmaConfiguration` gains `restart_strategy: Option<RestartStrategy>` (default `None` = no restarts, existing behavior unchanged) and a builder method `with_restart_strategy(RestartStrategy) -> Self`. This is the only change to `CmaConfiguration`.

### Best Tracking Across Restarts

- **D-04:** `CmaResult.best` is the global best individual found across ALL restart runs. `CmaResult.best_fitness` is the corresponding fitness. The engine tracks the global best during its loop and updates it on improvement across restart boundaries.

### CmaResult Extension

- **D-05:** `CmaResult` gains one new field: `total_restarts: usize` (default `0` when no restart strategy is configured or no restarts occurred). No per-restart history. Backward-compatible: `CmaResult` struct literal construction is not part of the public API (users construct via `.run()`).

### on_restart Hook

- **D-06:** `GaObserver` gains a new 13th hook with a default no-op body:
  ```
  fn on_restart(&self, _event: &RestartEvent) {}
  ```
  `RestartEvent` is a new `#[derive(Debug, Clone, Copy)]` struct:
  ```
  pub struct RestartEvent {
      pub restart_number: usize,        // 1-based restart count
      pub generation: usize,            // generation at which the restart was triggered
      pub population_size_before: usize,
      pub population_size_after: usize,
      pub kind: RestartKind,
  }
  ```
  `RestartKind` is a new `#[derive(Debug, Clone, Copy)]` enum:
  ```
  pub enum RestartKind {
      Ipop,
      BipopLarge,
      BipopSmall,
  }
  ```

### State Reset on Restart

- **D-07:** On each restart, the engine fully resets `CmaState` (sigma back to `config.sigma0`, covariance matrix to identity, evolution paths pc/ps to zero, mean re-derived from a fresh initial population sample). The population is re-initialized via the user's `init_fn`. This gives each restart a clean search start from a new random sample.

### Claude's Discretion

- Default values for `population_scale` in example/docs (common: `2.0` for IPOP per Hansen)
- Default value for `stagnation_threshold` if omitted in docs (common: `100` or `10*n` where n is dimension)
- Whether `small_population_size = 0` in BIPOP auto-computes to `max(1, floor(default_lambda / 5))` or some other formula
- Internal bookkeeping variable names and struct layout
- Whether `RestartStrategy`, `RestartEvent`, `RestartKind` are added to `src/observe/observer/mod.rs` or to a new `src/engines/cma/restart.rs` module

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CMA-ES Engine (extend this)
- `src/engines/cma/engine.rs` — `CmaEngine`, `CmaState`, full `run()` loop; restart logic inserts into this file
- `src/engines/cma/configuration.rs` — `CmaConfiguration`; add `restart_strategy: Option<RestartStrategy>` and builder method here
- `src/engines/cma/mod.rs` — module re-exports; add new public types here

### Observer Trait (add on_restart hook here)
- `src/observe/observer/mod.rs` — `GaObserver<U>` trait with 12 existing hooks; `on_restart` becomes the 13th hook; `RestartEvent` and `RestartKind` are defined here (or imported from a cma restart module)

### Pattern Reference (most recent engine)
- `src/engines/eda/engine.rs` — Most recent engine; observer wiring and `notify()` helper pattern to follow
- `src/engines/pso/engine.rs` — PSO engine; another recent engine pattern reference

### lib.rs Re-exports
- `src/lib.rs` — add `pub use engines::cma::{RestartStrategy, RestartEvent, RestartKind}` re-exports

### Result Struct
- `src/engines/cma/engine.rs` (`CmaResult`) — add `total_restarts: usize` field; verify this is constructed only inside `run()` so adding a field is non-breaking

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CmaState::new(n, lambda, &config, initial_mean)` — state constructor; restart calls this again with a new `lambda` value derived from the strategy
- `CmaEngine.notify()` helper (`fn notify<F: Fn(&dyn GaObserver<U>)>`) — use the same pattern to call `obs.on_restart(&event)` after triggering a restart
- `CmaEngine.find_best(&pop)` — used after init; same call used after each restart re-init
- `crate::rng::make_rng()` — already used for sampling; re-use for fresh restart init

### Established Patterns
- `Option<Arc<dyn GaObserver<U> + Send + Sync>>` — observer pattern used in every engine; no change needed, just call `on_restart` via the existing `notify()` helper
- `CmaConfiguration` builder methods (`.with_cc()`, `.with_cs()`, etc.) — `with_restart_strategy()` follows the same pattern
- `ExtensionEvent` / `on_extension_triggered` in `observer/mod.rs` — structural template for `RestartEvent` / `on_restart`

### Integration Points
- `CmaEngine::run()` — the stagnation counter tracking and restart loop both live here; the existing `best_fitness` tracking becomes cross-restart global tracking
- `CmaResult` struct constructed at the end of `run()` — add `total_restarts` field there
- `GaObserver` trait in `src/observe/observer/mod.rs` — add `on_restart` default method; any existing observer implementations continue to compile unchanged (default no-op)

</code_context>

<specifics>
## Specific Ideas

- BIPOP alternation is strict (odd restart = large, even restart = small) — not budget-based. This is a deliberate simplification over Hansen 2009.
- `RestartEvent.restart_number` is 1-based (first restart = 1, matches user expectation when logging "Restart #1 triggered").
- The `stagnation_threshold` meaning: if best fitness at generation `g` equals best fitness at generation `g - stagnation_threshold`, a restart is triggered.

</specifics>

<deferred>
## Deferred Ideas

- Sigma-collapse stagnation detection (`sigma < sigma_min`) — could be a future addition to `RestartStrategy` variants without breaking changes
- Budget-based BIPOP alternation (Hansen 2009 original) — deferred; strict alternation is chosen for this phase
- Per-restart `RestartSummary` history in `CmaResult` — deferred; `total_restarts: usize` is sufficient for now
- Restart strategies for other engines (PSO, EDA) — out of scope for this phase

</deferred>

---

*Phase: 59-restart-strategies-ipop-bipop*
*Context gathered: 2026-06-05*
