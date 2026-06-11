---
phase: 25-alternative-metaheuristics
reviewed: 2026-06-10T00:00:00Z
depth: standard
files_reviewed: 29
files_reviewed_list:
  - src/engines/alps/mod.rs
  - src/engines/cellular/mod.rs
  - src/engines/de/mod.rs
  - src/engines/ga.rs
  - src/engines/island/configuration.rs
  - src/engines/island/migration.rs
  - src/engines/island/mod.rs
  - src/engines/island/nsga2.rs
  - src/engines/island/topology.rs
  - src/engines/nsga2/configuration.rs
  - src/engines/nsga2/crowding_distance.rs
  - src/engines/nsga2/mod.rs
  - src/engines/nsga2/non_dominated_sort.rs
  - src/engines/nsga2/pareto.rs
  - src/engines/scatter/mod.rs
  - src/lib.rs
  - src/observe/checkpoint.rs
  - src/observe/observer/composite.rs
  - src/observe/observer/log.rs
  - src/observe/observer/metrics_observer.rs
  - src/observe/observer/mod.rs
  - src/observe/observer/tracing_observer.rs
  - src/observe/visualization/mod.rs
  - src/types/chromosomes/binary.rs
  - src/types/chromosomes/list.rs
  - src/types/chromosomes/mod.rs
  - src/types/chromosomes/range.rs
  - src/types/genotypes/binary.rs
  - src/types/genotypes/list.rs
  - src/types/genotypes/mod.rs
  - src/types/genotypes/range.rs
findings:
  critical: 3
  warning: 7
  info: 3
  total: 13
status: issues_found
---

# Phase 25: Code Review Report

**Reviewed:** 2026-06-10T00:00:00Z
**Depth:** standard
**Files Reviewed:** 29 (5 listed files in scope were not found on disk)
**Status:** issues_found

## Summary

Phase 25 restructured the source tree into `src/engines/`, `src/types/`, and `src/observe/` groups and added placeholder stubs for future engines (DE, Scatter, Cellular, ALPS). The public API is re-exported through `#[path]` attributes in `src/lib.rs`. Most existing code was moved without logical changes; however, the review found three blockers, seven warnings, and three informational items across the existing engine and observer code. Five files listed in the review scope (`src/observe/reporter/*.rs`) do not exist on disk — those are noted but not counted as reviewed.

---

## Structural Findings (fallow)

No structural pre-pass was provided for this review.

---

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Missing migration at generation 0 — off-by-one skips first migration window

**File:** `src/engines/island/mod.rs:469`

**Issue:** The migration condition is `gen > 0 && gen % migration_interval == 0`. This means migration never fires at `gen == 0`. Assuming the default `migration_interval = 10`, the first actual migration occurs at generation 10, but the window `gen == 0..migration_interval` is silently skipped. For small `max_generations` values this can mean zero migrations occur when `max_generations <= migration_interval`. The same guard is present in `src/engines/island/nsga2.rs:337` with identical logic.

**Fix:**
```rust
// Remove the `gen > 0` guard; let modulo arithmetic handle gen == 0 naturally.
// Generation 0 only fires if migration_interval == 1, which is intentional.
if self.island_config.migration_interval > 0
    && gen % self.island_config.migration_interval == 0
{
    migrate(&mut self.islands, &self.island_config, problem_solving)?;
    // ...
}
```
If generation-0 migration is intentionally undesired, use `gen > 0 && (gen + 1) % migration_interval == 0` and document why. The current guard is inconsistent with the documentation comment which says "every N generations."

---

### CR-02: `IslandGa::evolve_islands_one_generation` uses `rayon::prelude::*` unconditionally — WASM compilation failure

**File:** `src/engines/island/mod.rs:493`

**Issue:** The method body contains `use rayon::prelude::*;` and calls `self.islands.par_iter_mut()` (line 524) without any `#[cfg(not(target_arch = "wasm32"))]` gate. CLAUDE.md mandates WASM compatibility and CI enforces it via `.github/workflows/wasm-check.yml`. This will cause a compilation error on `wasm32-unknown-unknown` because rayon is unavailable there.

