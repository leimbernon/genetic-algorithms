# Phase 60: Batch Fitness / Fitness Cache Extension - Research

**Researched:** 2026-06-07
**Domain:** Rust GA engine extension — fitness evaluation pipeline (batch evaluator trait, LRU cache stats, CMA engine wiring)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `BatchFitnessEvaluator<U: ChromosomeT>` is a public trait with signature `fn evaluate_batch(&self, chromosomes: &[U]) -> Vec<f64>`. Takes typed chromosomes (not DNA slices). `Send + Sync` required for `Arc<dyn ...>`.
- **D-02:** When a `BatchFitnessEvaluator` is configured on `Ga`, it **fully replaces** the individual-level `calculate_fitness()` path.
- **D-03:** Builder method `.with_batch_evaluator(Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>) -> Self` on both `Ga` and `CmaEngine`. Mutually exclusive with `fitness_fn`.
- **D-04:** `CmaEngine`'s run loop is modified to collect all offspring, then call `evaluate_batch` once for the full offspring slice. Individual-level `(self.fitness_fn)(ind.dna())` calls replaced.
- **D-05:** `CmaEngine` gains `.with_fitness_cache(size)` support with same `Arc<Mutex<FitnessCache>>` pattern.
- **D-06:** Batch + cache together: cache wraps the batch path — hits skip `evaluate_batch`, only misses are batched.
- **D-07:** `GenerationStats` gains `pub cache_hits: Option<u64>` and `pub cache_misses: Option<u64>` (delta per generation, `None` when no cache).
- **D-08:** `wrap_with_cache` refactored to return `(Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)`. `Ga` stores the handle and reads delta hits/misses around each generation loop.

### Claude's Discretion

- Whether `BatchFitnessEvaluator` lives in `src/traits/` or `src/fitness/` module.
- Internal variable names for the batch evaluation pass in `ga.rs`.
- How the builder signals mutual exclusivity between `fitness_fn` and `with_batch_evaluator` (panic vs `Result` vs `GaError` at `run()` time).
- Whether `CmaEngine` refactors fitness evaluation into a shared helper or duplicates the batch/cache logic inline.

### Deferred Ideas (OUT OF SCOPE)

- Batch evaluator or cache support for PSO, EDA, ALPS, ScatterSearch, CellularGA, DE engines.
- Async `BatchFitnessEvaluator`.
- Per-observer cache event hooks (`on_cache_hit` / `on_cache_miss`).
- `FitnessCache` with `Hash`-based keys.
</user_constraints>

---

## Summary

Phase 60 extends `Ga` and `CmaEngine` with two complementary fitness evaluation enhancements: a `BatchFitnessEvaluator<U>` trait that replaces per-chromosome evaluation with a single batch call, and `Arc<Mutex<FitnessCache>>` exposure so per-generation cache delta statistics can be written into `GenerationStats`.

The codebase is already well-prepared. `FitnessCache` (LRU) exists in `src/fitness/cache.rs` with `hits()`/`misses()` accessors. The `wrap_with_cache` function needs a signature change to also return the shared cache handle. The `Ga` struct already has a `fitness_cache_size: Option<usize>` field — it needs a companion `fitness_cache: Option<Arc<Mutex<FitnessCache>>>` field to hold the external reference after wrapping. `GenerationStats` needs two new `Option<u64>` fields with `serde(default)` and `serde(skip_serializing_if)` treatment matching the existing `avg_node_count` pattern.

The core complexity of the batch path in `Ga` is that fitness evaluation happens inside `parent_crossover` (a free function), not in the main generation loop. To fully replace `calculate_fitness()` with batch evaluation, one of two designs is needed: either (a) defer fitness assignment to after `parent_crossover` returns and insert a post-crossover batch-evaluate pass in the main loop, or (b) thread the `batch_evaluator` into `parent_crossover` alongside `fitness_fn`. Option (a) is cleaner — offspring can be returned without fitness set, then the batch call assigns all at once. This matches D-02 precisely.

