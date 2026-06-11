# Phase 30: Observer Wiring & DE Benchmark - Research

**Researched:** 2026-04-28
**Domain:** Rust observer pattern wiring, criterion benchmarking
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Observer type is `Option<Arc<dyn GaObserver<U> + Send + Sync>>` on all four engines — identical to `ga.rs`. Zero overhead when `None`.
- **D-02:** No per-engine sub-traits. All four engines use the base `GaObserver<U>` trait only.
- **D-03:** Wire **5 required hooks only** on all four engines: `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_run_end`. Do NOT wire operator-timing hooks (`on_selection_complete`, `on_mutation_complete`, etc.) in this phase.
- **D-04:** Builder method: `with_observer(Arc<dyn GaObserver<U> + Send + Sync>) -> Self` on each engine's configuration or engine struct, matching the `ga.rs` pattern.
- **D-05:** All four engines call `GenerationStats::from_fitness_values(generation, &fitness_slice, is_maximization)` to build stats passed to `on_generation_end`.
- **D-06:** `on_generation_end` receives **merged stats across all layers** for ALPS — flatten all layer populations into one fitness slice, compute a single `GenerationStats`.
- **D-07:** `on_new_best` fires when the **global best across all layers** improves (not per-layer tracking) for ALPS.
- **D-08:** Extend `benches/de.rs` — add a GA run alongside existing DE benchmarks. No new bench file or Cargo.toml entry needed.
- **D-09:** Both DE and GA run on **sphere(5D)** with the **same `max_generations`** (e.g., 100). Comparison is wall-time per run as reported by criterion — no evaluation-count normalization.
- **D-10:** The GA run uses the standard `Ga<U>` engine with `RangeChromosome<f64>` and default operators on the same sphere(5D) fitness function.

### Claude's Discretion

- `with_observer()` placement: either on the engine struct directly (like `ga.rs`) or on the configuration struct — follow whatever pattern is cleanest for each engine's existing API.
- `sample_size(10)` on the DE-vs-GA benchmark group (matches existing alps/de bench convention for faster CI runs).

### Deferred Ideas (OUT OF SCOPE)

- Operator-timing hooks for new engines (`on_mutation_complete` for DE trial vectors, `on_selection_complete` for Cellular local tournament, etc.)
- Per-layer observer stats for ALPS
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OBS-01 | User can attach a `GaObserver` to `DeEngine` and receive 5 lifecycle hooks | D-01/D-03: `ga.rs` wiring pattern verified; `DeEngine::run()` has clear per-generation and run-level insertion points |
| OBS-02 | User can attach a `GaObserver` to `ScatterEngine` and receive 5 lifecycle hooks | D-01/D-03: `ScatterEngine::run()` uses `_iter` loop variable — need to rename to `gen` for hook calls |
| OBS-03 | User can attach a `GaObserver` to `CellularEngine` and receive 5 lifecycle hooks | D-01/D-03: `CellularEngine::run()` uses `_gen` — need to rename for hook calls; best tracking is in-loop (on_new_best fires inside inner sweep) |
| OBS-04 | User can attach a `GaObserver` to `AlpsEngine` and receive 5 lifecycle hooks | D-06/D-07: multi-layer flatten for stats; global best tracking already exists as `best` variable |
| OBS-05 | `cargo bench --bench de` compares DE vs GA convergence on sphere(5D) | D-08/D-09/D-10: `bench_mutation_strategies` group already exists; new `de_vs_ga` group added to same file |
</phase_requirements>

---

## Summary

Phase 30 is a wiring phase: no new algorithms, no new traits, no breaking changes. The canonical implementation pattern already exists in `src/engines/ga.rs` and must be replicated verbatim across the four new engines. The `GaObserver<U>` trait is already fully defined in `src/observe/observer/mod.rs` with all 12 hooks (only 5 are required here). `GenerationStats::from_fitness_values()` is the single stats constructor all engines will use.

