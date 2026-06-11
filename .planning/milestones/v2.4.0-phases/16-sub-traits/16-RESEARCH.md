# Phase 16: Sub-Traits - Research

**Researched:** 2026-03-26
**Domain:** Rust trait design — sub-observer traits for `IslandGa<U>` and `Nsga2Ga<U>`
**Confidence:** HIGH

## Summary

Phase 16 is an internal Rust refactoring/extension phase. There are no external dependencies to add and no new crates to evaluate. All patterns are established by Phase 13 (`GaObserver`) and repeated here for two additional engines. The research task is therefore about understanding the exact code surfaces to touch, the log calls to remove, and the threading constraints that apply.

The island engine runs islands in parallel via `rayon::par_iter_mut()`. The observer `Arc` must be cloned once before entering the parallel closure — the same constraint documented in STATE.md for Phase 13. The `Nsga2Ga` run loop is sequential (only `create_offspring` uses rayon internally), so no special observer-cloning ceremony is needed there.

The two existing `log!()` call sites in `src/nsga2/mod.rs` (one `info!` at run start, one `debug!` per generation) map cleanly to the three `Nsga2Observer` hooks via timing wrappers. The four `log!()` call sites in `src/island/mod.rs` (one `info!` at run start, one `info!` at fitness-target-reached, one `debug!` after migration) map to the four `IslandGaObserver` hooks. The fitness-target-reached path returns early and must fire `on_island_run_end` before returning, or the hook is silently skipped.

**Primary recommendation:** Replicate the `notify()` helper + `Option<Arc<dyn Trait>>` field pattern from `Ga<U>` verbatim in both `IslandGa<U>` and `Nsga2Ga<U>`. No new crates. No architectural novelty.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**IslandGaObserver hook surface** — exactly 4 hooks:
```rust
pub trait IslandGaObserver<U: ChromosomeT>: Send + Sync {
    fn on_island_run_start(&self, island_id: usize) {}
    fn on_island_run_end(&self, island_id: usize) {}
    fn on_island_generation_end(&self, island_id: usize, generation: usize, stats: &GenerationStats) {}
    fn on_migration_triggered(&self, generation: usize, migration_count: usize) {}
}
```
- `on_island_generation_end` carries full `&GenerationStats`
- `on_migration_triggered` carries `generation` + `migration_count` only
- All hooks `&self`, all `Send + Sync` supertraits

**Nsga2Observer hook surface** — exactly 3 hooks:
```rust
pub trait Nsga2Observer<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize) {}
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {}
    fn on_crowding_distance_calculated(&self, generation: usize, duration_ms: f64) {}
}
```
- Timing hooks use `duration_ms: f64`
- `on_pareto_front_assigned` carries scalar counts (allocation-free)

**Trait relationship** — `IslandGaObserver<U>` and `Nsga2Observer<U>` are independent traits (no supertrait relationship to `GaObserver<U>`). `LogObserver` gets three separate `impl` blocks.

**Observer storage and attachment:**
```rust
// IslandGa<U>
observer: Option<Arc<dyn IslandGaObserver<U> + Send + Sync>>
pub fn with_observer(mut self, obs: Arc<dyn IslandGaObserver<U> + Send + Sync>) -> Self

// Nsga2Ga<U>
observer: Option<Arc<dyn Nsga2Observer<U> + Send + Sync>>
pub fn with_observer(mut self, obs: Arc<dyn Nsga2Observer<U> + Send + Sync>) -> Self
```

**Module structure** — both sub-traits live in `src/observer/mod.rs` alongside `GaObserver<U>`. Re-exported from `src/lib.rs` alongside `GaObserver`.

**Island/nsga2 log migration** — Phase 16 removes ALL remaining `log!()` calls from `src/island/mod.rs` and `src/nsga2/mod.rs`. Acceptance: `grep -rn "info!\|debug!\|trace!\|warn!" src/island/ src/nsga2/` returns zero results.

