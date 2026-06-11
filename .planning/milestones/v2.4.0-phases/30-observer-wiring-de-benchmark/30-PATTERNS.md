# Phase 30: Observer Wiring & DE Benchmark - Pattern Map

**Mapped:** 2026-04-28
**Files analyzed:** 9 (4 engine files, 4 test files, 1 bench file)
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/engines/de/engine.rs` | engine | event-driven (run loop) | `src/engines/ga.rs` | exact (same observer wiring pattern) |
| `src/engines/scatter/engine.rs` | engine | event-driven (run loop) | `src/engines/ga.rs` | exact |
| `src/engines/cellular/engine.rs` | engine | event-driven (run loop) | `src/engines/ga.rs` | exact |
| `src/engines/alps/engine.rs` | engine | event-driven (run loop) | `src/engines/ga.rs` | exact |
| `tests/engines/de/test_de.rs` | test | request-response | `tests/observe/observer/test_observer.rs` | exact |
| `tests/engines/scatter/test_scatter.rs` | test | request-response | `tests/observe/observer/test_observer.rs` | exact |
| `tests/engines/cellular/test_cellular.rs` | test | request-response | `tests/observe/observer/test_observer.rs` | exact |
| `tests/engines/alps/test_alps.rs` | test | request-response | `tests/observe/observer/test_observer.rs` | exact |
| `benches/de.rs` | benchmark | batch | `benches/de.rs` (self, extending) | exact |

---

## Pattern Assignments

### `src/engines/de/engine.rs` (engine, event-driven)

**Analog:** `src/engines/ga.rs`

**Imports to add** (top of file, alongside existing imports):
```rust
// Currently in engine.rs lines 1-13 — add these two:
use crate::observe::observer::GaObserver;
use crate::stats::GenerationStats;
use crate::ga::TerminationCause;
use std::sync::Arc;
```

**Observer field on struct** (`src/engines/ga.rs` lines 135, 157):
```rust
// Add to DeEngine<U> struct (src/engines/de/engine.rs lines 50-57):
pub struct DeEngine<U: ChromosomeT>
where
    U::Gene: DeGene,
{
    config: DeConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,  // NEW
}
```

**Default field initializer** (in `DeEngine::new`, lines 74-79):
```rust
// Add observer: None to the Self { ... } block
Self {
    config,
    init_fn: Arc::new(init_fn),
    fitness_fn: Arc::new(fitness_fn),
    observer: None,  // NEW
}
```

**`with_observer` + `notify` methods** (`src/engines/ga.rs` lines 539-550):
```rust
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}

#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

**5 hook call sites in `run()`** — insertions relative to `src/engines/de/engine.rs`:

```rust
// Before the main loop (after existing init block, ~line 107):
let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
let mut stats: Vec<GenerationStats> = Vec::new();
let mut prev_best_fitness = best_fitness;
self.notify(|obs| obs.on_run_start());  // Hook 1

// Rename `_gen` → `gen` at line 110:
for gen in 0..self.config.max_generations {
    self.notify(|obs| obs.on_generation_start(gen));  // Hook 2

    // ... existing engine work (inner i-loop, adaptive update) ...

    // After the post-generation re-locate block (~lines 192-198):
    // Hook 3: fire on_new_best ONCE per generation (not inside inner i-loop)
    if self.is_better(best_fitness, prev_best_fitness) {
        prev_best_fitness = best_fitness;
        self.notify(|obs| obs.on_new_best(gen, best.clone()));
    }

    // Hook 4: build stats and fire on_generation_end
    let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    let gen_stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
    stats.push(gen_stats);
    self.notify(|obs| obs.on_generation_end(stats.last().unwrap()));

    generations += 1;

    // Early stopping (unchanged)
    if let Some(target) = self.config.fitness_target {
        if self.reached_target(best_fitness, target) { break; }
    }
}

// After the loop (~line 207, before DeResult return):
let cause = if generations < self.config.max_generations {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
self.notify(|obs| obs.on_run_end(cause, &stats));  // Hook 5
```

