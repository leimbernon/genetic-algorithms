# Phase 39: Multi-objective quality indicators — Hypervolume, GD, IGD, Spread - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 39-multi-objective-quality-indicators-hypervolume-gd-igd-spread
**Areas discussed:** Module location, API style, Hypervolume scope, Reference sets

---

## Module Location

| Option | Description | Selected |
|--------|-------------|----------|
| multi_objective/indicators/ | Create src/engines/multi_objective/indicators/ directory — follows Phase 35's shared-MOO-utility pattern | ✓ |
| multi_objective/indicators.rs | Single file — simpler, all 4 indicators in one file | |
| nsga2/indicators.rs | As roadmap originally suggested — but Phase 38 engines would couple to nsga2 unnecessarily | |

**User's choice:** multi_objective/indicators/ directory — follows the established shared-MOO-utility pattern from Phase 35
**Notes:** Roadmap's original nsga2/indicators.rs suggestion superseded. The multi_objective module extracted in Phase 35 is the canonical home for shared MOO utilities.

---

## API Style

| Option | Description | Selected |
|--------|-------------|----------|
| Pure functions | Simple, stateless, no pre-computation. Phase 38 engines call these in loops | ✓ |
| Struct-based precompute | HypervolumeIndicator struct pre-sorts points for repeated contribution queries | |
| Both | Primary API is pure functions, optional struct for hot-loop use | |

**User's choice:** Pure functions — stateless, simple signatures
**Notes:** If SMS-EMOA hot-loop optimization is needed, Phase 38 handles that layer.

---

## Hypervolume Scope

| Option | Description | Selected |
|--------|-------------|----------|
| 2-objective exact only | O(n log n) Lebesgue-based algorithm | ✓ |
| Exact any-d WFG | General d-dimensional exact using WFG algorithm, O(n^{d-1}) | |
| Exact 2D + Monte Carlo dD | Exact 2-objective + Monte Carlo sampling for 3+ dimensions | |

**User's choice:** 2-objective exact only — covers the primary use cases (ZDT/DTLZ benchmarks, SMS-EMOA)
**Notes:** 3+ objective hypervolume deferred. The current multi-objective engines (NSGA-II, NSGA-III, MOEA/D, SPEA2) all support 2+ objectives, but hypervolume computation above 2D is exponentially expensive and not needed for SMS-EMOA's typical 2-3 objective range.

---

## Reference Sets

| Option | Description | Selected |
|--------|-------------|----------|
| User provides always | All indicator functions take reference front as parameter. Tests define fronts inline | ✓ |
| Library constants module | tests/ provides hardcoded 1000-point samples of ZDT/DTLZ Pareto fronts | |

**User's choice:** User provides always — no hardcoded reference front data in the library
**Notes:** Tests will define analytically-known reference fronts inline for validation. Library stays free of domain-specific reference data.

---

## Claude's Discretion

- Exact algorithm for each indicator: 2D hypervolume (sort-then-sweep Lebesgue), GD/IGD (standard Euclidean nearest-neighbor), Spread (Deb et al. 2002)
- Internal validation helpers for input checking
- GD/IGD power parameter defaults to 2.0
- No new feature flags
- No cfg-gating needed (pure math, no Instant/rayon)

## Deferred Ideas

None — discussion stayed within phase scope.