**Primary recommendation:** Separate the "produce offspring" step from the "evaluate fitness" step in `Ga::run()`. When a batch evaluator is configured, call `evaluate_batch(&offspring)` once after `parent_crossover` returns, assign returned `Vec<f64>` values back, then skip `set_fitness_fn`/`calculate_fitness()` inside `parent_crossover` by passing `fitness_fn = None` when batch mode is active.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `BatchFitnessEvaluator<U>` trait definition | `src/fitness/` or `src/traits/` | — | Trait belongs alongside fitness abstractions; same tier as `FitnessFn<G>` alias |
| Batch evaluation pass in `Ga::run()` | `src/engines/ga.rs` | — | Engine orchestrates fitness; batch call replaces individual eval inside the generation loop |
| Cache delta stats collection in `Ga::run()` | `src/engines/ga.rs` | `src/fitness/cache.rs` | Engine reads from cache handle; cache holds the counters |
| `wrap_with_cache` refactor | `src/fitness/cache.rs` | — | Cache module owns the wrapping logic |
| `GenerationStats` new fields | `src/stats.rs` | — | Stats struct is the single source of per-generation metrics |
| CMA batch + cache wiring | `src/engines/cma/engine.rs` | `src/engines/cma/configuration.rs` | Engine loop + config builder |
| Re-exports | `src/lib.rs` | — | `pub use` for `BatchFitnessEvaluator` so users can import it cleanly |

---

## Standard Stack

No new external dependencies. This phase uses only existing codebase infrastructure:

| Asset | Location | Purpose |
|-------|----------|---------|
| `FitnessCache` | `src/fitness/cache.rs` | Existing LRU cache — reuse unchanged |
| `hash_dna<G: Debug>()` | `src/fitness/cache.rs` | DNA hashing for cache keys — reuse unchanged |
| `wrap_with_cache()` | `src/fitness/cache.rs` | Refactor return type to expose `Arc<Mutex<FitnessCache>>` |
| `Arc<dyn GaObserver<U> + Send + Sync>` | `src/engines/ga.rs` | Pattern to replicate for `BatchFitnessEvaluator` field |
| `GenerationStats` | `src/stats.rs` | Add two `Option<u64>` fields |
| `parent_crossover()` | `src/engines/ga.rs` line 2482 | Free function that currently sets fitness; batch path bypasses its fitness step |

**Installation:** No `cargo add` required.

---

## Package Legitimacy Audit

No external packages are added in this phase.

---

## Architecture Patterns

### System Architecture Diagram

```
Ga::run() generation loop
  │
  ├── [existing] parent_crossover(fitness_fn = None when batch_evaluator configured)
  │     └── returns Vec<U> with NO fitness set
  │
  ├── [NEW] if batch_evaluator configured:
  │     1. (optional) partition offspring by cache hit/miss
  │     2. call evaluate_batch(&miss_slice)  ← single call
  │     3. merge cached hits + new values back into offspring
  │     4. set offspring[i].set_fitness(values[i])
  │
  ├── [existing] if individual fitness_fn configured:
  │     └── (unchanged) calculate_fitness() per chromosome via set_fitness_fn
  │
  ├── [NEW] read cache delta (cache.hits() - prev_hits, etc.)
  │     └── stored in gen_stats.cache_hits / cache_misses
  │
  └── push gen_stats into self.stats
```

```
wrap_with_cache() [REFACTORED]
  Before: returns Arc<FitnessFn<G>>
  After:  returns (Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)

Ga::build()
  └── if fitness_cache_size set AND fitness_fn set:
        (wrapped_fn, cache_handle) = wrap_with_cache(fitness_fn, size)
        self.fitness_fn = Some(wrapped_fn)
        self.fitness_cache = Some(cache_handle)   ← NEW field
```

### Recommended Project Structure

No new directories required. New/modified files:

```
src/
├── fitness/
│   ├── cache.rs          # Refactor wrap_with_cache return type
│   └── mod.rs            # Add pub use BatchFitnessEvaluator (if defined here)
├── traits/               # OR: BatchFitnessEvaluator trait defined here
│   └── (new file or appended to existing)
├── stats.rs              # Add cache_hits / cache_misses fields
├── engines/
│   ├── ga.rs             # New field, batch path in run(), cache delta reads
│   └── cma/
│       ├── engine.rs     # Batch + cache wiring in run loop
│       └── configuration.rs  # New builder methods
└── lib.rs                # pub use BatchFitnessEvaluator

tests/
├── fitness/
│   └── (extend test_cache.rs with new wrap_with_cache signature)
├── engines/
│   ├── test_ga.rs        # Batch evaluator + cache stats tests for Ga
│   └── cma/
│       └── test_cma.rs   # Batch + cache tests for CmaEngine
```