**Key DE-specific note:** `on_new_best` must NOT fire inside the inner `for i in 0..pop_size` loop. The existing inner-loop best update (lines 174-178) still updates `best_fitness`/`best` eagerly; the observer hook fires once per generation after the re-locate block, using the `prev_best_fitness` snapshot.

---

### `src/engines/scatter/engine.rs` (engine, event-driven)

**Analog:** `src/engines/ga.rs`

**Same struct/method additions as DeEngine** — observer field, `with_observer`, `notify`.

**5 hook call sites in `run()`** — insertions relative to `src/engines/scatter/engine.rs`:

```rust
// Before the main loop (~line 111):
let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
let mut stats: Vec<GenerationStats> = Vec::new();
let mut prev_best_fitness = best_fitness;
self.notify(|obs| obs.on_run_start());  // Hook 1

// Rename `_iter` → `iter` at line 114:
for iter in 0..self.config.max_iterations {
    self.notify(|obs| obs.on_generation_start(iter));  // Hook 2

    // ... existing combine + evaluate + update ref_set work ...

    // After existing best-update block (~lines 141-146):
    if self.is_better(best_fitness, prev_best_fitness) {
        prev_best_fitness = best_fitness;
        self.notify(|obs| obs.on_new_best(iter, best.clone()));  // Hook 3
    }

    // Hook 4: fitness slice from ref_set
    let fitness_values: Vec<f64> = ref_set.iter().map(|c| c.fitness()).collect();
    let gen_stats = GenerationStats::from_fitness_values(iter, &fitness_values, is_maximization);
    stats.push(gen_stats);
    self.notify(|obs| obs.on_generation_end(stats.last().unwrap()));

    iterations += 1;

    if let Some(target) = self.config.fitness_target {
        if self.reached_target(best_fitness, target) { break; }
    }
}

// After loop (~line 157):
let cause = if iterations < self.config.max_iterations {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
self.notify(|obs| obs.on_run_end(cause, &stats));  // Hook 5
```

**Key Scatter-specific note:** `on_generation_start` / `on_generation_end` use `iter` (not `gen`) as the generation counter — this is fine; `GaObserver` uses `usize` with no semantic constraint.

---

### `src/engines/cellular/engine.rs` (engine, event-driven)

**Analog:** `src/engines/ga.rs`

**Same struct/method additions as DeEngine** — observer field, `with_observer`, `notify`.

**5 hook call sites in `run()`** — insertions relative to `src/engines/cellular/engine.rs`:

```rust
// Before the main loop (~line 136):
let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
let mut stats: Vec<GenerationStats> = Vec::new();
self.notify(|obs| obs.on_run_start());  // Hook 1

// Rename `_gen` → `gen` at line 139:
for gen in 0..self.config.max_generations {
    self.notify(|obs| obs.on_generation_start(gen));  // Hook 2

    // Record best before the sweep (snapshot for per-generation on_new_best detection):
    let prev_best_fitness = best_fitness;

    // ... existing inner double-loop (rows × cols) unchanged ...
    // The inner loop already updates best_fitness + best in-place (lines 206-209) — leave untouched.

    // Apply synchronous replacements (unchanged, lines 220-224)

    generations += 1;

    // Hook 3: fire on_new_best ONCE after sweep, not per-cell
    if self.is_better(best_fitness, prev_best_fitness) {
        self.notify(|obs| obs.on_new_best(gen, best.clone()));
    }

    // Hook 4: fitness from full grid pop
    let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    let gen_stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
    stats.push(gen_stats);
    self.notify(|obs| obs.on_generation_end(stats.last().unwrap()));

    if let Some(target) = self.config.fitness_target {
        if self.reached_target(best_fitness, target) { break; }
    }
}

// After loop (~line 236):
let cause = if generations < self.config.max_generations {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
self.notify(|obs| obs.on_run_end(cause, &stats));  // Hook 5
```