### Claude's Discretion
- Whether `GenerationStats` computation per island is inlined or extracted into a helper
- `notify_island` helper pattern (mirroring existing `notify` helper in ga.rs)
- Exact `GenerationStats` fields available from island context (may need `diversity` computed from island population)
- Test coverage strategy for the 3 new trait impls

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SUB-01 | User can attach `IslandGaObserver` to `IslandGa<U>` via `with_observer()` and receive island-specific events | Covered by `notify` helper pattern from Phase 13; island log call sites mapped below |
| SUB-02 | User can attach `Nsga2Observer` to `Nsga2Ga<U>` via `with_observer()` and receive NSGA-II-specific events | Covered by `notify` helper pattern; nsga2 log call sites mapped below; timing via `std::time::Instant` |
| SUB-03 | `LogObserver` implements all three observer traits providing complete log migration coverage | Covered by two new `impl<U: ChromosomeT> XxxObserver<U> for LogObserver` blocks in `src/observer/log.rs` |
</phase_requirements>

---

## Standard Stack

### Core

No new dependencies. All libraries already in `Cargo.toml`.

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::sync::Arc` | std | Thread-safe observer sharing across rayon | Required for `Send + Sync` trait object in par_iter |
| `std::time::Instant` | std | `duration_ms` timing for Nsga2Observer hooks | Already used in `ga.rs` for operator hooks |
| `log` 0.4 | existing | `LogObserver` island/nsga2 impls emit via log facade | Already a direct dependency |

### Supporting

No new supporting libraries needed.

### Alternatives Considered

None — all choices are locked by Phase 13 precedent.

**Installation:** No new packages to install.

---

## Architecture Patterns

### Recommended Project Structure

No new files or folders needed. Changes are confined to:

```
src/
├── observer/
│   ├── mod.rs          # Add IslandGaObserver<U> and Nsga2Observer<U> trait defs
│   └── log.rs          # Add two new impl blocks for LogObserver
├── island/
│   └── mod.rs          # Add observer field, with_observer(), notify_island(), hook calls, remove log!()
├── nsga2/
│   └── mod.rs          # Add observer field, with_observer(), notify_nsga2(), hook calls, remove log!()
└── lib.rs              # Add IslandGaObserver, Nsga2Observer to pub use re-exports
```

### Pattern 1: notify() helper (replicate from ga.rs)

**What:** An `#[inline]` private method that takes a closure and invokes it only if an observer is attached.
**When to use:** At every hook call site — avoids scattered `if let Some(ref obs) = self.observer` everywhere.

```rust
// Source: src/ga.rs line 554-560 — exact pattern to copy
#[inline]
fn notify<F: FnOnce(&dyn IslandGaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

Name the helpers `notify` on both structs (each struct has its own `notify` that captures its own observer type).

### Pattern 2: Observer field declaration

**What:** `Option<Arc<dyn Trait + Send + Sync>>` field added to the engine struct.

```rust
// Add to IslandGa<U> struct (src/island/mod.rs)
pub struct IslandGa<U> where U: ChromosomeT {
    // ... existing fields ...
    observer: Option<Arc<dyn IslandGaObserver<U> + Send + Sync>>,
}
```

The field must be `Option` defaulting to `None` — the `IslandGa::new()` constructor(s) must initialise it to `None`.

### Pattern 3: Clone-once-before-parallel for island par_iter

**What:** The observer `Arc` must be cloned before entering `par_iter_mut()`. Inside the rayon closure the `Arc` clone is moved in and used via `if let Some(ref obs) = observer_clone`.

```rust
// In evolve_islands_one_generation()
let observer_clone: Option<Arc<dyn IslandGaObserver<U> + Send + Sync>> =
    self.observer.as_ref().map(Arc::clone);

self.islands
    .par_iter_mut()
    .enumerate()
    .try_for_each(|(idx, island)| {
        // ... operators ...
        if let Some(ref obs) = observer_clone {
            obs.on_island_generation_end(idx, gen, &stats);
        }
        Ok(())
    })?;
