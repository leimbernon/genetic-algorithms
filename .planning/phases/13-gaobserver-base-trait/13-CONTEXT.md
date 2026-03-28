# Phase 13: GaObserver Base Trait - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Define the `GaObserver<U>` trait and integrate it into `Ga<U>`. This phase delivers the notification architecture that all later observer implementations (LogObserver, TracingObserver, sub-traits) depend on. Scope is strictly `Ga<U>` — Island and NSGA-II sub-traits are Phase 16.

</domain>

<decisions>
## Implementation Decisions

### Hook surface

Full hook set for the base trait (11 hooks):

**Lifecycle:**
- `on_run_start()` — no payload
- `on_generation_start(generation: usize)` — fires before operators run; enables wall-clock timing by observers
- `on_generation_end(stats: &GenerationStats)` — fires after stats collected, passes full stats
- `on_run_end(cause: TerminationCause, all_stats: &[GenerationStats])` — mirrors Reporter's on_finish

**Special events:**
- `on_new_best(generation: usize, best: U)` — owned clone of chromosome (U: Clone, already guaranteed by ChromosomeT)
- `on_stagnation(generation: usize, stagnation_count: usize)` — fires when stagnation counter increments
- `on_extension_triggered(event: ExtensionEvent)` — typed struct payload (see below)

**Operator hooks:**
- `on_selection_complete(generation: usize, duration: Duration, population_size: usize)`
- `on_crossover_complete(generation: usize, duration: Duration, offspring_count: usize)`
- `on_mutation_complete(generation: usize, duration: Duration, population_size: usize)`
- `on_survivor_selection_complete(generation: usize, duration: Duration, population_size: usize)`
- `on_fitness_evaluation_complete(generation: usize, duration: Duration, population_size: usize)` — fitness evaluation is a major step and worth its own hook

All hooks have default no-op bodies — users implement only the hooks they need.

All hooks are zero-cost when no observer is attached (`Option::None` branch eliminates all dispatch and `Instant` measurement).

### ExtensionEvent struct

Stack-allocated struct with:
```
pub struct ExtensionEvent {
    pub generation: usize,
    pub diversity: f64,
    pub extension_type: &'static str,  // e.g. "MassExtinction", "MassGenesis"
}
```

### Operator timing measurement scope

Each operator's `Duration` is measured as: `Instant::now()` immediately before the operator call, `.elapsed()` immediately after. Includes only the operator execution — fitness re-evaluation after crossover is measured separately by `on_fitness_evaluation_complete`.

### Storage and thread safety

- `observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>` field on `Ga<U>`
- Builder: `pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self`
- All hooks are `&self` (not `&mut self`) — required for Arc sharing and rayon compatibility
- `GaObserver<U>: Send + Sync` supertraits are mandatory from day one (adding later is a breaking change)

### Reporter<U> deprecation

Soft-deprecate in v2.2.0:
- `Reporter` trait gets `#[deprecated(since = "2.2.0", note = "use GaObserver<U> instead. Reporter will be removed in v3.0.0.")]`
- `with_reporter()` builder method gets the same `#[deprecated]` attribute
- Reporter continues to work and compiles — zero breakage for existing users
- Both `reporter` and `observer` fields coexist on `Ga<U>` in v2.2.0

### Claude's Discretion

- Module structure: `src/observer/mod.rs` mirroring `src/reporter/` layout
- Exact naming of the `notify_*` helper (inline or method on `Ga<U>`)
- Whether `NoopObserver` is a public struct or just documented as "implement with all defaults"
- `Instant` measurement: whether to skip it when `observer.is_none()` (should be — avoids measurement overhead when unused)
- Re-export from `src/lib.rs` prelude

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Observer Trait (GaObserver base — #182) — OBS-01 through OBS-04

### Structural precedent
- `src/reporter/mod.rs` — `Reporter<U>` trait definition; `GaObserver<U>` follows the same layout (`src/observer/mod.rs`). Note: Reporter uses `&mut self` and `Box` — GaObserver uses `&self` and `Arc`.
- `src/ga.rs` lines 127-129 — existing `reporter` field on `Ga<U>`; `observer` field is added alongside, not replacing
- `src/ga.rs` lines 520-524 — existing `with_reporter()` builder; `with_observer()` follows the same pattern
- `src/ga.rs` lines 711, 878, 1048, 1095 — existing Reporter call sites (4 points); GaObserver adds more

### Types used in hook signatures
- `src/stats.rs` — `GenerationStats` struct (payload for `on_generation_end`)
- `src/ga.rs` `TerminationCause` enum (payload for `on_run_end`)
- `src/traits/chromosome.rs` — `ChromosomeT` bounds; `on_new_best` receives `U` which requires `Clone` (already guaranteed)

### Research
- `.planning/research/ARCHITECTURE.md` — integration points, notification flow in ga.rs, observer storage decision rationale
- `.planning/research/PITFALLS.md` — critical: Arc clone pattern for rayon, Send+Sync supertrait must be Phase 1, zero-overhead verification

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TerminationCause` (in `src/ga.rs`) — public enum, reused as `on_run_end` parameter
- `GenerationStats` (in `src/stats.rs`) — public struct, reused as `on_generation_end` parameter
- `src/reporter/mod.rs` — entire module structure is the direct template for `src/observer/mod.rs`
- `improved` boolean in ga.rs run loop — existing new-best detection logic; `on_new_best` fires when `improved == true` (same condition)

### Established Patterns
- Optional fields on `Ga<U>`: `Option<Box<...>>` for single-owner (Reporter), `Option<Arc<...>>` for shared (observer, fitness_fn, initialization_fn)
- Builder methods: `fn with_X(mut self, ...) -> Self` (not trait methods)
- Module exposure: `pub mod observer;` in `src/lib.rs` + prelude re-exports
- Zero-overhead gate: `if let Some(ref obs) = self.observer { obs.on_X(...); }` — same pattern as reporter

### Integration Points
- `src/ga.rs` — add `observer` field, `with_observer()` builder, `Instant` measurements at 5 operator phases + fitness eval, notification calls at 11 points in the run loop
- `src/ga.rs` `Default` impl — `observer: None`
- `src/lib.rs` — `pub mod observer;` + prelude re-exports for `GaObserver`, `ExtensionEvent`, `NoopObserver`

</code_context>

<specifics>
## Specific Ideas

- `ExtensionEvent.extension_type` is `&'static str` (not `String`) — zero allocation, matches the enum-based extension system where type names are known at compile time
- Operator hooks receive both `Duration` AND a count (population_size or offspring_count) so a single hook call gives the observer enough context to compute throughput (e.g., chromosomes/ms)
- `on_fitness_evaluation_complete` is included even though fitness eval is parallelized internally — the hook receives wall-clock total time (not per-chromosome), which is still the most actionable profiling signal
- `Instant` measurement should be skipped (`observer.is_none()` check before `Instant::now()`) — avoids any measurement cost when no observer is set

</specifics>

<deferred>
## Deferred Ideas

- Per-operator timing hooks with individual chromosome counts (e.g., per-offspring fitness time) — Phase 13 hooks give wall-clock total; per-chromosome breakdown would require threading the observer through operator implementations (EXT-01, deferred to v2.3+)
- `on_checkpoint_saved` hook — EXT-02, low priority, deferred to v2.3+

</deferred>

---

*Phase: 13-gaobserver-base-trait*
*Context gathered: 2026-03-25*
