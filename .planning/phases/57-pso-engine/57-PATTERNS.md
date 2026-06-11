# Phase 57: PSO Engine - Pattern Map

**Mapped:** 2026-06-02
**Files analyzed:** 7 new/modified files
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/engines/pso/engine.rs` | engine | request-response (iterative loop) | `src/engines/cma/engine.rs` | exact |
| `src/engines/pso/configuration.rs` | config | — | `src/engines/cma/configuration.rs` | exact |
| `src/engines/pso/mod.rs` | module wiring | — | `src/engines/cma/mod.rs` | exact |
| `src/traits/real_gene.rs` | trait | — | `src/traits/real_gene.rs` (extension) | self-extension |
| `src/lib.rs` | lib wiring | — | `src/lib.rs` (cma block) | exact |
| `tests/engines/pso/test_pso.rs` | test | — | `tests/engines/cma/test_cma.rs` | exact |
| `examples/pso_rastrigin.rs` | example | — | `examples/cma_es_rastrigin.rs` | exact |

---

## Pattern Assignments

### `src/engines/pso/engine.rs` (engine, iterative loop)

**Analog:** `src/engines/cma/engine.rs`

**Imports pattern** (lines 13–24):
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

use super::configuration::PsoConfiguration;
```

**Internal state struct pattern** (lines 174–218 of analog — `CmaState`):
```rust
// Private struct allocated once before the run loop.
// PSO equivalent: replace CMA matrix/vector fields with PSO bookkeeping.
struct PsoState {
    /// Problem dimension (genes per chromosome).
    dim: usize,
    /// Number of particles.
    n_particles: usize,
    /// Velocities: [particle][gene].
    velocities: Vec<Vec<f64>>,
    /// Personal best positions: [particle][gene].
    pbest_positions: Vec<Vec<f64>>,
    /// Personal best fitness values: [particle].
    pbest_fitness: Vec<f64>,
    /// Global best position (gbest topology).
    gbest_position: Vec<f64>,
    /// Global best fitness.
    gbest_fitness: f64,
    /// v_max per gene (= hi_i - lo_i); auto-derived.
    v_max: Vec<f64>,
}
```

**Result struct pattern** (lines 297–306 of analog — `CmaResult`):
```rust
// Source: src/engines/cma/engine.rs lines 297-306
pub struct CmaResult<U: LinearChromosome> {
    pub population: Vec<U>,
    pub best: U,
    pub best_fitness: f64,
    pub generations: usize,
}
// PsoResult<U> follows the identical field set.
```

**Engine struct and constructor pattern** (lines 332–363 of analog):
```rust
// Source: src/engines/cma/engine.rs lines 332-363
pub struct CmaEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: CmaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: LinearChromosome + Clone> CmaEngine<U>
where
    U::Gene: RealGene,
{
    pub fn new(
        config: CmaConfiguration,
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
```

**is_better / find_best helpers** (lines 382–416 of analog):
```rust
// Source: src/engines/cma/engine.rs lines 382-416
#[inline]
fn is_better(&self, candidate: f64, current: f64) -> bool {
    match self.config.problem_solving {
        ProblemSolving::Minimization => candidate < current,
        ProblemSolving::Maximization => candidate > current,
        ProblemSolving::FixedFitness => {
            if let Some(t) = self.config.fitness_target {
                (candidate - t).abs() < (current - t).abs()
            } else {
                candidate < current
            }
        }
    }
}

fn find_best(&self, pop: &[U]) -> (usize, f64) {
    let mut best_idx = 0;
    let mut best_fit = pop[0].fitness();
    for (i, ind) in pop.iter().enumerate().skip(1) {
        if self.is_better(ind.fitness(), best_fit) {
            best_fit = ind.fitness();
            best_idx = i;
        }
    }
    (best_idx, best_fit)
}
```

