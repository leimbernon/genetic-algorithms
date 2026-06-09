# Phase 62: Surrogate-Assisted Evaluation - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 62 delivers a `SurrogateModel<U>` trait that allows users to attach a surrogate pre-screener to `Ga` to reduce true fitness calls on expensive black-box problems. Each generation, the surrogate ranks the newly generated offspring batch by predicted fitness; only the top `prescreening_fraction` proceed to actual fitness evaluation (scalar or batch). Rejected offspring are dropped entirely before evaluation. True fitness call count per generation is exposed in `GenerationStats` and observable via `GaObserver`.

**Out of scope:** CMA-ES surrogate support, IslandGa surrogate support, online surrogate training hooks, async surrogates, surrogate for initial population screening.

</domain>

<decisions>
## Implementation Decisions

### SurrogateModel Trait

- **D-01:** `SurrogateModel<U>` trait has exactly one method:
  ```rust
  pub trait SurrogateModel<U: ChromosomeT>: Send + Sync {
      fn predict(&self, chromosome: &U) -> f64;
  }
  ```
  Training is entirely user-managed outside the trait. The GA treats the surrogate as a pre-trained oracle — no `train()` or `update()` hooks. Users who want online learning implement it themselves inside their `SurrogateModel` impl.

- **D-02:** Trait lives in `src/fitness/surrogate.rs`, parallel to `BatchFitnessEvaluator` in `src/fitness/batch.rs`. `Send + Sync` required for `Arc<dyn SurrogateModel<U>>` across rayon threads.

- **D-03:** Builder method on `Ga`: `.with_surrogate(model: Arc<dyn SurrogateModel<U> + Send + Sync>, prescreening_fraction: f64) -> Self`. The `prescreening_fraction` is in `(0.0, 1.0]` (validated at build/run time).

### Prescreening Mechanics

- **D-04:** Rejected offspring (bottom `1 - prescreening_fraction` by surrogate score) are **dropped entirely** — they never enter the fitness evaluation path. The surrogate is a pure filter, not a fitness predictor.

- **D-05:** Minimum floor: always pass at least 1 offspring, regardless of fraction. Formula: `max(1, floor(n * prescreening_fraction))` where `n` is offspring count.

- **D-06:** Prescreening applies to **offspring only** — the per-generation crossover+mutation output batch. The existing population (already evaluated) is never re-screened.

### Engine Scope

- **D-07:** Surrogate support is added to **`Ga` only** in Phase 62. `CmaEngine` and `IslandGa` are explicitly out of scope.

### Pipeline Ordering

- **D-08:** Evaluation pipeline order when surrogate is configured:
  1. **Surrogate prescreens offspring batch** — rank all offspring by `predict()`, keep top fraction
  2. **FitnessCache check** on surviving offspring
  3. **BatchFitnessEvaluator** (if configured) or scalar `fitness_fn` on cache misses

  Surrogate runs first, maximizing cache and batch evaluator efficiency (fewer items enter the expensive path).

- **D-09:** Surrogate and `BatchFitnessEvaluator` are **compatible** — they compose cleanly. Surrogate narrows the offspring slice; batch evaluates what remains. No mutual exclusivity.

### GenerationStats

- **D-10:** Add `true_fitness_calls: Option<u64>` to `GenerationStats`. `None` when no surrogate is configured; `Some(n)` is the count of offspring that actually reached fitness evaluation (post-prescreening) in this generation. Follows the `cache_hits`/`cache_misses` pattern from Phase 60.

- **D-11:** `GaObserver` receives `true_fitness_calls` via the existing `GenerationStats` parameter in `on_generation_complete` — no new observer method needed.

### Claude's Discretion

- Internal variable names for the prescreened offspring sub-slice
- Whether `prescreening_fraction` is stored as a field in `GaConfiguration` or inline in the surrogate builder tuple
- Whether `SurrogateModel` is re-exported from `src/lib.rs` at crate root (follow `BatchFitnessEvaluator` re-export pattern)
- How the prescreening sort handles NaN surrogate predictions (treat as worst score)
- Whether to add a `with_surrogate` validation step to `src/validators/`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 60 Foundations (batch + cache — MUST read)
- `src/fitness/batch.rs` — `BatchFitnessEvaluator<U>` trait; pattern for `SurrogateModel` trait definition
- `src/engines/ga.rs` lines 289, 949-952, 2584 — `batch_evaluator` field and wiring; surrogate prescreening inserts before the batch path
- `src/fitness/cache.rs` — `FitnessCache` (LRU); cache check follows surrogate prescreening in the pipeline