Each engine's `run()` method currently uses discarded loop variables (`_gen`, `_iter`) — these must be un-prefixed to make generation numbers available for hook call arguments. This is the most significant "non-trivial" edit per engine. Everything else is additive struct fields and method calls.

The DE-vs-GA benchmark is straightforward: the existing `benches/de.rs` already has `make_pop()` and `sphere()` helpers, and the `bench_mutation_strategies` group demonstrates the `sample_size(10)` pattern. A new group `de_vs_ga` is added after the existing one.

**Primary recommendation:** Treat `ga.rs` lines 135, 539-550, 737, 742, 931-953, 1053, 1131, 1195 as the copy-paste template for every engine. Do not invent variations.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Observer storage | Engine struct | — | All four engines own their `Option<Arc<dyn GaObserver<U>>>` field directly, like `Ga<U>` |
| Observer dispatch | Engine struct (`notify` helper) | — | Inline `fn notify<F: FnOnce(...)>(&self, f: F)` avoids repeated `if let Some` boilerplate |
| Stats construction | Engine `run()` method | `GenerationStats::from_fitness_values` | One call per generation with flattened fitness slice |
| Best-change detection | Engine `run()` main loop | — | Already implemented as `is_better()` helper on each engine; observer hook fires on improvement |
| Benchmark execution | Criterion framework | `benches/de.rs` | No new bench entry needed; same file, new group |

---

## Standard Stack

### Core (already in Cargo.toml — no new dependencies)

| Item | Version/Location | Purpose | Notes |
|------|-----------------|---------|-------|
| `GaObserver<U>` trait | `src/observe/observer/mod.rs` | Observer contract | 12 hooks, all default no-op; only 5 needed here |
| `GenerationStats::from_fitness_values` | `src/stats.rs` | Stats construction | Takes `(generation: usize, &[f64], is_maximization: bool)` |
| `Arc<dyn GaObserver<U> + Send + Sync>` | std | Thread-safe observer storage | Zero-cost when `None`; required by rayon contexts |
| `TerminationCause` | `src/engines/ga.rs` | `on_run_end` parameter | Each engine needs to determine its own cause (generation limit vs. fitness target) |
| `criterion` | `benches/de.rs` | Benchmark harness | Already used; `sample_size(10)` pattern already established |

### Supporting (no installation needed)

| Item | Location | Purpose | When to Use |
|------|----------|---------|-------------|
| `NoopObserver` | `src/observe/observer/mod.rs` | Compile tests | Use in new test cases to verify observer field accepts `Arc<dyn GaObserver>` |
| `Instant::now()` | std | Timing guard | Only needed if operator-timing hooks are wired — deferred, not needed here |

**Installation:** None required. All dependencies are already present.

---

## Architecture Patterns

### System Architecture Diagram

```
User code                     Engine (e.g. DeEngine)         GaObserver<U>
───────────                   ──────────────────────         ─────────────
Arc<MyObserver> ─with_obs─→   self.observer = Some(arc)
                              │
engine.run()                  │
  │                           notify(|obs| obs.on_run_start())  →  on_run_start()
  │                           │
  │  for gen in 0..max_gens   │
  │    │                      notify(|obs| obs.on_generation_start(gen))  →  on_generation_start(gen)
  │    │                      │
  │    │  [algorithm work]    │
  │    │                      │
  │    │  if improved best    notify(|obs| obs.on_new_best(gen, best.clone()))  →  on_new_best(gen, best)
  │    │                      │
  │    │  build stats from    GenerationStats::from_fitness_values(gen, &fitness_slice, is_maximization)
  │    │  fitness slice        │
  │    │                      notify(|obs| obs.on_generation_end(&stats))  →  on_generation_end(&stats)
  │    │                      │
  │                           notify(|obs| obs.on_run_end(cause, &stats_history))  →  on_run_end(cause, &stats_history)
  │
  ←── result
```

### Recommended Engine File Structure (additive changes only)

