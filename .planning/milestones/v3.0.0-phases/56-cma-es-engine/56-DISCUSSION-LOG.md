# Phase 56: CMA-ES Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-01
**Phase:** 56-cma-es-engine
**Areas discussed:** Gene arithmetic trait, Restart scope, Configuration depth, Example coverage, RealGene rename approach, Adaptation parameter granularity

---

## Gene Arithmetic Trait

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse DeGene | CmaEngine bounds on U::Gene: DeGene — same trait DE uses, no new type | |
| New CmaGene trait | Separate CmaGene with identical interface, clean module separation | |
| Generalize to RealGene | Rename/extract DeGene → RealGene shared by both DE and CMA-ES | ✓ |

**User's choice:** Generalize to RealGene  
**Notes:** Hard rename (no alias) in this phase. DeEngine and CmaEngine both bound on `U::Gene: RealGene`. Fits v3.0.0 breaking-change milestone.

---

## RealGene Rename Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Hard rename in this phase | DeGene removed; RealGene replaces everywhere; fits v3.0.0 | ✓ |
| Type alias bridge | Keep `pub type DeGene = RealGene` as deprecated alias | |
| Keep DeGene, add RealGene | Blanket impl bridges the two names | |

**User's choice:** Hard rename  
**Notes:** All DeEngine bounds update in this phase. Downstream users must update to RealGene.

---

## Restart Scope

| Option | Description | Selected |
|--------|-------------|----------|
| No restarts — defer to #255 | Pure CMA-ES only; IPOP/BIPOP in separate phase | ✓ |
| IPOP only | Basic increasing-population restart bundled here | |
| Restart config hook | Add disabled RestartConfig placeholder now | |

**User's choice:** No restarts — defer to #255  
**Notes:** Restarts (issue #255) explicitly out of scope.

---

## Configuration Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal — automatic defaults | Only sigma0, population_size, max_generations, fitness_target | |
| Standard — expose tuning params | Also cc, cs, c1, cmu as Option<f64> overrides | ✓ |

**User's choice:** Standard — expose tuning params  
**Notes:** Four separate Option<f64> fields (not grouped in a struct). None = Hansen auto-formula.

---

## Adaptation Parameter Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Four separate Option<f64> fields | with_cc(), with_cs(), with_c1(), with_cmu() builder methods | ✓ |
| AdaptationRates struct | One with_adaptation_rates() builder method | |
| You decide | Leave grouping to planner | |

**User's choice:** Four separate Option<f64> fields  
**Notes:** Fine-grained control preferred.

---

## Example Coverage

| Option | Description | Selected |
|--------|-------------|----------|
| cma_es_rastrigin example | Standard benchmark showing CMA-ES strength vs plain GA | |
| CMA-ES vs DE convergence bench | Criterion benchmark comparing engines | |
| You decide | Leave to planner/executor | ✓ |

**User's choice:** You decide  
**Notes:** Planner/executor chooses based on what best complements existing examples.

---

## Claude's Discretion

- File placement of `RealGene` trait (new shared module vs. re-export from de/)
- `GenerationStats` population for CMA-ES
- Internal bookkeeping structures (path vectors, eigendecomposition scheduling)
- Example benchmark choice (see above)

## Deferred Ideas

- **Restart strategies (IPOP/BIPOP)** — Issue #255; separate phase
- **Active CMA-ES** — negative update variant; future CMA-ES enhancement
- **CMA-ES-MO / MO-CMA-ES** — multi-objective variants; not in scope
