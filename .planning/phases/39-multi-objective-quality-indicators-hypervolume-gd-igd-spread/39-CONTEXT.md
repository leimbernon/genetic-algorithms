# Phase 39: Multi-objective quality indicators — Hypervolume, GD, IGD, Spread - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a shared `src/engines/multi_objective/indicators/` directory exposing Hypervolume, Generational Distance, Inverted Generational Distance, and Spread as pure functions. Phase 38 (SMS-EMOA, IBEA) depends on this module — SMS-EMOA consumes hypervolume-contribution for steady-state removal; IBEA may use epsilon-indicator computations. The functions are also callable standalone for post-run Pareto-front analysis.

**In scope:**
- `src/engines/multi_objective/indicators/` directory — `mod.rs` (re-exports), `hypervolume.rs` (2-objective exact Lebesgue-based), `generational_distance.rs`, `inverted_generational_distance.rs`, `spread.rs`
- `src/engines/multi_objective/mod.rs` — `pub mod indicators;`
- `src/lib.rs` — indicators accessible via `crate::multi_objective::indicators` (no new re-export needed — `pub mod multi_objective` already exported)
- `tests/engines/multi_objective/indicators/` — integration tests with analytically-verified expected values against ZDT/DTLZ reference fronts (inline test data, no hardcoded library constants)
- Each function returns `Result<f64, GaError>` with appropriate error variants for invalid inputs

**Out of scope:**
- 3+ objective hypervolume (exact WFG or Monte Carlo)
- Hardcoded Pareto-front reference sets in the library
- Epsilon-indicator function (I_eps+) — that's Phase 38's domain
- Indicator-based engine integration (SMS-EMOA/IBEA call these functions, but wiring is Phase 38)
- `GaObserver` hooks or observer integration — indicators are pure functions, no lifecycle to observe
- `AllObserver<U>` updates
- WASM-specific examples (functions are pure math — WASM-compatible by construction)

</domain>

<decisions>
## Implementation Decisions

### Module Architecture

- **D-01:** Indicators live in `src/engines/multi_objective/indicators/` — a directory with one file per indicator + a `mod.rs` that re-exports all four public functions. This follows the shared-MOO-utility pattern established in Phase 35 (where `multi_objective/` was extracted from nsga2). All multi-objective engines import via `crate::multi_objective::indicators::*`. The roadmap's `nsga2/indicators.rs` suggestion is superseded — nsga2 can add a re-export if backward compat is needed, but the canonical home is `multi_objective`.

### API Design

- **D-02:** All four indicators are pure functions:
  - `fn hypervolume(points: &[Vec<f64>], reference_point: &[f64]) -> Result<f64, GaError>`
  - `fn generational_distance(approx_front: &[Vec<f64>], true_front: &[Vec<f64>], power: f64) -> Result<f64, GaError>`
  - `fn inverted_generational_distance(approx_front: &[Vec<f64>], true_front: &[Vec<f64>], power: f64) -> Result<f64, GaError>`
  - `fn spread(approx_front: &[Vec<f64>], extreme_points: &[Vec<f64>]) -> Result<f64, GaError>`

  No structs, no pre-computation. Stateless. Phase 38 engines call these directly in their loop — if hot-loop optimization is needed, it's Phase 38's responsibility.

### Hypervolume Scope

- **D-03:** 2-objective exact only. Algorithm: sort points by first objective, then accumulate rectangular slices (Lebesgue measure). O(n log n) complexity. Returns `GaError::InvalidIndicatorConfiguration` if objectives != 2 or if reference point is not strictly dominated by all points.

### Reference Fronts

- **D-04:** No hardcoded Pareto-front data in the library. GD and IGD require the true front as a `&[Vec<f64>]` parameter — user provides it. Tests define analytically-known ZDT/DTLZ reference fronts inline in test code only, not as library constants.

### Error Handling

- **D-05:** All indicator functions return `Result<f64, GaError>` using the error variant pattern. New `GaError` variant: `InvalidIndicatorConfiguration(String)` — covers empty point sets, dimension mismatches, invalid reference points, non-positive powers.

### Claude's Discretion

