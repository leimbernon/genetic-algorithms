# Phase 60: Batch Fitness / Fitness Cache Extension - Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/fitness/cache.rs` | utility | transform | self (refactor) | exact |
| `src/stats.rs` | model | transform | self (extend) | exact |
| `src/engines/ga.rs` | engine/orchestrator | CRUD + batch | self (extend) | exact |
| `src/engines/cma/engine.rs` | engine/orchestrator | batch | `src/engines/eda/engine.rs` | role-match |
| `src/engines/cma/configuration.rs` | config | request-response | self (extend) | exact |
| `src/lib.rs` | config | — | self (extend) | exact |
| `tests/engines/test_ga.rs` (extend) | test | — | existing test file | exact |
| `tests/engines/cma/test_cma.rs` (extend) | test | — | existing test file | exact |

---

## Pattern Assignments

### `src/fitness/cache.rs` — refactor `wrap_with_cache` return type

**Analog:** `src/fitness/cache.rs` (self — signature change only)

**Current signature** (lines 119–147):
```rust
pub fn wrap_with_cache<G>(fitness_fn: Arc<FitnessFn<G>>, cache_size: usize) -> Arc<FitnessFn<G>>
where
    G: GeneT + Debug + 'static,
{
    let cache = Arc::new(Mutex::new(FitnessCache::new(cache_size)));

    Arc::new(move |dna: &[G]| {
        let key = hash_dna(dna);
        {
            let mut cache = cache.lock().expect("fitness cache lock poisoned");
            if let Some(fitness) = cache.get(key) {
                return fitness;
            }
        }
        let fitness = fitness_fn(dna);
        {
            let mut cache = cache.lock().expect("fitness cache lock poisoned");
            cache.put(key, fitness);
        }
        fitness
    })
}
```

**New signature pattern** — return tuple, expose external handle:
```rust
pub fn wrap_with_cache<G>(
    fitness_fn: Arc<FitnessFn<G>>,
    cache_size: usize,
) -> (Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)
where
    G: GeneT + Debug + 'static,
{
    let cache = Arc::new(Mutex::new(FitnessCache::new(cache_size)));
    let cache_for_fn = Arc::clone(&cache);
    // ... closure body unchanged ...
    (wrapped, cache)
}
```

**Cache accessors already present** (lines 77–84):
```rust
pub fn hits(&self) -> u64 { self.hits }
pub fn misses(&self) -> u64 { self.misses }
```

---

### `src/stats.rs` — add `cache_hits` / `cache_misses` fields

**Analog:** `src/stats.rs` lines 41–53 — existing `Option<f64>` / `Option<u64>` optional fields with serde pattern

**Existing serde pattern to copy** (lines 41–53):
```rust
/// Current dynamic mutation probability, if dynamic mutation is enabled.
/// `None` when dynamic mutation is disabled.
#[cfg_attr(feature = "serde", serde(default))]
pub dynamic_mutation_probability: Option<f64>,
/// Average node count across the surviving population (GP only).
///
/// Set to `0.0` for non-GP engines. Used by `GpGa` for bloat monitoring
/// (CHR-05). The `serde(default)` attribute ensures backward-compatible
/// deserialization of checkpoints created before this field was added.
#[cfg_attr(feature = "serde", serde(default))]
pub avg_node_count: f64,
```

**New fields follow the exact same pattern:**
```rust
/// LRU cache hits during this generation (delta). `None` when no cache configured.
#[cfg_attr(feature = "serde", serde(default))]
pub cache_hits: Option<u64>,
/// LRU cache misses during this generation (delta). `None` when no cache configured.
#[cfg_attr(feature = "serde", serde(default))]
pub cache_misses: Option<u64>,
```

**Construction sites to update** — both must add `cache_hits: None, cache_misses: None`:

Site 1 — empty-population early return (lines 66–78):
```rust
return GenerationStats {
    generation,
    best_fitness: 0.0,
    worst_fitness: 0.0,
    avg_fitness: 0.0,
    fitness_std_dev: 0.0,
    population_size: 0,
    diversity: 0.0,
    dynamic_mutation_probability: None,
    avg_node_count: 0.0,
    // ADD:
    cache_hits: None,
    cache_misses: None,
};
```

Site 2 — normal construction (lines 107–118):
```rust
GenerationStats {
    generation,
    best_fitness: best,
    worst_fitness: worst,
    avg_fitness: avg,
    fitness_std_dev: std_dev,
    population_size: n,
    diversity: std_dev,
    dynamic_mutation_probability: None,
    avg_node_count: 0.0,
    // ADD:
    cache_hits: None,
    cache_misses: None,
}
```

---

### `src/engines/ga.rs` — new fields + batch path + cache delta stats