The project pattern for this case is documented in CLAUDE.md:
```rust
#[cfg(not(target_arch = "wasm32"))]
let results: Vec<_> = items.par_iter().map(...).collect();
#[cfg(target_arch = "wasm32")]
let results: Vec<_> = items.iter().map(...).collect();
```

`src/engines/island/nsga2.rs` has the same unconditional `use rayon::prelude::*` at line 58 and `par_iter_mut` at line 368 and `into_par_iter` at line 224.

**Fix:** Wrap every `par_iter_mut` / `into_par_iter` call in `evolve_islands_one_generation` and `initialize_islands` in `cfg` gates, providing a sequential fallback for `wasm32`.

---

### CR-03: `limit_reached` uses exact float equality for `Minimization` stopping criterion — silent no-stop on real-valued problems

**File:** `src/engines/ga.rs:2784`

**Issue:** For `ProblemSolving::Minimization`, the function returns `true` only when `chromosome.fitness() == 0.0` (exact float equality). On any real-valued continuous problem (Rastrigin, Rosenbrock, etc.) where the true minimum is 0.0, the GA will never trigger this stop condition due to floating-point rounding. Users who do not set a stagnation or convergence threshold will silently run for all `max_generations` with the appearance of non-termination.

For `FixedFitness` (line 2792), the same exact-equality pattern is used for arbitrary `fitness_target` values, which is equally fragile for `f64` targets.

**Fix:**
```rust
// Minimization: stop when fitness is within epsilon of zero
if (chromosome.fitness() - 0.0).abs() < f64::EPSILON {
    result = true;
    break;
}

// FixedFitness: stop when fitness is within epsilon of target
if (chromosome.fitness() - target).abs() < f64::EPSILON {
    result = true;
    break;
}
```
Document the epsilon comparison in the function-level doc comment. Alternatively, expose a `fitness_epsilon` config field so users control the tolerance.

---

## Warnings

### WR-01: `initialize_with_seeds` dedup loop can exhaust max_attempts with small gene alphabets

**File:** `src/engines/ga.rs:1383-1453`

**Issue:** `max_attempts = fill_count * 10`. When the gene alphabet is small (e.g., binary chromosomes with chromosome_length=4 → only 16 unique individuals) and seed count is high, the retry budget is easily exhausted. The error message suggests "reducing the number of seeds" but does not guide users toward the root cause. The algorithm also re-checks all seed DNAs on every retry attempt, making the worst-case O(max_attempts × seeds.len() × chromosome_length) — potentially expensive for large seed sets.

**Fix:** Increase the multiplier or document the limit in the builder doc comment. Add a more descriptive error message that mentions the alphabet-size constraint: "Population space exhaustion: chromosome_length=N with binary genes allows only 2^N unique individuals."

---

### WR-02: `IslandGa::run` calls `validate` and `initialize` unconditionally on every `run()` call, discarding any existing population

**File:** `src/engines/island/mod.rs:440-441`

**Issue:** `run()` always calls `self.initialize()`, which overwrites `self.islands` (line 393: `self.islands = Vec::with_capacity(num_islands)`). Any population manually set on `self.islands` before calling `run()` is silently discarded. This is different from `Ga::run()` behavior, which respects a pre-set population. If a user pre-populates islands and calls `run()`, their data is silently overwritten without error.

**Fix:** Add a guard:
```rust
if self.islands.is_empty() {
    self.initialize()?;
}
```
Or document explicitly in the `run()` doc comment that it always re-initializes.

---

### WR-03: `crowding_distance.rs` — boundary individuals overwrite already-accumulated distance with `INFINITY`

**File:** `src/engines/nsga2/crowding_distance.rs:45-46`

**Issue:** When processing multiple objectives, the boundary-assignment code unconditionally overwrites `crowding[sorted_indices[0]]` and `crowding[sorted_indices[n - 1]]` with `f64::INFINITY` on every objective iteration (lines 45-46). This is correct behavior because boundary individuals always get infinity, but the interior guard `if crowding[idx].is_finite()` on line 62 silently skips updating any individual that was assigned infinity as a boundary in a previous objective — even if that individual is an interior point for the current objective.

