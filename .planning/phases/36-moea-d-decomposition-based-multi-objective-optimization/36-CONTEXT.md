# Phase 36: MOEA/D — Decomposition-based multi-objective optimization - Context

**Gathered:** 2026-05-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a new `MoeaDGa<U>` engine in `src/engines/moead/` implementing MOEA/D (Zhang & Li 2007). The algorithm decomposes the multi-objective problem into N scalar sub-problems using weight vectors; each sub-problem uses Tchebycheff or PBI scalarization; offspring compete only within a T-nearest-neighbour neighbourhood. The engine reuses the `multi_objective` shared utilities (non_dominated_sort, ParetoIndividual, ObjectiveFn) extracted in Phase 35.

**In scope:**
- `src/engines/moead/` — `MoeaDGa<U>` engine, `MoeaDConfiguration`, `ScalarizationFn` enum, weight vector generation (reusing Das-Dennis simplex lattice from Phase 35), neighbourhood computation, ideal point tracking
- `MoeaDObserver<U>` sub-trait in `src/observe/observer/mod.rs` — generation-level hooks only (mirrors Nsga3Observer pattern)
- `LogObserver` gains an `impl MoeaDObserver<U>` block in `src/observe/observer/log.rs` (debug-level logs on `"moead_events"` target)
- `src/lib.rs` — `pub mod moead` re-export via `#[path]`
- `tests/engines/moead/` — integration tests mirroring NSGA-III/NSGA-II test structure
- `examples/moead_dtlz2.rs` — runnable example (3-objective DTLZ2, mirrors `examples/nsga3_dtlz2.rs`)

**Out of scope:**
- Constraint handling for MOEA/D
- External decomposition strategies beyond Tchebycheff and PBI (e.g., weighted-sum)
- Updating `AllObserver<U>` to include `MoeaDObserver<U>` (defer, same rationale as Phase 35 D-10)
- WASM-specific example for MOEA/D
- `on_new_best` tracking (ambiguous in decomposition-based MOO)
- Sub-problem-level observer hooks (on_neighbour_updated, on_subproblem_update) — generation-level only in this phase

</domain>

<decisions>
## Implementation Decisions

### Return Type / Output API

- **D-01:** `MoeaDGa<U>::run()` returns `Result<ParetoFront<U>, GaError>` — post-hoc non-dominated sorting applied to all N sub-problem representative solutions to extract the Pareto front. Identical return type to `Nsga2Ga<U>::run()` and `Nsga3Ga<U>::run()`. Provides a uniform multi-objective engine API.

### Scalarization Function

- **D-02:** Expose scalarization as a public enum `ScalarizationFn` with variants:
  - `ScalarizationFn::Tchebycheff` — classic MOEA/D scalarization, no parameters
  - `ScalarizationFn::Pbi { theta: f64 }` — penalty-based boundary intersection; `theta` is user-configurable (common values: 1.0–10.0; Zhang & Li use 5.0)
  - Mirrors how `ObjectiveDirection` works in `Nsga2Configuration`.
- **D-03:** `MoeaDConfiguration` exposes `.with_scalarization(ScalarizationFn)`. Default (if not called): `ScalarizationFn::Tchebycheff`. `validate()` does NOT fail when scalarization is not set — the default is safe and well-understood.

### Weight Vectors

- **D-04:** `MoeaDConfiguration` exposes `.with_weight_vectors_auto(p: usize)` — triggers Das-Dennis simplex lattice generation (same generator already written for NSGA-III in Phase 35). With M objectives and subdivision p, generates C(p+M-1, M-1) uniformly spaced points.
- **D-05:** `MoeaDConfiguration` also exposes `.with_weight_vectors(Vec<Vec<f64>>)` for user-supplied custom weight vectors. Library validates at `validate()` time that each inner Vec has length == `num_objectives`.
- **D-06:** If neither `with_weight_vectors_auto` nor `with_weight_vectors` is called, `validate()` returns `GaError::InvalidMoeaDConfiguration` with a descriptive message. (Same fail-fast stance as NSGA-III reference points — weight vectors are mandatory.)
- **D-07:** Auto and custom weight vectors are mutually exclusive; last builder call wins.

### Neighbourhood

- **D-08:** `MoeaDConfiguration` exposes `.with_neighborhood_size(t: usize)`. Default: T = 20 (Zhang & Li 2007 baseline for populations of 100+). `validate()` passes with the default — no explicit call required.
- **D-09:** Include `with_max_neighbor_replacements(nr: usize)` with default `nr = 2` (Zhang & Li 2007). Limits how many neighbours each offspring can replace per generation — canonical MOEA/D parameter, prevents premature convergence via over-exploitation. `validate()` passes with the default.