**Analog:** `src/engines/ga.rs` — existing `observer` field, `fitness_cache_size` field, and `parent_crossover` fitness-assignment pattern

**Existing Arc trait-object field pattern** (lines 278 + 274):
```rust
/// Optional LRU fitness cache size.
fitness_cache_size: Option<usize>,

/// Optional structured lifecycle observer.
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
```

**New fields follow the same pattern — add after `fitness_cache_size`:**
```rust
/// External handle to the LRU fitness cache, populated by `build()` when
/// `fitness_cache_size` is set. Used to read per-generation delta stats.
fitness_cache: Option<Arc<Mutex<FitnessCache>>>,

/// Optional batch fitness evaluator. When set, fully replaces the
/// individual-level `calculate_fitness()` path. Mutually exclusive with
/// `fitness_fn`.
batch_evaluator: Option<Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>>,
```

**Existing builder pattern for optional fields** (lines 893–908):
```rust
/// Enables an LRU fitness cache with the given capacity.
pub fn with_fitness_cache_size(mut self, size: usize) -> Self {
    self.fitness_cache_size = Some(size);
    self
}
```

**New builder method follows the same shape:**
```rust
/// Attaches a batch fitness evaluator. Mutually exclusive with a scalar `fitness_fn`.
pub fn with_batch_evaluator(
    mut self,
    evaluator: Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>,
) -> Self {
    self.batch_evaluator = Some(evaluator);
    self
}
```

**Existing cache-wiring call site in `build()`** (lines 761–768) — this is the one call site for `wrap_with_cache`, must be updated to destructure tuple:
```rust
// Before:
if let Some(cache_size) = self.fitness_cache_size {
    if let Some(fitness_fn) = self.fitness_fn.take() {
        self.fitness_fn = Some(crate::fitness::cache::wrap_with_cache(
            fitness_fn, cache_size,
        ));
    }
}

// After refactor (D-08):
if let Some(cache_size) = self.fitness_cache_size {
    if let Some(fitness_fn) = self.fitness_fn.take() {
        let (wrapped, cache_handle) =
            crate::fitness::cache::wrap_with_cache(fitness_fn, cache_size);
        self.fitness_fn = Some(wrapped);
        self.fitness_cache = Some(cache_handle);
    }
}
```

**Mutual exclusivity validation pattern** — copy from seeds/checkpoint check (lines 773–779):
```rust
if self.seeds.is_some() && self.checkpoint_path.is_some() {
    return Err(GaError::ConfigurationError(
        "Cannot use both with_seeds() and with_checkpoint() — they are mutually exclusive"
            .to_string(),
    ));
}
// New (in build(), after cache wiring):
if self.fitness_fn.is_some() && self.batch_evaluator.is_some() {
    return Err(GaError::ConfigurationError(
        "Cannot use both fitness_fn and with_batch_evaluator() — they are mutually exclusive"
            .to_string(),
    ));
}
```

**`parent_crossover` fitness-assignment site** (lines 2753–2762) — batch path passes `None` as `fitness_fn`:
```rust
// Current: inject and calculate fitness inline
if let Some(ref ff) = fitness_fn {
    let ff1 = Arc::clone(ff);
    child_1.set_fitness_fn(move |genes| ff1(genes));
    let ff2 = Arc::clone(ff);
    child_2.set_fitness_fn(move |genes| ff2(genes));
}
child_1.calculate_fitness();
child_2.calculate_fitness();
```
When `self.batch_evaluator.is_some()`, pass `fitness_fn = None` to `parent_crossover` — children return with default 0.0 fitness, then the batch pass overwrites.

**Cache delta reading pattern** — wrap `gen_stats` construction (around line 1919):
```rust
// At generation loop start (before parent selection):
let (prev_cache_hits, prev_cache_misses): (u64, u64) =
    if let Some(ref ch) = self.fitness_cache {
        let c = ch.lock().expect("fitness cache lock poisoned");
        (c.hits(), c.misses())
    } else {
        (0, 0)
    };

// ... generation body (parent_crossover, survivors, etc.) ...

// After gen_stats constructed (around line 1919), before push:
if let Some(ref ch) = self.fitness_cache {
    let c = ch.lock().expect("fitness cache lock poisoned");
    gen_stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
    gen_stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
}
```

**Dynamic mutation probability pattern to copy for gen_stats field assignment** (lines 1954–1955):
```rust
// Set the field directly on gen_stats before push (no last_mut needed)
gen_stats.dynamic_mutation_probability = Some(self.dynamic_mutation_probability);
// Same style for cache fields:
gen_stats.cache_hits = Some(delta_hits);
gen_stats.cache_misses = Some(delta_misses);
```