This means an individual that is extreme in objective 0 but interior in objective 1 will not accumulate the contribution from objective 1, understating its crowding distance. The standard NSGA-II algorithm assigns infinity to boundaries and accumulates for interior; the boundary check should be: "is this individual a boundary for THIS objective" (i.e., local sort position 0 or n-1), not a global infinity test.

**Fix:**
```rust
// Interior individuals — skip those that are boundary for THIS objective
for k in 1..(n - 1) {
    let prev = sorted_indices[k - 1];
    let next = sorted_indices[k + 1];
    let idx = sorted_indices[k];
    // Always accumulate — boundary individuals were already set to INFINITY above
    // and further addition has no effect, but interior individuals might have
    // received INFINITY from a different objective, so we must NOT skip them here.
    if crowding[idx].is_finite() {
        crowding[idx] += (obj_col[next] - obj_col[prev]) / range;
    }
}
```
The correct fix is to remove the `if crowding[idx].is_finite()` guard entirely; individuals that were set to infinity as a boundary in a previous objective should remain at infinity. The guard is redundant for true boundaries (infinity + finite = infinity) but incorrectly skips individuals that happened to be a boundary in any earlier objective.

---

### WR-04: `log.rs` — `on_generation_end` unconditionally emits trace-level messages about `limit_reached` that are factually wrong

**File:** `src/observe/observer/log.rs:151-153`

**Issue:** Lines 151-153 emit trace-level log messages "limit reached for minimization" and "limit reached for fixed fitness" unconditionally on every generation end, regardless of whether the limit was actually reached. These are annotated with the comment "Emit both unconditionally at trace level for coverage." Emitting false positive "limit reached" messages every generation (even at trace level) will confuse users who enable trace logging during debugging, suggesting premature termination that never happened.

**Fix:** Remove these unconditional trace lines, or move them inside `on_run_end` where they can be gated on the actual `TerminationCause`:
```rust
fn on_run_end(&self, cause: TerminationCause, _all_stats: &[GenerationStats]) {
    match cause {
        TerminationCause::FitnessTargetReached => {
            log::trace!(target="ga_events", method="limit_reached"; "limit reached");
        }
        _ => {}
    }
}
```

---

### WR-05: `Ga::run_with_callback` re-validates configuration on every call but skips operator-compat check

**File:** `src/engines/ga.rs:1512`

**Issue:** `run_with_callback` calls `ValidatorFactory::validate` at line 1512, but does NOT call `crate::validators::generic_validator::operator_compat_check` (which is only called in `build()`). A user who bypasses `build()` (which is allowed — `run()` does not require `build()` to have been called) and calls `run()` directly will skip the operator compatibility validation. This is a silent quality hole: incompatible operators won't be caught until they produce incorrect results or a runtime error inside the crossover/mutation factory.

**Fix:** Add the operator-compat check to `run_with_callback`:
```rust
crate::validators::generic_validator::operator_compat_check::<U>(&self.configuration)?;
```
Or document that `build()` must always be called and enforce it (return an error if the GA has never been built).

---

### WR-06: `adaptive_penalty` uses `penalty_coefficient * 1.1` but only after `penalty_coefficient` has been initialized to `0.0`, producing permanent zero penalty

**File:** `src/engines/ga.rs:2619-2653`

**Issue:** `penalty_coefficient` is initialized to `0.0` (line 327 default, confirmed in `Default` impl at line 400). The `Adaptive` penalty branch (line 2615) sets `coeff = initial_coefficient` when `penalty_coefficient == 0.0`, which correctly handles the first call. However, the update logic at line 2647 computes `let new_coeff = self.penalty_coefficient * 1.1` — but at this point `self.penalty_coefficient` is still `0.0` (it is only written to here, not read from `initial_coefficient`). Multiplying `0.0 * 1.1 = 0.0` means the penalty coefficient is permanently stuck at zero for the entire run.

```rust
// BUG: self.penalty_coefficient is 0.0 at this point — it was never assigned
// from initial_coefficient before this multiplication.
let new_coeff = self.penalty_coefficient * 1.1;  // = 0.0 always
self.penalty_coefficient = new_coeff;             // still 0.0
```

