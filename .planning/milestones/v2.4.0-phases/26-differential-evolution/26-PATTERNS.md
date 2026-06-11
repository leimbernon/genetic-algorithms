# Phase 26: Differential Evolution Engine (gap-closure) - Pattern Map

**Mapped:** 2026-04-26
**Files analyzed:** 3 (engine.rs modify, benches/de.rs extend, tests/test_de.rs extend)
**Analogs found:** 3 / 3

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/engines/de/engine.rs` | engine/service | event-driven, request-response | `src/engines/ga.rs` | exact |
| `benches/de.rs` | benchmark | batch | `benches/scatter.rs` (structure), `benches/ga_run.rs` (GA setup) | exact |
| `tests/test_de.rs` | test | request-response | existing file (extend in-place) | n/a |

---

## Pattern Assignments

### `src/engines/de/engine.rs` — add observer field + hooks

**Analog:** `src/engines/ga.rs`

**Imports to add** (ga.rs lines 30–33, 49):
```rust
use crate::observer::GaObserver;
use crate::stats::GenerationStats;
use std::sync::Arc;
// std::time::Instant — only needed for timed hooks; for DE parity just the
// four lifecycle hooks suffice (on_run_start / on_generation_end / on_new_best / on_run_end)
```

**Observer field on struct** (ga.rs lines 135–136):
```rust
/// Optional structured lifecycle observer. When `None` (the default),
/// no hook calls are performed (zero overhead).
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
```

**Constructor — add `new_with_observer`** (model after ga.rs `with_observer` line 539):
```rust
/// Attach a lifecycle observer.  All hooks fire with the same semantics as
/// the standard [`Ga`] engine.
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}
```

**`notify` helper** (ga.rs lines 545–550):
```rust
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

**Hook call sites in `run()`** — insert at the marked positions in the existing loop:

1. Before the main loop (ga.rs line 737):
```rust
self.notify(|obs| obs.on_run_start());
```

2. At end of each generation, after `generations += 1` and best re-check, build `GenerationStats` and call (ga.rs lines 932, 1052–1053):
```rust
let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
let gen_stats = GenerationStats::from_fitness_values(
    _gen,
    &fitness_values,
    matches!(self.config.problem_solving, ProblemSolving::Maximization),
);
self.notify(|obs| obs.on_generation_end(&gen_stats));
```

3. When a new best is found (ga.rs line 1131):
```rust
self.notify(|obs| obs.on_new_best(_gen, best.clone()));
```

4. After the loop exits (ga.rs line 1195):
```rust
// Pass an empty stats slice — DE does not accumulate a history Vec by default.
// If the caller needs full history, they collect it via their observer.
self.notify(|obs| obs.on_run_end(/* no TerminationCause in DE */ /* use a dummy */ , &[]));
```

> **Note on `on_run_end`:** `ga.rs` passes `TerminationCause` and `&[GenerationStats]`.
> `DeEngine` has no `TerminationCause` enum. The simplest approach:
> - Pass `TerminationCause::GenerationLimitReached` (or define a local alias) for early-stopped runs via target.
> - Pass `&[]` for stats since DE does not currently accumulate a Vec. This matches the "simplest given existing fields" discretion from D-05.
> - Import `crate::ga::TerminationCause` — it is already `pub` in `src/engines/ga.rs`.

**`Default` impl update** (ga.rs lines 139–159) — add `observer: None` to the `DeEngine` default/constructor:
```rust
// In DeEngine::new():
Self {
    config,
    init_fn: Arc::new(init_fn),
    fitness_fn: Arc::new(fitness_fn),
    observer: None,   // ← add this field
}
```

---

### `benches/de.rs` — extend with DE-vs-GA comparison group

**Analog:** `benches/scatter.rs` (group structure) + `benches/ga_run.rs` (Ga setup pattern)

**New imports needed** (add to existing import block at top of `benches/de.rs`):
```rust
// Already present — reuse RangeChromosome, RangeGene, sphere, make_pop helpers

// Add for Ga side:
use genetic_algorithms::ga::Ga;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, StoppingConfig};
// Note: RangeChromosome does NOT implement ValueMutable — the bench must use the
// same chromosome type for both engines OR define a minimal wrapper for the Ga side.
// Simplest path: keep the Ga bench independent with its own inline chromosome
// (copy pattern from ga_run.rs lines 22–90) and use make_pop() for DE only.
```

**New benchmark function** (model after `bench_scatter_vs_local_search` in `benches/scatter.rs` lines 28–66):
```rust
fn bench_de_vs_ga(c: &mut Criterion) {
    let mut group = c.benchmark_group("de_vs_ga_sphere_5d");
    group.sample_size(10);

    // DE side — Rand1/Binomial, same sphere function, pop=30, 100 gen
    group.bench_function("de_rand1_binomial", |b| {
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

    // GA side — minimal Ga<RangeChromosome> on sphere, same pop/gen budget
    // (RangeChromosome must implement ValueMutable — check if it does, or skip GA
    //  side with a note. See ga_run.rs for the full chromosome boilerplate if needed.)
    group.bench_function("ga_uniform_swap", |b| {
        b.iter(|| {
            // build_ga_range(30, 5, 100)  ← helper defined inline
        });
    });

    group.finish();
}
```