```
src/engines/<name>/engine.rs     ← add: observer field, notify helper, 5 hook calls
src/engines/<name>/configuration.rs  ← optional: with_observer() builder here OR on engine struct
tests/engines/<name>/test_<name>.rs  ← add: observer smoke test (attach + run + assert hook fired)
benches/de.rs                    ← add: de_vs_ga benchmark group
```

### Pattern 1: Observer Field and Dispatch Helper (verbatim from ga.rs)

**What:** Store observer as `Option<Arc<...>>`, dispatch via `notify` helper.
**When to use:** In the engine struct for every new engine.

```rust
// Source: src/engines/ga.rs lines 135, 539-550

// In the engine struct:
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

// Default value:
observer: None,

// Builder method on engine struct (or config struct):
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}

// Dispatch helper — inline, zero overhead when None:
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### Pattern 2: The 5 Hook Call Sites (per ga.rs ordering)

```rust
// Source: src/engines/ga.rs lines 737, 742, 1053, 1131, 1195

// 1. Before the generation loop:
self.notify(|obs| obs.on_run_start());

// 2. At the top of each loop iteration:
for gen in 0..self.config.max_generations {
    self.notify(|obs| obs.on_generation_start(gen));

    // ... engine-specific work ...

    // 3. When global best improves (inside or after engine work):
    if self.is_better(new_fitness, best_fitness) {
        best_fitness = new_fitness;
        best = /* clone best individual */;
        self.notify(|obs| obs.on_new_best(gen, best.clone()));
    }

    // 4. After building stats from fitness slice:
    let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
    let stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
    self.notify(|obs| obs.on_generation_end(&stats));
}

// 5. After the loop exits:
self.notify(|obs| obs.on_run_end(/* cause */, &stats_history));
```

### Pattern 3: ALPS Multi-Layer Stats Flattening (D-06, D-07)

**What:** Flatten all layer populations into a single fitness slice for `on_generation_end`.
**When to use:** AlpsEngine only.

```rust
// Per-generation, after evolving all layers:
let fitness_values: Vec<f64> = layers
    .iter()
    .flat_map(|layer| layer.iter().map(|ind| ind.fitness()))
    .collect();
let stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
self.notify(|obs| obs.on_generation_end(&stats));
```

### Pattern 4: Stats History for on_run_end

**What:** Collect `GenerationStats` per generation into a `Vec<GenerationStats>`, pass to `on_run_end`.
**When to use:** All four engines need this. `ga.rs` does `self.stats.push(gen_stats)` and passes `&self.stats` to `on_run_end`.

Each new engine must:
1. Add `stats: Vec<GenerationStats>` field to its struct (or build it locally in `run()`).
2. Push per-generation stats.
3. Pass `&stats` to `on_run_end`.

The simplest approach for engines that don't currently expose a stats accessor: use a local `Vec<GenerationStats>` variable inside `run()`.

### Pattern 5: TerminationCause for on_run_end

**What:** Each engine needs to report why the run ended.
**When to use:** `on_run_end(cause: TerminationCause, &[GenerationStats])`.

All four new engines exit via one of:
- `max_generations` exhausted → `TerminationCause::GenerationLimitReached`
- `fitness_target` reached (early stop) → `TerminationCause::FitnessTargetReached`

Determine cause after the loop exits using the existing early-stop boolean or by checking `generations == max_generations`.

```rust
// Source: src/engines/ga.rs line 1188-1190
use crate::ga::TerminationCause;

let cause = if /* fitness target was reached */ {
    TerminationCause::FitnessTargetReached
} else {
    TerminationCause::GenerationLimitReached
};
self.notify(|obs| obs.on_run_end(cause, &stats));
```

### Pattern 6: DE-vs-GA Benchmark Addition

**What:** Add a `de_vs_ga` benchmark group to `benches/de.rs`.
**When to use:** OBS-05. Reuse existing `make_pop()` and `sphere()` helpers.

```rust
// Source: benches/de.rs (extend after existing bench_mutation_strategies)

