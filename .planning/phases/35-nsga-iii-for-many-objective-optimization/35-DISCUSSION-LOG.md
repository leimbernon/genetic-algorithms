# Phase 35: NSGA-III for many-objective optimization - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 35-nsga-iii-for-many-objective-optimization
**Areas discussed:** Code sharing with NSGA-II, Reference point API, Observer pattern, Return type / output API

---

## Code Sharing with NSGA-II

### Q1: How should shared utilities be structured?

| Option | Description | Selected |
|--------|-------------|----------|
| Extract to shared module | Create src/engines/multi_objective/ with shared utilities; nsga2 re-exports (no API break); nsga3 and future engines import from multi_objective | ✓ |
| Import from nsga2 in nsga3 | nsga3 uses crate::nsga2::non_dominated_sort and crate::nsga2::pareto directly | |
| Duplicate in nsga3 | Copy files into src/engines/nsga3/ — zero coupling, but maintenance divergence risk | |

**User's choice:** Extract to shared module
**Notes:** Motivated by phases 36-38 (MOEA/D, SPEA2, SMS-EMOA) all needing the same utilities.

### Q2: Public module name?

| Option | Description | Selected |
|--------|-------------|----------|
| multi_objective | pub mod multi_objective — clear intent, doesn't over-commit to algorithm family | ✓ |
| pareto | pub mod pareto — named after core concept | |
| moo | pub mod moo — short abbreviation | |

**User's choice:** multi_objective

### Q3: Backward compatibility for moved types?

| Option | Description | Selected |
|--------|-------------|----------|
| Re-export from nsga2 | Items move but nsga2 keeps pub use re-exports — no API break | ✓ |
| No re-exports (clean break) | Remove from nsga2, put only in multi_objective — breaking change | |

**User's choice:** Re-export from nsga2

---

## Reference Point API

### Q1: How does user trigger Das-Dennis auto-generation?

| Option | Description | Selected |
|--------|-------------|----------|
| with_reference_points_auto(p) | Separate builder method for subdivision count — explicit, validatable | ✓ |
| Enum in with_reference_points() | Single builder taking ReferencePoints::DasDennis(p) or ReferencePoints::Custom(vec) | |
| Default to auto in config | Das-Dennis default with default p; user only calls with_reference_points() to override | |

**User's choice:** with_reference_points_auto(p)

### Q2: Type for user-supplied custom reference points?

| Option | Description | Selected |
|--------|-------------|----------|
| Vec<Vec<f64>> | Simple, no new types; library validates lengths at validate() | ✓ |
| Vec<[f64; N]> with const generic | Compile-time dimension check — adds complexity, const generic ripples | |
| Custom ReferencePoint struct | New pub struct ReferencePoint { coords: Vec<f64> } | |

**User's choice:** Vec<Vec<f64>>

### Q3: What if neither builder method is called?

| Option | Description | Selected |
|--------|-------------|----------|
| Return Err at validate() | GaError::ConfigurationError if reference points missing — fail-fast | ✓ |
| Default to Das-Dennis p=4 | Silently use default — hides meaningful parameter choice | |
| Panic at run() time | Runtime panic — less ergonomic than Err | |

**User's choice:** Return Err at validate()

---

## Observer Pattern

### Q1: Which observer approach for NSGA-III?

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse Nsga2Observer<U> | Store Option<Arc<dyn Nsga2Observer<U>>>; both are multi-objective with same lifecycle | |
| Create Nsga3Observer<U> sub-trait | New trait mirroring Nsga2Observer but named for NSGA-III | ✓ |
| Just GaObserver<U> | No MOO-specific sub-trait; simplest, avoids fragmentation | |

**User's choice:** Create Nsga3Observer<U> sub-trait
**Notes:** Consistent with how NSGA-II was treated; prioritizes API symmetry over the v2.4.0 general policy (which targeted non-NSGA engines).

### Q2: What hooks beyond basic lifecycle?

| Option | Description | Selected |
|--------|-------------|----------|
| Basic lifecycle only | Mirror Nsga2Observer: on_pareto_front_assigned, on_non_dominated_sort_complete with Duration params | ✓ |
| Add on_reference_association hook | Extra hook after niche assignment with association counts | |
| You decide | Claude chooses based on Nsga2Observer precedent | |

**User's choice:** Basic lifecycle only

---

## Return Type / Output API

### Q1: What does run() return?

| Option | Description | Selected |
|--------|-------------|----------|
| Same ParetoFront<U> type | Result<ParetoFront<U>, GaError> — identical to NSGA-II; ParetoFront from multi_objective | ✓ |
| Return all fronts Vec<ParetoFront<U>> | Richer output — all ranked fronts, not just first | |
| New Nsga3Result struct | Wrap ParetoFront + reference point association data | |

**User's choice:** Same ParetoFront<U> type

### Q2: on_new_best semantics?

| Option | Description | Selected |
|--------|-------------|----------|
| Track first-front best by objective 0 | Same as NSGA-II — predictable, consistent across engines | ✓ |
| You decide | Claude chooses based on NSGA-III semantics | |

**User's choice:** Track first-front best by objective 0

---

## Claude's Discretion

- Whether Nsga3Ga<U> holds a single observer field (Nsga3Observer<U>) or separate GaObserver<U> + Nsga3Observer<U> fields — match Nsga2Ga pattern as closely as possible
- Das-Dennis generator implementation details (recursive vs iterative, function names)
- Reference point normalization: raw or normalized on construction
- Exact WASM cfg-gate placement (module-level use vs inline)

## Deferred Ideas

- Two-layer Das-Dennis reference points for large M (future builder method)
- Constraint handling for NSGA-III (follow-up after Phase 35)
- Updating AllObserver<U> to include Nsga3Observer<U> (deferred to avoid breaking changes)
- Adaptive normalization (online ideal/nadir estimation)
