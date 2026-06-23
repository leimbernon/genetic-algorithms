# Phase 77: Extend Fitness Cache to More Engines - Pattern Map

**Mapped:** 2026-06-19
**Files analyzed:** 9 (3 engine configs, 3 engine impls, 3 test files)
**Analogs found:** 2 / 2 reference engines (Ga, CMA)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/engines/pso/configuration.rs` | config | transform | `src/engines/cma/configuration.rs` | exact |
| `src/engines/pso/engine.rs` | engine | transform | `src/engines/cma/engine.rs` | exact |
| `src/engines/eda/configuration.rs` | config | transform | `src/engines/cma/configuration.rs` | exact |
| `src/engines/eda/engine.rs` | engine | transform | `src/engines/cma/engine.rs` | exact |
| `src/engines/de/configuration.rs` | config | transform | `src/engines/cma/configuration.rs` | exact |
| `src/engines/de/engine.rs` | engine | transform | `src/engines/cma/engine.rs` | exact |
| `tests/engines/pso/test_pso.rs` | test | transform | `tests/engines/pso/test_pso.rs` (self) | extend |
| `tests/engines/eda/test_eda.rs` | test | transform | `tests/engines/eda/test_eda.rs` (self) | extend |
| `tests/engines/de/test_de.rs` | test | transform | `tests/engines/de/test_de.rs` (self) | extend |

## Pattern Assignments

### `src/engines/pso/configuration.rs` (config, transform)

**Analog:** `src/engines/cma/configuration.rs`

**Struct field pattern** (CMA line 83, same location in each config):
```rust
/// Fitness cache capacity in entries.
///
/// When set, `run()` wraps the scalar `fitness_fn` with an LRU cache of this
/// size. When `None`, no caching is performed (zero overhead).
pub fitness_cache_size: Option<usize>,
```

**Default pattern** (CMA line 99):
```rust
fitness_cache_size: None,
```

**Builder method pattern** (CMA lines 199-202):
```rust
/// Enables an LRU fitness cache with the given capacity.
///
/// When enabled, fitness evaluations are cached by DNA hash. Chromosomes
/// with identical genes will reuse cached fitness values, avoiding
/// redundant (and potentially expensive) fitness function calls.
///
/// # Arguments
///
/// * `size` — Maximum number of entries in the cache. A typical value
///   is 2-10x the population size.
pub fn with_fitness_cache_size(mut self, size: usize) -> Self {
    self.fitness_cache_size = Some(size);
    self
}
```

---

### `src/engines/pso/engine.rs` (engine, transform)

**Analog:** `src/engines/cma/engine.rs`

**Struct fields** (CMA line 344):
```rust
// Add to PsoEngine struct after the existing fields:
fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
```

**Import** (add `use std::sync::Mutex;` — already has `use std::sync::Arc;`)

**Run-time wrapping at run() start** (CMA lines 582-598, adapted for PSO):
```rust
// Place at the top of run(), after `let mut rng = make_rng();`
// and before `self.notify(|obs| obs.on_run_start());`
//
// PSO wraps at run() start (CMA pattern), not in new().
// Guard: wrap only once; if re-run() is called, cache already exists.
if let Some(size) = self.config.fitness_cache_size {
    if self.fitness_cache.is_none() {
        let (wrapped_fn, cache_handle) =
            crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), size);
        self.fitness_fn = wrapped_fn;
        self.fitness_cache = Some(cache_handle);
    }
}
```

**Per-generation snapshot** (before the `for gen in 0..` loop, PSO line ~351):
```rust
// Place before the main loop:
let (mut prev_cache_hits, mut prev_cache_misses): (u64, u64) =
    match &self.fitness_cache {
        Some(ch) => {
            let c = ch.lock().expect("fitness cache lock poisoned");
            (c.hits(), c.misses())
        }
        None => (0, 0),
    };