### Pattern 1: BatchFitnessEvaluator Trait Definition

**What:** Public trait matching the existing `Arc<dyn ... + Send + Sync>` pattern for observer and repair operator.
**When to use:** Users implement this when fitness evaluation is vectorizable (GPU, REST API, external process).

```rust
// Source: modelled on existing GaObserver pattern in src/observer/mod.rs
pub trait BatchFitnessEvaluator<U: ChromosomeT>: Send + Sync {
    fn evaluate_batch(&self, chromosomes: &[U]) -> Vec<f64>;
}
```

The `BatchFitnessEvaluator` field on `Ga<U>` follows the exact pattern of `observer`:
```rust
// In Ga<U> struct (src/engines/ga.rs)
batch_evaluator: Option<Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>>,
fitness_cache: Option<Arc<Mutex<FitnessCache>>>,  // external handle from wrap_with_cache
```

### Pattern 2: Batch Evaluation Pass in Ga::run()

**What:** After `parent_crossover` produces offspring (without fitness when batch mode active), run one batch call.
**When to use:** When `self.batch_evaluator.is_some()`.

```rust
// Source: [ASSUMED] — derived from D-02 and existing ga.rs patterns

// Step A: produce offspring WITHOUT fitness (pass fitness_fn = None to parent_crossover)
let mut offspring = parent_crossover(
    &parents,
    &self.population.chromosomes,
    &self.configuration,
    age,
    self.population.f_max,
    self.population.f_avg,
    dynamic_prob,
    if self.batch_evaluator.is_some() { None } else { self.fitness_fn.clone() },
    // ... AOS params unchanged ...
)?;

// Step B: batch evaluate
if let Some(ref evaluator) = self.batch_evaluator {
    let fitness_values = evaluator.evaluate_batch(&offspring);
    debug_assert_eq!(fitness_values.len(), offspring.len(),
        "evaluate_batch must return exactly one fitness per chromosome");
    for (c, f) in offspring.iter_mut().zip(fitness_values) {
        c.set_fitness(f);
    }
}
```

### Pattern 3: Batch + Cache Partition (D-06)

**What:** When both batch evaluator and fitness cache are configured, check cache per chromosome, batch-evaluate only cache misses, merge results.

```rust
// Source: [ASSUMED] — derived from D-06 specification

if let (Some(ref evaluator), Some(ref cache_handle)) =
    (&self.batch_evaluator, &self.fitness_cache)
{
    let mut cache = cache_handle.lock().expect("fitness cache lock poisoned");
    let mut miss_indices: Vec<usize> = Vec::new();
    let mut fitness_values: Vec<f64> = vec![0.0; offspring.len()];

    // Partition: resolve hits, collect miss indices
    for (i, c) in offspring.iter().enumerate() {
        let key = crate::fitness::cache::hash_dna(c.dna());
        match cache.get(key) {
            Some(f) => fitness_values[i] = f,
            None => miss_indices.push(i),
        }
    }
    drop(cache); // release lock before potentially expensive batch call

    // Evaluate only misses
    if !miss_indices.is_empty() {
        let miss_slice: Vec<&U> = miss_indices.iter().map(|&i| &offspring[i]).collect();
        // BatchFitnessEvaluator takes &[U] — need owned slice or adjust
        let miss_chromosomes: Vec<U> = miss_indices.iter().map(|&i| offspring[i].clone()).collect();
        let miss_fitness = evaluator.evaluate_batch(&miss_chromosomes);

        let mut cache = cache_handle.lock().expect("fitness cache lock poisoned");
        for (idx, (&orig_i, &f)) in miss_indices.iter().zip(miss_fitness.iter()).enumerate() {
            fitness_values[orig_i] = f;
            let key = crate::fitness::cache::hash_dna(offspring[orig_i].dna());
            cache.put(key, f);
        }
    }

    // Assign fitness back
    for (c, f) in offspring.iter_mut().zip(fitness_values) {
        c.set_fitness(f);
    }
}
```