**run() loop scaffold** (lines 419–720 of analog — abridged to key structural points):
```rust
// Source: src/engines/cma/engine.rs lines 419-720
pub fn run(&mut self) -> CmaResult<U> {
    let mut rng = make_rng();
    let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);

    self.notify(|obs| obs.on_run_start());                        // (1) on_run_start

    // --- Init population ---
    let mut pop: Vec<U> = (self.init_fn)(self.config.population_size.max(1));
    if pop.is_empty() { panic!("...init_fn returned empty population"); }
    let n = pop[0].dna().len();

    // --- Evaluate initial fitness ---
    for ind in &mut pop {
        let f = (self.fitness_fn)(ind.dna());
        ind.set_fitness(f);
    }

    // --- Initialize internal state (CmaState / PsoState) ---
    let mut state = CmaState::new(n, lambda, &self.config, initial_mean);

    // --- Initial best ---
    let (mut best_idx, mut best_fitness) = self.find_best(&pop);
    let mut best = pop[best_idx].clone();
    self.notify(|obs| obs.on_new_best(0, best.clone()));          // (2) initial on_new_best

    let mut termination_cause = TerminationCause::GenerationLimitReached;
    let mut all_stats: Vec<GenerationStats> = Vec::with_capacity(self.config.max_generations);

    // --- Main loop ---
    for gen in 0..self.config.max_generations {
        self.notify(|obs| obs.on_generation_start(gen));           // (3) on_generation_start

        // ... engine-specific update (CMA sampling / PSO velocity+position update) ...

        // --- Best tracking ---
        let (bi, bf) = self.find_best(&pop);
        if self.is_better(bf, best_fitness) {
            best_fitness = bf;
            best_idx = bi;
            best = pop[best_idx].clone();
            let best_clone = best.clone();
            self.notify(|obs| obs.on_new_best(gen, best_clone));   // (4) on_new_best
        }

        // --- Stats ---
        let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
        let stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
        self.notify(|obs| obs.on_generation_end(&stats));          // (5) on_generation_end
        all_stats.push(stats);

        // --- Early stopping ---
        if let Some(target) = self.config.fitness_target {
            if self.reached_target(best_fitness, target) {
                termination_cause = TerminationCause::FitnessTargetReached;
                break;
            }
        }
    }

    let generations = all_stats.len();
    let all_stats_ref = all_stats.as_slice();
    self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref)); // (6) on_run_end

    CmaResult { population: pop, best, best_fitness, generations }
}
```

**Gene read/write pattern for position update** (lines 527–534 of analog):
```rust
// Source: src/engines/cma/engine.rs lines 527-534
// Read: g.real_value()
// Write (non-mutating): g.with_real_value(new_x)
// Apply to chromosome:
let new_dna: Vec<U::Gene> = template
    .dna()
    .iter()
    .enumerate()
    .map(|(j, g)| g.with_real_value(x_k[j]))
    .collect();
let mut child = template.clone();
child.set_dna(Cow::Owned(new_dna));
```

**WASM-safe timing gate** (per CLAUDE.md pattern — used throughout engine code):
```rust
#[cfg(not(target_arch = "wasm32"))]
let t0 = std::time::Instant::now();
// ... work ...
#[cfg(not(target_arch = "wasm32"))]
let elapsed = t0.elapsed();
// PSO has no Instant usage needed (no timing in core loop), but the pattern
// must be used if timing is added for diagnostics.
```

**Warn on empty init_fn return** (lines 432–447 of analog):
```rust
// Source: src/engines/cma/engine.rs lines 432-447
if pop.is_empty() {
    log::warn!(
        target: "cma_events",
        "CmaEngine: init_fn returned an empty population; returning empty result"
    );
    self.notify(|obs| {
        obs.on_run_end(TerminationCause::GenerationLimitReached, &[])
    });
    panic!("CmaEngine: init_fn returned an empty population");
}
// PSO: same guard pattern, log target "pso_events"
```

---

### `src/engines/pso/configuration.rs` (config)

**Analog:** `src/engines/cma/configuration.rs`

**Imports pattern** (lines 1–4):
```rust
// Source: src/engines/cma/configuration.rs lines 1-4
use crate::configuration::ProblemSolving;
```

**Struct layout with doc-comments** (lines 6–69):
```rust
// Source: src/engines/cma/configuration.rs lines 6-69
// Pattern: each field gets a /// doc comment explaining purpose and defaults.
// PSO equivalent:
#[derive(Debug, Clone)]
pub struct PsoConfiguration {
    pub population_size: usize,      // 0 = auto (default 30); see default impl
    pub max_generations: usize,      // default: 1000
    pub problem_solving: ProblemSolving,
    pub fitness_target: Option<f64>,
    pub inertia: PsoInertia,         // default: LinearDecay { w_start: 0.9, w_end: 0.4 }
    pub c1: f64,                     // cognitive coefficient; default: 2.0
    pub c2: f64,                     // social coefficient; default: 2.0
    pub topology: PsoTopology,       // default: PsoTopology::Global
}
```

**Default impl pattern** (lines 71–85):
```rust
// Source: src/engines/cma/configuration.rs lines 71-85
impl Default for CmaConfiguration {
    fn default() -> Self {
        Self {
            sigma0: 0.3,
            population_size: 0,
            max_generations: 1000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
            cc: None, cs: None, c1: None, cmu: None,
        }
    }
}
// PsoConfiguration::default() follows the same shape.
```

