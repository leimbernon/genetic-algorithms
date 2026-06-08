# Phase 61: Performance — Clone Reduction & Parallel Survivor - Context

**Gathered:** 2026-06-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 61 delivers measurable GA throughput improvements on two fronts:

1. **Clone reduction** — Eliminate unnecessary chromosome clones in the GA hot path, focusing on the crossover fallback path (`parent_1.clone()` / `parent_2.clone()` at `ga.rs` lines 2916-2917) plus a uniform `&U` refactor across all `GaObserver` callback signatures.

2. **Parallel survivor selection** — Apply `par_sort_unstable_by` to the four sort-based survivor operators (`fitness_based`, `mu_plus_lambda`, `age_based`, `mu_comma_lambda`) behind `#[cfg(not(target_arch = "wasm32"))]` gates. `DeterministicCrowding` is order-dependent (parent-offspring pairing) and is explicitly excluded from parallelization.

3. **Benchmark harness** — Create `benches/rastrigin.rs` as a dedicated benchmark file using `RangeChromosome<f64>` at pop=500 across 10, 20, and 50 dimensions. This is the measurement instrument for the ROADMAP ≥10% success criterion.

**Out of scope:** Clone reduction at the selection output collect (line 3091), observer clone deferral via intermediate types, parallelizing `DeterministicCrowding`, surrogate-assisted evaluation, batch fitness evaluation.

</domain>

<decisions>
## Implementation Decisions

### Clone Reduction — Crossover Fallback

- **D-01:** Primary clone target is the crossover fallback path at `src/engines/ga.rs` lines 2916-2917. When crossover returns `< 2` children, the current code clones `parent_1` and `parent_2` as fallback children. Replace this by **taking ownership of the parent chromosomes** from the couple and mutating them in-place, preserving existing behavioral semantics (generation always has offspring) while eliminating both clones.

- **D-02:** The selection output collect at line 3091 (`indices.iter().map(|&i| chromosomes[i].clone()).collect()`) is **not in scope** for Phase 61. It stays sequential and clone-based for now.

### GaObserver Callback Signature

- **D-03:** All `GaObserver<U>` callbacks that currently accept `U` (owned chromosome) are changed to `&U` (reference) uniformly across the trait. This eliminates the per-generation clone that currently precedes every observer notification (e.g., `on_new_best`, `on_generation_complete`). This is a **breaking change** — acceptable under v3.0.0 major semver.

- **D-04:** The change applies to ALL observer callbacks uniformly (not just the ones that currently trigger a clone), for a consistent, cleaner API.

- **D-05:** All built-in observer implementations (`LogObserver`, `TracingObserver`, `MetricsObserver`, `CompositeObserver`) must be updated to match the new `&U` signatures.

### Parallel Survivor Selection

- **D-06:** Use **`par_sort_unstable_by`** (not score-precompute + sequential sort) in the four eligible survivor operators. Unstable sort is acceptable — tie-breaking in fitness sort was already non-deterministic in practice.

- **D-07:** Operators that receive `par_sort_unstable_by`:
  - `src/operations/survivor/fitness.rs` — both branches (`fitness()` sort and `fitness_distance()` sort)
  - `src/operations/survivor/mu_plus_lambda.rs` — same two branches
  - `src/operations/survivor/age.rs` — `sort_by_key(|a| Reverse(a.age()))`
  - `src/operations/survivor/mu_comma_lambda.rs` — the fitness sort on the age==0 survivors sub-vec

- **D-08:** `DeterministicCrowding` is **explicitly excluded** — it pairs each parent with its nearest offspring, making the operation order-dependent. No parallelism is added there.

- **D-09:** All `par_sort_unstable_by` calls are gated behind `#[cfg(not(target_arch = "wasm32"))]` with a sequential `sort_unstable_by` fallback in the `#[cfg(target_arch = "wasm32")]` branch. WASM must compile and behave correctly.

### Benchmark Harness

- **D-10:** Create `benches/rastrigin.rs` as a new dedicated benchmark file. Do NOT add to `benches/ga_run.rs`.

- **D-11:** Use `RangeChromosome<f64>` with bounds `[-5.12, 5.12]` per gene. Parameterize over **three dimensionalities**: 10, 20, and 50. Population size is **500** for all configurations (the ROADMAP success criterion target). Run for a fixed generation count (planner decides exact value — enough to show meaningful wall-time at pop=500).

- **D-12:** The Rastrigin fitness function: `f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))` where `A=10`. Implemented inline in the benchmark file (not a new public function in the library).

- **D-13:** The benchmark must be run **before** and **after** the clone + parallelism changes to confirm ≥10% wall-time reduction. The CONTEXT.md does not pre-assume what the baseline is — the researcher and planner must treat this as a measurement-driven success criterion.

### Claude's Discretion