**Key Cellular-specific note (critical):** `on_new_best` MUST NOT fire inside the inner `for row … for col` loop. The existing per-cell best update (lines 206-209 of engine.rs) is correct to keep — it maintains the accurate `best`/`best_fitness`. The observer fires ONCE after the sweep using the `prev_best_fitness` snapshot.

---

### `src/engines/alps/engine.rs` (engine, event-driven)

**Analog:** `src/engines/ga.rs`

**Same struct/method additions as DeEngine** — observer field, `with_observer`, `notify`.

**5 hook call sites in `run()`** — insertions relative to `src/engines/alps/engine.rs`:

```rust
// Before the main loop (~line 136):
let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
let mut stats: Vec<GenerationStats> = Vec::new();
self.notify(|obs| obs.on_run_start());  // Hook 1

// `gen` already available (no rename needed — line 139 uses `gen`):
for gen in 0..self.config.max_generations {
    self.notify(|obs| obs.on_generation_start(gen));  // Hook 2

    // ... existing layer evolution, age increment, promotion, injection (unchanged) ...

    // After the existing "Track global best from all layers" block (~lines 241-249):
    // Record prev_best before the tracking scan — declare before the loop:
    // (Place `let prev_best_fitness = best_fitness;` just before the tracking block)
    let prev_best_fitness_before_scan = /* snapshot taken before the tracking block */;

    // After the tracking block completes:
    if self.is_better(best_fitness, prev_best_fitness_before_scan) {
        self.notify(|obs| obs.on_new_best(gen, best.clone()));  // Hook 3
    }

    // Hook 4: merged fitness slice from all layers (D-06)
    let fitness_values: Vec<f64> = layers
        .iter()
        .flat_map(|layer| layer.iter().map(|ind| ind.fitness()))
        .collect();
    let gen_stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
    stats.push(gen_stats);
    self.notify(|obs| obs.on_generation_end(stats.last().unwrap()));

    generations += 1;

    if let Some(target) = self.config.fitness_target {
        if self.reached_target(best_fitness, target) { break; }
    }
}

// After loop (~line 275):
let cause = if generations < self.config.max_generations {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
self.notify(|obs| obs.on_run_end(cause, &stats));  // Hook 5
```

**Key ALPS-specific notes:**
- `gen` loop variable already exists (not prefixed with `_`) — no rename needed.
- Fitness slice is flattened across all layers per D-06. Empty layers produce no values — `from_fitness_values` handles empty slices gracefully.
- `on_new_best` fires at most once per generation (global best across all layers per D-07). Take `prev_best_fitness` snapshot before the global-best scanning block (~lines 241-249); compare after.

---

### `tests/engines/de/test_de.rs` (test, request-response)

**Analog:** `tests/observe/observer/test_observer.rs`

**SpyObserver pattern** (from `test_observer.rs` lines 17-100):
```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::observer::{GaObserver, NoopObserver};
use genetic_algorithms::stats::GenerationStats;

#[derive(Default)]
struct SpyData {
    run_start:         AtomicUsize,
    generation_start:  AtomicUsize,
    new_best:          AtomicUsize,
    generation_end:    AtomicUsize,
    run_end:           AtomicUsize,
    run_end_cause:     std::sync::Mutex<Option<TerminationCause>>,
    run_end_stats_len: AtomicUsize,
}

struct SpyObserver { data: Arc<SpyData> }

impl GaObserver<RangeChromosome<f64>> for SpyObserver {
    fn on_run_start(&self) {
        self.data.run_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_start(&self, _g: usize) {
        self.data.generation_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_new_best(&self, _g: usize, _best: RangeChromosome<f64>) {
        self.data.new_best.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.data.generation_end.fetch_add(1, Ordering::Relaxed);
    }
    fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        self.data.run_end.fetch_add(1, Ordering::Relaxed);
        *self.data.run_end_cause.lock().unwrap() = Some(cause);
        self.data.run_end_stats_len.store(all_stats.len(), Ordering::Relaxed);
    }
}
```

