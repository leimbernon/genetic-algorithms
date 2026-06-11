# Phase 37: SPEA2 — Strength Pareto Evolutionary Algorithm 2 - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a new `Spea2Ga<U>` engine in `src/engines/spea2/` implementing SPEA2 (Zitzler, Laumanns & Thiele 2001). SPEA2 maintains a fixed-size external archive; fitness is computed from raw strength (domination count) + density (k-nearest-neighbour distance); archive truncation uses iterative nearest-neighbour removal. The engine reuses `multi_objective` shared utilities (non-dominated sort, ParetoIndividual, ObjectiveFn) and follows the established multi-objective engine pattern from NSGA-II, NSGA-III, and MOEA/D.

**In scope:**
- `src/engines/spea2/` — `Spea2Ga<U>` engine, `Spea2Configuration`, archive management, strength + density fitness assignment, environmental selection with truncation
- `Spea2Observer<U>` sub-trait in `src/observe/observer/mod.rs` — two generation-level hooks (fitness assigned, archive updated)
- `LogObserver` gains an `impl Spea2Observer<U>` block in `src/observe/observer/log.rs` (debug-level logs on `"spea2_events"` target)
- `src/lib.rs` — `pub mod spea2` re-export via `#[path]`
- `GaError::InvalidSpea2Configuration(String)` variant
- `tests/engines/spea2/` — integration tests mirroring NSGA-III/MOEA/D test structure
- `examples/spea2_zdt1.rs` — runnable example (2-objective ZDT1, canonical SPEA2 benchmark)

**Out of scope:**
- Archive size adaptation (fixed size only — per the original SPEA2 paper)
- Alternative truncation strategies (nearest-neighbour removal only)
- Constraint handling for SPEA2
- `AllObserver<U>` update to include `Spea2Observer<U>` (deferred — same rationale as Phase 35 D-10 and Phase 36 D-13)
- WASM-specific example

</domain>

<decisions>
## Implementation Decisions

### Archive Sizing

- **D-01:** `Spea2Configuration` exposes `.with_archive_size(usize)`. Default: archive size equals population size (canonical SPEA2). `validate()` rejects `archive_size > population_size` or `archive_size == 0` as `InvalidSpea2Configuration`.

### Density k Parameter

- **D-02:** k-nearest-neighbour parameter for density estimation is auto-calculated as `k = floor(sqrt(N_pop + N_archive))` — matches the SPEA2 paper exactly. No configuration method — users cannot override.

### Truncation Strategy

- **D-03:** Archive truncation uses the exact SPEA2 algorithm: iteratively remove the individual with the smallest Euclidean distance to its nearest neighbour in objective space, recomputing distances after each removal. No alternative strategies — this is the canonical SPEA2 algorithm.

### Observer Hooks

- **D-04:** `Spea2Observer<U>` sub-trait exposes two generation-level hooks:
  - `fn on_fitness_assigned(&self, generation: usize, duration_ms: f64, pop_size: usize, archive_size: usize) {}` — after raw strength R(i) + density D(i) assignment to all individuals
  - `fn on_archive_updated(&self, generation: usize, archive_size: usize, non_dominated_count: usize) {}` — after environmental selection + archive truncation
  - All methods have default no-op implementations. `Send + Sync` supertraits.
  - Mirrors `Nsga3Observer<U>` and `MoeaDObserver<U>` hook structure (two hooks, generation-level).
- **D-05:** `Spea2Ga<U>` stores `Option<Arc<dyn Spea2Observer<U> + Send + Sync>>` — zero overhead when `None`. Same `with_observer()` + `notify()` pattern as all prior multi-objective engines.
- **D-06:** `LogObserver` MUST implement `Spea2Observer<U>` in this phase. Debug-level log messages on `"spea2_events"` target. Mirrors existing observer impl blocks for NSGA-III and MOEA/D.
- **D-07:** `AllObserver<U>` is NOT updated in this phase to include `Spea2Observer<U>` — avoids breaking existing implementors (same rationale as Phase 35 D-10 and Phase 36 D-13).

### Example Benchmark