**WASM gate pattern for rayon** (lines 1142–1176) — batch evaluation path is inherently sequential (no rayon), so no gate required for the batch call itself:
```rust
#[cfg(not(target_arch = "wasm32"))]
let result: Vec<U> = items.into_par_iter().map(|_| { ... }).collect();
#[cfg(target_arch = "wasm32")]
let result: Vec<U> = items.into_iter().map(|_| { ... }).collect();
```

---

### `src/engines/cma/engine.rs` — add `batch_evaluator` field + batch path at both eval sites

**Analog:** `src/engines/eda/engine.rs` — most recent engine; observer field + `notify()` helper pattern

**EDA engine struct layout for optional Arc trait-object** (lines 107–112):
```rust
pub struct EdaEngine<U: LinearChromosome> {
    config: EdaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}
```

**New fields for `CmaEngine` — same optional Arc pattern:**
```rust
batch_evaluator: Option<Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>>,
fitness_cache_size: Option<usize>,
// Runtime-only — populated in run():
fitness_cache: Option<Arc<Mutex<FitnessCache>>>,
```

**EDA `notify()` helper** (lines 152–157) — `CmaEngine` already has the same helper; use it for any new observer calls:
```rust
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

**CMA fitness eval site 1 — initial population** (lines 562–566):
```rust
// Before:
for ind in &mut pop {
    let f = (self.fitness_fn)(ind.dna());
    ind.set_fitness(f);
}

// After — batch path replaces inline eval:
if let Some(ref evaluator) = self.batch_evaluator {
    let fitness_values = evaluator.evaluate_batch(&pop);
    for (ind, f) in pop.iter_mut().zip(fitness_values) {
        ind.set_fitness(f);
    }
} else {
    for ind in &mut pop {
        let f = (self.fitness_fn)(ind.dna());
        ind.set_fitness(f);
    }
}
```

**CMA fitness eval site 2 — per-generation offspring** (lines 609–633):
```rust
// Before — inline per-offspring:
let mut child = template.clone();
child.set_dna(Cow::Owned(new_dna));
let f = (self.fitness_fn)(child.dna());
child.set_fitness(f);
offspring.push(child);

// After — collect without fitness, then batch evaluate:
let mut child = template.clone();
child.set_dna(Cow::Owned(new_dna));
// Do NOT evaluate fitness here when batch mode active
offspring.push(child);
// ... end of offspring build loop ...

// Batch evaluate all offspring at once:
if let Some(ref evaluator) = self.batch_evaluator {
    let fitness_values = evaluator.evaluate_batch(&offspring);
    for (c, f) in offspring.iter_mut().zip(fitness_values) {
        c.set_fitness(f);
    }
} else {
    for c in offspring.iter_mut() {
        let f = (self.fitness_fn)(c.dna());
        c.set_fitness(f);
    }
}
```

**Cache wiring in `run()`** — same pattern as `Ga::build()` but done at `run()` start since `CmaEngine` has no separate `build()`:
```rust
// At run() start, after fitness_fn is confirmed present:
if let Some(cache_size) = self.fitness_cache_size {
    let (wrapped, cache_handle) =
        crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), cache_size);
    self.fitness_fn = wrapped;
    self.fitness_cache = Some(cache_handle);
}
```

---

### `src/engines/cma/configuration.rs` — add builder methods

**Analog:** `src/engines/cma/configuration.rs` lines 116–197 — existing builder method pattern

**Existing builder method shape** (lines 119–122):
```rust
/// Builder: set initial step size σ₀.
pub fn with_sigma0(mut self, s: f64) -> Self {
    self.sigma0 = s;
    self
}
```

**New methods follow the same shape exactly:**
```rust
/// Builder: set fitness cache size.
///
/// When set, fitness evaluations are cached by DNA hash. Useful when CMA-ES
/// repeatedly evaluates chromosomes with similar or identical gene values.
pub fn with_fitness_cache(mut self, size: usize) -> Self {
    self.fitness_cache_size = Some(size);
    self
}