**Builder method pattern** (lines 107–176):
```rust
// Source: src/engines/cma/configuration.rs lines 107-176
// Each builder method: pub fn with_X(mut self, v: T) -> Self { self.x = v; self }
// Examples:
pub fn with_population_size(mut self, n: usize) -> Self {
    self.population_size = n;
    self
}
pub fn with_max_generations(mut self, n: usize) -> Self {
    self.max_generations = n;
    self
}
pub fn with_fitness_target(mut self, t: f64) -> Self {
    self.fitness_target = Some(t);
    self
}
// PSO adds: with_inertia, with_c1, with_c2, with_topology, with_problem_solving
```

**Named constructor pattern** (lines 87–105):
```rust
// Source: src/engines/cma/configuration.rs lines 87-105
// CmaConfiguration::default_for_dim(n) sets population_size from Hansen's formula.
// PSO equivalent: PsoConfiguration::default() uses population_size = 30 directly
// (PSO doesn't use a dimension-aware formula per RESEARCH.md recommendation).
```

---

### `src/engines/pso/mod.rs` (module wiring)

**Analog:** `src/engines/cma/mod.rs` (lines 1–7)

**Full file pattern:**
```rust
// Source: src/engines/cma/mod.rs lines 1-7
//! CMA-ES engine. ...
pub mod configuration;
pub mod engine;

pub use configuration::CmaConfiguration;
pub use engine::{CmaEngine, CmaResult};

// PSO equivalent:
//! PSO engine. Particle Swarm Optimization for real-valued continuous optimization.
pub mod configuration;
pub mod engine;

pub use configuration::{PsoConfiguration, PsoInertia, PsoTopology};
pub use engine::{PsoEngine, PsoResult};
```

---

### `src/traits/real_gene.rs` (trait extension)

**Analog:** `src/traits/real_gene.rs` (self — non-breaking addition)

**Current trait surface** (lines 23–29):
```rust
// Source: src/traits/real_gene.rs lines 23-29
pub trait RealGene: GeneT {
    fn real_value(&self) -> f64;
    fn with_real_value(&self, value: f64) -> Self;
}
```

**Extension to add** (new method + two impls):
```rust
// Add to RealGene trait (non-breaking — default impl):
pub trait RealGene: GeneT {
    fn real_value(&self) -> f64;
    fn with_real_value(&self, value: f64) -> Self;

    /// Returns the `(lo, hi)` bounds for this gene, if available.
    ///
    /// Used by PSO for velocity initialization and boundary enforcement.
    /// Returns `None` for gene types that have no explicit bounds (PSO falls
    /// back to `v_max = 1.0` per gene when `None`).
    fn bounds(&self) -> Option<(f64, f64)> {
        None
    }
}

// Add to Range<f64> impl (lines 32-44 of analog):
// Source: src/traits/real_gene.rs lines 32-44
impl RealGene for Range<f64> {
    #[inline]
    fn real_value(&self) -> f64 { self.value }
    #[inline]
    fn with_real_value(&self, value: f64) -> Self { let mut g = self.clone(); g.value = value; g }
    // NEW:
    #[inline]
    fn bounds(&self) -> Option<(f64, f64)> {
        self.ranges.first().copied()
    }
}

// Add to MultiRangeGenotype<f64> impl (lines 47-58 of analog):
// Source: src/traits/real_gene.rs lines 47-58 — same pattern
impl RealGene for MultiRangeGenotype<f64> {
    // ... existing methods ...
    #[inline]
    fn bounds(&self) -> Option<(f64, f64)> {
        self.ranges.first().copied()
    }
}
```

**Range<f64> internal struct** (for reference — `src/types/genotypes/range.rs` lines 44–48):
```rust
// Source: src/types/genotypes/range.rs lines 44-48
pub struct Range<T> {
    pub id: i32,
    pub ranges: Arc<[(T, T)]>,   // ← bounds stored here; .first() gives (lo, hi)
    pub value: T,
}
```

---

### `src/lib.rs` (modification — add PSO module + re-exports)

**Analog:** `src/lib.rs` lines 330–332 (cma block) and lines 362–363 (trait re-exports)

**Module declaration insertion point** (after cma block, lines 330–331):
```rust
// Source: src/lib.rs lines 330-331
#[path = "engines/cma/mod.rs"]
pub mod cma;

// ADD AFTER:
#[path = "engines/pso/mod.rs"]
pub mod pso;
```