fn bench_de_vs_ga(c: &mut Criterion) {
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::configuration::ProblemSolving;

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
                .with_fitness_fn(|dna: &[RangeGene<f64>]| {
                    dna.iter().map(|g| g.value() * g.value()).sum::<f64>()
                })
                .with_initialization_fn(/* range_random_initialization or closure */ ...)
                .with_selection_method(Selection::Tournament)
                .with_crossover_method(Crossover::Uniform)
                .with_mutation_method(Mutation::Gaussian)
                .with_survivor_method(Survivor::Fitness)
                .with_problem_solving(ProblemSolving::Minimization)
                .with_max_generations(100)
                .build()
                .expect("valid config");
            ga.run().expect("GA run")
        });
    });

    group.finish();
}

criterion_group!(benches, bench_mutation_strategies, bench_de_vs_ga);
```

**Note on GA initialization:** `Ga<RangeChromosome<f64>>` requires a `with_initialization_fn`. The cleanest approach reuses `make_pop(n, 5)` logic inside a closure, or uses the crate's `range_random_initialization` helper. Confirm the exact initializer signature during implementation — the closure form avoids importing an extra public function.

### Anti-Patterns to Avoid

- **Using `_gen` / `_iter` with hooks:** The discarded loop variables in `DeEngine`, `ScatterEngine`, `CellularEngine`, and `AlpsEngine` must be renamed to `gen` (or `iter` for Scatter) to pass to `on_generation_start(gen)`. Forgetting this causes a compile error.
- **Cloning the observer Arc per generation:** Never `Arc::clone` the observer inside the generation loop. `notify()` borrows `self.observer` via `if let Some(ref obs)` — no clone needed.
- **Calling `on_new_best` outside the improvement guard:** `on_new_best` must only fire when `is_better(new_fitness, best_fitness)` is true. Do not fire it unconditionally.
- **Passing per-layer stats separately to `on_generation_end` for ALPS:** D-06 locks merged stats. Do not fire `on_generation_end` once per layer.
- **Adding `TerminationCause` as a new field:** Use the local variable pattern — no struct field needed unless the engine exposes termination cause as part of its `Result` type (none of the four do currently).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Fitness statistics | Custom mean/std computation | `GenerationStats::from_fitness_values()` | Already handles empty slice, std dev, diversity, is_maximization distinction |
| Observer dispatch | `match self.observer { Some(o) => o.hook(), None => {} }` everywhere | `fn notify<F: FnOnce(...)>` helper | Eliminates 10+ repetitive match blocks; zero overhead |
| Thread-safe observer | `Box<dyn Observer + Send>` | `Arc<dyn GaObserver<U> + Send + Sync>` | Required for potential rayon use; Arc allows sharing without ownership |
| Benchmark boilerplate | New bench file or new `[[bench]]` entry | Extend `benches/de.rs` | D-08 is locked; criterion group API handles multiple groups in one file |

**Key insight:** This phase is a pure replication exercise. Every design decision is already locked in `ga.rs`. The value is in clean, consistent implementation — not invention.

---

## Per-Engine Wiring Analysis

### DeEngine

**Current state:** `engine.rs` has `for _gen in 0..self.config.max_generations` — `_gen` must become `gen`.

**`is_maximization` flag:** Derive from `self.config.problem_solving` — same pattern as `ga.rs`:
```rust
let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
```

**`on_new_best` placement:** The existing best-tracking code updates `best` and `best_fitness` at two places in the loop (inside the per-individual greedy selection and after re-locating best at end of generation). The simplest correct placement: fire `on_new_best` immediately after `best = pop[best_idx].clone()` in the post-generation re-locate block (lines 192-198 in `engine.rs`). This avoids firing it multiple times per generation from the inner `i` loop.

**`with_observer()` placement:** On the engine struct (not config), matching `ga.rs`. `DeConfiguration` has no observer — keep config clean.

**Stats history:** Use a local `Vec<GenerationStats>` in `run()`. `DeResult` does not expose stats — no need to add a field to `DeResult`.

### ScatterEngine

**Current state:** `for _iter in 0..self.config.max_iterations` — `_iter` must become `iter`.

**Generation semantics:** Scatter Search calls iterations "iterations" not "generations", but `on_generation_start(iter)` and `on_generation_end(&stats)` use `iter` as the generation number. This is fine — the `GaObserver` contract uses `usize` with no semantic constraint.

**Fitness slice:** After updating `ref_set` and calling `sort_by_fitness`, the fitness slice is:
```rust
let fitness_values: Vec<f64> = ref_set.iter().map(|c| c.fitness()).collect();
```

**`on_new_best` placement:** Fire when `self.is_better(bf, best_fitness)` is true (already exists at the post-combine update block). Wrap the existing improvement block.

### CellularEngine

**Current state:** `for _gen in 0..self.config.max_generations` — `_gen` must become `gen`.

**`on_new_best` placement:** The cellular engine fires `is_better` per cell inside the inner double-loop (`for row … for col`). The global best variable `best` and `best_fitness` are already updated per-cell. However, `on_new_best` should NOT fire per-cell — it should fire at most once per generation, after the full sweep, if `best` improved during that sweep. Implementation:

```rust
// Before inner loop:
let prev_best_fitness = best_fitness;