### Observer

- **D-10:** Create `MoeaDObserver<U>` sub-trait in `src/observe/observer/mod.rs` alongside `Nsga2Observer<U>` and `Nsga3Observer<U>`. Generation-level hooks only, mirroring `Nsga3Observer` exactly:
  - `fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {}`
  - `fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize) {}`
  - All methods have default no-op implementations.
- **D-11:** `MoeaDGa<U>` stores `Option<Arc<dyn MoeaDObserver<U> + Send + Sync>>` — zero overhead when `None`. Same `with_observer()` + `notify()` pattern as NSGA-II and NSGA-III.
- **D-12:** `LogObserver` (`src/observe/observer/log.rs`) MUST implement `MoeaDObserver<U>` in this phase. Debug-level log messages on a `"moead_events"` target. Mirrors the existing `impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver` block (Phase 35 D-14).
- **D-13:** `AllObserver<U>` is NOT updated in this phase to include `MoeaDObserver<U>` — avoids breaking existing `AllObserver` implementors. Deferred (same rationale as Phase 35 D-10).

### Claude's Discretion

- Internal neighbourhood computation: precompute T nearest reference-point neighbours at initialisation time (Euclidean distance in weight-vector space) — store as `Vec<Vec<usize>>` indexed by sub-problem.
- Ideal point update strategy: update incrementally after each offspring evaluation (update only changed components).
- WASM cfg-gating: apply `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` to all `Instant::now()` and `par_iter()` call sites in `MoeaDGa` (mandatory — CLAUDE.md constraint).
- Internal normalisation for PBI: use the current ideal point to shift objectives before computing the PBI value; no explicit nadir tracking in Phase 36.
- Example DTLZ2 setup details (population size, number of generations, subdivision p).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### NSGA-III Engine (primary pattern source for Phase 36)
- `src/engines/nsga3/mod.rs` — Full engine pattern for a reference-point-based MOO engine: weight/reference-point storage, with_observer/notify() pattern, run() structure returning ParetoFront<U>, WASM cfg-gating
- `src/engines/nsga3/configuration.rs` — Builder pattern with `with_reference_points_auto(p)` and `with_reference_points(custom)` — D-04/D-05/D-06/D-07 mirror this exactly for weight vectors
- `src/engines/nsga3/` — Das-Dennis simplex lattice generator — REUSE this in MoeaDConfiguration for weight vector auto-generation (do not duplicate)

### Shared Multi-Objective Utilities (from Phase 35)
- `src/engines/multi_objective/non_dominated_sort.rs` — `non_dominated_sort_with_directions()` — post-hoc sort applied to all N sub-problem solutions in `run()` (per D-01)
- `src/engines/multi_objective/pareto.rs` — `ParetoIndividual<U>`, `ParetoFront<U>` types; used for the return value and for wrapping each sub-problem representative
- `src/engines/multi_objective/mod.rs` — ObjectiveFn type alias and module exports

### NSGA-II Engine (secondary pattern source)
- `src/engines/nsga2/mod.rs` — Baseline engine pattern: objective_fns as `Arc<ObjectiveFn<U::Gene>>`, run() structure, crowding distance (NOT reused in MOEA/D but good structural reference)
- `src/engines/nsga2/configuration.rs` — `ObjectiveDirection` enum pattern — `ScalarizationFn` enum mirrors this style

### Observer Infrastructure
- `src/observe/observer/mod.rs` lines 150–187 — `Nsga2Observer<U>` and `Nsga3Observer<U>` trait definitions: hook signatures, default no-op pattern, Send+Sync supertraits. **MoeaDObserver<U> goes in the same file, mirrors Nsga3Observer exactly.**
- `src/observe/observer/log.rs` — `impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver` block — exact pattern to mirror for `impl MoeaDObserver<U> for LogObserver` (per D-12)

### Module Placement Pattern
- `src/lib.rs` lines 109–110 — `#[path = "engines/nsga3/mod.rs"] pub mod nsga3;` — replicate for `moead`

### Tests and Example Patterns
- `tests/engines/nsga3/` — Integration test structure for the NSGA-III engine (mirror for moead tests)
- `examples/nsga3_dtlz2.rs` — Example structure to mirror for `examples/moead_dtlz2.rs`

