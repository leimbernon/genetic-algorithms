# Phase 35: NSGA-III for many-objective optimization - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a new `Nsga3Ga<U>` engine in `src/engines/nsga3/` that implements NSGA-III: reference-point based many-objective optimization. The algorithm uses the same non-dominated sorting as NSGA-II but replaces crowding-distance survivor selection with reference-point niche association (Das-Dennis simplex lattice or user-supplied points).

As part of this phase, the shared utilities currently in `src/engines/nsga2/` (`non_dominated_sort`, `ParetoIndividual`, `ParetoFront`) are extracted to a new `src/engines/multi_objective/` module. The `nsga2` module re-exports from there for backward compatibility.

**In scope:**
- `src/engines/multi_objective/` — extract non_dominated_sort.rs, pareto.rs from nsga2; expose as `pub mod multi_objective` in lib.rs; nsga2 keeps pub use re-exports
- `src/engines/nsga3/` — Nsga3Ga<U> engine, Nsga3Configuration, Das-Dennis reference point generator
- `Nsga3Observer<U>` sub-trait in `src/observe/observer/mod.rs` — mirrors Nsga2Observer pattern, NSGA-III algorithm-specific hooks only (on_pareto_front_assigned, on_non_dominated_sort_complete), no on_reference_association hook
- `tests/engines/nsga3/` — integration tests mirroring NSGA-II test structure
- `examples/nsga3_dtlz2.rs` — runnable example for 3-objective DTLZ2

**Out of scope:**
- Constraint handling for NSGA-III (no constraint_fns in Phase 35)
- Two-layer Das-Dennis reference points for very large M (can be added later)
- Updating AllObserver<U> to include Nsga3Observer<U> — impacts existing implementors; defer to a follow-up phase
- WASM-specific example for NSGA-III

</domain>

<decisions>
## Implementation Decisions

### Code Sharing — Shared Multi-Objective Module

- **D-01:** Extract `non_dominated_sort.rs` and `pareto.rs` from `src/engines/nsga2/` to a new `src/engines/multi_objective/` module. These utilities are shared across NSGA-II, NSGA-III, and future phases 36-38.
- **D-02:** Expose the new module as `pub mod multi_objective` in `src/lib.rs` via `#[path = "engines/multi_objective/mod.rs"]` — consistent with the v2.3.0 restructure pattern.
- **D-03:** `nsga2` keeps `pub use crate::multi_objective::pareto::*` and `pub use crate::multi_objective::non_dominated_sort::*` re-exports so existing `genetic_algorithms::nsga2::pareto::ParetoIndividual` paths continue to work. Zero breaking change.

### Reference Point API

- **D-04:** `Nsga3Configuration` exposes `with_reference_points_auto(p: usize)` — triggers Das-Dennis simplex lattice generation with subdivision count `p`. With `M` objectives, generates `C(p+M-1, M-1)` uniformly spaced points on the unit hyperplane.
- **D-05:** `Nsga3Configuration` also exposes `with_reference_points(Vec<Vec<f64>>)` for user-supplied custom reference points. Library validates at `validate()` time that each inner Vec has length == `num_objectives`.
- **D-06:** If neither `with_reference_points_auto` nor `with_reference_points` is called, `validate()` returns `GaError::ConfigurationError` with a descriptive message. Fail-fast before any computation starts — consistent with how Nsga2Configuration validates `num_objectives`.
- **D-07:** Auto and custom are mutually exclusive. The last builder call wins (same pattern as other config fields).

### Observer Pattern

- **D-08:** Create `Nsga3Observer<U>` sub-trait in `src/observe/observer/mod.rs` alongside `Nsga2Observer<U>`. Mirrors the Nsga2Observer pattern. Algorithm-specific hooks only — does NOT add an `on_reference_association` hook (user decision: basic lifecycle only).
  - `on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize)`
  - `on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64)`
  - All methods have default no-op implementations.