/// Builder: attach a batch fitness evaluator.
///
/// When set, replaces the individual-level fitness function entirely.
/// Mutually exclusive with a scalar `fitness_fn`.
pub fn with_batch_evaluator(
    mut self,
    evaluator: Arc<dyn BatchFitnessEvaluator<U> + Send + Sync>,
) -> Self {
    self.batch_evaluator = Some(evaluator);
    self
}
```

Note: `CmaConfiguration` holds only plain data (no generics). `batch_evaluator` must live on `CmaEngine` directly (as a field), not in `CmaConfiguration`, since `CmaConfiguration` is not generic over `U`. Add `fitness_cache_size: Option<usize>` to `CmaConfiguration`; add `batch_evaluator` as a field on `CmaEngine` itself with its own builder method on `CmaEngine`.

---

### `src/lib.rs` — add `pub use BatchFitnessEvaluator`

**Analog:** `src/lib.rs` lines 347–363 — observer re-export block

**Existing re-export pattern:**
```rust
pub use observer::GaObserver;
pub use observer::NoopObserver;
// etc.
```

**New re-export — add to the fitness / traits block:**
```rust
pub use fitness::BatchFitnessEvaluator;
```

Or if placed in `src/traits/`:
```rust
pub use traits::BatchFitnessEvaluator;
```

---

### `src/fitness/` — new `BatchFitnessEvaluator` trait

**Analog:** `src/observe/observer/mod.rs` line 66 — `GaObserver` trait declaration pattern

**`GaObserver` pattern:**
```rust
pub trait GaObserver<U: ChromosomeT>: Send + Sync {
    fn on_run_start(&self) {}
    // ...
}
```

**`BatchFitnessEvaluator` follows the same shape:**
```rust
/// Trait for batch fitness evaluation — replaces the per-chromosome path.
///
/// Implement this trait when fitness evaluation can be vectorized (GPU kernels,
/// REST API calls, external process pipes). When configured on `Ga` or
/// `CmaEngine`, `evaluate_batch` is called once per generation with all
/// offspring; the individual-level `calculate_fitness()` path is never called.
///
/// # Contract
///
/// The returned `Vec<f64>` must have exactly `chromosomes.len()` elements.
/// `fitness_values[i]` is assigned to `chromosomes[i]`.
pub trait BatchFitnessEvaluator<U: ChromosomeT>: Send + Sync {
    fn evaluate_batch(&self, chromosomes: &[U]) -> Vec<f64>;
}
```

**Placement decision (Claude's discretion):** Put in `src/fitness/batch.rs`, declare `pub mod batch;` in `src/fitness` inline declarations, re-export as `pub use batch::BatchFitnessEvaluator` from `src/fitness`.

---

## Shared Patterns

### Arc Trait-Object Optional Field
**Source:** `src/engines/ga.rs` line 278 + `src/engines/eda/engine.rs` lines 111
**Apply to:** `batch_evaluator` field on both `Ga` and `CmaEngine`
```rust
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
// pattern: Option<Arc<dyn Trait<U> + Send + Sync>>
```

### Mutex Lock with Expect
**Source:** `src/fitness/cache.rs` lines 130, 141
**Apply to:** Every `fitness_cache.lock()` call site
```rust
let mut cache = cache.lock().expect("fitness cache lock poisoned");
```

### saturating_sub for u64 Delta
**Source:** Standard Rust pattern — critical here to avoid debug-mode overflow panics
**Apply to:** Cache delta calculation in `Ga::run()`
```rust
gen_stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
gen_stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
```

### GaError::ConfigurationError for Mutual Exclusivity
**Source:** `src/engines/ga.rs` lines 773–779
**Apply to:** `batch_evaluator` + `fitness_fn` mutual exclusivity check in `Ga::build()`
```rust
return Err(GaError::ConfigurationError(
    "Cannot use both ... — they are mutually exclusive".to_string(),
));
```

### serde(default) for Optional Stats Fields
**Source:** `src/stats.rs` lines 41–53
**Apply to:** `cache_hits` and `cache_misses` fields in `GenerationStats`
```rust
#[cfg_attr(feature = "serde", serde(default))]
pub cache_hits: Option<u64>,
```

### Tests in `tests/` Directory
**Source:** Project convention (CLAUDE.md)
**Apply to:** All new test functions — must go in `tests/engines/test_ga.rs` and `tests/engines/cma/test_cma.rs`, never inline in `src/`

---

## No Analog Found

All files in this phase have close analogs in the codebase. No entries in this section.

---

## Anti-Patterns (Extracted from RESEARCH.md)

These must be explicitly avoided by the planner:

1. **Do not pass `fitness_fn` to `parent_crossover` when batch mode active.** Pass `None` instead — children return with 0.0 fitness, batch pass overwrites. Line 2753 in `ga.rs` is the injection point.

2. **Do not hold the cache lock across `evaluate_batch`.** Release before calling the batch evaluator; re-acquire after to store miss results.

3. **Do not omit `serde(default)` on new `GenerationStats` fields.** Old checkpoints will fail to deserialize without it.

4. **Do not add `Instant::now()` unconditionally** in the batch path. Gate with `#[cfg(not(target_arch = "wasm32"))]`.

5. **Do not call `calculate_fitness()` on offspring when batch mode is active.** It reads the chromosome's internally-stored fitness fn (not the batch evaluator) and returns 0.0 (no-op when fn is None).

---

## Metadata

**Analog search scope:** `src/engines/`, `src/fitness/`, `src/stats.rs`, `src/observe/`, `src/traits/`, `src/lib.rs`
**Files scanned:** 12 source files read directly
**Pattern extraction date:** 2026-06-07