**Register new group** (ga.rs pattern lines 179–185):
```rust
criterion_group!(benches, bench_mutation_strategies, bench_de_vs_ga);
criterion_main!(benches);
```

> **Implementer note:** Check whether `RangeChromosome<f64>` implements `operations::mutation::ValueMutable`. If not, the GA bench must use either a stub chromosome (copy the `SimpleChromosome` pattern from `ga_run.rs` lines 22–90) or the comparison should be clearly noted as "DE vs GA on equivalent discrete problem". The simplest approach is to document this limitation in a comment rather than introducing a new chromosome type in the bench.

---

### `tests/test_de.rs` — add observer smoke test

**Analog:** existing file (extend in-place per CLAUDE.md: all tests in `tests/`)

**Imports to add** (after existing imports):
```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::traits::ChromosomeT;
```

**Observer smoke test pattern** — minimal counter observer (no existing analog; use simplest interior-mutability pattern matching `GaObserver` contract from `src/observe/observer/mod.rs` lines 65–120):
```rust
struct CountingObserver {
    generation_ends: AtomicUsize,
    run_starts: AtomicUsize,
    run_ends: AtomicUsize,
    new_bests: AtomicUsize,
}

impl<U: ChromosomeT> GaObserver<U> for CountingObserver {
    fn on_run_start(&self) {
        self.run_starts.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_end(&self, _stats: &genetic_algorithms::stats::GenerationStats) {
        self.generation_ends.fetch_add(1, Ordering::Relaxed);
    }
    fn on_new_best(&self, _generation: usize, _best: U) {
        self.new_bests.fetch_add(1, Ordering::Relaxed);
    }
    fn on_run_end(
        &self,
        _cause: genetic_algorithms::ga::TerminationCause,
        _all_stats: &[genetic_algorithms::stats::GenerationStats],
    ) {
        self.run_ends.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn test_observer_hooks_fire() {
    let obs = Arc::new(CountingObserver {
        generation_ends: AtomicUsize::new(0),
        run_starts: AtomicUsize::new(0),
        run_ends: AtomicUsize::new(0),
        new_bests: AtomicUsize::new(0),
    });

    let config = DeConfiguration::default()
        .with_population_size(10)
        .with_max_generations(5)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = DeEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 99), sphere)
        .with_observer(Arc::clone(&obs) as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);

    let _ = engine.run();

    assert_eq!(obs.run_starts.load(Ordering::Relaxed), 1, "on_run_start must fire once");
    assert_eq!(obs.run_ends.load(Ordering::Relaxed), 1, "on_run_end must fire once");
    assert_eq!(obs.generation_ends.load(Ordering::Relaxed), 5, "on_generation_end fires each generation");
    // on_new_best fires >= 0 times; just assert it doesn't panic
}
```

---

## Shared Patterns

### Observer field + notify helper
**Source:** `src/engines/ga.rs` lines 135–136, 545–550
**Apply to:** `src/engines/de/engine.rs`
```rust
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### `with_observer` builder method
**Source:** `src/engines/ga.rs` lines 539–542
**Apply to:** `src/engines/de/engine.rs` (`impl DeEngine<U>` block)
```rust
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}
```

### `GenerationStats::from_fitness_values`
**Source:** `src/stats.rs` lines 40–97
**Apply to:** `src/engines/de/engine.rs` run loop (build stats each generation for `on_generation_end`)
```rust
let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
let gen_stats = GenerationStats::from_fitness_values(
    _gen,
    &fitness_values,
    matches!(self.config.problem_solving, ProblemSolving::Maximization),
);
```

### Criterion benchmark group with `sample_size(10)`
**Source:** `benches/scatter.rs` lines 28–66, `benches/de.rs` lines 38–55
**Apply to:** new `bench_de_vs_ga` function in `benches/de.rs`
```rust
let mut group = c.benchmark_group("de_vs_ga_sphere_5d");
group.sample_size(10);
// ... bench_function calls ...
group.finish();
```

---

## No Analog Found

None — all three files have close analogs.

---

## Metadata

**Analog search scope:** `src/engines/`, `benches/`, `tests/`, `src/observe/`, `src/stats.rs`
**Files scanned:** 10 (ga.rs, observer/mod.rs, de/engine.rs, de/configuration.rs, stats.rs, de.rs bench, ga_run.rs bench, scatter.rs bench, test_de.rs, lib.rs)
**Pattern extraction date:** 2026-04-26