- **D-08:** User-facing example is `examples/spea2_zdt1.rs` — ZDT1 (2-objective, 30 variables). ZDT1 is the canonical SPEA2 benchmark from the original Zitzler et al. 2001 paper. Mirrors `examples/nsga2_zdt1.rs` structure with SPEA2 adaptations.

### Return Type

- **D-09:** `Spea2Ga<U>::run()` returns `Result<ParetoFront<U>, GaError>` — identical return type to all existing multi-objective engines (Nsga2Ga, Nsga3Ga, MoeaDGa). The Pareto front is extracted from the final archive via non-dominated sorting.

### Claude's Discretion

- Internal archive management: maintain archive as a `Vec<U>` alongside the population; after fitness assignment, copy non-dominated individuals to archive, then truncate if over capacity
- Mating selection: binary tournament from the archive (standard SPEA2)
- SPEA2 fitness algorithm: combine population + archive, compute strength S(i) = count of dominated individuals, raw fitness R(i) = sum of S(j) for all j dominating i, density D(i) = 1/(σ_k + 2)
- WASM cfg-gating: apply `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` to all `Instant::now()` and `par_iter()` call sites (mandatory — CLAUDE.md constraint)
- Builder methods return `Self` (fluent pattern)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Engine Pattern (primary source)
- `src/engines/nsga2/mod.rs` — Baseline multi-objective engine: `with_observer()` + `notify()` pattern, `run()` returning `ParetoFront<U>`, WASM cfg-gating
- `src/engines/nsga3/mod.rs` — Newest engine pattern (Phase 35): weight/reference-point storage, ideal point tracking patterns
- `src/engines/moead/mod.rs` — Most recent engine (Phase 36): neighbourhood precomputation, scalarization dispatch, observer wiring — closest structural analog for SPEA2

### Configuration Pattern
- `src/engines/moead/configuration.rs` — `MoeaDConfiguration` builder, `ScalarizationFn` enum — pattern for `Spea2Configuration`
- `src/engines/nsga3/configuration.rs` — Builder pattern with `with_reference_points_auto(p)` — `with_archive_size(N)` mirrors this

### Shared Multi-Objective Utilities
- `src/engines/multi_objective/non_dominated_sort.rs` — `non_dominated_sort_with_directions()` — used in environmental selection and for post-hoc Pareto front extraction
- `src/engines/multi_objective/pareto.rs` — `ParetoIndividual<U>`, `ParetoFront<U>` types — return type for `run()`

### Observer Infrastructure
- `src/observe/observer/mod.rs` — `Nsga2Observer<U>`, `Nsga3Observer<U>`, `MoeaDObserver<U>` trait definitions — `Spea2Observer<U>` goes here
- `src/observe/observer/log.rs` — `impl Nsga3Observer<U> for LogObserver`, `impl MoeaDObserver<U> for LogObserver` — exact pattern for `impl Spea2Observer<U> for LogObserver`

### Module Placement
- `src/lib.rs` lines 109–110 — `#[path = "engines/nsga3/mod.rs"] pub mod nsga3;` — replicate for `spea2`

### Tests and Example Patterns
- `tests/engines/nsga3/` — Integration test structure for multi-objective engines (mirror for spea2 tests)
- `examples/nsga2_zdt1.rs` — Example structure to mirror for `examples/spea2_zdt1.rs`

### Requirements and Issue
- `.planning/ROADMAP.md` §Phase 37 — Goal: "Users can run SPEA2 with a configurable archive size; fitness is computed from raw strength + density (k-nearest-neighbour), and the archive is truncated using the Euclidean crowding criterion"
- Issue #205 — Original feature request (MOO-03 requirement)

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Mandatory cfg-gating rules for `Instant::now()` and `par_iter()`
- `src/engines/moead/mod.rs` — Most recent WASM cfg-gating applied (Phase 36) — exact pattern to copy