```

This is the same pattern STATE.md explicitly documents: "Phase 16: Island `par_iter_mut()` requires same clone-once-before-parallel pattern as Phase 13."

### Pattern 4: Timing wrappers for Nsga2Observer

**What:** `std::time::Instant::now()` + `.elapsed().as_secs_f64() * 1000.0` wraps the sort and distance operations.

```rust
// Only incur Instant::now() cost when observer is attached
let t = if self.observer.is_some() { Some(std::time::Instant::now()) } else { None };
let fronts = self.perform_sorting(&population, &directions, has_constraints);
if let Some(start) = t {
    self.notify(|obs| obs.on_non_dominated_sort_complete(gen, start.elapsed().as_secs_f64() * 1000.0));
}
```

This preserves the zero-overhead guarantee (OBS-03) when no observer is attached.

### Anti-Patterns to Avoid

- **Calling `obs.on_island_generation_end()` outside the parallel region:** The generation index (`gen`) is only available in the outer `run()` loop. The `evolve_islands_one_generation()` method currently does not receive `gen`. Either thread `gen` as a parameter, or compute stats in the outer loop after the call returns. See "Open Questions" below.
- **Skipping `on_island_run_end` on the fitness-target-reached early return:** `src/island/mod.rs` line 349 does `return Ok(best)` without a run-end hook. The planner must add a notification before that return.
- **Using `&mut self` hooks:** All hooks are `&self`. Do not introduce `&mut self` — it breaks `Send + Sync`.
- **Cloning the `Arc` inside the rayon closure per iteration:** Clone once before the parallel region, not inside `.par_iter_mut()`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Thread-safe observer sharing | Custom `RwLock<Box<dyn Trait>>` | `Arc<dyn Trait + Send + Sync>` | Already proven in Phase 13; `Arc` is sufficient since all hooks are `&self` |
| Timing measurement | Platform-specific timing | `std::time::Instant` | Portable, zero-dependency, consistent with existing `ga.rs` operator timing |
| Log output in LogObserver impls | Custom formatting logic | Mirror exact target/level/format strings from removed `log!()` calls | Backward-compatibility contract (LOG-01, LOG-02) |

**Key insight:** This phase is almost entirely copy-paste-and-adapt of Phase 13 patterns. The only novel engineering decisions are the `GenerationStats` construction per island and the timing gate for Nsga2Observer hooks.

---

## Common Pitfalls

### Pitfall 1: Early-return path skips on_island_run_end

**What goes wrong:** The fitness-target-reached path (`dist < 1e-10`) at `src/island/mod.rs` line 344 returns immediately. If `on_island_run_end` is only placed at the bottom of `run()`, it is never called on a successful early exit.
**Why it happens:** Rust's early return bypasses any code below it in the function.
**How to avoid:** Call `self.notify(|obs| obs.on_island_run_end(0))` on both the early-return path and the normal exit path. Or extract into a helper that is called from both paths.
**Warning signs:** Test that attaches a counting observer and triggers fitness-target early exit — `on_island_run_end` count is 0 instead of 1.

### Pitfall 2: Generation index unavailable inside evolve_islands_one_generation

**What goes wrong:** `on_island_generation_end` needs `generation: usize`, but `evolve_islands_one_generation()` currently takes no `gen` parameter.
**Why it happens:** The method was not designed to receive or report generation context.
**How to avoid:** Add `gen: usize` as a parameter to `evolve_islands_one_generation()`. This is a private method — it is not a breaking change.
**Warning signs:** Compiler error "use of undeclared variable `gen`" inside the method, or always passing `0` as the generation.

### Pitfall 3: GenerationStats construction per island

**What goes wrong:** `GenerationStats` needs fitness values from the island population. The island struct holds `Population<U>` but `GenerationStats::from_fitness_values()` takes `&[f64]`.
**Why it happens:** `evolve_islands_one_generation` operates on `&mut Vec<Population<U>>` via `par_iter_mut` — each island's `chromosomes` must be iterated to collect fitness values.
**How to avoid:** Inside the parallel closure, after survivor selection, collect `island.chromosomes.iter().map(|c| c.fitness()).collect::<Vec<_>>()` and call `GenerationStats::from_fitness_values(gen, &fitnesses, is_maximization)`. The `diversity` field will be populated as `fitness_std_dev` (same as `Ga<U>`). The `dynamic_mutation_probability` field should be `None` (island model does not implement dynamic mutation).
**Warning signs:** Panic on `unwrap()` of an empty fitness slice; wrong `generation` index (off by one).

### Pitfall 4: Missing use declarations after log removal

**What goes wrong:** `src/island/mod.rs` and `src/nsga2/mod.rs` both have `use log::{debug, info};` at the top. After removing all `log!()` calls, those `use` items become dead imports causing `unused_imports` warnings (or errors under `#![deny(warnings)]`).
**Why it happens:** The `use` line is not automatically removed when all usages are deleted.
**How to avoid:** Remove the `use log::{debug, info};` line at the same time as the last log call site.
**Warning signs:** `cargo clippy` reports unused imports.

### Pitfall 5: is_maximization not available inside evolve_islands_one_generation

