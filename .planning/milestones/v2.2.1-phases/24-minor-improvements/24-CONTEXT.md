# Phase 24: Minor Improvements - Context

**Gathered:** 2026-04-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Five internal allocations/sorts eliminated across stats, selection, and island migration — completing the full set of v2.2.1 optimizations. No public API changes, no observable behavior changes, no new dependencies.

Requirements: MISC-01, MISC-02, MISC-03, MISC-04, MISC-05

</domain>

<decisions>
## Implementation Decisions

### MISC-01: Stats push restructuring (gen_stats ownership)
- **Push-last pattern**: Use `gen_stats` normally throughout the generation loop for all four usages (diversity checks, extension trigger, reporter, observer). Move the push to the end of the loop, just before the observer notification.
- **dynamic_mutation_probability**: Set the field directly on `gen_stats` before pushing — `gen_stats.dynamic_mutation_probability = Some(new_p)` — then push. Eliminates the `self.stats.last_mut()` call entirely.
- **Observer notification**: After push, borrow `self.stats.last().unwrap()` for the observer call instead of cloning.
- Result: `self.stats.push(gen_stats)` is the move, `self.stats.last().unwrap()` replaces the `cloned().unwrap_or(gen_stats.clone())` pattern.

### MISC-02: Truncation selection O(n) partitioning
- Replace `indexed.sort_by(...)` with `select_nth_unstable_by()` to get top-k in O(n).
- **Elite member trace logging**: Keep the per-member trace loop but drop the rank number — log `"Elite member -> index {} fitness {}"` without rank. Honest about the unordered result from select_nth_unstable.
- Debug summary log (population size, truncation size) unchanged.

### MISC-03: Best chromosome scan deduplication
- Claude's Discretion — `fitness_calculation()` already finds `best_idx` internally; planner should ensure that result propagates without a second scan. No user preference on exact approach.

### MISC-04: Island migration O(n) sort elimination
- Replace `indices.sort_by(...)` in `select_best()` and `replace_worst()` with `select_nth_unstable_by()`.
- Claude's Discretion on exact restructuring within the migration functions.

### MISC-05: Island migration migrant sharing
- Use `Arc<Vec<U>>` to share migrant data across neighbors — one allocation, ref-count bumps per neighbor.
- `replace_worst()` signature changed to accept `&[U]` (not `Arc<Vec<U>>`) — callers deref the Arc. Function stays Arc-agnostic, works with any slice source.

### Claude's Discretion
- Variable naming and code structure within the above patterns
- MISC-03 exact implementation (already well-understood from requirements)
- Exact reborrow patterns within the stats push restructure (as long as push-last is used)

</decisions>

<canonical_refs>
## Canonical References

No external specs — requirements fully captured in decisions above and REQUIREMENTS.md.

### Requirements
- `.planning/REQUIREMENTS.md` — MISC-01 through MISC-05 definitions and acceptance criteria

### Source files being modified
- `src/ga.rs` — stats push (MISC-01), best-chromosome scan (MISC-03)
- `src/operations/selection/truncation.rs` — O(n) truncation partitioning (MISC-02)
- `src/island/migration.rs` — select_nth_unstable (MISC-04), Arc<Vec<U>> sharing (MISC-05)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `select_nth_unstable_by()`: Already used in `src/ga.rs:1397` and `:1419` for elitism — same pattern applies to truncation and migration
- `Arc::clone()`: Already used for `FitnessFnWrapper` sharing — same idiom for migrants

### Established Patterns
- **Push-last ownership**: Common Rust pattern — use the value, move it at the end, borrow from the collection for remaining uses
- **`&[U]` over `Vec<U>` in fn signatures**: Consistent with existing `fitness_calculation(&mut self)` style — prefer slice refs over owned types in internal helpers
- **select_nth_unstable_by**: Already proven correct in elitism path (ga.rs) — safe to replicate in selection and migration

### Integration Points
- `self.stats` vec in `ga.rs`: Push-last changes the point where GenerationStats enters the vec — dynamic mutation no longer needs `last_mut()`
- Observer `on_generation_end`: Must receive the stats entry that includes `dynamic_mutation_probability` — guaranteed by setting the field before push
- `migration.rs` topology loop: `for neighbor in topology.neighbors(island_idx)` — the Arc clone goes inside this loop

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 24-minor-improvements*
*Context gathered: 2026-04-04*