**Re-export insertion point** (after existing pub use blocks, ~line 362):
```rust
// Source: src/lib.rs lines 362-363
pub use traits::{LinearChromosome, OperatorCompat, RealGene, Strategy, VectorFitness};

// ADD (in the pub use block, alphabetical or grouped by engine):
pub use pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoResult, PsoTopology};
```

---

### `tests/engines/pso/test_pso.rs` (test)

**Analog:** `tests/engines/cma/test_cma.rs`

**Imports pattern** (lines 7–20):
```rust
// Source: tests/engines/cma/test_cma.rs lines 7-20
use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::rng;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome, RealGene};
use genetic_algorithms::ga::TerminationCause;
use rand::Rng;
// PSO equivalent: swap cma:: imports for pso:: imports; rest identical.
```

**SpyObserver pattern** (lines 52–80):
```rust
// Source: tests/engines/cma/test_cma.rs lines 52-80
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
// Copy verbatim for PSO tests — same type parameters.
```

**random_pop helper** (lines 30–46):
```rust
// Source: tests/engines/cma/test_cma.rs lines 30-46
fn random_pop(n: usize, dim: usize, lo: f64, hi: f64, seed: u64) -> Vec<RangeChromosome<f64>> {
    rng::set_seed(Some(seed));
    let mut r = rng::make_rng();
    (0..n)
        .map(|_| {
            let dna: Vec<RangeGene<f64>> = (0..dim)
                .map(|j| {
                    let v = r.random::<f64>() * (hi - lo) + lo;
                    RangeGene::new(j as i32, vec![(lo, hi)], v)
                })
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}
// Copy verbatim — PSO uses same chromosome type.
```

**Observer lifecycle test pattern** (lines 222–258):
```rust
// Source: tests/engines/cma/test_cma.rs lines 222-258
// on_run_start fires exactly once, on_run_end fires exactly once,
// on_generation_start == on_generation_end == result.generations.
assert_eq!(spy.run_start_count.load(Ordering::SeqCst), 1);
assert_eq!(spy.run_end_count.load(Ordering::SeqCst), 1);
assert_eq!(
    spy.generation_start_count.load(Ordering::SeqCst),
    result.generations,
);
assert_eq!(
    spy.generation_end_count.load(Ordering::SeqCst),
    result.generations,
);
```

**WASM placeholder test pattern** (lines 291–295):
```rust
// Source: tests/engines/cma/test_cma.rs lines 291-295
#[test]
#[ignore = "Plan 04 verifies WASM via cargo check --target wasm32-unknown-unknown"]
fn test_pso_wasm_compiles() {
    unimplemented!("WASM verification gate")
}
```

---

### `examples/pso_rastrigin.rs` (example)

**Analog:** `examples/cma_es_rastrigin.rs`

**Full file structure** (lines 1–96):
```rust
// Source: examples/cma_es_rastrigin.rs lines 1-96
// PSO version: replace cma:: with pso::, DIMENSIONS = 10, add topology/inertia config.

use std::f64::consts::PI;
use std::sync::Arc;
use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::LogObserver;
use rand::Rng;

const DIMENSIONS: usize = 10;      // 10D (vs 5D for CMA)
const SEARCH_LO: f64 = -5.12;
const SEARCH_HI: f64 = 5.12;

fn rastrigin(dna: &[RangeGene<f64>]) -> f64 {
    let n = dna.len() as f64;
    10.0 * n + dna.iter()
        .map(|g| { let x = g.real_value(); x * x - 10.0 * (2.0 * PI * x).cos() })
        .sum::<f64>()
}

fn init_population(n: usize) -> Vec<RangeChromosome<f64>> {
    // Source: examples/cma_es_rastrigin.rs lines 49-65 — identical pattern
    rng::set_seed(Some(42));
    let mut r = rng::make_rng();
    (0..n).map(|_| {
        let dna: Vec<RangeGene<f64>> = (0..DIMENSIONS)
            .map(|j| { let v = r.random::<f64>() * (SEARCH_HI - SEARCH_LO) + SEARCH_LO;
                       RangeGene::new(j as i32, vec![(SEARCH_LO, SEARCH_HI)], v) })
            .collect();
        let mut c = <RangeChromosome<f64> as Default>::default();
        c.set_dna(Cow::Owned(dna));
        c
    }).collect()
}

fn main() {
    let config = PsoConfiguration {
        population_size: 30,
        max_generations: 1000,
        problem_solving: ProblemSolving::Minimization,
        fitness_target: Some(1e-3),
        inertia: PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 },
        c1: 2.0,
        c2: 2.0,
        topology: PsoTopology::Global,
    };

    let mut engine = PsoEngine::new(config, init_population, rastrigin)
        .with_observer(Arc::new(LogObserver));

    println!("== PSO: {DIMENSIONS}D Rastrigin Minimization ==");
    let result = engine.run();
    println!("Generations: {}", result.generations);
    println!("Best fitness: {:.6}", result.best_fitness);
    // ... print best DNA as in cma_es_rastrigin.rs lines 85-91 ...
}
```