**What goes wrong:** `GenerationStats::from_fitness_values()` takes `is_maximization: bool`, but the method doesn't currently receive `ProblemSolving`.
**Why it happens:** `_problem_solving` is currently an unused parameter (prefixed with `_`). It is passed in from `run()` but ignored.
**How to avoid:** Remove the `_` prefix from `_problem_solving` and use it: `problem_solving == ProblemSolving::Maximization`.
**Warning signs:** Stats show wrong best/worst even though fitness values are correct.

---

## Code Examples

Verified patterns from official sources (`src/ga.rs`):

### IslandGaObserver trait definition

```rust
// src/observer/mod.rs — add after GaObserver<U> definition
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

pub trait IslandGaObserver<U: ChromosomeT>: Send + Sync {
    fn on_island_run_start(&self, _island_id: usize) {}
    fn on_island_run_end(&self, _island_id: usize) {}
    fn on_island_generation_end(&self, _island_id: usize, _generation: usize, _stats: &GenerationStats) {}
    fn on_migration_triggered(&self, _generation: usize, _migration_count: usize) {}
}

pub trait Nsga2Observer<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(&self, _generation: usize, _front_count: usize, _population_size: usize) {}
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
    fn on_crowding_distance_calculated(&self, _generation: usize, _duration_ms: f64) {}
}
```

### LogObserver impl for IslandGaObserver

```rust
// src/observer/log.rs — new impl block
impl<U: ChromosomeT> IslandGaObserver<U> for LogObserver {
    fn on_island_run_start(&self, _island_id: usize) {
        // Absorbs island/mod.rs: info!(target: "island_events", "Starting island model GA: ...")
        // and: debug!(target: "island_events", "Initialized island {} with {} chromosomes")
        log::info!(target: "island_events", "Island model GA started");
    }
    fn on_island_run_end(&self, _island_id: usize) {
        // Absorbs: info!(target: "island_events", "Fitness target reached at generation {}")
        log::info!(target: "island_events", "Island model GA ended");
    }
    fn on_island_generation_end(&self, _island_id: usize, generation: usize, _stats: &GenerationStats) {
        // No pre-existing log call at generation granularity in island — new hook
        log::debug!(target: "island_events", "Island generation {} complete", generation);
    }
    fn on_migration_triggered(&self, generation: usize, migration_count: usize) {
        // Absorbs: debug!(target: "island_events", "Migration performed at generation {}")
        log::debug!(target: "island_events", "Migration performed at generation {} (count={})", generation, migration_count);
    }
}
```

**Note:** The exact format strings for `LogObserver` impls must match the removed `log!()` calls verbatim for LOG-02 compliance. The examples above are indicative; the planner must cross-reference against the actual removed call sites.

### Island log calls to remove (confirmed from source)

| Line | Call | Maps to hook |
|------|------|--------------|
| ~329 | `info!(target: "island_events", "Starting island model GA: ...")` | `on_island_run_start(0)` |
| ~345 | `info!(target: "island_events", "Fitness target reached at generation {}")` | `on_island_run_end(0)` (before early return) |
| ~359 | `debug!(target: "island_events", "Migration performed at generation {}")` | `on_migration_triggered(gen, migration_count)` |

Note: `on_island_generation_end` fires per-island per-generation and has no pre-existing `log!()` counterpart — it is a new capability.

### Nsga2 log calls to remove (confirmed from source)

| Line | Call | Maps to hook |
|------|------|--------------|
| ~208 | `info!(target: "nsga2_events", "Starting NSGA-II: ...")` | No direct mapping — absorbed into run-start (not a hook in `Nsga2Observer`). Consider omitting or keeping in LogObserver as `on_pareto_front_assigned` first call. |
| ~280 | `debug!(target: "nsga2_events", "Generation {} complete, population size = {}")` | `on_pareto_front_assigned(gen, fronts.len(), population.len())` |

**Key finding:** The nsga2 `info!` at run start does not map to any `Nsga2Observer` hook (which only covers pareto/sort/distance events). The LogObserver impl for `Nsga2Observer` cannot reproduce that message — it will be silently dropped unless the planner adds it to `on_pareto_front_assigned` generation-0 logic, or accepts it as dropped log output. Verify with user if LOG-02 strictness requires it.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Direct `log!()` calls in island/nsga2 | Observer hooks + LogObserver | Phase 16 | No duplicate output; users control what gets logged |
| No per-engine observer specialization | `IslandGaObserver` and `Nsga2Observer` sub-traits | Phase 16 | Users can attach typed observers to each engine |

