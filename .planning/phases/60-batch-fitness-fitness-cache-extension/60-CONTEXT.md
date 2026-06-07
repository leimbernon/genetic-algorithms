# Phase 60: Batch Fitness / Fitness Cache Extension - Context

**Gathered:** 2026-06-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 60 adds two complementary fitness evaluation extensions to `Ga` and `CmaEngine`:

1. **`BatchFitnessEvaluator<U>` trait** — Users implement this trait to evaluate a batch of chromosomes in a single call, enabling GPU/API-backed evaluators. When configured, the individual-level `calculate_fitness()` path is fully replaced (never called). `Ga` and `CmaEngine` both get a `.with_batch_evaluator(Arc<dyn BatchFitnessEvaluator<U>>)` builder method.

2. **`FitnessCache` stats exposure** — The existing LRU `FitnessCache` (already wired into `Ga` via `fitness_cache_size`) is refactored so `Ga` holds an external `Arc<Mutex<FitnessCache>>` reference. Per-generation delta `cache_hits: Option<u64>` and `cache_misses: Option<u64>` are added to `GenerationStats`. `CmaEngine` also gains `.with_fitness_cache(size)` support.

**Out of scope:** Batch evaluator or cache support for PSO / EDA / ALPS / ScatterSearch / CellularGA / DE engines in this phase. Async evaluators. Per-gene cache keys.

</domain>

<decisions>
## Implementation Decisions

### BatchFitnessEvaluator Trait

- **D-01:** `BatchFitnessEvaluator<U>` is a public trait with signature:
  ```rust
  pub trait BatchFitnessEvaluator<U: ChromosomeT>: Send + Sync {
      fn evaluate_batch(&self, chromosomes: &[U]) -> Vec<f64>;
  }
  ```
  Takes typed chromosomes (`&[U]`), not DNA slices. Matches ROADMAP spec exactly. `Send + Sync` required for `Arc<dyn ...>` across rayon threads.

- **D-02:** When a `BatchFitnessEvaluator` is configured on `Ga`, it **fully replaces** the individual-level `calculate_fitness()` path. `Ga` collects all offspring into a slice, calls `evaluate_batch` once, assigns returned fitness values back to each chromosome. No dual-path confusion.

- **D-03:** Builder method: `.with_batch_evaluator(Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>) -> Self` on both `Ga` and `CmaEngine`. Mutually exclusive with `fitness_fn` (builder panics or returns error if both are configured — planner decides mechanism).

### CmaEngine Scope

- **D-04:** `CmaEngine`'s run loop is modified to collect all offspring after sampling, then call `evaluate_batch` once for the full offspring slice. The individual-level `(self.fitness_fn)(ind.dna())` calls are replaced. This is a structural change to the CMA run loop.

- **D-05:** `CmaEngine` also gains `.with_fitness_cache(size)` support. Same `Arc<Mutex<FitnessCache>>` pattern as `Ga`.

### Batch + Cache Interaction

- **D-06:** `BatchFitnessEvaluator` and `FitnessCache` can be used together. When both are configured, the cache wraps the batch path:
  1. For each chromosome in the batch, check cache by DNA hash
  2. Collect cache misses into a sub-slice
  3. Call `evaluate_batch` only on the miss sub-slice
  4. Merge returned fitness values back in original order, store results in cache
  5. Cache hits skip `evaluate_batch` entirely
  This is the "partition: cache hits skip batch" design.

### FitnessCache Stats in GenerationStats

- **D-07:** `GenerationStats` gains two new `Option<u64>` fields:
  ```rust
  pub cache_hits: Option<u64>,    // delta hits for THIS generation
  pub cache_misses: Option<u64>,  // delta misses for THIS generation
  ```
  Both are `None` when no cache is configured; `Some(n)` when cache is active. Values are **delta per generation** (hits/misses that occurred during this generation only), not cumulative totals.

- **D-08:** To enable stats access, `wrap_with_cache` is refactored to return `(Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)` instead of just the wrapped fn. `Ga` stores the `Arc<Mutex<FitnessCache>>` reference and reads delta hits/misses before and after each generation loop to populate `GenerationStats`.

### Claude's Discretion

- Whether `BatchFitnessEvaluator` lives in `src/traits/` or `src/fitness/` module
- Internal variable names for the batch evaluation pass in `ga.rs`
- How the builder signals mutual exclusivity between `fitness_fn` and `with_batch_evaluator` (panic vs `Result` vs `GaError` at `run()` time)
- Whether `CmaEngine` refactors fitness evaluation into a shared helper or duplicates the batch/cache logic inline

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Fitness Evaluation (extend these)
- `src/fitness/cache.rs` — `FitnessCache` (LRU), `wrap_with_cache()`, `hash_dna()` — refactor `wrap_with_cache` to return external handle
- `src/fitness.rs` — module entry point; `BatchFitnessEvaluator` trait likely added here or in `src/traits/`
- `src/fitness/fitness_fn_wrapper.rs` — `FitnessFnWrapper` — reference for how fitness fn wrapping currently works