```

**Per-generation delta fill** (after `GenerationStats` construction, PSO line ~454):
```rust
// Place after `let stats = GenerationStats::from_fitness_values(...)`:
if let Some(ref ch) = self.fitness_cache {
    let c = ch.lock().expect("fitness cache lock poisoned");
    stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
    stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
    prev_cache_hits = c.hits();
    prev_cache_misses = c.misses();
}
```

---

### `src/engines/eda/configuration.rs` (config, transform)

**Analog:** `src/engines/cma/configuration.rs`

**Same pattern as PSO config** — add `fitness_cache_size: Option<usize>` field, default `None`, builder `with_fitness_cache_size(size: usize) -> Self`.

---

### `src/engines/eda/engine.rs` (engine, transform)

**Analog:** `src/engines/cma/engine.rs`

**Struct fields** — add to both `EdaEngine` and `EdaRealEngine`:
```rust
fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
```

**Import** — add `use std::sync::Mutex;`

**Run-time wrapping** — same CMA pattern, placed at the top of each `run()` method.

**EDA-specific: fitness call sites** — The wrapped `self.fitness_fn` is already used at:
- EDA parallel path (line 350-354): `let fitness_fn = Arc::clone(&self.fitness_fn);` — no change needed, cache is inside the Arc.
- EDA sequential path (line 361-364): `(self.fitness_fn)(ind.dna())` — no change needed.
- Same for EdaRealEngine parallel (line 678) and sequential (line 689).

**Per-generation snapshot + delta** — identical pattern to PSO, placed around the `GenerationStats` construction.

---

### `src/engines/de/configuration.rs` (config, transform)

**Analog:** `src/engines/cma/configuration.rs`

**Same pattern** — add `fitness_cache_size: Option<usize>` field, default `None`, builder `with_fitness_cache_size(size: usize) -> Self`.

---

### `src/engines/de/engine.rs` (engine, transform)

**Analog:** `src/engines/cma/engine.rs`

**Struct fields** — add to `DeEngine`:
```rust
fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
```

**Import** — add `use std::sync::Mutex;`

**Run-time wrapping** — same CMA pattern, at top of `run()`.

**DE-specific: fitness call site** — `(self.fitness_fn)(&trial_dna)` at line 167 is the single fitness call site; no change needed, cache intercepts inside the wrapped fn.

**Per-generation stats** — DE does NOT currently construct `GenerationStats` inside the main loop (it tracks `generations += 1` and `best_fitness` directly). The stats pattern needs to be added:
```rust
// After the `for i in 0..pop_size` inner loop and before `generations += 1`:
let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
let mut stats = GenerationStats::from_fitness_values(
    generations,
    &fitness_values,
    matches!(self.config.problem_solving, ProblemSolving::Maximization),
);
// D-07: populate per-generation cache delta stats
if let Some(ref ch) = self.fitness_cache {
    let c = ch.lock().expect("fitness cache lock poisoned");
    stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
    stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
    prev_cache_hits = c.hits();
    prev_cache_misses = c.misses();
}
```
This requires adding `use crate::stats::GenerationStats;` and `use crate::configuration::ProblemSolving;` (already used via `self.config.problem_solving`) to the imports.

---

### `tests/engines/pso/test_pso.rs` (test, transform)

**Analog:** existing test file (self-extension)

**Test pattern** (from existing test file):
```rust
#[test]
fn test_pso_cache_enabled_hits() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let eval_count = Arc::new(AtomicUsize::new(0));
    let eval_count_clone = Arc::clone(&eval_count);

    let config = PsoConfiguration::default()
        .with_population_size(20)
        .with_max_generations(10)
        .with_fitness_cache_size(128);

    let mut engine = PsoEngine::<RangeChromosome<f64>>::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        move |dna: &[RangeGene<f64>]| {
            eval_count_clone.fetch_add(1, Ordering::SeqCst);
            dna.iter().map(|g| g.real_value().powi(2)).sum()
        },
    );
    let result = engine.run();

    // With cache, duplicate DNA should reduce evaluations
    // (PSO may produce duplicate particles across generations)
    assert!(eval_count.load(Ordering::SeqCst) > 0);
}

