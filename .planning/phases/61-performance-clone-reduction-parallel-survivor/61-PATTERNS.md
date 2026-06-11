# Phase 61: Performance — Clone Reduction & Parallel Survivor - Pattern Map

**Mapped:** 2026-06-08
**Files analyzed:** 10 (8 modified, 1 new source, 1 new bench)
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/engines/ga.rs` | engine/orchestrator | event-driven | self (existing hot path) | exact (in-place edit) |
| `src/observe/observer/mod.rs` | trait definition | request-response | self (existing trait) | exact (in-place edit) |
| `src/observe/observer/log.rs` | observer impl | request-response | self + `composite.rs` | exact (mechanical sig change) |
| `src/observe/observer/composite.rs` | observer impl (fan-out) | request-response | self (existing fan-out) | exact (in-place edit) |
| `src/observe/observer/tracing_observer.rs` | observer impl (feature-gated) | request-response | `log.rs` | role-match |
| `src/observe/observer/metrics_observer.rs` | observer impl (feature-gated) | request-response | `log.rs` | role-match |
| `src/operations/survivor/fitness.rs` | operator | CRUD/transform | self (existing sort) | exact (mechanical sort upgrade) |
| `src/operations/survivor/mu_plus_lambda.rs` | operator | CRUD/transform | `fitness.rs` | exact (identical sort structure) |
| `src/operations/survivor/age.rs` | operator | CRUD/transform | `fitness.rs` | role-match (sort_by_key variant) |
| `src/operations/survivor/mu_comma_lambda.rs` | operator | CRUD/transform | `fitness.rs` | role-match (partial — sort on sub-vec) |
| `tests/observe/observer/test_observer.rs` | test | request-response | self (existing test) | exact (mechanical sig change) |
| `tests/gp.rs` | test | request-response | self (existing test) | exact (mechanical sig change) |
| `Cargo.toml` | config | — | self (existing `[[bench]]` entries) | exact |
| `benches/rastrigin.rs` | benchmark | batch | `benches/ga_run.rs` | exact |

---

## Pattern Assignments

### `src/operations/survivor/fitness.rs` (operator, CRUD/transform)

**Analog:** self — this file is the primary pattern that `mu_plus_lambda.rs`, `age.rs`, and `mu_comma_lambda.rs` will also follow.

**Current imports** (lines 1-12 of fitness.rs):
```rust
pub(crate) use crate::{
    configuration::{LimitConfiguration, ProblemSolving},
    traits::ChromosomeT,
};
use log::{debug, trace};
```

**Rayon import to add** (mirror of `src/engines/ga.rs` lines 157-158):
```rust
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
```

**Current sort pattern** (fitness.rs lines 32-47 — the full `sort_by` block to replace):
```rust
if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
    chromosomes.sort_by(|a, b| {
        b.fitness()
            .partial_cmp(&a.fitness())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
} else {
    let target = limit_configuration.fitness_target.unwrap_or(0.0);
    chromosomes.sort_by(|a, b| {
        b.fitness_distance(&target)
            .partial_cmp(&a.fitness_distance(&target))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
```

**Target pattern — WASM-gated parallel sort** (replace both `sort_by` calls):
```rust
if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
    #[cfg(not(target_arch = "wasm32"))]
    chromosomes.par_sort_unstable_by(|a, b| {
        b.fitness()
            .partial_cmp(&a.fitness())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    #[cfg(target_arch = "wasm32")]
    chromosomes.sort_unstable_by(|a, b| {
        b.fitness()
            .partial_cmp(&a.fitness())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
} else {
    let target = limit_configuration.fitness_target.unwrap_or(0.0);
    #[cfg(not(target_arch = "wasm32"))]
    chromosomes.par_sort_unstable_by(|a, b| {
        b.fitness_distance(&target)
            .partial_cmp(&a.fitness_distance(&target))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    #[cfg(target_arch = "wasm32")]
    chromosomes.sort_unstable_by(|a, b| {
        b.fitness_distance(&target)
            .partial_cmp(&a.fitness_distance(&target))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
// truncate/drain logic unchanged (lines 52-65)
```

**Key:** `target: f64` is `Copy` — captured in both parallel and sequential closures without issue. Truncate/drain logic after the sort is unchanged.

---

### `src/operations/survivor/mu_plus_lambda.rs` (operator, CRUD/transform)

**Analog:** `src/operations/survivor/fitness.rs` — identical `sort_by` structure (two branches, same comparators).

**Current sort block** (mu_plus_lambda.rs lines 31-44 — exact copy of fitness.rs structure):
```rust
if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
    chromosomes.sort_by(|a, b| {
        b.fitness()
            .partial_cmp(&a.fitness())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
} else {
    let target = limit_configuration.fitness_target.unwrap_or(0.0);
    chromosomes.sort_by(|a, b| {
        b.fitness_distance(&target)
            .partial_cmp(&a.fitness_distance(&target))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
```

**Target pattern:** Apply the exact same transformation as `fitness.rs` above — add `#[cfg(not(target_arch = "wasm32"))] use rayon::prelude::*;` at top, replace both `sort_by` with the dual-cfg `par_sort_unstable_by` / `sort_unstable_by` blocks.

---

### `src/operations/survivor/age.rs` (operator, CRUD/transform)

**Analog:** `src/operations/survivor/fitness.rs` — same role, single-sort variant.

**Current sort** (age.rs line 20):
```rust
chromosomes.sort_by_key(|a| std::cmp::Reverse(a.age()));
```

**Target pattern** (replace line 20 with dual-cfg block):
```rust
#[cfg(not(target_arch = "wasm32"))]
chromosomes.par_sort_unstable_by(|a, b| b.age().cmp(&a.age()));
#[cfg(target_arch = "wasm32")]
chromosomes.sort_unstable_by(|a, b| b.age().cmp(&a.age()));
```

**Note:** `age() -> usize` implements `Ord` — `cmp` is the right comparator. The `Reverse` wrapper is replaced by reversing the operands (`b.age().cmp(&a.age())`). No rayon import needed in the `#[cfg(target_arch = "wasm32")]` path.

---

### `src/operations/survivor/mu_comma_lambda.rs` (operator, CRUD/transform)

**Analog:** `src/operations/survivor/fitness.rs` — same sort pattern applied to sub-vec only.

**Current sort block** (mu_comma_lambda.rs lines 41-54 — applies only when `chromosomes.len() > population_size`):
```rust
if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
    chromosomes.sort_by(|a, b| {
        b.fitness()
            .partial_cmp(&a.fitness())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
} else {
    let target = limit_configuration.fitness_target.unwrap_or(0.0);
    chromosomes.sort_by(|a, b| {
        b.fitness_distance(&target)
            .partial_cmp(&a.fitness_distance(&target))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
```

**Context:** At this point `chromosomes` contains ONLY offspring (age==0) — the `retain(|c| c.age() == 0)` call at line 32 already discarded parents. The sort applies to the offspring sub-vec, not the full combined population. The pattern is otherwise identical to `fitness.rs`.

**Target pattern:** Same dual-cfg transformation as `fitness.rs`. Add rayon import at top. Replace both `sort_by` calls in this block with `par_sort_unstable_by` / `sort_unstable_by` dual-cfg.

---

### `src/observe/observer/mod.rs` (trait definition, request-response)

**Analog:** self — in-place breaking change to the trait signature.

**Current signature** (mod.rs line 112):
```rust
fn on_new_best(&self, _generation: usize, _best: U) {}
```

**Target signature:**
```rust
fn on_new_best(&self, _generation: usize, _best: &U) {}
```

**Propagation:** This single change forces all implementing types to update their signatures. The trait's `Send + Sync` supertrait bounds and the `Arc<dyn GaObserver<U>>` ownership model are unchanged.

---

### `src/observe/observer/log.rs` (observer impl, request-response)

**Analog:** self — mechanical signature update only.

**Current signature** (log.rs line 111):
```rust
fn on_new_best(&self, _generation: usize, _best: U) {
    // No direct log call existed for new best
}
```

**Target signature:**
```rust
fn on_new_best(&self, _generation: usize, _best: &U) {
    // No direct log call existed for new best
}
```

**Note:** `_best` is unused in the body — the rename from `U` to `&U` is truly a no-op in the function body. All other hooks in `log.rs` pass values by reference or scalar types and are unchanged.

---

### `src/observe/observer/composite.rs` (observer impl fan-out, request-response)

**Analog:** self — this is the critical secondary clone site.

**Current fan-out** (composite.rs lines 149-153):
```rust
fn on_new_best(&self, generation: usize, best: U) {
    for obs in &self.observers {
        obs.on_new_best(generation, best.clone());
    }
}
```

**Target fan-out** (zero-copy — `best: &U` passes to all inner observers without clone):
```rust
fn on_new_best(&self, generation: usize, best: &U) {
    for obs in &self.observers {
        obs.on_new_best(generation, best);
    }
}
```

**Why this matters:** With `best: U` (owned), each inner observer consumed the value, requiring `best.clone()` for all-but-last. With `best: &U` (reference), all inner observers receive the same reference — zero clones in the loop.

---

### `src/observe/observer/tracing_observer.rs` (feature-gated impl, request-response)

**Analog:** `src/observe/observer/log.rs` — same mechanical signature change.

**Pattern:** Locate `on_new_best` in the file, change parameter from `best: U` to `best: &U`. If the body uses `best`, it must now borrow (e.g., clone internally if needed — that's the observer's choice, not the caller's).

---

### `src/observe/observer/metrics_observer.rs` (feature-gated impl, request-response)

**Analog:** `src/observe/observer/log.rs` — same mechanical signature change.

**Pattern:** Same as `tracing_observer.rs` above. Locate `on_new_best`, change `best: U` to `best: &U`.

---

### `src/engines/ga.rs` (engine orchestrator, event-driven)

**Analog:** self — two targeted edits in the crossover inner loop and the observer call site.

**Edit 1 — on_new_best call site** (ga.rs line 2285, current):
```rust
self.notify(|obs| obs.on_new_best(i, self.population.best_chromosome.clone()));
```

**Target** (drop the `.clone()` — pass reference matching new `&U` signature):
```rust
self.notify(|obs| obs.on_new_best(i, &self.population.best_chromosome));
```

**Edit 2 — crossover fallback clones** (ga.rs lines 2915-2917, current):
```rust
} else {
    child_1 = parent_1.clone();
    child_2 = parent_2.clone();
}
```

**Context:** `parent_1` and `parent_2` are `&U` references from `chromosomes.get(key/value)` within a rayon `par_iter` closure that borrows `chromosomes: &[U]`. Direct ownership transfer is not possible without restructuring the borrow. The correct approach is a conditional clone that fires only on the no-crossover branch rather than the current unconditional clone:

The `child_2 = children.pop().unwrap_or_else(|| parent_1.clone())` at line 2914 (single-offspring path) is a separate clone site already present for the multi-parent dispatch path — keep it as-is per D-02 scope note.

The primary target is the `else` block (lines 2915-2917). The existing comment at line 2980 states "Children from `parent.clone()` (the else branch above) already carry the correct fitness fn from their parent" — the fitness-fn injection block at lines 2982-2987 re-applies the fn unconditionally, so the else-branch children are safe regardless of how they were constructed.

**WASM gate reference** — existing dual-cfg pattern in ga.rs (lines 3026-3029):
```rust
#[cfg(not(target_arch = "wasm32"))]
let results: Vec<Result<Vec<U>, GaError>> = parents.par_iter().map(process_pair).collect();
#[cfg(target_arch = "wasm32")]
let results: Vec<Result<Vec<U>, GaError>> = parents.iter().map(process_pair).collect();
```

---

### `tests/observe/observer/test_observer.rs` (test, request-response)

**Analog:** self — mechanical signature update to match new `&U` trait.

**Current `SpyObserver::on_new_best`** (line 79):
```rust
fn on_new_best(&self, _generation: usize, _best: BinaryChromosome) {
    self.data.new_best.fetch_add(1, Ordering::Relaxed);
}
```

**Target:**
```rust
fn on_new_best(&self, _generation: usize, _best: &BinaryChromosome) {
    self.data.new_best.fetch_add(1, Ordering::Relaxed);
}
```

**Note:** `CountingObserver` (line 247) does NOT implement `on_new_best` — it only implements `on_generation_end`. No change needed for `CountingObserver`. Verify no other `on_new_best` impls exist in this file.

---

### `tests/gp.rs` (test, request-response)

**Analog:** self — mechanical signature update.

**Current `StatsCollector::impl GaObserver<GpChromosome<TestNode>>`** (lines 480-484):
```rust
impl GaObserver<GpChromosome<TestNode>> for StatsCollector {
    fn on_generation_end(&self, stats: &GenerationStats) {
        self.stats.lock().unwrap().push(stats.clone());
    }
}
```

**Note:** `StatsCollector` does NOT implement `on_new_best` — it uses the default no-op. No explicit change needed here unless other `GaObserver` impls in `tests/gp.rs` override `on_new_best`. Verify by searching the file before marking done.

---

### `benches/rastrigin.rs` (benchmark, batch) — NEW FILE

**Analog:** `benches/ga_run.rs` — follow exactly.

**Imports pattern** (from ga_run.rs lines 1-17, adapted for RangeChromosome):
```rust
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{ConfigurationT, SelectionConfig, CrossoverConfig,
    MutationConfig, StoppingConfig, SurvivorConfig};
// RangeChromosome and RangeGenotype imports — verify exact public path
```

**Rastrigin fitness function** (inline, not a public library function):
```rust
fn rastrigin(genes: &[RangeGenotype<f64>]) -> f64 {
    let a = 10.0_f64;
    let n = genes.len() as f64;
    a * n + genes.iter().map(|g| {
        let x = g.value();  // RangeGenotype::value() confirmed at range.rs line 110
        x * x - a * (2.0 * std::f64::consts::PI * x).cos()
    }).sum::<f64>()
}
```

**build_ga helper** (follow ga_run.rs `build_ga` pattern, lines 123-139):
```rust
fn build_rastrigin_ga(population_size: usize, dims: usize, max_generations: usize)
    -> Ga<RangeChromosome<f64>>
{
    // Construct population of RangeChromosome<f64> with dims genes, bounds [-5.12, 5.12]
    // Then:
    Ga::new()
        .with_problem_solving(ProblemSolving::Minimization)  // Rastrigin is minimization
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(max_generations)
        .with_population(population)
}
```

**Benchmark body** (follow ga_run.rs lines 146-179 pattern exactly):
```rust
fn benchmark_rastrigin(c: &mut Criterion) {
    let mut group = c.benchmark_group("rastrigin");
    let dims: Vec<usize> = vec![10, 20, 50];
    for &dim in &dims {
        group.bench_with_input(
            BenchmarkId::new("Ga::run", format!("pop_500_dim_{}", dim)),
            &dim,
            |b, &d| {
                b.iter_batched(
                    || build_rastrigin_ga(500, d, 50),
                    |mut ga| { let _ = ga.run(); },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = rastrigin_benchmarks;
    config = Criterion::default();
    targets = benchmark_rastrigin
}
criterion_main!(rastrigin_benchmarks);
```

**Key discretion decisions:**
- `BatchSize::SmallInput` — matches ga_run.rs; `SmallInput` is correct for setup closures that construct a full `Ga` struct.
- `max_generations = 50` — 25,000 fitness evaluations at pop=500 provides enough measurement signal while keeping each iteration under ~1-2 seconds.
- Problem solving: `Minimization` — Rastrigin's global minimum is 0.0; GA tries to minimize.

---

### `Cargo.toml` (config)

**Analog:** existing `[[bench]]` entries in Cargo.toml.

**Add entry** (pattern: add after existing `[[bench]]` blocks):
```toml
[[bench]]
name = "rastrigin"
harness = false
```

---

## Shared Patterns

### WASM-Gated Rayon Import
**Source:** `src/engines/ga.rs` lines 157-158
**Apply to:** All four survivor operator files (`fitness.rs`, `mu_plus_lambda.rs`, `age.rs`, `mu_comma_lambda.rs`)
```rust
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
```

### WASM-Gated Parallel vs Sequential Expression (duplicate iterator, share closure body)
**Source:** `src/engines/ga.rs` lines 3026-3029
**Apply to:** All `sort_by` replacements in the four survivor operator files
```rust
#[cfg(not(target_arch = "wasm32"))]
collection.par_sort_unstable_by(|a, b| { /* comparator */ });
#[cfg(target_arch = "wasm32")]
collection.sort_unstable_by(|a, b| { /* same comparator */ });
```

### Observer Fan-out (CompositeObserver)
**Source:** `src/observe/observer/composite.rs` lines 149-153
**Change:** `best: U` + `best.clone()` in loop → `best: &U` + `best` (no clone) in loop
**Apply to:** `composite.rs::on_new_best` only

### Observer Impl Signature (no-op body)
**Source:** `src/observe/observer/log.rs` line 111
**Apply to:** `LogObserver`, `TracingObserver`, `MetricsObserver` — all change `_best: U` to `_best: &U`; body is unchanged for no-op impls

### Logging Style in Survivor Ops
**Source:** `src/operations/survivor/fitness.rs` lines 31, 51, 67
**Apply to:** All survivor files — keep existing `debug!(target="survivor_events", method=...; ...)` and `trace!(target="survivor_events", method=...; ...)` calls unchanged around the sort block. The log calls bracket the sort; only the sort expression itself changes.

---

## No Analog Found

No files in this phase lack a codebase analog. All patterns are derived from existing project code.

---

## Metadata

**Analog search scope:**
- `src/engines/ga.rs`
- `src/observe/observer/` (mod.rs, log.rs, composite.rs)
- `src/operations/survivor/` (fitness.rs, mu_plus_lambda.rs, age.rs, mu_comma_lambda.rs)
- `benches/ga_run.rs`
- `tests/observe/observer/test_observer.rs`
- `tests/gp.rs`
- `src/types/genotypes/range.rs` (value accessor verification)

**Files scanned:** 14
**Pattern extraction date:** 2026-06-08