// After inner loop (after applying sync replacements):
if self.is_better(best_fitness, prev_best_fitness) {
    self.notify(|obs| obs.on_new_best(gen, best.clone()));
}
```

**Fitness slice:** After the generation sweep:
```rust
let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
```

### AlpsEngine

**Current state:** `for gen in 0..self.config.max_generations` — `gen` is already available (not prefixed with `_`).

**`on_new_best` placement:** The existing `is_better` scan at the end of each generation (the "Track global best from all layers" block, lines 241-249) is the correct location. Wrap the improvement branch:

```rust
for layer in &layers {
    for ind in layer {
        if self.is_better(ind.fitness(), best_fitness) {
            best_fitness = ind.fitness();
            best = ind.clone();
            // on_new_best fires once per generation at most
            // (last improvement wins if multiple layers improve in same gen)
        }
    }
}
// Fire once after scanning all layers:
if self.is_better(best_fitness, prev_best_fitness) {
    self.notify(|obs| obs.on_new_best(gen, best.clone()));
}
```

**Merged fitness slice (D-06):**
```rust
let fitness_values: Vec<f64> = layers
    .iter()
    .flat_map(|layer| layer.iter().map(|ind| ind.fitness()))
    .collect();
let stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
self.notify(|obs| obs.on_generation_end(&stats));
```

**Note on empty layers:** `GenerationStats::from_fitness_values` handles empty slices gracefully (returns zeroed stats). ALPS layers start empty for all layers except layer 0, so early generations will have sparse flattened slices — this is correct behavior.

---

## Common Pitfalls

### Pitfall 1: `on_new_best` Fires Multiple Times Per Generation (CellularEngine)
**What goes wrong:** If `on_new_best` fires inside the inner per-cell loop, it can fire dozens of times per generation, violating user expectations.
**Why it happens:** `best_fitness` is updated per-cell — naively placing the notify there fires it once per improving cell.
**How to avoid:** Record `prev_best_fitness` before the inner loop; fire `on_new_best` once after the sweep if `best_fitness` improved.
**Warning signs:** Test observer spy records `new_best_count > max_generations` after a run.

### Pitfall 2: `_gen` / `_iter` Compile Errors After Rename
**What goes wrong:** Renaming `_gen` to `gen` in engines where the variable was genuinely unused (not passed to hooks) causes an unused variable warning or triggers `clippy::let_underscore_future`-style lints.
**Why it happens:** After wiring, `gen` is used in hook calls, so the warning disappears. But if any hook is omitted, the warning returns.
**How to avoid:** Wire all 5 hooks in the same commit. Do not partially wire.
**Warning signs:** `cargo clippy` warns about unused `gen` variable.

### Pitfall 3: Missing `TerminationCause` Import
**What goes wrong:** `TerminationCause` is defined in `src/engines/ga.rs` and re-exported from the crate root. New engine files must import it.
**Why it happens:** Engine files are in separate modules that don't automatically have `ga.rs` in scope.
**How to avoid:** Add `use crate::ga::TerminationCause;` at the top of each engine file.
**Warning signs:** Compile error "cannot find type `TerminationCause` in this scope".

### Pitfall 4: Stats History Not Initialized Before Loop
**What goes wrong:** If `stats: Vec<GenerationStats>` is created after the loop, `on_run_end` receives an empty slice.
**Why it happens:** Forgetting to declare and push stats inside the generation loop.
**How to avoid:** Declare `let mut stats: Vec<GenerationStats> = Vec::new();` before the generation loop; push inside it.
**Warning signs:** Test that checks `all_stats.len() == max_generations` fails with 0.

### Pitfall 5: GA Initialization in Benchmark
**What goes wrong:** `Ga<RangeChromosome<f64>>` requires `with_initialization_fn` — omitting it causes a runtime `InitializationError`.
**Why it happens:** The standard `Ga` engine doesn't auto-initialize like `DeEngine` (which takes a closure returning `Vec<U>`).
**How to avoid:** Provide an initialization closure reusing `make_pop` logic from the same bench file, or inline the initialization.
**Warning signs:** Benchmark panics on first iteration with "No initialization function set".

---

## Code Examples

### Adding Observer Field to Engine Struct

```rust
// Source: src/engines/ga.rs line 135 (verified)
// Pattern to replicate in DeEngine, ScatterEngine, CellularEngine, AlpsEngine:

use crate::observer::GaObserver;
use std::sync::Arc;

pub struct DeEngine<U: ChromosomeT>
where
    U::Gene: DeGene,
{
    config: DeConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    // NEW:
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}
```

### `with_observer` and `notify` on Engine Struct

```rust
// Source: src/engines/ga.rs lines 539-550 (verified)

impl<U: ChromosomeT + Clone> DeEngine<U>
where
    U::Gene: DeGene,
{
    pub fn new( /* unchanged */ ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            observer: None,  // NEW field initializer
        }
    }

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
}
```

### Complete 5-Hook Call Sequence in run()

```rust
// Source: derived from src/engines/ga.rs lines 737, 742, 931, 1053, 1131, 1195 (verified)

pub fn run(&mut self) -> DeResult<U> {
    // ... init ...
    let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
    let mut stats: Vec<GenerationStats> = Vec::new();
    let mut prev_best_fitness = best_fitness;

    self.notify(|obs| obs.on_run_start());  // Hook 1

    for gen in 0..self.config.max_generations {  // renamed from _gen
        self.notify(|obs| obs.on_generation_start(gen));  // Hook 2

        // ... engine work ...

        // Hook 3: on_new_best — fire if global best improved this generation
        if self.is_better(best_fitness, prev_best_fitness) {
            prev_best_fitness = best_fitness;
            self.notify(|obs| obs.on_new_best(gen, best.clone()));
        }

        // Hook 4: on_generation_end
        let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
        let gen_stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
        stats.push(gen_stats);
        self.notify(|obs| obs.on_generation_end(stats.last().unwrap()));

        generations += 1;

        // Early stop check ...
        if let Some(target) = self.config.fitness_target {
            if self.reached_target(best_fitness, target) {
                break;
            }
        }
    }

    let cause = if /* early stopped */ {
        TerminationCause::FitnessTargetReached
    } else {
        TerminationCause::GenerationLimitReached
    };
    self.notify(|obs| obs.on_run_end(cause, &stats));  // Hook 5

    DeResult { population: pop, best, best_fitness, generations }
}
```

### Benchmark Group Addition (benches/de.rs)

```rust
// Source: benches/de.rs (verified — extend after existing criterion_group!)