#[test]
fn test_pso_cache_disabled_zero_overhead() {
    let config = PsoConfiguration::default()
        .with_population_size(20)
        .with_max_generations(10);
    // No with_fitness_cache_size() — cache is disabled
    let mut engine = PsoEngine::<RangeChromosome<f64>>::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    );
    let result = engine.run();
    assert!(result.best_fitness >= 0.0);
}
```

---

### `tests/engines/eda/test_eda.rs` (test, transform)

**Analog:** existing test file (self-extension)

**Test pattern** — add two tests (one per model):
- `test_eda_bernoulli_cache_enabled` — uses `EdaEngine` with `with_fitness_cache_size(128)`
- `test_eda_gaussian_cache_enabled` — uses `EdaRealEngine` with `with_fitness_cache_size(128)`
- `test_eda_no_cache_disabled` — uses engine without cache to verify zero-overhead default

---

### `tests/engines/de/test_de.rs` (test, transform)

**Analog:** existing test file (self-extension)

**Test pattern** — add two tests:
- `test_de_cache_enabled` — uses `DeEngine` with `with_fitness_cache_size(128)`, verifies fitness evaluation works
- `test_de_no_cache_disabled` — uses engine without cache

---

## Shared Patterns

### Cache Struct Fields (applies to all 3 engine structs)
**Source:** `src/engines/cma/engine.rs` line 344
**Apply to:** PSO, EDA (both), DE engine structs
```rust
fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
```
Initialize as `None` in `new()`.

### Cache Config Field (applies to all 3 config structs)
**Source:** `src/engines/cma/configuration.rs` line 83
**Apply to:** PsoConfiguration, EdaConfiguration, DeConfiguration
```rust
pub fitness_cache_size: Option<usize>,
```
Initialize as `None` in `Default::default()`. Add `with_fitness_cache_size(size: usize) -> Self` builder.

### Run-time Cache Wrapping (applies to all 3 engine run() methods)
**Source:** `src/engines/cma/engine.rs` lines 582-598
**Apply to:** PSO `run()`, EDA `run()` (both engines), DE `run()`
```rust
if let Some(size) = self.config.fitness_cache_size {
    if self.fitness_cache.is_none() {
        let (wrapped_fn, cache_handle) =
            crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), size);
        self.fitness_fn = wrapped_fn;
        self.fitness_cache = Some(cache_handle);
    }
}
```

### Per-generation Cache Delta Stats (applies to all 3 engine run() loops)
**Source:** `src/engines/cma/engine.rs` lines 919-924 (inline snapshot pattern)
**Apply to:** PSO, EDA (both), DE main loops
```rust
// Snapshot before generation (at loop start):
let (prev_hits, prev_misses) = match &self.fitness_cache {
    Some(ch) => {
        let c = ch.lock().expect("fitness cache lock poisoned");
        (c.hits(), c.misses())
    }
    None => (0, 0),
};

// Delta after generation (after fitness evaluations):
if let Some(ref ch) = self.fitness_cache {
    let c = ch.lock().expect("fitness cache lock poisoned");
    stats.cache_hits = Some(c.hits().saturating_sub(prev_hits));
    stats.cache_misses = Some(c.misses().saturating_sub(prev_misses));
}
```

### Import Pattern (applies to all 3 engine files)
**Source:** `src/engines/cma/engine.rs` line 344 (uses `Arc`, `Mutex`)
**Apply to:** PSO, EDA, DE engine.rs — add `use std::sync::Mutex;` (Arc already imported)

## No Analog Found

None — all target files have exact analogs in the CMA engine.

## Metadata

**Analog search scope:** `src/engines/`, `src/fitness/`, `src/stats.rs`, `tests/engines/`
**Files scanned:** ~30 (engine configs, engine impls, fitness cache, stats, tests)
**Pattern extraction date:** 2026-06-19