---

## Shared Patterns

### Observer wiring (`notify` helper)
**Source:** `src/engines/cma/engine.rs` lines 372–377
**Apply to:** `src/engines/pso/engine.rs`
```rust
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### Observer import path
**Source:** `src/lib.rs` line 282 and `tests/engines/cma/test_cma.rs` line 15
**Apply to:** all PSO source and test files
```rust
use crate::observer::GaObserver;           // in src/ files
use genetic_algorithms::observer::GaObserver;  // in tests/ files
```

### RNG initialization
**Source:** `src/engines/cma/engine.rs` line 421
**Apply to:** `src/engines/pso/engine.rs`
```rust
let mut rng = make_rng();
// use rng.random::<f64>() for r1, r2 per-gene draws
```

### GenerationStats construction
**Source:** `src/engines/cma/engine.rs` lines 696–698
**Apply to:** `src/engines/pso/engine.rs`
```rust
let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
let stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
self.notify(|obs| obs.on_generation_end(&stats));
all_stats.push(stats);
```

### WASM timing gate
**Source:** `src/engines/cma/engine.rs` (no timing in CMA core loop — consistent with PSO)
**Apply to:** `src/engines/pso/engine.rs`
```rust
// PSO core loop needs no timing. If diagnostics are added later:
#[cfg(not(target_arch = "wasm32"))]
let t0 = std::time::Instant::now();
```

### Cow::Owned DNA write
**Source:** `src/engines/cma/engine.rs` lines 533–534
**Apply to:** PSO position update step
```rust
child.set_dna(Cow::Owned(new_dna));
```

### Log target naming convention
**Source:** `src/engines/cma/engine.rs` line 434
**Apply to:** `src/engines/pso/engine.rs`
```rust
log::warn!(target: "pso_events", "PsoEngine: ...");
// pattern: engine name + "_events" as target string
```

---

## No Analog Found

All 7 files have strong analogs. No files require falling back to RESEARCH.md patterns alone.

| File | Notes |
|------|-------|
| PSO velocity update algorithm | No analog exists (novel algorithm). Use the velocity formula from `57-CONTEXT.md` / `57-RESEARCH.md` Pattern 6. The structural shell (loop, state struct, observer hooks) copies from `CmaEngine`. |

---

## Critical Implementation Notes for Planner

### Bounds access decision (Research Option A — recommended)
`RealGene` has no `bounds()` method (verified: `src/traits/real_gene.rs` lines 23–29). PSO requires per-gene `(lo, hi)` for velocity init and boundary enforcement. The pattern assignment above includes the non-breaking addition of `fn bounds(&self) -> Option<(f64, f64)>` with `default None`. The `Range<f64>` impl returns `self.ranges.first().copied()` (field `ranges: Arc<[(T, T)]>` verified at `src/types/genotypes/range.rs` line 46).

### Population size default
`CmaEngine` uses 0 for auto-compute; PSO should default `population_size = 30` (PSO literature standard, dimension-independent). When `population_size == 0`, auto-default to 30.

### Ring topology index wrapping
Use `(i + n_particles - offset) % n_particles` for left-side wrap (Rust `%` on negative values is not `n-1`). Clamp `neighborhood_size` to `n_particles - 1` if `neighborhood_size >= n_particles`.

### Synchronous gbest update
Update gbest once after the full particle sweep per generation (not mid-sweep). This matches the CONTEXT.md formula spec and produces deterministic behaviour with a fixed seed.

### Test file location
`tests/engines/pso/test_pso.rs` — requires creating `tests/engines/pso/` directory and a `mod.rs` or `Cargo.toml` discovery entry. Follow the pattern in `tests/engines/cma/` (single file, no extra mod.rs needed if using integration test discovery).

---

## Metadata

**Analog search scope:** `src/engines/cma/`, `src/engines/de/`, `src/traits/`, `src/lib.rs`, `tests/engines/cma/`, `examples/`
**Files scanned:** 9 source files read in full
**Pattern extraction date:** 2026-06-02
