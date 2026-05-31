# Phase 51: Multi-parent Crossover + Self-Adaptive Mutation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-23
**Phase:** 51-multi-parent-crossover-self-adaptive-mutation
**Areas discussed:** Multi-parent dispatch, RealValued trait vs downcast, Sigma inheritance in crossover, SelfAdaptive built-in vs user-impl

---

## Multi-parent Dispatch

### Q1: How should ga.rs collect extra parents for UNDX/SPX/PCX?

| Option | Description | Selected |
|--------|-------------|----------|
| New factory_multi_parent() | Like factory_lexicase: ga.rs checks method and calls separate function taking &[U] parents. Selection still picks pairs, engine grabs num_parents random extras. | ✓ |
| New MultiParentCrossover trait | Add a second trait fn crossover_multi(&self, parents: &[&U]) alongside CrossoverOperator. More structural but adds a second trait hierarchy. | |

**User's choice:** New `factory_multi_parent()` — consistent with `factory_lexicase` precedent.

### Q2: How are extra parents collected beyond the primary pair?

| Option | Description | Selected |
|--------|-------------|----------|
| Random from population | ga.rs takes primary pair (i, j) then picks (num_parents - 2) random indices from population. Simple, stateless. | ✓ |
| Selection provides all parents | Modify selection to output Vec<usize> groups. Requires SelectionOperator changes. | |

**User's choice:** Random from population.

### Q3: How many offspring per multi-parent crossover call?

| Option | Description | Selected |
|--------|-------------|----------|
| 1 offspring per call | Standard for UNDX/PCX. Engine loops over pairs, calls once per pair. | ✓ |
| 2 offspring per call | Mirrors 2-parent operators. Simpler bookkeeping. | |

**User's choice:** 1 offspring per call.

---

## RealValued Trait vs Downcast

### Q1: How should UNDX/SPX/PCX guard against non-real-valued chromosomes?

| Option | Description | Selected |
|--------|-------------|----------|
| New RealValued marker trait | `pub trait RealValued: LinearChromosome {}` in `src/traits/`. factory_multi_parent generic over U: LinearChromosome + RealValued. Compile-time error for Binary/Unique. | ✓ |
| Runtime downcast (SBX precedent) | Follow existing try_sbx() pattern. No new trait. Consistent with existing real-valued operators. | |

**User's choice:** New `RealValued` marker trait — compile-time safety over SBX downcast pattern.

### Q2: Which chromosome types should implement RealValued?

| Option | Description | Selected |
|--------|-------------|----------|
| RangeChromosome<T> + MultiRangeChromosome<T> | Both built-in real-valued types. Users can also impl for custom chromosomes. | ✓ |
| RangeChromosome<T> only | Narrower scope; add MultiRange in Phase 48. | |

**User's choice:** Both `RangeChromosome<T>` and `MultiRangeChromosome<T>`.

---

## Sigma Inheritance in Crossover

### Q1: Where should sigma blending happen?

| Option | Description | Selected |
|--------|-------------|----------|
| Inside SelfAdaptiveGaussian::mutate() only | Child inherits primary parent sigma via clone; mutate() applies log-normal update. No crossover changes. | ✓ |
| In ga.rs after crossover, before mutation | ga.rs blends sigma if chromosome implements SelfAdaptive. Requires ga.rs SelfAdaptive awareness. | |
| Inside UNDX/SPX/PCX crossover ops | Crossover functions detect SelfAdaptive and blend sigmas. Mixes concerns. | |

**User's choice:** Mutation-only — sigma blending in `SelfAdaptiveGaussian::mutate()` only.

### Q2: Where should τ and τ' defaults come from?

| Option | Description | Selected |
|--------|-------------|----------|
| Standard ES heuristics as defaults | τ = 1/sqrt(2n), τ' = 1/sqrt(2*sqrt(n)). User can override via MutationConfiguration. | ✓ |
| Fixed user-provided tau values only | No automatic heuristics. User must set both. | |

**User's choice:** Standard ES heuristics as defaults.

### Q3: Update all sigmas or only the selected gene's sigma?

| Option | Description | Selected |
|--------|-------------|----------|
| Update all sigmas, then mutate one gene | Standard (1+λ)-ES: all sigmas updated via log-normal per mutate() call, then one gene perturbed. | ✓ |
| Update only the selected gene's sigma | Cheaper but sigmas for unmutated genes stagnate. | |

**User's choice:** Update all sigmas, mutate one gene — correct ES strategy parameter evolution.

---

## SelfAdaptive Built-in vs User-impl

### Q1: Should RangeChromosome<T> get a built-in SelfAdaptive implementation?

| Option | Description | Selected |
|--------|-------------|----------|
| Built-in impl on RangeChromosome<T> | Add Vec<f64> sigma field; implement SelfAdaptive. Users can immediately use SelfAdaptiveGaussian on RangeChromosome<f64> without boilerplate. | ✓ |
| User-impl only (like MultiCaseFitness) | Purely opt-in. RangeChromosome stays unchanged. Consistent with MultiCaseFitness precedent. | |

**User's choice:** Built-in impl on `RangeChromosome<T>`.

### Q2: How should sigma be initialized?

| Option | Description | Selected |
|--------|-------------|----------|
| Lazy-init to 1.0 per gene on first strategy_params() call | Auto-populates vec![1.0; n] if empty. No user action required. | ✓ |
| User calls with_strategy_params() builder | Explicit init. Forces user to set initial sigma scale. | |

**User's choice:** Lazy init to 1.0.

### Q3: Should sigma vector be serialized with serde?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, include in serde derive | Sigma survives checkpoint save/restore. Evolved strategy parameters preserved. | ✓ |
| No, reinitialize on restore | Simpler but destroys evolved sigmas. | |

**User's choice:** Yes, include in serde.

---

## Claude's Discretion

None — user provided clear direction on all areas.

## Deferred Ideas

None — discussion stayed within phase scope.