### Requirements and Issue
- `.planning/ROADMAP.md` §Phase 36 — Goal: "Users can run MOEA/D with configurable weight vectors and either Tchebycheff or PBI scalarisation; each sub-problem maintains a neighbourhood of similar weight vectors and offspring compete only within that neighbourhood"
- Issue #204 — Original feature request (MOO-02 requirement)

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Mandatory cfg-gating rules for `Instant::now()` and `par_iter()`
- `src/engines/nsga3/mod.rs` — Recent cfg-gating applied in Phase 35; exact pattern to copy

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/nsga3/` Das-Dennis lattice generator — weight vector auto-generation for MOEA/D (D-04) should call this directly, not duplicate it
- `src/engines/multi_objective/non_dominated_sort.rs::non_dominated_sort_with_directions()` — post-hoc Pareto sort for the run() return value (D-01)
- `src/engines/multi_objective/pareto.rs::ParetoIndividual<U>` — wraps each sub-problem's representative solution with objectives vector and rank
- `src/observe/observer/mod.rs` — `MoeaDObserver<U>` gets added here, below `Nsga3Observer<U>`
- `src/observe/observer/log.rs` — `impl MoeaDObserver<U> for LogObserver` gets added below the `impl Nsga3Observer<U> for LogObserver` block
- `crate::rng::make_rng()` — used in MoeaDGa for crossover/mutation

### Established Patterns
- `with_observer()` + `fn notify<F: FnOnce(&dyn ObserverTrait)>()` inline dispatch — zero-cost when None; copy from nsga3/mod.rs
- `#[path]` in lib.rs for directory restructure — proven non-breaking; use for `pub mod moead`
- `pub use` re-exports (nsga2 re-exports from multi_objective) — not needed for moead; moead consumes multi_objective directly
- Config builder methods return `Self` (fluent) — mandatory for MoeaDConfiguration
- `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` — add to MoeaDConfiguration and ScalarizationFn enum variants
- `GaError::InvalidMoeaDConfiguration` — new variant for validate() failures (mirrors InvalidNsga3Configuration)

### Integration Points
- `src/lib.rs` — add `pub mod moead` re-export
- `src/observe/observer/mod.rs` — add `MoeaDObserver<U>` trait definition
- `src/observe/observer/log.rs` — add `impl<U: ChromosomeT> MoeaDObserver<U> for LogObserver`
- `src/error.rs` — add `InvalidMoeaDConfiguration(String)` variant to `GaError`
- `src/lib.rs::observer` re-export — add `MoeaDObserver` to public observer exports

</code_context>

<specifics>
## Specific Ideas

- MOEA/D weight-vector neighbourhood: precompute at initialisation by sorting all weight vector pairs by Euclidean distance and retaining the T closest (including self). Store as `Vec<Vec<usize>>` — avoids recomputation per generation.
- Tchebycheff scalarization: `g_tch(f, w, z*) = max_i { w_i * |f_i - z*_i| }`. Minimise over all sub-problems.
- PBI scalarization: `g_pbi(f, w, z*, theta) = d1 + theta * d2` where d1 is distance to ideal along weight vector direction, d2 is perpendicular distance. theta default = 5.0 per Zhang & Li.
- Ideal point z*: updated after each offspring evaluation — `z*_i = min(z*_i, f_i(offspring))` for each objective i.
- Example: DTLZ2 3-objective (sphere), population size 91 (C(12,2) with p=10 for 3 objectives), 300 generations.
- Neighbourhood update loop: for each sub-problem i, generate one offspring (via standard crossover/mutation from neighbourhood), evaluate objectives, update ideal point, replace neighbours where offspring dominates (by scalarized value), capped at `max_neighbor_replacements` = 2.

</specifics>

<deferred>
## Deferred Ideas

- Two-layer weight vectors for M > 5 objectives (outer + inner layer) — not needed for the initial implementation
- Constraint handling for MOEA/D — follow-up phase (mirrors NSGA-III constraint deferral)
- `AllObserver<U>` updated to include `MoeaDObserver<U>` — deferred to avoid breaking existing implementors
- Sub-problem-level observer hooks (`on_neighbour_updated`, `on_subproblem_update`) — generation-level only in Phase 36; add in a follow-up if detailed introspection is needed
- Weighted-sum scalarization — not requested; Tchebycheff and PBI are sufficient for the canonical implementation
- Adaptive weight vector adjustment — advanced; defer to future phases

</deferred>

---

*Phase: 36-moea-d-decomposition-based-multi-objective-optimization*
*Context gathered: 2026-05-09*