**Test assertions pattern** (from `test_observer.rs` lines 122-165):
```rust
#[test]
fn test_de_observer_fires_5_hooks() {
    let max_gens = 10usize;
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver { data: Arc::clone(&data) });

    let config = DeConfiguration::default()
        .with_population_size(10)
        .with_max_generations(max_gens)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = DeEngine::new(config, |n| random_pop(n, 3, -5.0, 5.0, 42), sphere)
        .with_observer(spy);
    engine.run();

    assert_eq!(data.run_start.load(Ordering::Relaxed), 1);
    assert_eq!(data.generation_start.load(Ordering::Relaxed), max_gens);
    assert_eq!(data.generation_end.load(Ordering::Relaxed), max_gens);
    assert_eq!(data.run_end.load(Ordering::Relaxed), 1);
    assert_eq!(
        *data.run_end_cause.lock().unwrap(),
        Some(TerminationCause::GenerationLimitReached)
    );
    assert_eq!(data.run_end_stats_len.load(Ordering::Relaxed), max_gens);
    // on_new_best fires >= 1 on sphere (minimization always improves from random start)
    assert!(data.new_best.load(Ordering::Relaxed) >= 1);
}

#[test]
fn test_de_no_observer_no_panic() {
    // NoopObserver compile check and no-observer path
    let obs: Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync> = Arc::new(NoopObserver);
    let config = DeConfiguration::default()
        .with_population_size(10)
        .with_max_generations(5)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = DeEngine::new(config, |n| random_pop(n, 3, -5.0, 5.0, 1), sphere)
        .with_observer(obs);
    engine.run(); // must not panic
}
```

**Same pattern applies to:** `test_scatter.rs`, `test_cellular.rs`, `test_alps.rs` — substitute the appropriate engine/result types and population builders from their respective existing test helpers.

---

### `tests/engines/scatter/test_scatter.rs` (test, request-response)

**Analog:** `tests/observe/observer/test_observer.rs` + existing `test_scatter.rs` helpers

**SpyObserver** uses `RangeChromosome<f64>` (same chromosome as DE tests). Reuse `random_pop`/`sphere` helpers already in the file. `ScatterEngine::new(...).with_observer(spy)` — `with_observer` goes on the engine struct. Assertions: same 5-hook counts as DE test.

---

### `tests/engines/cellular/test_cellular.rs` (test, request-response)

**Analog:** `tests/observe/observer/test_observer.rs` + existing `test_cellular.rs` helpers

**Key assertion difference:** `on_new_best` count must be `<= max_generations` (once per generation at most — validates the per-generation, not per-cell, firing). Add explicit upper-bound assertion:
```rust
assert!(data.new_best.load(Ordering::Relaxed) <= max_gens,
    "on_new_best must fire at most once per generation, not per-cell");
```

---

### `tests/engines/alps/test_alps.rs` (test, request-response)

**Analog:** `tests/observe/observer/test_observer.rs` + existing `test_alps.rs` helpers

**`run_end_stats_len` assertion:** Stats length equals `generations` returned by `AlpsResult` (may be less than `max_generations` if early stopped). Assert `>= 1` if early stopping is enabled; assert `== max_gens` if no fitness target.

---

### `benches/de.rs` (benchmark, batch — extending existing file)

**Analog:** `benches/de.rs` itself (existing `bench_mutation_strategies`)

**Existing patterns to reuse** (lines 1-58):
- `sphere()` fitness function (lines 11-13) — reuse directly
- `make_pop()` initializer (lines 15-27) — reuse directly for DE side
- `group.sample_size(10)` (line 39) — copy for new group
- `DeConfiguration::default().with_*` builder chain (lines 44-49) — copy for DE side of new group