**Note for planner:** The clone of miss chromosomes above can be avoided if `evaluate_batch` is changed to take `&[&U]` — but D-01 locks the signature as `&[U]`. The clone is therefore mandatory for the miss-only call path. This is a known cost; document it in code comments.

### Pattern 4: wrap_with_cache Signature Change (D-08)

**What:** Return external cache handle alongside the wrapped function.

```rust
// src/fitness/cache.rs — refactored signature
pub fn wrap_with_cache<G>(
    fitness_fn: Arc<FitnessFn<G>>,
    cache_size: usize,
) -> (Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)
where
    G: GeneT + Debug + 'static,
{
    let cache = Arc::new(Mutex::new(FitnessCache::new(cache_size)));
    let cache_for_fn = Arc::clone(&cache);

    let wrapped = Arc::new(move |dna: &[G]| {
        let key = hash_dna(dna);
        {
            let mut c = cache_for_fn.lock().expect("fitness cache lock poisoned");
            if let Some(fitness) = c.get(key) {
                return fitness;
            }
        }
        let fitness = fitness_fn(dna);
        {
            let mut c = cache_for_fn.lock().expect("fitness cache lock poisoned");
            c.put(key, fitness);
        }
        fitness
    });

    (wrapped, cache)
}
```

**Call site in `Ga::build()`:**
```rust
if let Some(cache_size) = self.fitness_cache_size {
    if let Some(fitness_fn) = self.fitness_fn.take() {
        let (wrapped, cache_handle) =
            crate::fitness::cache::wrap_with_cache(fitness_fn, cache_size);
        self.fitness_fn = Some(wrapped);
        self.fitness_cache = Some(cache_handle);
    }
}
```

### Pattern 5: GenerationStats New Fields

**What:** Add `cache_hits` and `cache_misses` with the same serde treatment as `avg_node_count`.

```rust
// src/stats.rs
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenerationStats {
    // ... existing fields ...
    /// Cache hits during this generation (delta). None when no cache configured.
    #[cfg_attr(feature = "serde", serde(default))]
    pub cache_hits: Option<u64>,
    /// Cache misses during this generation (delta). None when no cache configured.
    #[cfg_attr(feature = "serde", serde(default))]
    pub cache_misses: Option<u64>,
}
```

Both fields must be added to every construction site:
- `GenerationStats::from_fitness_values()` — add `cache_hits: None, cache_misses: None`
- The empty-population early-return inside `from_fitness_values()` — same

### Pattern 6: Cache Delta Reading in Ga::run()

**What:** Read `cache.hits()` and `cache.misses()` at generation start and end, compute delta.

```rust
// At generation loop start (before parent selection)
let (prev_cache_hits, prev_cache_misses): (u64, u64) =
    if let Some(ref ch) = self.fitness_cache {
        let c = ch.lock().expect("fitness cache lock poisoned");
        (c.hits(), c.misses())
    } else {
        (0, 0)
    };

// ... generation body ...

// Before push to self.stats
if let Some(ref ch) = self.fitness_cache {
    let c = ch.lock().expect("fitness cache lock poisoned");
    gen_stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
    gen_stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
}
```

### Pattern 7: CmaEngine Batch + Cache Wiring

**What:** `CmaEngine` has two fitness eval sites:
1. Initial population eval (line ~563): `for ind in &mut pop { let f = (self.fitness_fn)(ind.dna()); ... }`
2. Per-generation offspring loop (lines ~609-632): offspring are built and evaluated one by one.

For batch support, site 2 is restructured: build all offspring first (no fitness), then call `evaluate_batch` or individual `fitness_fn` once all are collected.

```rust
// CmaEngine struct additions
pub struct CmaEngine<U: LinearChromosome> {
    // ... existing fields ...
    batch_evaluator: Option<Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>>,
    fitness_cache_size: Option<usize>,
    // Runtime — populated in run():
    fitness_cache: Option<Arc<Mutex<FitnessCache>>>,
}
```