fn bench_de_vs_ga(c: &mut Criterion) {
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::initializers::range_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};

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
            let alleles = {
                let mut rng = rand::rng();
                (0..5).map(|j| RangeGene::new(j as i32, vec![(-5.0_f64, 5.0)], rng.random::<f64>() * 10.0 - 5.0)).collect::<Vec<_>>()
            };
            let alleles_clone = alleles.clone();
            let mut ga = Ga::new()
                .with_population_size(30)
                .with_genes_per_chromosome(5)
                .with_problem_solving(ProblemSolving::Minimization)
                .with_max_generations(100)
                .with_fitness_fn(sphere)
                .with_initialization_fn(move |n, _, _| range_random_initialization(n, Some(&alleles_clone), Some(false)))
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

criterion_group!(benches, bench_mutation_strategies, bench_de_vs_ga);
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (no external harness) |
| Config file | none (cargo test discovers tests/) |
| Quick run command | `cargo test engines` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OBS-01 | DeEngine observer fires 5 hooks | integration | `cargo test engines::de` | ❌ Wave 0 |
| OBS-02 | ScatterEngine observer fires 5 hooks | integration | `cargo test engines::scatter` | ❌ Wave 0 |
| OBS-03 | CellularEngine observer fires 5 hooks | integration | `cargo test engines::cellular` | ❌ Wave 0 |
| OBS-04 | AlpsEngine observer fires 5 hooks | integration | `cargo test engines::alps` | ❌ Wave 0 |
| OBS-05 | `cargo bench --bench de` runs without error | smoke | `cargo bench --bench de -- --test` | ❌ Wave 0 (new group) |

**Existing test files to extend (not create):**
- `tests/engines/de/test_de.rs` — add observer smoke test
- `tests/engines/scatter/test_scatter.rs` — add observer smoke test
- `tests/engines/cellular/test_cellular.rs` — add observer smoke test
- `tests/engines/alps/test_alps.rs` — add observer smoke test

**Pattern for each engine observer test (based on `tests/observe/observer/test_observer.rs`):**

```rust
// Verify: on_run_start fires 1x, on_generation_start fires max_gens times,
//         on_generation_end fires max_gens times, on_run_end fires 1x,
//         on_new_best fires >= 0 times (sphere improves so fires >= 1 in practice).
```

