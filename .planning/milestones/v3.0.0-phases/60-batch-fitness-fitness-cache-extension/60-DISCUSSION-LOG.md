# Phase 60: Batch Fitness / Fitness Cache Extension - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-07
**Phase:** 60-batch-fitness-fitness-cache-extension
**Areas discussed:** Batch evaluator wiring, Batch + cache interaction, CmaEngine batch scope, Cache stats in GenerationStats

---

## Batch Evaluator Wiring

| Option | Description | Selected |
|--------|-------------|----------|
| Fully replaces individual path | When batch evaluator is set, Ga collects all offspring, calls evaluate_batch once, assigns fitness values. Individual calculate_fitness never called. | ✓ |
| Fallback — individual path still used when needed | Batch handles main pass; individual path remains for elitism reinsertion, extension regrowth, etc. | |
| You decide | Claude picks the approach that fits the existing run loop. | |

**User's choice:** Fully replaces it

---

| Option | Description | Selected |
|--------|-------------|----------|
| Trait object: `.with_batch_evaluator(Arc<dyn BatchFitnessEvaluator<U>>)` | Consistent with GaObserver pattern. Structured impl blocks, testable, composable. | ✓ |
| Closure: `.with_batch_evaluator(Arc<dyn Fn(&[U]) -> Vec<f64>>)` | Consistent with existing fitness_fn pattern. Simpler for quick lambdas. | |
| You decide | Claude picks whichever fits the existing builder ergonomics better. | |

**User's choice:** Trait object (GaObserver pattern)

---

| Option | Description | Selected |
|--------|-------------|----------|
| `evaluate_batch(&self, chromosomes: &[U]) -> Vec<f64>` | Takes typed chromosomes. Matches ROADMAP spec exactly. | ✓ |
| `evaluate_batch(&self, dna: &[&[U::Gene]]) -> Vec<f64>` | Takes DNA slices only. More minimal, consistent with individual fitness_fn. | |

**User's choice:** `&[U]` (typed chromosomes)

---

## Batch + Cache Interaction

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — cache wraps the batch path | Cache checks first. Misses collected, passed to evaluate_batch, results stored back. Maximally efficient. | ✓ |
| Mutually exclusive | Simpler: if BatchFitnessEvaluator is set, FitnessCache is ignored. Avoids partial-batch cache lookups. | |
| You decide | Claude picks whichever is less complex to implement. | |

**User's choice:** Can coexist — cache wraps batch path

---

| Option | Description | Selected |
|--------|-------------|----------|
| Partition: check cache for all, call evaluate_batch only on misses | Cache-hit chromosomes skip batch call entirely. Misses form a sub-slice. Results merged back in original order. | ✓ |
| Full batch always: don't split the slice | Always call evaluate_batch with full slice. Cache only used for individual fallback. Simpler but wastes calls. | |

**User's choice:** Partition — evaluate_batch called only on cache misses

---

## CmaEngine Batch Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — modify CMA run loop to collect-then-batch | Structural change: collect all offspring after sampling, call evaluate_batch once. Fully delivers SC-1 for CmaEngine. | ✓ |
| Yes — surface wiring only (no loop restructure) | Add trait + builder, but internally still loop per individual. Technically satisfies SC-1 but doesn't truly batch in CMA. | |
| No — Ga only for batch in Phase 60 | CmaEngine defers to future phase. Reduces scope, avoids restructuring CMA run loop. | |

**User's choice:** Full structural change to CMA run loop

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — add .with_fitness_cache(size) to CmaEngine | Mirrors Ga's existing pattern. Cache wraps fitness fn, hit rate in CmaEngine stats. | ✓ |
| No — cache only for Ga | CmaEngine defers cache support to Phase 61+. | |

**User's choice:** CmaEngine also gets FitnessCache support

---

## Cache Stats in GenerationStats

| Option | Description | Selected |
|--------|-------------|----------|
| Hit rate only: `cache_hit_rate: Option<f64>` | Single f64 in [0,1]. Simple, useful for tuning. | |
| Raw counts: `cache_hits: Option<u64>` and `cache_misses: Option<u64>` | Two raw counters per generation. User can compute hit rate themselves. More detailed. | ✓ |
| You decide | Claude picks whichever is most useful and least intrusive. | |

**User's choice:** Raw counts (hits + misses)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Ga holds Arc<Mutex<FitnessCache>> externally | Refactor `wrap_with_cache` to return both the wrapped fn AND an external Arc handle. Ga reads delta stats per generation. | ✓ |
| FitnessCache uses atomics for stats (lock-free) | hits/misses become AtomicU64. Ga holds Arc to atomic counters. Lock-free reads at stats time. | |
| You decide | Claude picks whichever design is simpler. | |

**User's choice:** External Arc<Mutex<FitnessCache>> reference

---

| Option | Description | Selected |
|--------|-------------|----------|
| Delta per generation | GenerationStats shows hits/misses for THAT generation only. Ga computes delta before/after each generation loop. | ✓ |
| Cumulative totals | Monotonically increasing counters since run start. Simpler — just read current totals. | |

**User's choice:** Delta per generation

---

## Claude's Discretion

- Whether `BatchFitnessEvaluator` lives in `src/traits/` or `src/fitness/` module
- Internal variable names for the batch evaluation pass in `ga.rs`
- How the builder signals mutual exclusivity between `fitness_fn` and `with_batch_evaluator`
- Whether `CmaEngine` refactors fitness evaluation into a shared helper or duplicates logic inline

## Deferred Ideas

- Batch evaluator support for PSO, EDA, ALPS, ScatterSearch, CellularGA, DE
- Async `BatchFitnessEvaluator` (sync-only for Phase 60)
- Per-observer cache event hooks (`on_cache_hit` / `on_cache_miss`)
- `FitnessCache` with `Hash`-based keys instead of `Debug`-repr hashing