Site 1 (initial population) needs special handling: `BatchFitnessEvaluator` works on whole chromosomes, so pass the full `pop` slice. Site 2 (offspring loop): collect all offspring first, then batch-evaluate.

### Anti-Patterns to Avoid

- **Calling `calculate_fitness()` on offspring when batch mode is active:** `calculate_fitness()` uses the chromosome's internally stored `fitness_fn`, not the batch evaluator. When batch mode is configured, `fitness_fn` is `None` (not set via `set_fitness_fn`), so `calculate_fitness()` would be a no-op or return default fitness. Never rely on `calculate_fitness()` in batch paths.
- **Holding cache lock across `evaluate_batch` call:** `evaluate_batch` may be expensive (GPU, API call). The cache lock must be released before calling the batch evaluator. Partition → release lock → call batch → re-acquire lock → store results.
- **Passing `fitness_fn` to `parent_crossover` when batch mode active:** The `fitness_fn` inside `parent_crossover` is cloned per-child via `set_fitness_fn` then `calculate_fitness()`. When batch mode is on, pass `None` as `fitness_fn` to suppress per-child evaluation entirely.
- **Omitting `serde(default)` on new `GenerationStats` fields:** Without `serde(default)`, old checkpoints deserialized after this phase will fail. Follow the `avg_node_count` pattern exactly.
- **Mutually exclusive validation at `build()` vs `run()`:** D-03 says both `fitness_fn` and `with_batch_evaluator` cannot be set simultaneously. The planner can choose panic or `GaError` at `build()` time — `run()` time is also valid but `build()` is preferable (earlier feedback). Either way it must be consistent for both `Ga` and `CmaEngine`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LRU cache | Custom eviction | `FitnessCache` in `src/fitness/cache.rs` | Already implemented with proper LRU ordering via `HashMap` + `VecDeque` |
| DNA hashing | Custom hasher | `hash_dna<G: Debug>()` in `src/fitness/cache.rs` | Handles `Range<f64>` which doesn't implement `Hash`; uses `Debug` repr |
| Trait-object field pattern | Custom dispatch | Follow `Option<Arc<dyn GaObserver<U> + Send + Sync>>` pattern | Established zero-overhead optional field pattern in this codebase |

**Key insight:** The `FitnessCache` implementation is already correct. The only structural change needed is exposing the cache handle externally via the refactored `wrap_with_cache` return type.

---

## Common Pitfalls

### Pitfall 1: `parent_crossover` Sets Fitness Inline

**What goes wrong:** `parent_crossover` (line 2753–2762) calls `set_fitness_fn` then `calculate_fitness()` on every child. If a `batch_evaluator` is configured but `fitness_fn` is also passed, children get individual evaluation instead of batch.
**Why it happens:** The batch evaluator is a new field; `parent_crossover` doesn't know about it.
**How to avoid:** Pass `fitness_fn = None` to `parent_crossover` when `self.batch_evaluator.is_some()`. Children return with fitness = 0.0 (default); the batch pass overwrites it.
**Warning signs:** Test shows fitness = 0.0 for all offspring; batch evaluator call count = 0.

### Pitfall 2: `wrap_with_cache` Return Type is a Breaking Change for Call Sites

**What goes wrong:** The existing call in `Ga::build()` (line 764) destructures the return of `wrap_with_cache`. After the refactor to return a tuple, any call that assigns the result to `self.fitness_fn` directly will fail to compile.
**Why it happens:** Signature change from `Arc<FitnessFn<G>>` to `(Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)`.
**How to avoid:** Search for all call sites of `wrap_with_cache` before writing the refactored version. Currently there is only one call site: `Ga::build()` at line ~764.
**Warning signs:** `cargo build` fails with type mismatch at the `Ga::build()` cache-wrapping block.

### Pitfall 3: Cache Delta Goes Negative Under Saturating Subtraction

**What goes wrong:** If the cache is cleared or replaced between generations (not applicable here, but logically possible), `prev_hits > current_hits`, leading to underflow.
**Why it happens:** `u64` subtraction panics in debug mode on underflow.
**How to avoid:** Use `saturating_sub`. `u64::saturating_sub` returns 0 on underflow instead of panicking.
**Warning signs:** `thread 'main' panicked: 'attempt to subtract with overflow'` in debug builds.