### GA Engine (extend batch + cache paths)
- `src/engines/ga.rs` — main `Ga` engine; lines 762–765 (cache wiring), line 1098/1113–1172 (individual fitness eval calls — replace with batch path)
- `src/stats.rs` — `GenerationStats` struct; add `cache_hits` and `cache_misses` fields here

### CMA Engine (add batch + cache support)
- `src/engines/cma/engine.rs` — `CmaEngine`; lines 564, 630 (individual fitness calls — replace with batch path); add `Arc<Mutex<FitnessCache>>` field
- `src/engines/cma/configuration.rs` — `CmaConfiguration`; add `batch_evaluator` and `fitness_cache_size` builder methods

### Trait Definitions
- `src/traits/chromosome.rs` — `ChromosomeT` base trait — `BatchFitnessEvaluator<U: ChromosomeT>` bound references this
- `src/lib.rs` — add `pub use` re-exports for `BatchFitnessEvaluator`

### Pattern References (most recent engines)
- `src/engines/eda/engine.rs` — most recent engine; observer wiring pattern to follow for new builder methods
- `src/engines/pso/engine.rs` — secondary reference for engine struct layout

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/fitness/cache.rs::FitnessCache` — already implemented LRU with `hits`/`misses` counters; refactor `wrap_with_cache` to also return `Arc<Mutex<FitnessCache>>`
- `src/fitness/cache.rs::hash_dna<G: Debug>()` — DNA hashing for cache keys; reuse unchanged
- `Ga.fitness_cache_size: Option<usize>` (already in builder, lines ~274/358/905) — existing field; extend to also store the external cache handle after `wrap_with_cache`
- `CmaEngine.notify()` helper — use same pattern for any new observer hooks needed

### Established Patterns
- `Option<Arc<dyn GaObserver<U> + Send + Sync>>` — the Arc trait-object pattern; `BatchFitnessEvaluator` follows the same ownership model
- `#[cfg(not(target_arch = "wasm32"))]` gates — fitness evaluation via rayon in `Ga` is already gated; batch evaluation must respect the same gate
- `ProblemSolving` enum — used for sort direction in fitness; not relevant to batch but needed for the population sort in the CMA batch pass
- `Arc<FitnessFn<G>>` type alias (`src/traits/mod.rs`) — existing individual fitness fn type; batch evaluator is a separate field, not a replacement of this type alias

### Integration Points
- `Ga::run()` — batch evaluation pass replaces the rayon-parallel individual `calculate_fitness()` calls; cache delta reading wraps the generation boundary
- `CmaEngine::run()` — offspring sampling loop (lines 564, 630) replaced with collect-then-batch pass
- `GenerationStats::new()` (or construction site) — add `cache_hits: None` / `cache_misses: None` defaults
- `src/engines/mod.rs` — no changes needed (batch evaluator lives in fitness/traits module)
- `tests/engines/ga.rs` (or equivalent) — new tests for batch evaluator; never inline in src/
- `tests/engines/cma.rs` — new tests for CMA batch + cache

</code_context>

<specifics>
## Specific Ideas

- **Batch + cache partition algorithm**: collect all chromosomes, partition into hits/misses by DNA hash, call `evaluate_batch` on miss slice only, merge results. This is the core implementation detail for D-06.
- **GenerationStats delta stats**: `Ga` reads `cache.hits()` and `cache.misses()` at the start of each generation loop iteration, stores as `prev_hits`/`prev_misses`, reads again at end, subtracts to get delta. This requires the external `Arc<Mutex<FitnessCache>>` handle from D-08.
- **WASM safety**: the batch evaluation pass itself is sequential (no rayon) — the trait takes `&[U]` synchronously. The user's implementation may be async/GPU-backed but the trait signature is sync. WASM-safe by default.

</specifics>

<deferred>
## Deferred Ideas

- Batch evaluator support for PSO, EDA, ALPS, ScatterSearch, CellularGA, DE — out of scope for this phase; these engines evaluate fitness in their own loops
- Async `BatchFitnessEvaluator` — sync-only for now; async trait impls require additional ecosystem complexity
- Per-observer cache event hook (`on_cache_hit` / `on_cache_miss`) — deferred; `GenerationStats` delta fields are sufficient for Phase 60
- `FitnessCache` with `Hash`-based keys (instead of `Debug`-repr hashing) — the existing `hash_dna` Debug approach is retained; a more robust hash strategy is a future improvement

</deferred>

---

*Phase: 60-batch-fitness-fitness-cache-extension*
*Context gathered: 2026-06-07*