- Whether `use rayon::prelude::*` is added to each survivor file or imported at the call site
- Internal variable name for the captured `fitness_target` in the parallel sort comparator closure
- Whether the benchmark uses `BatchSize::SmallInput` or `BatchSize::LargeInput` (match pattern from `ga_run.rs`)
- Exact `max_generations` parameter for rastrigin bench (balance between warmup time and measurement signal)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### GA Hot Path (clone sites)
- `src/engines/ga.rs` lines 2910-2920 — crossover fallback path (`parent_1.clone()` / `parent_2.clone()`); primary clone reduction target
- `src/engines/ga.rs` lines 2280-2290 — `on_new_best` observer notification clone; eliminated by D-03
- `src/engines/ga.rs` line 2209 — `notify_stats` clone; eliminated by D-03

### Survivor Operators (parallelize)
- `src/operations/survivor/fitness.rs` — `fitness_based()`; receives `par_sort_unstable_by`
- `src/operations/survivor/mu_plus_lambda.rs` — `mu_plus_lambda()`; receives `par_sort_unstable_by`
- `src/operations/survivor/age.rs` — `age_based()`; receives `par_sort_unstable_by`
- `src/operations/survivor/mu_comma_lambda.rs` — `mu_comma_lambda()`; sort step receives `par_sort_unstable_by`
- `src/operations/survivor/deterministic_crowding.rs` — **DO NOT parallelize**; order-dependent

### Observer Trait (breaking change)
- `src/observe/observer.rs` (or equivalent) — `GaObserver<U>` trait; all `U` params → `&U`
- `src/observe/log_observer.rs` — `LogObserver` implementation; update signatures
- `src/observe/tracing_observer.rs` — `TracingObserver` implementation; update signatures (behind `observer-tracing` feature)
- `src/observe/metrics_observer.rs` — `MetricsObserver` implementation; update signatures (behind `observer-metrics` feature)
- `src/observe/composite.rs` — `CompositeObserver`; update signatures

### Benchmark
- `benches/ga_run.rs` — existing GA benchmark; pattern reference for `iter_batched`, `BenchmarkId`, `Criterion` setup
- `benches/rastrigin.rs` — **new file**; pop=500, dims=[10,20,50], `RangeChromosome<f64>`, Rastrigin fitness fn

### WASM Gating Pattern
- `src/engines/ga.rs` — existing `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` rayon gates; follow this exact pattern for survivor parallelism

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `rayon::slice::ParallelSliceMut::par_sort_unstable_by` — already a dependency; no new crate needed
- `src/operations/survivor/fitness.rs` — the `sort_by` comparator closures can be mechanically converted to `par_sort_unstable_by` with identical comparison logic
- `benches/ga_run.rs` `build_ga()` helper — reference for how to construct a `Ga` in a bench; `benches/rastrigin.rs` should follow the same `iter_batched` / `BatchSize` pattern

### Established Patterns
- `#[cfg(not(target_arch = "wasm32"))] ... #[cfg(target_arch = "wasm32")]` — duplicate the iterator expression only, keep the closure body shared (from CLAUDE.md WASM rules)
- `Option<Arc<dyn GaObserver<U> + Send + Sync>>` — observer ownership pattern; updating signatures does not change the ownership model
- Observer notification always goes through `self.notify(|obs| obs.on_*(...))`; update `notify` helper or each call site depending on how `notify` is defined

### Integration Points
- `src/engines/ga.rs` crossover inner loop (rayon parallel closure, ~lines 2895-2985) — crossover fallback restructuring happens here; must preserve `Send` bounds and rayon safety
- `src/operations/survivor.rs` dispatcher — routes to each `*_based()` function; no changes needed to the dispatcher itself
- `src/traits/observers.rs` (or wherever `GaObserver` is defined) — the breaking `&U` change originates here and propagates to all impls
- `cargo check --target wasm32-unknown-unknown` — must pass after all changes; run locally before marking phase complete

</code_context>

<specifics>
## Specific Ideas

- **Parallel sort pattern for `FixedFitness` branch**: the `fitness_distance()` comparator captures `target` — this works fine in a `par_sort_unstable_by` closure since `target: f64` is `Copy`.
- **`mu_comma_lambda` parallel sort**: the age==0 filter produces a sub-vec of offspring; apply `par_sort_unstable_by` to that sub-vec only (not the full input).
- **Observer `&U` change**: if `GaObserver::on_new_best` currently has signature `fn on_new_best(&self, generation: usize, best: U)`, it becomes `fn on_new_best(&self, generation: usize, best: &U)`. The caller site in `ga.rs` drops the `.clone()` call.

</specifics>

<deferred>
## Deferred Ideas

- **Selection output collect (line 3091)** — `indices.iter().map(|&i| chromosomes[i].clone()).collect()` — this clone is also in the hot path but was explicitly descoped. Future performance phase could switch to index-passing to defer clones to the crossover step.
- **`DeterministicCrowding` parallelism** — not parallelizable as-is due to parent-offspring pairing. A future restructure could make it eligible.
- **Observer async support** — switching to `&U` signatures opens the door for async observers without ownership transfer, but async trait support is deferred.

</deferred>

---

*Phase: 61-performance-clone-reduction-parallel-survivor*
*Context gathered: 2026-06-08*