### Pitfall 4: Initial Population in `Ga::initialize_random()` Uses Individual Eval

**What goes wrong:** Batch evaluator skips initial population — chromosomes start with default 0.0 fitness.
**Why it happens:** `initialize_random()` and `initialize_with_seeds()` call `calculate_fitness()` per chromosome via `set_fitness_fn`. These code paths don't know about the batch evaluator.
**How to avoid:** After `self.initialization()` returns, check for `batch_evaluator` and run a batch pass over `self.population.chromosomes`. The same logic applies to initial population as to offspring. Add a `self.batch_evaluate_population()` helper called after initialization and after each generation's offspring evaluation.
**Warning signs:** First-generation stats show default fitness (0.0) for all individuals; subsequent generations correct themselves.

### Pitfall 5: CmaEngine Has Two Independent Fitness Eval Sites

**What goes wrong:** Modifying only the offspring loop (line ~630) misses the initial population eval (line ~563). Initial population gets individual evaluation even when batch mode configured.
**Why it happens:** CMA-ES evaluates initial population separately from the per-generation offspring.
**How to avoid:** Address both sites. Site 1 (initial): collect `pop` after `init_fn`, call `evaluate_batch(&pop)`. Site 2 (offspring): collect all offspring first, then call `evaluate_batch(&offspring)`.
**Warning signs:** Initial population has incorrect fitness when batch evaluator is configured.

### Pitfall 6: `serde` Feature Breaks Checkpoint Deserialization

**What goes wrong:** Old checkpoints (JSON files saved before Phase 60) fail to deserialize after adding `cache_hits`/`cache_misses` to `GenerationStats`.
**Why it happens:** `serde` by default requires all fields to be present.
**How to avoid:** Add `#[cfg_attr(feature = "serde", serde(default))]` to both new fields. This matches the exact pattern used for `dynamic_mutation_probability` and `avg_node_count` in `GenerationStats`.
**Warning signs:** `cargo test --features serde` fails with deserialization errors.

### Pitfall 7: WASM Compatibility for FitnessCache

**What goes wrong:** `std::sync::Mutex` is available on WASM but `std::time::Instant` is not. The cache path itself has no timing — it is WASM-safe as designed. The risk is accidentally importing `Instant` in a cache-adjacent code path.
**Why it happens:** Easy to accidentally add timing metrics to the new batch path.
**How to avoid:** Never add `Instant::now()` in the batch evaluation path unconditionally. If timing is needed, gate it `#[cfg(not(target_arch = "wasm32"))]`. Run `cargo check --target wasm32-unknown-unknown` before considering the feature complete.

---

## Code Examples

### Existing Cache Call Site (will change)

```rust
// src/engines/ga.rs, lines 762-768 [VERIFIED: read from source]
// Before refactor:
if let Some(cache_size) = self.fitness_cache_size {
    if let Some(fitness_fn) = self.fitness_fn.take() {
        self.fitness_fn = Some(crate::fitness::cache::wrap_with_cache(
            fitness_fn, cache_size,
        ));
    }
}
// After refactor: destructure tuple, store cache handle in self.fitness_cache
```

### Existing GenerationStats Construction Pattern

```rust
// src/stats.rs, lines 107-117 [VERIFIED: read from source]
// New fields added to all construction sites:
GenerationStats {
    generation,
    best_fitness: best,
    // ... existing fields ...
    avg_node_count: 0.0,
    cache_hits: None,    // NEW
    cache_misses: None,  // NEW
}
```

### Observer Pattern (model for BatchFitnessEvaluator field)

```rust
// src/engines/ga.rs, line 278 [VERIFIED: read from source]
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

// Notify helper pattern (from EdaEngine, src/engines/eda/engine.rs):
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### CmaEngine Fitness Eval Sites (both need patching)

```rust
// src/engines/cma/engine.rs, line ~563 [VERIFIED: read from source]
// Site 1 - Initial population:
for ind in &mut pop {
    let f = (self.fitness_fn)(ind.dna());
    ind.set_fitness(f);
}