**Fix:**
```rust
// Initialize penalty_coefficient from initial_coefficient on first use
if self.penalty_coefficient == 0.0 {
    self.penalty_coefficient = initial_coefficient;
}
// Now update based on feasibility tracking
if self.adaptive_penalty_counter > 0 {
    self.penalty_coefficient *= 1.1;
} else if self.adaptive_penalty_counter < 0 {
    self.penalty_coefficient = (self.penalty_coefficient / 1.1).max(0.001);
}
```

---

### WR-07: `IslandNsga2Ga::evolve_islands_one_generation` does not re-evaluate migrated individuals' objectives after Pareto migration

**File:** `src/engines/island/nsga2.rs:359`

**Issue:** After `migrate_pareto` runs (line 340 in `run()`), the migrants are cloned into destination islands with their original `rank` and `crowding_distance` values from the source island. The next call to `rank_and_crowd` (via `Self::rank_and_crowd(island)` inside `evolve_islands_one_generation` at line 414) does reassign these, so ranks are eventually corrected. However, `migrate_pareto` is called at line 340 in `run()`, which is **after** `evolve_islands_one_generation` has already run for that generation. So migrated individuals enter the destination with stale source-island ranks and participate in selection (`binary_tournament`) in the **next** generation before `rank_and_crowd` is re-run for them.

This is a minor correctness issue: for one generation, migration arrivals have ranks from a different population context and may be unfairly selected against or in favor of. The severity is low for large populations but is a latent bug.

**Fix:** After `migrate_pareto`, re-run `rank_and_crowd` on affected destination islands:
```rust
migrate_pareto(&mut self.islands, &self.island_config)?;
// Re-rank all islands after migration to correct stale ranks
self.islands.par_iter_mut().for_each(|island| Self::rank_and_crowd(island));
```

---

## Info

### IN-01: Five reporter files listed in review scope do not exist on disk

**File:** `src/observe/reporter/duration.rs`, `src/observe/reporter/mod.rs`, `src/observe/reporter/noop.rs`, `src/observe/reporter/simple.rs`

**Issue:** These four files were included in the `files` config for this review but do not exist at the specified paths. The `src/observe/reporter/` directory does not exist. This may indicate that the reporter module was never created as part of Phase 25, or it was planned but not implemented. The `src/lib.rs` does not reference a `reporter` module, so there is no compilation failure — but the review scope is incomplete relative to what was specified.

**Fix:** If these files are planned stubs for a future phase, remove them from the Phase 25 scope document. If they should have been created in Phase 25, add the stubs.

---

### IN-02: `src/engines/island/mod.rs` — `evolve_islands_one_generation` shadows the outer variable `idx` with a local binding

**File:** `src/engines/island/mod.rs:527`

**Issue:** The closure parameter `(idx, island)` in `par_iter_mut().enumerate()` (line 526-527) shadows the outer `idx` variable declared at line 548 inside the closure (`let idx_a = group[0]`). Although this does not cause incorrect behavior (Rust resolves the shadowing correctly), it reduces readability and may indicate the author intended `island_index` rather than re-using `idx` as both the island index and the parent index.

**Fix:** Rename the closure parameter to `island_idx` to avoid shadowing:
```rust
.enumerate()
.try_for_each(|(island_idx, island)| {
    let (selection_config, ...) = island_configs[island_idx].clone();
```

---

### IN-03: `src/lib.rs` — `#[path]` attributes for engine modules are non-idiomatic and can break IDE navigation

**File:** `src/lib.rs:269-376`

**Issue:** The restructure uses `#[path = "engines/ga.rs"] pub mod ga;` and similar for all moved modules. While this preserves the public API (`crate::ga`, `crate::island`, etc.), it means the physical file layout does not match the module hierarchy. Most Rust IDEs use the standard `src/module_name.rs` or `src/module_name/mod.rs` convention for module resolution — some tooling (e.g., rust-analyzer's "Go to file" from `use` statements) may navigate to `src/lib.rs` rather than the actual source file.

This is a known trade-off accepted in Phase 25 (the context notes "API paths were preserved via `#[path]` attributes"). No code change is required, but this should be tracked for future refactoring when a proper re-export layer (`pub use`) can be introduced.

**Fix:** Document in `src/lib.rs` why `#[path]` is used and note the planned future transition to `pub use` re-exports. No immediate change required.

---

_Reviewed: 2026-06-10T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