### GenerationStats (extend)
- `src/stats.rs` lines 26-65 — `GenerationStats` struct; add `true_fitness_calls: Option<u64>` following the `cache_hits`/`cache_misses` pattern (lines 59-65)

### GA Hot Path (surrogate insertion point)
- `src/engines/ga.rs` — main `Ga` engine; offspring are collected after crossover+mutation, before fitness evaluation — this is where surrogate prescreening fires

### Observer (no new methods needed)
- `src/observe/observer.rs` — `GaObserver<U>` trait; `on_generation_complete(&self, stats: &GenerationStats)` already passes stats, which will carry `true_fitness_calls`

### WASM Gating Pattern
- `src/engines/ga.rs` — existing `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` rayon gates; surrogate predict calls must be WASM-compatible (no par_iter for the prescreening sort step unless gated)

### Configuration / Builder Pattern
- `src/engines/ga.rs` builder methods — follow `.with_batch_evaluator()` pattern for `.with_surrogate(model, fraction)` builder

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `BatchFitnessEvaluator<U>` in `src/fitness/batch.rs` — direct model for `SurrogateModel<U>` trait definition (same `Send + Sync` bounds, same `Arc<dyn ...>` usage pattern)
- `GenerationStats::cache_hits: Option<u64>` — exact pattern to follow for `true_fitness_calls: Option<u64>` (field, serde default, None when feature inactive)
- `src/fitness/cache.rs` `FitnessCache` — the cache is already in the pipeline; surrogate inserts before it, not after

### Established Patterns
- Enum + factory for operators — not applicable here (surrogate is user-trait, not an enum variant)
- `Arc<dyn Trait + Send + Sync>` — ownership pattern for all user-provided engine extensions (batch evaluator, observer); surrogate follows same pattern
- `#[cfg(not(target_arch = "wasm32"))]` rayon gates — any sorting of offspring by surrogate score must be either sequential or cfg-gated

### Integration Points
- Offspring batch collection point in `src/engines/ga.rs` crossover loop — surrogate prescreening happens here, between offspring generation and fitness evaluation
- `src/stats.rs` `GenerationStats` — add `true_fitness_calls` field
- `src/lib.rs` re-export — add `SurrogateModel` to public API (follow `BatchFitnessEvaluator` re-export)
- `cargo check --target wasm32-unknown-unknown` must pass; `predict(&U) -> f64` is WASM-safe as long as no par_iter is used in the prescreening sort

</code_context>

<specifics>
## Specific Ideas

- **Prescreening sort**: sort offspring by `predict()` score descending, take top `max(1, floor(n * fraction))`. No rayon needed for the sort (offspring count is typically small relative to pop size); planner may opt for sequential sort unconditionally.
- **`true_fitness_calls` counting**: count the length of the post-prescreening offspring slice that actually reaches the `evaluate_batch` / `fitness_fn` path. This is the delta for this generation's stats.
- **Validation**: validate `prescreening_fraction` in `(0.0, 1.0]` — a fraction of 0.0 would call no true fitness at all; a fraction > 1.0 is nonsensical.

</specifics>

<deferred>
## Deferred Ideas

- **CmaEngine surrogate support** — CMA-ES is also used on expensive black-box problems; surrogate pre-screening would be valuable there. Deferred to a future phase.
- **IslandGa surrogate support** — surrogate per-island is conceivable but requires per-island model configuration. Deferred.
- **Online surrogate learning (`update` hook)** — `SurrogateModel::update(chromosome, true_fitness)` for feedback-based online learning. Users can implement this themselves via interior mutability (`Mutex<model>`) without API support. Deferred.
- **Surrogate for initial population** — pre-screening the generation-0 population before the first true fitness pass. Deferred.

</deferred>

---

*Phase: 62-surrogate-assisted-evaluation*
*Context gathered: 2026-06-09*
