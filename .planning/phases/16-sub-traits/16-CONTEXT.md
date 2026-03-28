# Phase 16: Sub-Traits - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Define `IslandGaObserver<U>` and `Nsga2Observer<U>` sub-traits, integrate them into `IslandGa<U>` and `Nsga2Ga<U>`, and have `LogObserver` implement all three traits. Phase 16 also completes the log migration by removing all remaining `log!()` calls from `src/island/` and `src/nsga2/`. Scope: `src/observer/mod.rs` (trait definitions), `src/island/mod.rs` (integration + log removal), `src/nsga2/mod.rs` (integration + log removal), `src/observer/log.rs` (new IslandGaObserver + Nsga2Observer impls), plus re-exports.

</domain>

<decisions>
## Implementation Decisions

### IslandGaObserver hook surface

Exactly 4 hooks (matching roadmap success criteria):

```rust
pub trait IslandGaObserver<U: ChromosomeT>: Send + Sync {
    fn on_island_run_start(&self, island_id: usize) {}
    fn on_island_run_end(&self, island_id: usize) {}
    fn on_island_generation_end(&self, island_id: usize, generation: usize, stats: &GenerationStats) {}
    fn on_migration_triggered(&self, generation: usize, migration_count: usize) {}
}
```

- `on_island_generation_end` carries full `&GenerationStats` (same as GaObserver's `on_generation_end`) — compute GenerationStats per island per generation
- `on_migration_triggered` carries `generation` + `migration_count` only (no per-pair island IDs)
- All hooks `&self`, all `Send + Sync` supertraits — required from day one (same as GaObserver)

### Nsga2Observer hook surface

Exactly 3 hooks (matching roadmap success criteria):

```rust
pub trait Nsga2Observer<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize) {}
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {}
    fn on_crowding_distance_calculated(&self, generation: usize, duration_ms: f64) {}
}
```

- Timing hooks use `duration_ms: f64` (consistent with GaObserver operator hooks in Phase 13/15)
- `on_pareto_front_assigned` carries scalar counts, not a slice of front sizes (allocation-free)

### Trait relationship

- `IslandGaObserver<U>` and `Nsga2Observer<U>` are **independent traits** — no supertrait relationship to `GaObserver<U>`
- Both have `Send + Sync` as supertraits (required for rayon island parallel execution)
- `LogObserver` gets three separate `impl` blocks: one per trait

### Observer storage and attachment

Same pattern as `Ga<U>`:

```rust
// IslandGa<U>
observer: Option<Arc<dyn IslandGaObserver<U> + Send + Sync>>

pub fn with_observer(mut self, obs: Arc<dyn IslandGaObserver<U> + Send + Sync>) -> Self

// Nsga2Ga<U>
observer: Option<Arc<dyn Nsga2Observer<U> + Send + Sync>>

pub fn with_observer(mut self, obs: Arc<dyn Nsga2Observer<U> + Send + Sync>) -> Self
```

- `Option::None` → zero overhead (same guard as Phase 13)
- Builder method `with_observer()` on both engines

### Module structure

Both sub-traits live in `src/observer/mod.rs` alongside `GaObserver<U>`:

```rust
// src/observer/mod.rs
pub trait GaObserver<U: ChromosomeT>: Send + Sync { ... }
pub trait IslandGaObserver<U: ChromosomeT>: Send + Sync { ... }
pub trait Nsga2Observer<U: ChromosomeT>: Send + Sync { ... }
```

- Re-exported from `src/lib.rs` prelude alongside `GaObserver`
- Paths: `genetic_algorithms::observer::IslandGaObserver`, `genetic_algorithms::observer::Nsga2Observer`

### Island/nsga2 log migration

Phase 16 **removes all remaining `log!()` calls** from `src/island/mod.rs` and `src/nsga2/mod.rs`. LogObserver implements the new hooks to reproduce them:

**Island mapping:**
- `info!(target: "island_events", "Starting island model GA: ...")` → `on_island_run_start(island_id)` (fires once per run start, before the generation loop)
- `debug!(target: "island_events", "Migration performed at generation {}")` → `on_migration_triggered(gen, count)`
- `info!(target: "island_events", "Fitness target reached at generation {}")` → absorbed into `on_island_run_end(island_id)` — the fitness-target path also ends the run, so run-end hook covers it
- `debug!(target: "island_events", "Initialized island {} with {} chromosomes")` → absorbed into `on_island_run_start(island_id)` (initialization happens before first generation)

**Nsga2 mapping:** read `src/nsga2/mod.rs` log calls and map to the 3 Nsga2Observer hooks; timing wraps the sort/distance operations.

**Acceptance criterion (Phase 16):** `grep -rn "info!\|debug!\|trace!\|warn!" src/island/ src/nsga2/` returns zero results.

### Claude's Discretion

- Whether `GenerationStats` computation per island is inlined or extracted into a helper
- `notify_island` helper pattern (mirroring existing `notify` helper in ga.rs)
- Exact `GenerationStats` fields available from island context (may need `diversity` computed from island population)
- Test coverage strategy for the 3 new trait impls

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Sub-Traits (#185) — SUB-01 through SUB-03

### Observer infrastructure (already in place)
- `src/observer/mod.rs` — `GaObserver<U>` trait definition + `LogObserver` re-export; sub-traits added here
- `src/observer/log.rs` — `LogObserver` struct; Phase 16 adds `impl<U: ChromosomeT> IslandGaObserver<U> for LogObserver` and `impl<U: ChromosomeT> Nsga2Observer<U> for LogObserver`

### Engine source (integration targets)
- `src/island/mod.rs` — `IslandGa<U>` struct, `run()` method (lines ~316-370), `evolve_islands_one_generation()` (lines ~380+), existing `log!()` call sites to remove
- `src/nsga2/mod.rs` — `Nsga2Ga<U>` struct, `run()` method (line ~194), existing `log!()` call sites to remove

### Phase 13 precedent (notification pattern)
- `src/ga.rs` — `observer` field, `with_observer()` builder, `notify()` helper — exact pattern to replicate in IslandGa and Nsga2Ga
- `.planning/phases/13-gaobserver-base-trait/13-CONTEXT.md` — storage/threading decisions that apply here unchanged

### Types used in hook signatures
- `src/stats.rs` — `GenerationStats` struct (payload for `on_island_generation_end`)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/ga.rs` `notify()` helper + `with_observer()` builder — copy pattern verbatim for `IslandGa` and `Nsga2Ga`
- `src/observer/log.rs` `LogObserver` — add two new `impl` blocks; unit struct already defined, no changes to struct itself
- `GenerationStats` (`src/stats.rs`) — already used in `on_generation_end`; same type for `on_island_generation_end`

### Established Patterns
- `Option<Arc<dyn Trait + Send + Sync>>` field + `if let Some(ref obs) = self.observer { obs.on_X(...); }` guard — zero-overhead pattern established in Phase 13
- Log target naming: `target: "island_events"` in island, `target: "nsga2_events"` in nsga2 — LogObserver implementations must preserve these targets

### Integration Points
- `src/island/mod.rs` — add `observer` field, `with_observer()` builder, notification calls at migration, run start/end, per-generation end; remove ~4 existing `log!()` calls
- `src/nsga2/mod.rs` — add `observer` field, `with_observer()` builder, notification calls at pareto assignment, non-dominated sort, crowding distance; remove ~2 existing `log!()` calls
- `src/lib.rs` — add `IslandGaObserver` and `Nsga2Observer` to observer re-exports
- `src/observer/mod.rs` — add both sub-trait definitions

</code_context>

<specifics>
## Specific Ideas

- The `on_island_generation_end` hook fires inside `evolve_islands_one_generation()` which runs inside `par_iter_mut()`. The observer Arc must be cloned before the parallel region (same pattern as Phase 13 island thread safety note in STATE.md: "Phase 16: Island `par_iter_mut()` requires same clone-once-before-parallel pattern as Phase 13")
- `on_island_run_start` fires in the outer `run()` loop, once before the generation loop starts — it's not per-island (there's one island model run, not one per island). The `island_id` parameter may be omitted or set to 0 if the run encompasses all islands. **Re-read the island run() method to determine exact semantics before implementing.**
- The fitness-target-reached path (`if dist < 1e-10 { return Ok(best); }`) ends the run without calling `on_island_run_end` unless explicitly placed before the return — LogObserver must fire the run-end log there too

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 16-sub-traits*
*Context gathered: 2026-03-26*