// src/engines/cma/engine.rs, lines ~629-631 [VERIFIED: read from source]
// Site 2 - Per-offspring (inside offspring build loop):
let f = (self.fitness_fn)(child.dna());
child.set_fitness(f);
offspring.push(child);
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| `wrap_with_cache` returns `Arc<FitnessFn<G>>` only | Returns `(Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)` | Phase 60 | Enables external delta stats |
| `Ga` has no batch evaluation | `Ga` holds `batch_evaluator: Option<Arc<dyn BatchFitnessEvaluator<U>>>` | Phase 60 | GPU/API-backed fitness |
| `GenerationStats` has no cache fields | Adds `cache_hits: Option<u64>`, `cache_misses: Option<u64>` | Phase 60 | Cache efficiency visible in observer hooks |
| `CmaEngine` has no cache support | `CmaEngine` gains `.with_fitness_cache(size)` | Phase 60 | CMA-ES benefits from cache on repeated function evaluations |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Passing `fitness_fn = None` to `parent_crossover` suppresses per-child evaluation cleanly (children with no fitness fn return default 0.0 from `calculate_fitness()`) | Architecture Patterns / Pattern 2 | If chromosome's default fitness is not 0.0, or if `calculate_fitness()` panics without a fitness fn, batch pass would not overwrite correctly. Verify `ChromosomeT::calculate_fitness()` behavior when no fn is set. |
| A2 | `CmaEngine` fields are `pub` or accessible for adding `batch_evaluator` and `fitness_cache` — or the engine can be extended via builder pattern without restructuring | Architecture / CmaEngine | If `CmaEngine` uses a sealed struct pattern, adding new fields may require deeper refactoring |
| A3 | Miss-only batch call requires cloning miss chromosomes because `evaluate_batch(&[U])` takes owned slice — this is the only way to satisfy the D-01 signature with a non-contiguous miss sub-slice | Patterns / Pattern 3 | If implementors of `evaluate_batch` are GPU-backed, the clone overhead is negligible vs GPU latency. But if planner wants to avoid the clone, D-01 signature would need `&[&U]`. Since D-01 is locked, clone is the correct approach. |

---

## Open Questions

1. **Mutual exclusivity mechanism for `fitness_fn` + `with_batch_evaluator`**
   - What we know: D-03 says they are mutually exclusive; implementation mechanism is Claude's discretion.
   - What's unclear: `build()` panic vs `GaError` return.
   - Recommendation: Return `Err(GaError::ConfigurationError("..."))` from `build()` — consistent with other validation in `build()` (e.g., seed/checkpoint mutual exclusivity at line 774).

2. **Initial population batch evaluation in `Ga`**
   - What we know: `initialize_random()` and `initialize_with_seeds()` call `calculate_fitness()` per chromosome (Pitfall 4 above). These are called from `self.initialization()` within `run()`.
   - What's unclear: Whether the planner wants a separate `batch_evaluate_population()` helper or inline batch logic after `self.initialization()`.
   - Recommendation: Add a private `fn batch_evaluate_pop(&self, pop: &mut Vec<U>)` helper that handles the null-evaluator case (no-op) and the batch evaluator case uniformly. Call it after `self.initialization()` and after `parent_crossover` returns offspring.