**New `bench_de_vs_ga` function to add** (after `bench_mutation_strategies`, before `criterion_group!`):
```rust
fn bench_de_vs_ga(c: &mut Criterion) {
    use std::borrow::Cow;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig,
                                      SelectionConfig, StoppingConfig};

    let mut group = c.benchmark_group("de_vs_ga");
    group.sample_size(10);

    group.bench_function("de_sphere_5d", |b| {
        b.iter(|| {
            let config = DeConfiguration::default()
                .with_population_size(30)
                .with_max_generations(100)
                .with_mutation_strategy(DeMutationStrategy::Rand1)
                .with_problem_solving(ProblemSolving::Minimization);
            let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    });

    group.bench_function("ga_sphere_5d", |b| {
        b.iter(|| {
            let mut ga = Ga::new()
                .with_population_size(30)
                .with_genes_per_chromosome(5)
                .with_problem_solving(ProblemSolving::Minimization)
                .with_max_generations(100)
                .with_fitness_fn(sphere)
                .with_initialization_fn(|n, _, _| {
                    // Reuse make_pop logic inline to avoid extra public import
                    let mut rng = rand::rng();
                    (0..n)
                        .map(|j| RangeGene::new(j as i32, vec![(-5.0_f64, 5.0)],
                                                rng.random::<f64>() * 10.0 - 5.0))
                        .collect::<Vec<_>>()
                })
                .with_selection_method(Selection::Tournament)
                .with_crossover_method(Crossover::Uniform)
                .with_mutation_method(Mutation::Gaussian)
                .with_survivor_method(Survivor::Fitness)
                .build()
                .expect("valid config");
            ga.run().expect("GA run")
        });
    });

    group.finish();
}
```

**Updated `criterion_group!` line** (replace line 57):
```rust
criterion_group!(benches, bench_mutation_strategies, bench_de_vs_ga);
```

**Note on GA initialization:** The `with_initialization_fn` closure returns `Vec<RangeGene<f64>>` (genes, not chromosomes). Check the exact signature during implementation — it may match `fn(usize, Option<&[Gene]>, Option<bool>) -> Vec<Gene>`. Inline the logic rather than importing `range_random_initialization` to keep the bench file self-contained (per assumption A1).

---

## Shared Patterns

### Observer Field + Dispatch Helper
**Source:** `src/engines/ga.rs` lines 135, 539-550
**Apply to:** All four engine files

```rust
// Field (in struct):
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

// Default initializer (in ::new()):
observer: None,

// Builder method:
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}

// Dispatch helper:
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### `is_maximization` Flag Derivation
**Source:** `src/engines/de/engine.rs` (`problem_solving` field pattern, e.g. `select_pbest` at line 227)
**Apply to:** All four engine `run()` methods

```rust
let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
```

### Stats History Local Variable
**Source:** `src/engines/ga.rs` (field `stats: Vec<GenerationStats>` — use local variable in new engines)
**Apply to:** All four engine `run()` methods

```rust
let mut stats: Vec<GenerationStats> = Vec::new();
// Inside generation loop:
stats.push(gen_stats);
// Passed to on_run_end:
self.notify(|obs| obs.on_run_end(cause, &stats));
```

### TerminationCause Import
**Source:** `tests/observe/observer/test_observer.rs` line 3
**Apply to:** All four engine files

```rust
use crate::ga::TerminationCause;
```

### SpyObserver Test Pattern
**Source:** `tests/observe/observer/test_observer.rs` lines 17-100
**Apply to:** All four engine test files (adapt chromosome type to match engine's chromosome)

Use `AtomicUsize` for all counters. Use `Mutex<Option<TerminationCause>>` for cause capture. Only implement the 5 required hooks (`on_run_start`, `on_generation_start`, `on_new_best`, `on_generation_end`, `on_run_end`).

---

## No Analog Found

All files in this phase have direct analogs in the codebase. No entries needed here.

---

## Metadata

**Analog search scope:** `src/engines/`, `tests/observe/observer/`, `benches/`
**Files scanned:** 11 source files read directly
**Pattern extraction date:** 2026-04-28