- Exact algorithm selection for each indicator: 2D hypervolume uses sort-then-sweep (Lebesgue measure). GD/IGD use standard Euclidean-distance-to-nearest formulas. Spread uses the Deb et al. 2002 definition (extreme-point distance + uniformity measure).
- Internal validation helpers for common checks (empty sets, dimension consistency, reference point dominance) — researcher/planner decides factoring.
- `power` parameter on GD/IGD defaults to 2.0 (standard p=2 Euclidean norm).
- WASM: no `Instant` or `rayon` needed — pure functions compile for wasm32 without cfg-gating.
- No new feature flags — indicators are always available.

### Established Patterns (carried forward)

- `#[path]` re-exports for any backward-compat nsga2::indicators paths if needed
- Tests in `tests/engines/multi_objective/indicators/` directory
- `GaError` variant naming: `InvalidIndicatorConfiguration`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Shared MOO Module (home for this code)
- `src/engines/multi_objective/mod.rs` — Existing shared module: `ObjectiveDirection` enum, `non_dominated_sort` submodule, `pareto` submodule, `ObjectiveFn` type alias. Indicators extend this module.
- `src/engines/multi_objective/pareto.rs` — `ParetoFront<U>` and `ParetoIndividual<U>`: the data types users extract from engines before passing to indicators
- `src/engines/multi_objective/non_dominated_sort.rs` — Non-dominated sort used by indicator functions for filtering dominated points

### Prior Phase Context
- `.planning/phases/38-indicator-based-moeas-sms-emoa-and-ibea/38-CONTEXT.md` — Phase 38 consumer: D-01 mandates Phase 39 build first; SMS-EMOA needs hypervolume contribution; IBEA needs epsilon-indicator
- `.planning/phases/37-spea2-strength-pareto-evolutionary-algorithm/37-CONTEXT.md` — SPEA2 engine pattern (most recent completed MOEA)
- `.planning/phases/35-nsga-iii-for-many-objective-optimization/35-CONTEXT.md` — Phase 35: multi_objective module extraction, Das-Dennis generator

### Requirements
- `.planning/REQUIREMENTS.md` — MOO-05 (quality indicators), MOO-04 (SMS-EMOA/IBEA, consumer of this module)

### Existing Engine Patterns
- `src/engines/nsga2/mod.rs` — NSGA-II: how ParetoFront is produced from run(), consumed by indicator users
- `src/engines/nsga3/mod.rs` — NSGA-III: multi-objective reference point patterns
- `src/error.rs` — `GaError` enum: existing variant naming convention

### Codebase Maps
- `.planning/codebase/ARCHITECTURE.md` — Overall architecture: trait-driven GA library with pluggable operators
- `.planning/codebase/STRUCTURE.md` — Directory layout and where new code goes

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ParetoFront<U>`** (`multi_objective/pareto.rs`): Already stores objective values per individual — indicator users will extract `Vec<Vec<f64>>` from this type before calling indicator functions
- **`non_dominated_sort`** (`multi_objective/non_dominated_sort.rs`): Used by GD/IGD to pre-process the approximate front — indicators assume non-dominated input
- **`ObjectiveDirection`** (`multi_objective/mod.rs`): Minimize/Maximize enum — needed for hypervolume reference point validation and Spread extreme-point handling

### Established Patterns
- Shared MOO utilities live in `src/engines/multi_objective/` — one directory, one `mod.rs` re-export, submodules for logical groupings
- `#[path]` re-exports in `src/lib.rs` for backward compatibility
- `GaError` enum follows `Invalid{Feature}Configuration(String)` naming
- Integration tests mirror `src/` structure in `tests/engines/`

### Integration Points
- `src/engines/multi_objective/mod.rs` — add `pub mod indicators;` line
- `src/error.rs` — add `InvalidIndicatorConfiguration(String)` variant
- Phase 38 engines import `crate::multi_objective::indicators::hypervolume` directly
</code_context>

<specifics>
## Specific Ideas

No specific references beyond the standard definitions:
- Hypervolume: Zitzler & Thiele 1999, 2D Lebesgue measure (sort by f1, accumulate f2 rectangles)
- GD: Van Veldhuizen & Lamont 2000, average Euclidean distance from each approx point to nearest true-front point
- IGD: Coello Coello & Cruz Cortés 2005, average distance from each true-front point to nearest approx point
- Spread: Deb et al. 2002 — extreme-point distance + uniformity of distribution metric

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 39-multi-objective-quality-indicators-hypervolume-gd-igd-spread*
*Context gathered: 2026-05-10*
