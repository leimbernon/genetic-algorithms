# Phase 58: EDA / UMDA Engine - Pattern Map

**Mapped:** 2026-06-04
**Files analyzed:** 6
**Analogs found:** 6 / 6

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/engines/eda/engine.rs` | engine | request-response | `src/engines/pso/engine.rs` | exact |
| `src/engines/eda/configuration.rs` | config | — | `src/engines/pso/configuration.rs` | exact |
| `src/engines/eda/mod.rs` | module | — | `src/engines/pso/mod.rs` | exact |
| `src/lib.rs` (modification) | registration | — | `src/lib.rs` lines 333–366 | exact |
| `tests/engines/eda/test_eda.rs` | test | — | `tests/engines/pso/test_pso.rs` | exact |
| `examples/eda_trap.rs` | example | — | `examples/pso_rastrigin.rs` | exact |

---

## Pattern Assignments

### `src/engines/eda/engine.rs` (engine, request-response)

**Analog:** `src/engines/pso/engine.rs`

**Imports pattern** (lines 14–27):
```rust
use std::borrow::Cow;
use std::sync::Arc;

use rand::Rng;

use crate::configuration::ProblemSolving;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::rng::make_rng;
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, LinearChromosome, RealGene};

use super::configuration::EdaConfiguration;
```

**Result struct pattern** (lines 30–40):
```rust
pub struct PsoResult<U: LinearChromosome> {
    pub population: Vec<U>,
    pub best: U,
    pub best_fitness: f64,
    pub generations: usize,
}
```
Copy verbatim as `EdaResult<U>`, adding `pub learned_model: EdaModel` field per D-03.

**EdaModel enum** (new, no direct analog — defined per D-04):
```rust
pub enum EdaModel {
    Bernoulli(Vec<f64>),
    Gaussian { means: Vec<f64>, stds: Vec<f64> },
}
```
Derive `Debug`, `Clone`; add `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` following the `Binary` chromosome pattern (`src/types/chromosomes/binary.rs` line 23).

**Engine struct pattern** (lines 155–163):
```rust
pub struct PsoEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: PsoConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}
```
For `EdaEngine`, the `where U::Gene: RealGene` bound is NOT placed on the struct — the struct accepts any `U: LinearChromosome` (D-01). The `RealGene` bound appears only on the Gaussian-path `impl` block.

**Constructor + observer builder + notify pattern** (lines 165–200):
```rust
impl<U: LinearChromosome + Clone> PsoEngine<U>
where
    U::Gene: RealGene,
{
    pub fn new(
        config: PsoConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            observer: None,
        }
    }

    pub fn with_observer(mut self, obs: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
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
Copy verbatim; remove `where U::Gene: RealGene` from the base `impl` block. Add `is_better` and `reached_target` helpers copied from lines 205–226.

**`run()` method structure** (lines 285–458, key skeleton):
```rust
pub fn run(&mut self) -> PsoResult<U> {
    let mut rng = make_rng();
    let is_maximization =
        matches!(self.config.problem_solving, ProblemSolving::Maximization);

    // Observer: run start
    self.notify(|obs| obs.on_run_start());

    // Determine population size (default 100 when 0 for EDA)
    let pop_size = if self.config.population_size == 0 { 100 } else { self.config.population_size };

    // Build and evaluate initial population
    let mut pop: Vec<U> = (self.init_fn)(pop_size.max(1));
    if pop.is_empty() { panic!("EdaEngine: init_fn returned an empty population"); }
    for ind in &mut pop {
        let f = (self.fitness_fn)(ind.dna());
        ind.set_fitness(f);
    }

    // Identify best from initial population
    let (best_idx, mut best_fitness) = self.find_best(&pop);
    let mut best = pop[best_idx].clone();

    // Observer: initial best
    self.notify(|obs| obs.on_new_best(0, best.clone()));

    let mut termination_cause = TerminationCause::GenerationLimitReached;
    let mut all_stats: Vec<GenerationStats> =
        Vec::with_capacity(self.config.max_generations);

    let mut last_model = EdaModel::Bernoulli(vec![]);  // updated each generation

    // Main loop
    for gen in 0..self.config.max_generations {
        self.notify(|obs| obs.on_generation_start(gen));

        // [EDA-specific: select parents, estimate model, sample offspring, evaluate]

        // Update best
        // ...
        if self.is_better(new_best_fit, best_fitness) {
            best_fitness = new_best_fit;
            best = /* updated best */;
            let best_clone = best.clone();
            self.notify(|obs| obs.on_new_best(gen, best_clone));
        }

        // Stats
        let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
        let stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
        self.notify(|obs| obs.on_generation_end(&stats));
        all_stats.push(stats);

        // Early stopping
        if let Some(target) = self.config.fitness_target {
            if self.reached_target(best_fitness, target) {
                termination_cause = TerminationCause::FitnessTargetReached;
                break;
            }
        }
    }

    // Observer: run end
    let generations = all_stats.len();
    let all_stats_ref = all_stats.as_slice();
    self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));

    EdaResult { population: pop, best, best_fitness, generations, learned_model: last_model }
}
```

**WASM gate pattern** — PSO engine uses NO `Instant::now()` at all (confirmed: no `std::time` import in `pso/engine.rs`). EDA follows the same approach: omit all timing. No `#[cfg]` gates needed beyond the rayon gate for fitness evaluation:
```rust
// Fitness evaluation of offspring (rayon gate per CLAUDE.md):
#[cfg(not(target_arch = "wasm32"))]
let evaluated: Vec<U> = offspring.par_iter_mut().map(|ind| { ... }).collect();
#[cfg(target_arch = "wasm32")]
let evaluated: Vec<U> = offspring.iter_mut().map(|ind| { ... }).collect();
```

**Bernoulli estimation + sampling** (sourced from 58-RESEARCH.md §Code Examples):
```rust
// Estimation: gene.id() == 1 used as binary indicator (user contract: Binary genes
// must be constructed with id = value as i32, NOT id = position).
fn estimate_bernoulli(selected: &[&U], dna_len: usize) -> Vec<f64> {
    let n = selected.len() as f64;
    (0..dna_len)
        .map(|i| {
            let count: f64 = selected.iter()
                .map(|c| if c.dna()[i].id() == 1 { 1.0 } else { 0.0 })
                .sum();
            (count / n).clamp(0.01, 0.99)
        })
        .collect()
}

// Sampling: clone template gene and set id to 0 or 1 via GeneT::set_id()
fn sample_bernoulli<U: LinearChromosome + Clone, R: Rng>(
    probs: &[f64], template: &U, rng: &mut R,
) -> U {
    let new_dna: Vec<U::Gene> = probs.iter().enumerate().map(|(i, &p)| {
        let one = rng.random::<f64>() < p;
        let mut g = template.dna()[i].clone();
        g.set_id(if one { 1 } else { 0 });
        g
    }).collect();
    let mut offspring = template.clone();
    offspring.set_dna(Cow::Owned(new_dna));
    offspring
}
```

**Gaussian estimation + sampling** (requires `U::Gene: RealGene` bound on impl block):
```rust
fn estimate_gaussian<U>(selected: &[&U], dna_len: usize) -> (Vec<f64>, Vec<f64>)
where U: LinearChromosome, U::Gene: RealGene,
{
    let n = selected.len() as f64;
    let means: Vec<f64> = (0..dna_len).map(|i| {
        selected.iter().map(|c| c.dna()[i].real_value()).sum::<f64>() / n
    }).collect();
    let stds: Vec<f64> = means.iter().enumerate().map(|(i, &mean)| {
        let variance = selected.iter()
            .map(|c| { let d = c.dna()[i].real_value() - mean; d * d })
            .sum::<f64>() / n;
        variance.sqrt().max(1e-6)
    }).collect();
    (means, stds)
}

fn sample_gaussian<U: LinearChromosome + Clone, R: Rng>(
    means: &[f64], stds: &[f64], template: &U, rng: &mut R,
) -> U
where U::Gene: RealGene,
{
    use std::f64::consts::PI;
    let new_dna: Vec<U::Gene> = template.dna().iter().enumerate().map(|(i, g)| {
        let u1: f64 = rng.random::<f64>().max(1e-300);
        let u2: f64 = rng.random();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        let v = means[i] + stds[i] * z;
        let v = if let Some((lo, hi)) = g.bounds() { v.clamp(lo, hi) } else { v };
        g.with_real_value(v)
    }).collect();
    let mut offspring = template.clone();
    offspring.set_dna(Cow::Owned(new_dna));
    offspring
}
```

**Dispatch mechanism** — Rust stable has no specialization. Recommended approach (decided for planner): provide a single `run()` method in the base `impl<U: LinearChromosome + Clone> EdaEngine<U>` block that calls a private `run_bernoulli_inner()`. Separately, provide `run_gaussian()` in a second `impl<U: LinearChromosome + Clone> EdaEngine<U> where U::Gene: RealGene` block. Alternatively (cleaner user API): a single `run()` that dispatches at runtime via a private `EdaSampler` trait with two implementations — but Rust stable prevents blanket-impl-based specialization. The simplest stable option: expose `EdaEngine::bernoulli(config, init_fn, fitness_fn)` and `EdaEngine::gaussian(config, init_fn, fitness_fn)` as separate constructors that store a `Box<dyn EdaSamplerInner<U>>` internally, OR simply use a single `run()` that always calls the Bernoulli path and document `run_gaussian()` as a separate method for `U::Gene: RealGene`. The planner must resolve this; the PATTERNS.md presents both options and extracts no further opinion.

---

### `src/engines/eda/configuration.rs` (config)

**Analog:** `src/engines/pso/configuration.rs`

**Imports pattern** (line 3):
```rust
use crate::configuration::ProblemSolving;
```

**Struct pattern** (lines 62–116):
```rust
#[derive(Debug, Clone)]
pub struct PsoConfiguration {
    pub population_size: usize,
    pub max_generations: usize,
    pub problem_solving: ProblemSolving,
    pub fitness_target: Option<f64>,
    // ... PSO-specific fields ...
}
```
`EdaConfiguration` mirrors this with `selection_ratio: f64` replacing PSO-specific fields:
```rust
#[derive(Debug, Clone)]
pub struct EdaConfiguration {
    pub population_size: usize,   // 0 → default 100 at run() time
    pub max_generations: usize,
    pub problem_solving: ProblemSolving,
    pub fitness_target: Option<f64>,
    pub selection_ratio: f64,     // default 0.5; clamped [1/pop_size, 1.0] at run time
}
```

**Default impl pattern** (lines 118–134):
```rust
impl Default for PsoConfiguration {
    fn default() -> Self {
        Self {
            population_size: 30,
            max_generations: 1000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
            // ...
        }
    }
}
```
For EDA: `population_size: 100`, `max_generations: 500`, `problem_solving: ProblemSolving::Maximization`, `fitness_target: None`, `selection_ratio: 0.5`.

**Builder method pattern** (lines 136–189):
```rust
impl PsoConfiguration {
    pub fn with_population_size(mut self, n: usize) -> Self {
        self.population_size = n;
        self
    }
    pub fn with_max_generations(mut self, n: usize) -> Self {
        self.max_generations = n;
        self
    }
    pub fn with_problem_solving(mut self, ps: ProblemSolving) -> Self {
        self.problem_solving = ps;
        self
    }
    pub fn with_fitness_target(mut self, t: f64) -> Self {
        self.fitness_target = Some(t);
        self
    }
    // Add: pub fn with_selection_ratio(mut self, r: f64) -> Self { ... }
}
```

---

### `src/engines/eda/mod.rs` (module wiring)

**Analog:** `src/engines/pso/mod.rs` (all 7 lines)

```rust
//! EDA engine. Univariate Marginal Distribution Algorithm (UMDA) for
//! binary and continuous optimization.

pub mod configuration;
pub mod engine;

pub use configuration::EdaConfiguration;
pub use engine::{EdaEngine, EdaModel, EdaResult};
```

---

### `src/lib.rs` (modification — engine registration)

**Analog:** `src/lib.rs` lines 333–334 (PSO registration) and line 366 (PSO re-export).

**Registration pattern** (lines 333–334):
```rust
#[path = "engines/pso/mod.rs"]
pub mod pso;
```
Add immediately after:
```rust
#[path = "engines/eda/mod.rs"]
pub mod eda;
```

**Re-export pattern** (line 366):
```rust
pub use pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoResult, PsoTopology};
```
Add:
```rust
pub use eda::{EdaConfiguration, EdaEngine, EdaModel, EdaResult};
```

---

### `tests/engines/eda/test_eda.rs` (test)

**Analog:** `tests/engines/pso/test_pso.rs`

**Imports pattern** (lines 7–21):
```rust
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::rng;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
```
For EDA tests: replace `pso::` with `eda::`, use `genetic_algorithms::chromosomes::Binary as BinaryChromosome` and `genetic_algorithms::genotypes::Binary as BinaryGene` for Bernoulli tests; use `RangeChromosome<f64>` / `RangeGene<f64>` for Gaussian tests.

**SpyObserver pattern** (lines 49–81):
```rust
#[derive(Default)]
struct SpyObserver {
    new_best_count: AtomicUsize,
    run_start_count: AtomicUsize,
    run_end_count: AtomicUsize,
    generation_start_count: AtomicUsize,
    generation_end_count: AtomicUsize,
}

impl GaObserver<RangeChromosome<f64>> for SpyObserver {
    fn on_run_start(&self) { self.run_start_count.fetch_add(1, Ordering::SeqCst); }
    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {
        self.run_end_count.fetch_add(1, Ordering::SeqCst);
    }
    fn on_new_best(&self, _generation: usize, _best: RangeChromosome<f64>) {
        self.new_best_count.fetch_add(1, Ordering::SeqCst);
    }
    fn on_generation_start(&self, _generation: usize) {
        self.generation_start_count.fetch_add(1, Ordering::SeqCst);
    }
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.generation_end_count.fetch_add(1, Ordering::SeqCst);
    }
}
```
Copy verbatim; parameterize for `BinaryChromosome` for Bernoulli observer tests.

**Result shape test pattern** (lines 87–103):
```rust
#[test]
fn test_pso_run_returns_result() {
    rng::set_seed(Some(1));
    let init_pop = random_pop(20, 10, -5.12, 5.12, 1);
    let config = PsoConfiguration::default()
        .with_max_generations(20)
        .with_population_size(20);
    let mut engine = PsoEngine::new(config, move |_n| init_pop.clone(), sphere);
    let result = engine.run();
    assert_eq!(result.population.len(), 20);
    assert_eq!(result.generations, 20);
    assert!(result.best_fitness.is_finite());
}
```
Adapt as `test_eda_run_returns_result` using `BinaryChromosome` + onemax fitness function.

**Observer hook count pattern** (lines 163–189):
```rust
assert_eq!(spy.generation_start_count.load(Ordering::SeqCst), result.generations);
assert_eq!(spy.generation_end_count.load(Ordering::SeqCst), result.generations);
```

**Wiring observer to engine** (line 151):
```rust
.with_observer(spy.clone() as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);
```

---

### `examples/eda_trap.rs` (example)

**Analog:** `examples/pso_rastrigin.rs`

**File header pattern** (lines 1–17):
```rust
/*!
# EDA (UMDA): Deceptive Trap Function

[Description block explaining algorithm and problem...]

Run with:
```sh
cargo run --release --example eda_trap
```
*/
```

**Imports pattern** (lines 19–30):
```rust
use std::borrow::Cow;
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};
use genetic_algorithms::rng;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
use genetic_algorithms::LogObserver;
use rand::Rng;
```
Adapt: replace `pso::` with `eda::`, replace `RangeChromosome/RangeGene` with `Binary as BinaryChromosome` / `Binary as BinaryGene`.

**init_population pattern** (lines 53–68) — copy structure; for `eda_trap`, genes must be constructed with `id = value as i32` (not positional) to satisfy Bernoulli indicator contract:
```rust
fn init_population(n: usize) -> Vec<BinaryChromosome> {
    let mut rng = rng::make_rng();
    (0..n).map(|_| {
        let dna: Vec<BinaryGene> = (0..CHROMOSOME_LEN).map(|_| {
            let v = rng.random_bool(0.5);
            BinaryGene { id: if v { 1 } else { 0 }, value: v }
        }).collect();
        let mut c = BinaryChromosome::new();
        c.set_dna(Cow::Owned(dna));
        c
    }).collect()
}
```

**main() pattern** (lines 70–109):
```rust
fn main() {
    rng::set_seed(Some(42));
    let config = EdaConfiguration {
        population_size: 200,
        max_generations: 500,
        problem_solving: ProblemSolving::Maximization,
        fitness_target: Some(/* trap max */),
        selection_ratio: 0.5,
    };
    let mut engine = EdaEngine::new(config, init_population, trap_fitness)
        .with_observer(Arc::new(LogObserver));
    println!("== EDA (UMDA): Deceptive Trap ==");
    let result = engine.run();
    println!("Generations: {}", result.generations);
    println!("Best fitness: {:.6}", result.best_fitness);
    // Print EdaModel probabilities to show convergence
    if let EdaModel::Bernoulli(probs) = &result.learned_model {
        println!("Learned probabilities: {:?}", probs);
    }
    assert!(result.best_fitness.is_finite());
}
```

---

## Shared Patterns

### Observer Wiring
**Source:** `src/engines/pso/engine.rs` lines 291, 322, 329–331, 412–427, 434, 447–449
**Apply to:** `src/engines/eda/engine.rs`
```rust
self.notify(|obs| obs.on_run_start());
// ... after initial population evaluated:
self.notify(|obs| obs.on_new_best(0, best.clone()));
// ... per generation:
self.notify(|obs| obs.on_generation_start(gen));
// ... when best improves:
let best_clone = best.clone();
self.notify(|obs| obs.on_new_best(gen, best_clone));
// ... end of generation:
self.notify(|obs| obs.on_generation_end(&stats));
// ... after loop:
self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));
```

### GenerationStats
**Source:** `src/stats.rs` line 60
**Apply to:** `src/engines/eda/engine.rs`
```rust
let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
let stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
```

### RNG Initialization
**Source:** `src/engines/pso/engine.rs` line 286
**Apply to:** All engine files and examples
```rust
let mut rng = make_rng();
// In examples and tests, seed first:
rng::set_seed(Some(42));
```

### serde Conditional Derivation
**Source:** `src/types/chromosomes/binary.rs` line 23, `src/types/genotypes/binary.rs` line 32
**Apply to:** `EdaModel` enum in `src/engines/eda/engine.rs`
```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdaModel { ... }
```

### lib.rs Re-export Placement
**Source:** `src/lib.rs` lines 333–334, 366
**Apply to:** `src/lib.rs` modification
Append `#[path]` mod declaration immediately after the `pso` registration block; append `pub use eda::` re-exports at the end of the existing `pub use` block.

### Population Size Default Guard
**Source:** `src/engines/pso/engine.rs` lines 294–298
**Apply to:** `src/engines/eda/engine.rs`
```rust
let pop_size = if self.config.population_size == 0 { 100 } else { self.config.population_size };
```

### Empty Population Guard
**Source:** `src/engines/pso/engine.rs` lines 304–306
**Apply to:** `src/engines/eda/engine.rs`
```rust
if pop.is_empty() {
    panic!("EdaEngine: init_fn returned an empty population");
}
```

---

## No Analog Found

No files in this phase lack an analog. All 6 files have exact or near-exact analogs in the codebase.

---

## Key Anti-Patterns (from 58-RESEARCH.md)

| Anti-Pattern | Source Finding | Mitigation |
|---|---|---|
| `gene.id()` as binary value indicator | `binary_initializer.rs` sets `id = i as i32` (positional) | `eda_trap` must use `id = value as i32`; document as user contract in engine doc comment |
| `par_iter` in model estimation | EDA estimation is sequential O(n·L) | Only parallelize fitness evaluation; gate with `#[cfg(not(target_arch = "wasm32"))]` |
| `Instant::now()` | PSO engine has NO timing code — EDA follows same | Do not import `std::time`; omit all timing |
| `EdaModel` NaN stds | Converged populations produce std=0 | `std_i = variance.sqrt().max(1e-6)` |
| `learned_model` captures initial state | Must update at end of each generation | Assign `last_model` at bottom of generation loop body, return at end |

---

## Metadata

**Analog search scope:** `src/engines/pso/`, `src/engines/cma/`, `tests/engines/pso/`, `examples/`, `src/lib.rs`, `src/types/`, `src/observe/`, `src/traits/`, `src/stats.rs`
**Files scanned:** 12
**Pattern extraction date:** 2026-06-04
