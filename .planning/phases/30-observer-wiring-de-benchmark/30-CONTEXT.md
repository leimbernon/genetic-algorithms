# Phase 30: Observer Wiring & DE Benchmark - Context

**Gathered:** 2026-04-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire `GaObserver<U>` into the four new engines (DeEngine, ScatterEngine, CellularEngine, AlpsEngine) so users can observe the same lifecycle events as the standard GA. Add a DE-vs-GA convergence benchmark to `benches/de.rs`. All existing engine tests must continue to pass — observer is purely additive.

</domain>

<decisions>
## Implementation Decisions

### Observer API

- **D-01:** Observer type is `Option<Arc<dyn GaObserver<U> + Send + Sync>>` on all four engines — identical to `ga.rs`. Zero overhead when `None`. (Locked in STATE.md prior to this phase.)
- **D-02:** No per-engine sub-traits. All four engines use the base `GaObserver<U>` trait only.
- **D-03:** Wire **5 required hooks only** on all four engines: `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_run_end`. Do NOT wire operator-timing hooks (`on_selection_complete`, `on_mutation_complete`, etc.) in this phase — those can be added per-engine later if needed.
- **D-04:** Builder method: `with_observer(Arc<dyn GaObserver<U> + Send + Sync>) -> Self` on each engine's configuration or engine struct, matching the `ga.rs` pattern.

### GenerationStats Construction

- **D-05:** All four engines call `GenerationStats::from_fitness_values(generation, &fitness_slice, is_maximization)` to build the stats passed to `on_generation_end`. No new stats struct needed.

### ALPS Multi-Layer Handling

- **D-06:** `on_generation_end` receives **merged stats across all layers** — flatten all layer populations into one fitness slice, compute a single `GenerationStats`. Consistent with single-population engine contract; no interface change needed.
- **D-07:** `on_new_best` fires when the **global best across all layers** improves (not per-layer tracking). Consistent with all other engines.

### DE-vs-GA Benchmark (OBS-05)

- **D-08:** Extend `benches/de.rs` — add a GA run alongside existing DE benchmarks. No new bench file or Cargo.toml entry needed.
- **D-09:** Both DE and GA run on **sphere(5D)** (same problem as existing DE benchmarks), with the **same `max_generations`** (e.g., 100). Comparison is wall-time per run as reported by criterion — no evaluation-count normalization.
- **D-10:** The GA run uses the standard `Ga<U>` engine with `RangeChromosome<f64>` and default operators on the same sphere(5D) fitness function.

### Claude's Discretion

- `with_observer()` placement: either on the engine struct directly (like `ga.rs`) or on the configuration struct — follow whatever pattern is cleanest for each engine's existing API.
- `sample_size(10)` on the DE-vs-GA benchmark group (matches existing alps/de bench convention for faster CI runs).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Observer Infrastructure

- `src/observe/observer/mod.rs` — `GaObserver<U>` trait definition (12 hooks, 5 required here), `NoopObserver`, `IslandGaObserver`, `Nsga2Observer`, `AllObserver` blanket impl
- `src/engines/ga.rs` — **canonical reference for observer wiring pattern**: `Option<Arc<dyn GaObserver<U>>>` field, `with_observer()` builder, `notify_observer()` dispatch helper, per-hook timing guards

### New Engines (all need wiring)

- `src/engines/de/engine.rs` — `DeEngine<U>` run loop
- `src/engines/de/configuration.rs` — `DeConfiguration` builder
- `src/engines/scatter/engine.rs` — `ScatterEngine<U>` run loop
- `src/engines/scatter/configuration.rs` — `ScatterConfiguration` builder
- `src/engines/cellular/engine.rs` — `CellularEngine<U>` run loop
- `src/engines/cellular/configuration.rs` — `CellularConfiguration` builder
- `src/engines/alps/engine.rs` — `AlpsEngine<U>` run loop (multi-layer: flatten for stats, global best for on_new_best)
- `src/engines/alps/configuration.rs` — `AlpsConfiguration` builder

### Stats

- `src/stats.rs` — `GenerationStats` struct and `from_fitness_values()` constructor

### Benchmark

- `benches/de.rs` — existing DE bench file to extend with GA comparison group
- `Cargo.toml` — `[[bench]]` entry for `de` (already exists, no change needed)

### Requirements

- `.planning/REQUIREMENTS.md` §OBS-01 through OBS-05 — exact acceptance criteria for each engine's observer wiring and benchmark

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `GenerationStats::from_fitness_values(generation, &[f64], is_maximization)` — computes best/worst/avg/std_dev/diversity from any flat fitness slice; directly usable by all four engines
- `src/observe/observer/mod.rs::NoopObserver` — zero-sized no-op impl; useful for compile tests
- `benches/de.rs::make_pop()` and `sphere()` — reuse for the GA side of the DE-vs-GA benchmark

### Established Patterns

- `ga.rs` observer wiring pattern: store `Option<Arc<dyn GaObserver<U>>>`, helper `fn notify_observer<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F)`, timing guard `if self.observer.is_some() { let t = Instant::now(); ... }` — replicate verbatim on each new engine
- All new engines are in `src/engines/<name>/engine.rs` and `configuration.rs` — no module restructuring needed
- `ProblemSolving` enum in `src/configuration.rs` — used by existing engines to determine minimization/maximization; reuse for `is_maximization` flag in `from_fitness_values`

### Integration Points

- Each engine's `run()` method is where hook call sites are inserted (before loop, each iteration start/end, on best update, after loop)
- The `with_observer()` builder on each engine/configuration must return `Self` for method chaining consistency

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond what's documented — open to standard approaches following `ga.rs` as the canonical reference.

</specifics>

<deferred>
## Deferred Ideas

- Operator-timing hooks for new engines (`on_mutation_complete` for DE trial vectors, `on_selection_complete` for Cellular local tournament, etc.) — out of scope for Phase 30; add per-engine in a later dedicated phase
- Per-layer observer stats for ALPS — deferred; merged stats chosen for Phase 30

</deferred>

---

*Phase: 30-Observer Wiring & DE Benchmark*
*Context gathered: 2026-04-27*
