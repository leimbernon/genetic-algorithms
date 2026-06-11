# Phase 36: MOEA/D — Decomposition-based multi-objective optimization - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-09
**Phase:** 36-moea-d-decomposition-based-multi-objective-optimization
**Areas discussed:** Return type, Scalarization enum, Observer hooks, Neighbour replacement cap

---

## Return Type

| Option | Description | Selected |
|--------|-------------|----------|
| `Result<ParetoFront<U>, GaError>` | Post-hoc non-dominated sort on all N sub-problem solutions; consistent with Nsga2Ga and Nsga3Ga API | ✓ |
| `Result<Vec<U>, GaError>` | Raw sub-problem representatives, no sorting; faithful to MOEA/D structure, user sorts manually if needed | |

**User's choice:** `Result<ParetoFront<U>, GaError>` — uniform multi-objective engine API
**Notes:** Consistency with NSGA-II and NSGA-III return type was the deciding factor.

---

## Scalarization Enum

### Question 1: Enum design

| Option | Description | Selected |
|--------|-------------|----------|
| Enum with builder method | `ScalarizationFn::Tchebycheff` \| `ScalarizationFn::Pbi { theta: f64 }`; `.with_scalarization(...)` on config | ✓ |
| Two separate builder methods | `with_tchebycheff()` / `with_pbi(theta: f64)`; no public enum | |

**User's choice:** Enum with builder method — mirrors `ObjectiveDirection` pattern in Nsga2Configuration.

### Question 2: Default

| Option | Description | Selected |
|--------|-------------|----------|
| Tchebycheff (default, validate passes) | Classic, no parameters, well-understood | ✓ |
| No default — validate() fails | Fail-fast, forces explicit choice | |

**User's choice:** Tchebycheff as default — sensible, consistent with literature.

---

## Observer Hooks

### Question 1: Hook granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Generation-level only | `on_non_dominated_sort_complete` + `on_pareto_front_assigned`; mirrors Nsga3Observer exactly | ✓ |
| Add on_ideal_point_updated | Also fires once per generation after ideal point refresh; useful for convergence monitoring | |

**User's choice:** Generation-level only — consistent across all multi-objective engines.

### Question 2: LogObserver implementation

| Option | Description | Selected |
|--------|-------------|----------|
| Include in Phase 36 | Consistent with D-14 from Phase 35; debug-level logs on `moead_events` | ✓ |
| Defer | Ship observer trait only, LogObserver impl later | |

**User's choice:** Include in Phase 36 — established pattern from Phase 35.

---

## Neighbour Replacement Cap

### Question 1: Include max_updates parameter

| Option | Description | Selected |
|--------|-------------|----------|
| Include with configurable default (nr=2) | `with_max_neighbor_replacements(2)`; canonical Zhang & Li 2007 parameter | ✓ |
| Defer — no cap | Unlimited replacements; simpler but deviates from canonical MOEA/D | |

**User's choice:** Include with nr=2 default — canonical algorithm compliance.

### Question 2: Neighbourhood size default

| Option | Description | Selected |
|--------|-------------|----------|
| T=20, validate() passes | Literature default for 100+ population sizes; no explicit call required | ✓ |
| No default — validate() fails | Force explicit neighbourhood size | |

**User's choice:** T=20 default — sensible, no friction for standard use cases.

---

## Claude's Discretion

- Internal neighbourhood computation details (precomputation strategy, data structure)
- Ideal point incremental update implementation
- WASM cfg-gating placement
- PBI internal normalisation (ideal-point shift, no nadir tracking in Phase 36)
- Example DTLZ2 setup parameters (population size, generations, subdivision p)

## Deferred Ideas

- Two-layer weight vectors for M > 5 objectives
- Constraint handling for MOEA/D
- `AllObserver<U>` updated to include `MoeaDObserver<U>`
- Sub-problem-level observer hooks
- Weighted-sum scalarization
- Adaptive weight vector adjustment
