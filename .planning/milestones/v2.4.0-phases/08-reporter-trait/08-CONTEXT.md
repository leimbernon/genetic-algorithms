# Phase 8: Reporter Trait - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a `Reporter<U>` trait that lets users attach structured lifecycle observers to `Ga<U>` via
`.with_reporter(Box::new(my_reporter))`. Four hooks fire at key execution points: `on_start`,
`on_generation_complete`, `on_new_best`, `on_finish`. Built-in reporters: `NoopReporter` (default,
zero overhead), `SimpleReporter` (stdout progress every N gens), `DurationReporter` (per-phase
timing at run end). The existing `run_with_callback()` API is unchanged.

</domain>

<decisions>
## Implementation Decisions

### Reporter trait shape
- `trait Reporter<U: ChromosomeT>` — generic over U because `on_new_best` receives the chromosome
- Other hooks are chromosome-agnostic (receive only stats / termination cause)
- Hook signatures:
  - `fn on_start(&mut self)` — pure notification, no payload
  - `fn on_generation_complete(&mut self, stats: &GenerationStats)` — stats only (generation, best/worst/avg fitness, diversity, population_size); no population access (avoids borrow issues and keeps Reporter object-safe for dyn)
  - `fn on_new_best(&mut self, generation: usize, best: U)` — full chromosome clone so users can inspect or store the best solution mid-run; requires `U: Clone` (already required by `ChromosomeT`)
  - `fn on_finish(&mut self, cause: TerminationCause, all_stats: &[GenerationStats])` — full stats history; enables DurationReporter to print a final timing summary
- Default impl for all hooks is a no-op (trait has default bodies) so users only override what they need

### Reporter dispatch mechanism
- Already decided (STATE.md): `Box<dyn Reporter<U> + Send>` — trait object, not generic param on `Ga`
- `Send` bound only — hooks are called sequentially from the main GA thread; no `Sync` required
- Field on `Ga<U>`: `reporter: Option<Box<dyn Reporter<U> + Send>>`
- Builder method: `fn with_reporter(mut self, reporter: Box<dyn Reporter<U> + Send>) -> Self`

### Reporter invocation points
- Reporter fires inside both `run()` and `run_with_callback()` — consistent behavior regardless of which run method is used
- Reporter does NOT replace or deprecate `run_with_callback()`; the two coexist independently
- `on_start` fires once before the first generation
- `on_generation_complete` fires at the end of every generation (after stats collection, before callback)
- `on_new_best` fires whenever the population's best chromosome improves vs. the previous best (same improvement detection logic already used for stagnation tracking)
- `on_finish` fires once after the loop exits (after termination cause is set)

### SimpleReporter behavior
- Prints to stdout every N generations, where N is set at construction: `SimpleReporter::new(n: usize)`
- Output format: `[Gen {current}/{max}] Best: {best_fitness:.4} | Diversity: {diversity:.4}`
- `max` comes from the total generations limit; if unknown, omit the denominator: `[Gen {current}]`
- Always prints at `on_finish` (final summary line) regardless of N
- Uses `on_generation_complete` for periodic printing and `on_finish` for the final line

### DurationReporter behavior
- Tracks wall-clock time per operator phase: selection, crossover, mutation, survivor
- Timing hooks called inside the GA loop via the reporter (not via separate instrumentation)
- Prints a final table at `on_finish` with per-phase totals and percentages
- Uses `on_finish(cause, all_stats)` to print; timing data stored as mutable state in the reporter struct

### NoopReporter
- Default reporter when no reporter is configured
- All hook methods have empty default bodies in the trait — `NoopReporter` simply uses the defaults
- `Ga<U>` stores `Option<Box<dyn Reporter<U> + Send>>` — `None` means no reporter, zero-overhead path (no virtual dispatch)

### Claude's Discretion
- Module location: `src/reporter/` with `mod.rs`, `noop.rs`, `simple.rs`, `duration.rs`
- Re-export from `src/lib.rs` / prelude
- Whether DurationReporter measures time via `std::time::Instant` stored per-phase or uses a single timer across the loop
- Exact formatting of the DurationReporter table (aligned columns, percentage, etc.)
- `on_new_best` trigger: same logic as the existing `improved` boolean in `run_with_callback` — avoids code duplication

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Reporter Trait — REP-01 through REP-04

### Existing patterns to follow
- `src/ga.rs` — `Ga<U>` struct, `run_with_callback()` implementation, existing callback invocation and `improved` boolean for new-best detection (lines ~990–1030)
- `src/stats.rs` — `GenerationStats` struct fields (what `on_generation_complete` receives)
- `src/ga.rs` `TerminationCause` enum — what `on_finish` receives as `cause`
- `src/traits/configuration.rs` — `ConfigurationT` builder trait supertrait pattern; `with_reporter()` extends `Ga<U>` directly (not via a new config trait, since reporter is not a `GaConfiguration` field)
- `src/traits/chromosome.rs` — `ChromosomeT` bounds on `U`; `on_new_best` receives `U` which already requires `Clone`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TerminationCause` enum (in `src/ga.rs`) — already public, used as `on_finish` parameter
- `GenerationStats` struct (in `src/stats.rs`) — already passed to the callback; same type flows to `on_generation_complete`
- `improved` boolean in `run_with_callback()` — existing new-best detection logic; `on_new_best` hook fires when this is `true`
- `stagnation_count` reset logic — shares the same condition as the reporter's `on_new_best` trigger

### Established Patterns
- Builder methods on `Ga<U>` are `fn with_X(mut self, ...) -> Self` (not trait methods for reporter, since `Reporter<U>` is `U`-specific and can't go in a general config trait)
- Optional fields use `Option<Arc<...>>` for shared refs (fitness_fn, initialization_fn) — reporter uses `Option<Box<...>>` since it's not shared
- `cfg_attr(feature = "serde", ...)` pattern for optional serde support — reporter types likely don't need serde; skip unless needed
- Modules exposed via `pub mod` in `src/lib.rs` and re-exported in prelude

### Integration Points
- `src/ga.rs` — add `reporter: Option<Box<dyn Reporter<U> + Send>>` field; add `with_reporter()` builder; add hook calls at four points in `run_with_callback()` (the single internal run loop)
- `src/lib.rs` — `pub mod reporter;` + prelude re-exports for `Reporter`, `SimpleReporter`, `DurationReporter`, `NoopReporter`
- `src/ga.rs` `Default` impl — `reporter: None`

</code_context>

<specifics>
## Specific Ideas

- The `Reporter<U>` trait's generic parameter (`U: ChromosomeT`) exists solely because `on_new_best` needs to hand the user the actual chromosome. All other hooks are chromosome-agnostic, so the generic is a minimal concession to expressiveness.
- `Box<dyn Reporter<U> + Send>` — the `Send` bound enables the reporter to be sent to island model worker threads in future without API change; no `Sync` needed since hooks are sequential.
- `SimpleReporter::new(n)` — `n` is the print interval in generations; passing `1` prints every generation. The user configures this at construction, not via the GA builder.
- DurationReporter timing: hook calls happen inside `run_with_callback()` — the reporter measures wall-clock slices around each operator phase by recording `Instant::now()` before and after.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 08-reporter-trait*
*Context gathered: 2026-03-21*