Use `AtomicUsize` spy observer (same pattern as `test_observer.rs`'s `SpyData`/`SpyObserver`). Use `NoopObserver` for a "no observer, no panic" test.

### Sampling Rate

- **Per task commit:** `cargo test engines`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] Observer smoke test in `tests/engines/de/test_de.rs` — covers OBS-01
- [ ] Observer smoke test in `tests/engines/scatter/test_scatter.rs` — covers OBS-02
- [ ] Observer smoke test in `tests/engines/cellular/test_cellular.rs` — covers OBS-03
- [ ] Observer smoke test in `tests/engines/alps/test_alps.rs` — covers OBS-04
- [ ] DE-vs-GA benchmark group in `benches/de.rs` (smoke: `cargo bench --bench de -- --test`) — covers OBS-05

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Reporter` (`Box<dyn Reporter + Send>`, `&mut self` hooks) | `GaObserver` (`Arc<dyn GaObserver + Send + Sync>`, `&self` hooks) | v2.2.0 | All new engines use `GaObserver`; `Reporter` is deprecated and not needed here |
| Per-engine sub-traits (e.g. `DeObserver`) | Single `GaObserver<U>` base trait | Locked in D-02 | Simpler user API; no fragmentation |

**Deprecated/outdated:**
- `Reporter` trait: deprecated since v2.2.0. New engines do not implement it — `GaObserver` only.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `range_random_initialization` is accessible for use in the GA benchmark | Benchmark Pattern 6 | Would need to inline the init logic from `make_pop()` instead — minor workaround |
| A2 | `on_new_best` for AlpsEngine should fire at most once per generation even if multiple layers improve | Per-Engine Analysis (ALPS) | If D-07 means "fire per improvement found", the snapshot-then-compare approach fires once; users expecting per-improvement events would not receive them — but D-07 confirms global-best-only |

**If this table were empty:** All claims in this research were verified or cited. The two assumed items (A1, A2) are low-risk: A1 has a known workaround, and A2 is clarified by D-07 in CONTEXT.md.

---

## Open Questions (RESOLVED)

1. **`on_new_best` for CellularEngine: per-cell or per-generation?**
   - What we know: The inner loop updates `best` per-cell; D-03 doesn't explicitly address granularity for Cellular.
   - What's unclear: Whether users would expect `on_new_best` to fire mid-sweep (per improving cell) or post-sweep (once per generation).
   - RESOLVED: Fire once per generation (post-sweep) using the snapshot pattern. This is consistent with all other engines, which fire at most once per generation. Record `prev_best_fitness` before the inner cell sweep; compare after the sweep and fire if improved.

2. **Stats history in engine Result structs**
   - What we know: `DeResult`, `ScatterResult`, `CellularResult`, `AlpsResult` do not currently expose a `stats` field.
   - What's unclear: Should the per-run stats be exposed as part of the Result, or only via the observer?
   - RESOLVED: Use a local `Vec<GenerationStats>` in `run()` — no Result field change, observer-only access. Adding to Result is a non-breaking enhancement deferred to a later phase.

---

## Environment Availability

Phase 2.6 SKIPPED — no external dependencies. All tools (`cargo`, `criterion`) are already in the project's `Cargo.toml`.

---

## Security Domain

Not applicable — this phase adds observer hook call sites and a benchmark group. No authentication, input validation, cryptography, or external service calls involved.

---

## Sources

### Primary (HIGH confidence — verified by direct code inspection this session)

- `src/engines/ga.rs` — canonical observer wiring pattern (field, `notify`, builder, 5 hook call sites)
- `src/observe/observer/mod.rs` — `GaObserver<U>` trait, `NoopObserver`, `TerminationCause` usage
- `src/stats.rs` — `GenerationStats::from_fitness_values(generation, &[f64], is_maximization)` signature
- `src/engines/de/engine.rs` — current `DeEngine` structure and run loop
- `src/engines/scatter/engine.rs` — current `ScatterEngine` structure and run loop
- `src/engines/cellular/engine.rs` — current `CellularEngine` structure and run loop
- `src/engines/alps/engine.rs` — current `AlpsEngine` structure and run loop
- `src/engines/de/configuration.rs`, `scatter/configuration.rs`, `cellular/configuration.rs`, `alps/configuration.rs` — builder patterns and struct fields
- `benches/de.rs` — existing benchmark structure (`make_pop`, `sphere`, `sample_size(10)`, `bench_mutation_strategies`)
- `tests/observe/observer/test_observer.rs` — `SpyData`/`SpyObserver` pattern for new engine tests
- `.planning/phases/30-observer-wiring-de-benchmark/30-CONTEXT.md` — all locked decisions (D-01 through D-10)

### Secondary (MEDIUM confidence)

- `.planning/STATE.md` — project accumulated decisions confirming `Option<Arc<dyn GaObserver<U>>>` pattern
- `.planning/REQUIREMENTS.md` — OBS-01 through OBS-05 acceptance criteria

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all types verified by direct code inspection
- Architecture: HIGH — wiring pattern lifted verbatim from `ga.rs`; per-engine analysis confirmed against each `engine.rs`
- Pitfalls: HIGH — all pitfalls are mechanically derived from engine code structure (prefixed loop vars, multi-cell best tracking, missing imports)
- Benchmark: HIGH — existing bench file inspected; pattern is clear

**Research date:** 2026-04-28
**Valid until:** 2026-06-01 (stable codebase; no fast-moving dependencies)