---

## Open Questions

1. **`on_island_run_start` semantics — one call for the whole run or one per island?**
   - What we know: The CONTEXT.md says "fires once per run start, before the generation loop" and notes the `island_id` parameter "may be omitted or set to 0 if the run encompasses all islands."
   - What's unclear: Whether `island_id` should always be `0` for the run-start/run-end hooks (since there is one island *model* run, not one run per island).
   - Recommendation: Pass `0` as `island_id` for `on_island_run_start` and `on_island_run_end` until island-level run semantics are clarified. The CONTEXT.md says "Re-read the island run() method to determine exact semantics before implementing." The planner should include a task to do this read explicitly.

2. **nsga2 run-start `info!` message has no hook counterpart — is it dropped or absorbed?**
   - What we know: `Nsga2Observer` has no `on_run_start` hook. The `info!(target: "nsga2_events", "Starting NSGA-II: ...")` at line ~208 must be removed (LOG-02) but cannot be reproduced by any of the 3 defined hooks.
   - What's unclear: Whether LOG-02 allows this message to be dropped.
   - Recommendation: The planner should include a clarification task or note that this message is intentionally dropped (it duplicates information visible from configuration, not from GA runtime behavior).

3. **`GenerationStats` `diversity` field computation for islands**
   - What we know: `GenerationStats::from_fitness_values()` sets `diversity = fitness_std_dev`. Island populations have fitness values available after survivor selection.
   - What's unclear: Whether the island context has `dynamic_mutation_probability` (likely `None`).
   - Recommendation: Always pass `None` for `dynamic_mutation_probability` in island stats. Set `diversity` to `fitness_std_dev` computed inline.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none — inline `#[cfg(test)] mod tests` or `tests/` directory |
| Quick run command | `cargo test observer` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SUB-01 | `IslandGa::with_observer()` stores observer; hooks fire at migration, run start/end, generation end | integration | `cargo test island_observer` | ❌ Wave 0 |
| SUB-02 | `Nsga2Ga::with_observer()` stores observer; hooks fire with correct args at sort/distance/pareto | integration | `cargo test nsga2_observer` | ❌ Wave 0 |
| SUB-03 | `LogObserver` implements `IslandGaObserver` and `Nsga2Observer`; compiles and passes trait bounds | unit (compile) | `cargo test log_observer` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test observer`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/island_observer.rs` — covers SUB-01: attach counting observer to `IslandGa`, run, assert hook call counts
- [ ] `tests/nsga2_observer.rs` — covers SUB-02: attach counting observer to `Nsga2Ga`, run, assert hook call counts and args
- [ ] `src/observer/log.rs` inline tests or `tests/log_observer_sub_traits.rs` — covers SUB-03: verify `LogObserver: IslandGaObserver<BinaryChromosome>` and `LogObserver: Nsga2Observer<BinaryChromosome>` at compile time

---

## Sources

### Primary (HIGH confidence)

- `src/ga.rs` lines 542-560 — `with_observer()` and `notify()` pattern
- `src/observer/mod.rs` — `GaObserver<U>` trait definition, `NoopObserver`
- `src/observer/log.rs` — `LogObserver` struct, existing `impl GaObserver<U> for LogObserver`
- `src/island/mod.rs` lines 303-367 — `IslandGa::run()`, log call sites, `evolve_islands_one_generation()` structure
- `src/nsga2/mod.rs` lines 194-284 — `Nsga2Ga::run()`, log call sites
- `src/stats.rs` — `GenerationStats` struct fields and `from_fitness_values()` constructor
- `src/lib.rs` lines 82-97 — current observer re-exports
- `.planning/phases/16-sub-traits/16-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)

- `.planning/STATE.md` — accumulated constraint "Phase 16: Island `par_iter_mut()` requires same clone-once-before-parallel pattern as Phase 13"

### Tertiary (LOW confidence)

None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all libraries already present
- Architecture: HIGH — exact patterns from Phase 13 confirmed by reading source
- Pitfalls: HIGH — derived directly from reading `island/mod.rs` and `nsga2/mod.rs` source, not from speculation
- Log call mapping: HIGH — confirmed by reading source lines with line numbers

**Research date:** 2026-03-26
**Valid until:** 2026-04-25 (stable codebase, no external dependencies)