### Prior Phase Context
- `.planning/phases/36-moea-d-decomposition-based-multi-objective-optimization/36-CONTEXT.md` — Phase 36 context (MOEA/D) — the most recent multi-objective engine, patterns carry forward
- `.planning/phases/35-nsga-iii-for-many-objective-optimization/35-CONTEXT.md` — Phase 35 context (NSGA-III) — shared multi_objective utilities, observer pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/multi_objective/non_dominated_sort.rs::non_dominated_sort_with_directions()` — used for post-hoc Pareto front extraction from archive + for dominance-based fitness components
- `src/engines/multi_objective/pareto.rs::ParetoIndividual<U>` — wraps each individual with objectives vector and rank
- `src/observe/observer/mod.rs` — `Spea2Observer<U>` gets added here, below `MoeaDObserver<U>`
- `src/observe/observer/log.rs` — `impl Spea2Observer<U> for LogObserver` gets added below the MOEA/D impl block
- `crate::rng::make_rng()` — used in Spea2Ga for tournament selection
- `src/operations/` — standard crossover/mutation operators reused (like NSGA-II, no MOEA/D-specific sub-problem restrictions)

### Established Patterns
- `with_observer()` + `fn notify<F: FnOnce(&dyn ObserverTrait)>()` inline dispatch — zero-cost when `None`; copy from moead/mod.rs or nsga3/mod.rs
- `#[path]` in lib.rs for directory restructure — proven non-breaking; use for `pub mod spea2`
- Config builder methods return `Self` (fluent) — mandatory for `Spea2Configuration`
- `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` — add to `Spea2Configuration`
- `GaError::InvalidSpea2Configuration(String)` — new variant (mirrors `InvalidMoeaDConfiguration`)

### Integration Points
- `src/lib.rs` — add `pub mod spea2` re-export
- `src/observe/observer/mod.rs` — add `Spea2Observer<U>` trait definition
- `src/observe/observer/log.rs` — add `impl<U: ChromosomeT> Spea2Observer<U> for LogObserver`
- `src/error.rs` — add `InvalidSpea2Configuration(String)` variant to `GaError`

### Differences from MOEA/D
- SPEA2 does NOT use weight vectors or neighbourhoods — it uses archive-based environmental selection
- SPEA2 applies crossover/mutation to the full population (not per-sub-problem), then uses archive truncation
- Selection is binary tournament from the archive, not neighbourhood-restricted
- Strength + density fitness replaces scalarization functions

</code_context>

<specifics>
## Specific Ideas

- SPEA2 fitness algorithm (Zitzler et al. 2001, Algorithm 1):
  1. Combine population P and archive A into union set U
  2. Strength S(i) = |{j in U : i dominates j}| — count of individuals that i dominates
  3. Raw fitness R(i) = sum of S(j) for all j in U that dominate i. R(i) = 0 means i is non-dominated
  4. Density D(i) = 1 / (σ_k + 2) where σ_k = distance to k-th nearest neighbour, k = floor(sqrt(|U|))
  5. Final fitness F(i) = R(i) + D(i) — lower is better (fitness is to be minimized)
- Archive update: copy all non-dominated (R(i) = 0) individuals from U to archive; if archive < target size, fill with best (lowest F) dominated individuals; if archive > target size, truncate iteratively
- Truncation: while |archive| > target, find the pair (i, j) with smallest Euclidean distance in objective space, remove the one with smaller distance to its second-nearest neighbour (the one "more crowded")
- ZDT1: f1 = x1, f2 = g * (1 - sqrt(f1/g)), g = 1 + 9*(sum_{i=2}^{n} x_i)/(n-1). 30 variables, each in [0,1]. Pareto front: f2 = 1 - sqrt(f1), f1 in [0,1]
- Example: population size 100, archive size 100, 250 generations (matching nsga2_zdt1.rs scale)

</specifics>

<deferred>
## Deferred Ideas

- Archive size adaptation (dynamic sizing) — stick with fixed-size per the original SPEA2 paper
- Alternative truncation strategies (random, farthest-first) — no user demand yet; keep single-strategy to match prior engine simplicity
- `AllObserver<U>` updated to include `Spea2Observer<U>` — deferred to avoid breaking existing implementors (consistent with Phase 35 and Phase 36 deferrals)
- Alternative density estimators (k-th vs fixed k) — standard squared-root formula used throughout the SPEA2 literature
- DTLZ2 or other 3-objective example — ZDT1 is the canonical SPEA2 benchmark; 3-objective can be added in a follow-up

</deferred>

---

*Phase: 37-spea2-strength-pareto-evolutionary-algorithm*
*Context gathered: 2026-05-10*