- **D-09:** `Nsga3Ga<U>` stores `Option<Arc<dyn Nsga3Observer<U> + Send + Sync>>` — zero overhead when `None`. Same `with_observer()` + `notify()` pattern as NSGA-II.
- **D-10:** `AllObserver<U>` is NOT updated in this phase to include `Nsga3Observer<U>` — avoids a breaking change on existing `AllObserver` implementors. Deferred.

### Return Type / Output API

- **D-11:** `Nsga3Ga<U>::run()` returns `Result<ParetoFront<U>, GaError>` — identical signature to `Nsga2Ga<U>::run()`. `ParetoFront<U>` is a type alias from `multi_objective`, re-exported by nsga2 and nsga3.
- **D-12:** `on_new_best` (from `GaObserver<U>`) tracks the individual in the first Pareto front with the best value on objective 0. Same semantics as NSGA-II — predictable, consistent across engines.
- **D-13:** `Nsga3Ga<U>` does NOT carry `GaObserver<U>` — it carries `Nsga3Observer<U>` (which covers the NSGA-III lifecycle hooks). Basic lifecycle hooks (on_start, on_finish, etc.) are exposed through `Nsga3Observer<U>` or through a separate `GaObserver<U>` field — **Claude's discretion:** pick whichever pattern requires the least duplication and is closest to how Nsga2Ga handles this.

### Claude's Discretion

- Whether `Nsga3Ga<U>` holds a single observer field (`Nsga3Observer<U>`) or separate `GaObserver<U>` + `Nsga3Observer<U>` fields — choose the pattern that matches Nsga2Ga most closely.
- Das-Dennis generator implementation details (recursive enumeration vs iterative, internal function name).
- Reference point normalization: store raw points as-is or normalize to unit hyperplane on construction.
- WASM cfg-gating: apply same `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` pattern to all `Instant::now()` and `par_iter()` call sites in Nsga3Ga (mandatory — CLAUDE.md constraint).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### NSGA-II Engine (primary pattern source)
- `src/engines/nsga2/mod.rs` — Full engine pattern: ParetoIndividual wrapper, objective_fns as `Arc<ObjectiveFn<U::Gene>>`, with_observer / notify() pattern, run() structure, WASM cfg gating
- `src/engines/nsga2/configuration.rs` — Builder pattern, `effective_directions()`, `ObjectiveDirection` enum — Nsga3Configuration mirrors this structure
- `src/engines/nsga2/non_dominated_sort.rs` — **Will move to multi_objective** — shared utility; NSGA-III reuses rank 0 (first front) selection
- `src/engines/nsga2/pareto.rs` — **Will move to multi_objective** — `ParetoIndividual<U>`, `ParetoFront<U>` types; dominates / dominates_with_directions predicates
- `src/engines/nsga2/crowding_distance.rs` — Stays in nsga2 only; replaced by reference-point association in nsga3

### Observer Infrastructure
- `src/observe/observer/mod.rs` lines 150–167 — `Nsga2Observer<U>` trait definition: hook signatures, default no-op pattern, Send+Sync supertraits. **Nsga3Observer<U> goes in the same file, mirrors this exactly.**
- `src/observe/observer/mod.rs` lines 177–187 — `AllObserver<U>` blanket impl — DO NOT modify in Phase 35 (deferred)

### Module Placement Pattern
- `src/lib.rs` lines 109–110 — `#[path = "engines/nsga2/mod.rs"] pub mod nsga2;` pattern — replicate for `nsga3` and `multi_objective`

### NSGA-II Tests (pattern to mirror)
- `tests/engines/nsga2/test_nsga2.rs` — Integration test structure for the engine
- `tests/engines/nsga2/test_nsga2_configuration.rs` — Configuration validation tests
- `tests/engines/nsga2/test_non_dominated_sort.rs` — Utility tests (move to multi_objective tests)
- `tests/engines/nsga2/test_pareto.rs` — Pareto utility tests (move to multi_objective tests)

### Example Pattern
- `examples/nsga2_zdt1.rs` — Example structure to mirror for `examples/nsga3_dtlz2.rs`

