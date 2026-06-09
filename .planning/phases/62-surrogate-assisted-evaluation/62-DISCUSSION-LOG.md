# Phase 62: Surrogate-Assisted Evaluation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-09
**Phase:** 62-surrogate-assisted-evaluation
**Areas discussed:** SurrogateModel trait API, Prescreening mechanics, Engine scope, Pipeline ordering

---

## SurrogateModel Trait API

### Trait methods

| Option | Description | Selected |
|--------|-------------|----------|
| predict only | Just fn predict(&self, chromosome: &U) -> f64. Training is entirely user-managed. | ✓ |
| predict + update | predict + fn update(&self, chromosome: &U, true_fitness: f64) for online learning. | |
| predict + train + update | Full three-method online learning API. | |

**User's choice:** predict only
**Notes:** Training stays outside the trait — surrogate is a pre-trained oracle from the GA's perspective.

### Trait location

| Option | Description | Selected |
|--------|-------------|----------|
| src/fitness/surrogate.rs, Send+Sync | Parallel to BatchFitnessEvaluator in src/fitness/batch.rs. | ✓ |
| src/traits/surrogate.rs, Send+Sync | Alongside core traits in src/traits/. | |
| You decide | Let the planner pick. | |

**User's choice:** src/fitness/surrogate.rs, Send+Sync

---

## Prescreening Mechanics

### What happens to rejected offspring

| Option | Description | Selected |
|--------|-------------|----------|
| Dropped entirely | Discarded before fitness evaluation. Surrogate is a pure filter. | ✓ |
| Assigned surrogate score as fitness | Stay in pool with predicted fitness. | |
| Passed through unevaluated | Pass to survivor selection without fitness. | |

**User's choice:** Dropped entirely

### Minimum floor

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — at least 1 offspring always passes | max(1, floor(n * fraction)) | ✓ |
| No floor — fraction is exact | Exact; user's responsibility. | |
| Configurable floor via .with_surrogate_min_survivors(n) | User-set floor. | |

**User's choice:** Yes — at least 1 offspring always passes

### Population scope

| Option | Description | Selected |
|--------|-------------|----------|
| Offspring only | Pre-screens each generation's crossover+mutation output. | ✓ |
| Also applied to initial population at run start | Screens generation-0 population too. | |

**User's choice:** Offspring only

---

## Engine Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Ga only | Focus on the main GA engine. CmaEngine deferred. | ✓ |
| Ga + CmaEngine | Parity with Phase 60 batch evaluator scope. | |
| Ga + IslandGa | Island model also runs Ga instances internally. | |

**User's choice:** Ga only

---

## Pipeline Ordering

### Surrogate position relative to cache/batch

| Option | Description | Selected |
|--------|-------------|----------|
| Surrogate first, then cache/batch | Surrogate → FitnessCache → BatchEvaluator/scalar. | ✓ |
| Cache first, then surrogate, then batch | Cache hits → surrogate on misses → batch. | |
| You decide | Let the planner determine ordering. | |

**User's choice:** Surrogate first, then cache/batch

### Surrogate + BatchFitnessEvaluator compatibility

| Option | Description | Selected |
|--------|-------------|----------|
| Compatible — surrogate screens, batch evaluates survivors | They compose cleanly. | ✓ |
| Mutually exclusive — surrogate or batch, not both | Simpler but limits usefulness. | |
| You decide | Let the planner sort it out. | |

**User's choice:** Compatible

---

## Claude's Discretion

- Internal variable names for the prescreened offspring sub-slice
- Whether `prescreening_fraction` is stored in `GaConfiguration` or inline in the surrogate builder tuple
- Whether `SurrogateModel` is re-exported from `src/lib.rs` at crate root (follow `BatchFitnessEvaluator` re-export pattern)
- How the prescreening sort handles NaN surrogate predictions
- Whether to add a `with_surrogate` validation step to `src/validators/`

## Deferred Ideas

- CmaEngine surrogate support — deferred to a future phase
- IslandGa surrogate support — deferred to a future phase
- Online surrogate learning (`update` hook) — users can implement via interior mutability
- Surrogate for initial population (generation-0 pre-screening) — deferred