3. **`BatchFitnessEvaluator` module placement** (Claude's discretion)
   - Recommendation: Place in `src/fitness/mod.rs` or a new `src/fitness/batch.rs`. The `FitnessFn<G>` alias lives in `src/traits/common.rs`, but that module is focused on internal type aliases. `BatchFitnessEvaluator` is a user-facing trait — `src/fitness/` is the right home. Export via `src/fitness/mod.rs` and re-export in `src/lib.rs`.

---

## Environment Availability

Step 2.6: SKIPPED — this phase has no external dependencies. All changes are pure code/config modifications to existing modules.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`cargo test`) + criterion for benchmarks |
| Config file | `Cargo.toml` (workspace root) |
| Quick run command | `cargo test --test test_fitness` or `cargo test batch_eval` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | `BatchFitnessEvaluator::evaluate_batch` called once; `calculate_fitness` NOT called | unit | `cargo test --test test_ga batch_eval` | No — Wave 0 |
| SC-1 | `Ga` builder method `.with_batch_evaluator(...)` accepted; run completes | integration | `cargo test --test test_ga with_batch_evaluator` | No — Wave 0 |
| SC-2 | `FitnessCache` enabled on `Ga`; unchanged-DNA chromosomes served from cache | unit | `cargo test --test test_ga fitness_cache_delta` | No — Wave 0 |
| SC-2 | `cache_hits`/`cache_misses` populated in `GenerationStats` when cache active | unit | `cargo test --test test_ga cache_stats_in_gen_stats` | No — Wave 0 |
| SC-2 | `cache_hits`/`cache_misses` are `None` in `GenerationStats` when no cache | unit | `cargo test --test test_stats` | Exists — extend |
| SC-3 | WASM: batch path compiles for `wasm32-unknown-unknown` | compile-only | `cargo check --target wasm32-unknown-unknown` | CI gate |
| SC-4 | `cargo clippy` zero warnings | lint | `cargo clippy` | CI gate |
| CMA | `CmaEngine` `.with_fitness_cache(size)` accepted; runs | integration | `cargo test --test test_cma with_fitness_cache` | Exists — extend |
| CMA | `CmaEngine` `with_batch_evaluator` — batch called once per generation | integration | `cargo test --test test_cma batch_evaluator` | Exists — extend |
| CACHE | `wrap_with_cache` returns tuple — call site compiles and cache handle accessible | unit | `cargo test --test test_fitness cache_handle` | Exists — extend |
| SERDE | Old `GenerationStats` JSON with no `cache_hits`/`cache_misses` deserializes OK | unit | `cargo test --features serde --test test_stats serde_compat` | No — Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --test test_fitness && cargo test --test test_ga batch` (< 10s)
- **Per wave merge:** `cargo test && cargo test --features serde`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/test_ga.rs` — new test functions for `batch_evaluator` and `cache_stats_in_gen_stats` (file exists at 70K, add new test mod section)
- [ ] `tests/engines/cma/test_cma.rs` — new test functions for CMA batch + cache (file exists, extend)
- [ ] `tests/fitness/test_cache.rs` — extend with `wrap_with_cache` tuple return test (file structure known from `tests/test_fitness.rs`)
- [ ] `tests/test_stats.rs` — extend with serde compatibility test for new `cache_hits`/`cache_misses` fields

---

## Security Domain

This phase introduces no authentication, session management, access control, cryptographic, or input-from-network code paths. The new fitness evaluation path accepts user-supplied `Arc<dyn BatchFitnessEvaluator<U>>` — this is a trust-boundary-internal extension (same threat model as existing `Arc<dyn GaObserver<U>>`). No ASVS categories apply.

---

## Sources

### Primary (HIGH confidence)

- `src/fitness/cache.rs` — `FitnessCache` implementation, `wrap_with_cache` signature, `hash_dna` — read directly
- `src/engines/ga.rs` — `Ga` struct fields, `build()` cache wiring (line 762), `parent_crossover` fitness assignment (lines 2753-2762), generation loop stats construction (line 1920), `with_fitness_cache_size` builder (line 905)
- `src/engines/cma/engine.rs` — CMA fitness eval sites (lines 563, 630), struct layout
- `src/engines/cma/configuration.rs` — `CmaConfiguration` builder methods, struct fields
- `src/stats.rs` — `GenerationStats` struct, serde attribute pattern, `from_fitness_values()` construction
- `src/traits/common.rs` — `FitnessFn<G>` type alias
- `.planning/phases/60-batch-fitness-fitness-cache-extension/60-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)

- `src/engines/eda/engine.rs` — most recent engine; observer wiring pattern confirmed
- `src/engines/ga.rs` struct field block (lines 260-330) — `observer` field as `BatchFitnessEvaluator` model

### Tertiary (LOW confidence)

- None — all claims verified from source code read.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all verified from codebase
- Architecture: HIGH — locked decisions in CONTEXT.md, verified against actual source code line numbers
- Pitfalls: HIGH — derived from direct code reading of call sites and existing patterns
- Test gaps: HIGH — test file structure verified via `ls` and file reads

**Research date:** 2026-06-07
**Valid until:** Stable (Rust library; no external ecosystem churn risk)