### Requirements and Issue
- `.planning/ROADMAP.md` §Phase 35 — Goal: "Users can run NSGA-III on problems with 3+ objectives; reference points are auto-generated (Das-Dennis simplex lattice) or user-supplied, and the algorithm selects survivors via reference-point association rather than crowding distance"
- Issue #203 — Original feature request (MOO-01 requirement)

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Mandatory cfg-gating rules for Instant::now() and par_iter()
- `src/engines/nsga2/mod.rs` — Recent cfg-gating applied in Phase 34; exact pattern to copy

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/nsga2/non_dominated_sort.rs::non_dominated_sort_with_directions()` — NSGA-III's first step is identical: sort all individuals into fronts by non-dominance. Will be in multi_objective after extraction.
- `src/engines/nsga2/pareto.rs::ParetoIndividual<U>` — NSGA-III wraps each chromosome the same way: objectives vector, constraint violation, rank. Will be in multi_objective.
- `src/observe/observer/mod.rs` — `Nsga3Observer<U>` gets added here, below the existing `Nsga2Observer<U>` definition.
- `crate::rng::make_rng()` — Das-Dennis generator doesn't need randomness, but nsga3's crossover/mutation step does.
- `src/engines/nsga2/mod.rs::ObjectiveFn<G>` type alias — move to `multi_objective` as shared type alias; both engines use it.

### Established Patterns
- `with_observer()` + `fn notify<F: FnOnce(&dyn ObserverTrait)>()` inline dispatch — zero-cost when observer is None; copy exactly from nsga2/mod.rs
- `#[path]` in lib.rs for directory restructure — proven non-breaking in v2.3.0 and used by nsga2 already
- `pub use` re-exports in nsga2 for backward compat — same pattern as v2.3.0 src/ restructure
- Config builder methods return `Self` (fluent) — all existing configs follow this; Nsga3Configuration must too
- `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` — add to Nsga3Configuration and enum variants

### Integration Points
- `src/lib.rs` — add `pub mod multi_objective` and `pub mod nsga3` re-exports
- `src/engines/nsga2/mod.rs` — add `pub use crate::multi_objective::pareto::*` and `pub use crate::multi_objective::non_dominated_sort::*`
- `src/observe/observer/mod.rs` — add `Nsga3Observer<U>` trait definition
- `src/lib.rs::observer` re-export — add `Nsga3Observer` to the public observer exports

</code_context>

<specifics>
## Specific Ideas

- Das-Dennis simplex lattice: for `M` objectives and subdivision `p`, enumerate all non-negative integer vectors `(n_1, ..., n_M)` with `n_1 + ... + n_M = p`, then normalize each to `(n_1/p, ..., n_M/p)`. Standard recursive enumeration produces exactly `C(p+M-1, M-1)` points.
- Reference-point association algorithm (from Deb & Jain 2014): normalize objective vectors to [0,1] hyperplane using ideal and intercept points; associate each individual to the nearest reference point by perpendicular distance; in the last front, select individuals from under-populated niches first, breaking ties randomly.
- The example should use DTLZ2 with 3 objectives (`f_1^2 + f_2^2 + f_3^2 = 1` sphere) — standard NSGA-III benchmark, simple to implement inline.

</specifics>

<deferred>
## Deferred Ideas

- Two-layer Das-Dennis reference points for M > 5 objectives (outer layer + inner layer) — not needed for the initial implementation; can be added as `with_reference_points_two_layer(p1, p2)` later
- Constraint handling for NSGA-III — Nsga2Ga already has constraint_fns; NSGA-III can add it in a follow-up
- Updating `AllObserver<U>` to include `Nsga3Observer<U>` — deferred to avoid breaking existing AllObserver implementors
- Adaptive normalization (online ideal/nadir point estimation) — advanced feature, Phase 35 uses batch normalization per generation

</deferred>

---

*Phase: 35-nsga-iii-for-many-objective-optimization*
*Context gathered: 2026-05-07*
