# Phase 56: CMA-ES Engine - Pattern Map

**Mapped:** 2026-06-01
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/traits/real_gene.rs` | trait | transform | `src/traits/real_valued.rs` + `src/engines/de/gene.rs` | exact (rename) |
| `src/engines/de/gene.rs` | trait (deleted/moved) | — | itself | — |
| `src/engines/de/engine.rs` | engine (modified) | batch | itself | self-update |
| `src/engines/de/mutation.rs` | operator (modified) | transform | itself | self-update |
| `src/engines/de/crossover.rs` | operator (modified) | transform | itself | self-update |
| `src/engines/de/mod.rs` | module (modified) | — | itself | self-update |
| `src/engines/scatter/engine.rs` | engine (modified) | batch | itself | self-update |
| `src/engines/cma/mod.rs` | module | — | `src/engines/de/mod.rs` | exact |
| `src/engines/cma/configuration.rs` | config | request-response | `src/engines/de/configuration.rs` | exact |
| `src/engines/cma/engine.rs` | engine | batch | `src/engines/de/engine.rs` + `src/engines/gp/engine.rs` | role-match (DE loop + GP observer) |
| `src/lib.rs` | module (modified) | — | itself | self-update |
| `tests/test_engines.rs` | test module (modified) | — | itself | self-update |
| `tests/engines/cma/test_cma.rs` | test | batch | `tests/engines/de/test_de.rs` | exact |

---

## Pattern Assignments

### `src/traits/real_gene.rs` (trait, transform)

**Analog:** `src/engines/de/gene.rs` (hard rename — same interface, new location, new method names)

**Full file pattern** (lines 1–47 of `src/engines/de/gene.rs`):
```rust
//! `RealGene` trait — continuous-value arithmetic for DE, Scatter, and CMA-ES engines.

use crate::genotypes::Range;
use crate::traits::GeneT;

/// Extension of [`GeneT`] that enables real-valued arithmetic.
///
/// Any gene type that can expose a `f64` value and create a new instance from a
/// `f64` may implement this trait. Used by [`DeEngine`], [`ScatterEngine`], and
/// [`CmaEngine`].
pub trait RealGene: GeneT {
    /// Returns the gene's continuous value as `f64`.
    fn real_value(&self) -> f64;

    /// Returns a new gene with the same metadata but a different value.
    fn with_real_value(&self, value: f64) -> Self;
}

/// `Range<f64>` genes work with real-valued engines out of the box.
impl RealGene for Range<f64> {
    #[inline]
    fn real_value(&self) -> f64 {
        self.value
    }

    #[inline]
    fn with_real_value(&self, value: f64) -> Self {
        let mut g = self.clone();
        g.value = value;
        g
    }
}
```

Note: Also add `impl RealGene for MultiRangeGenotype<f64>` here (same `self.value` field pattern — confirmed by research). Add `pub mod real_gene; pub use real_gene::RealGene;` to `src/traits/mod.rs`.

---

### `src/engines/de/engine.rs` (modified — cascade rename)

**Change:** `use super::gene::DeGene` → `use crate::traits::RealGene`; all `DeGene` bounds → `RealGene`.

**Import to replace** (line 8):
```rust
// Before:
use super::gene::DeGene;
// After:
use crate::traits::RealGene;
```

**Struct/impl bounds to replace** (lines 50–61):
```rust
// Before:
pub struct DeEngine<U: LinearChromosome>
where
    U::Gene: DeGene,
{ ... }

impl<U: LinearChromosome + Clone> DeEngine<U>
where
    U::Gene: DeGene,
{ ... }

// After: replace DeGene with RealGene throughout
```

No behavioral changes — only identifier replacements.

---

### `src/engines/de/mutation.rs` (modified — cascade rename)

**Change:** `use super::gene::DeGene` (or `use crate::de::gene::DeGene`) → `use crate::traits::RealGene`; all `DeGene` bounds and 7 call sites: `de_value()` → `real_value()`, `with_de_value()` → `with_real_value()`.

**Pattern for method call replacement** (from grep output):
```rust
// Before (all 7 occurrences in mutation.rs):
gene.de_value()
gene.with_de_value(v)
// After:
gene.real_value()
gene.with_real_value(v)
```

---

### `src/engines/de/crossover.rs` (modified — cascade rename)

**Change:** Update `DeGene` import and bounds. No method call changes (crossover delegates to mutation for arithmetic).

---

### `src/engines/de/mod.rs` (modified — cascade rename)

**Analog:** `src/engines/de/mod.rs` (self-update)

**Current re-export to update** (line 14 of `src/engines/de/mod.rs`):
```rust
// Before:
pub use gene::DeGene;
// After (no re-export of DeGene — it no longer exists in this module):
// Remove the gene module declaration and DeGene re-export entirely.
// The gene.rs file itself moves to src/traits/real_gene.rs.
```

**Updated mod.rs** (full file pattern from `src/engines/de/mod.rs` lines 1–14):
```rust
//! Differential Evolution engine.
//!
//! Provides a complete DE implementation with 5 mutation strategies, 2
//! crossover modes, and JADE / L-SHADE adaptive parameter control.

pub mod configuration;
pub mod crossover;
pub mod engine;
pub mod mutation;

pub use configuration::{DeAdaptive, DeConfiguration, DeCrossoverMode, DeMutationStrategy};
pub use engine::{DeEngine, DeResult};
// Note: pub mod gene and pub use gene::DeGene are removed; RealGene lives in crate::traits
```

---

### `src/engines/scatter/engine.rs` (modified — cascade rename)

**Change:** `use crate::de::gene::DeGene` (line 13) → `use crate::traits::RealGene`; all `DeGene` bounds → `RealGene`; 4 arithmetic call sites renamed.

**Import to replace** (line 13 of scatter/engine.rs):
```rust
// Before:
use crate::de::gene::DeGene;
// After:
use crate::traits::RealGene;
```

**Arithmetic call sites to rename** (scatter/engine.rs lines 183–184, 189–190, 210–211, 219, 300):
```rust
// Before:
x1.dna()[j].de_value()
x1.dna()[j].with_de_value(v)
ind.dna()[j].de_value()
ind.dna()[j].with_de_value(old_val + delta)
ind.dna()[j].with_de_value(old_val)
a[i].de_value()
b[i].de_value()
// After: replace de_value() → real_value(), with_de_value() → with_real_value()
```

---

### `src/engines/cma/mod.rs` (module, new)

**Analog:** `src/engines/de/mod.rs` (lines 1–14) — exact structural copy

```rust
//! CMA-ES engine.
//!
//! Covariance Matrix Adaptation Evolution Strategy for real-valued black-box
//! continuous optimization.

pub mod configuration;
pub mod engine;

pub use configuration::CmaConfiguration;
pub use engine::{CmaEngine, CmaResult};
```

---

### `src/engines/cma/configuration.rs` (config, new)

**Analog:** `src/engines/de/configuration.rs` — exact structural pattern for `Default`, builder methods, `ProblemSolving` field

**Imports pattern** (from `src/engines/de/configuration.rs` line 3):
```rust
use crate::configuration::ProblemSolving;
```

**Struct pattern** (from `src/engines/de/configuration.rs` lines 65–84):
```rust
#[derive(Debug, Clone)]
pub struct CmaConfiguration {
    /// Initial step size σ₀ (default 0.3).
    pub sigma0: f64,
    /// Population size λ. If 0, auto-computed as `4 + floor(3·ln(n))` when `run()` is called.
    pub population_size: usize,
    /// Maximum number of generations before stopping.
    pub max_generations: usize,
    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,
    /// Optional fitness target — engine stops early when reached.
    pub fitness_target: Option<f64>,
    /// Covariance matrix cumulation `cc`. `None` = Hansen's auto formula.
    pub cc: Option<f64>,
    /// Step-size control cumulation `cs`. `None` = Hansen's auto formula.
    pub cs: Option<f64>,
    /// Rank-one update rate `c1`. `None` = Hansen's auto formula.
    pub c1: Option<f64>,
    /// Rank-mu update rate `cmu`. `None` = Hansen's auto formula.
    pub cmu: Option<f64>,
}
```

**Default impl pattern** (from `src/engines/de/configuration.rs` lines 86–100):
```rust
impl Default for CmaConfiguration {
    fn default() -> Self {
        Self {
            sigma0: 0.3,
            population_size: 0,
            max_generations: 1000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
            cc: None,
            cs: None,
            c1: None,
            cmu: None,
        }
    }
}
```

**Builder methods pattern** (from `src/engines/de/configuration.rs` lines 102–148 — copy this style exactly):
```rust
impl CmaConfiguration {
    /// Auto-sized configuration for a problem of dimension `n`.
    /// Sets `population_size = 4 + floor(3·ln(n))`.
    pub fn default_for_dim(n: usize) -> Self {
        let lambda = 4 + (3.0 * (n as f64).ln()).floor() as usize;
        Self { population_size: lambda, ..Self::default() }
    }

    pub fn with_sigma0(mut self, s: f64) -> Self { self.sigma0 = s; self }
    pub fn with_population_size(mut self, n: usize) -> Self { self.population_size = n; self }
    pub fn with_max_generations(mut self, n: usize) -> Self { self.max_generations = n; self }
    pub fn with_problem_solving(mut self, ps: ProblemSolving) -> Self { self.problem_solving = ps; self }
    pub fn with_fitness_target(mut self, t: f64) -> Self { self.fitness_target = Some(t); self }
    pub fn with_cc(mut self, v: f64) -> Self { self.cc = Some(v); self }
    pub fn with_cs(mut self, v: f64) -> Self { self.cs = Some(v); self }
    pub fn with_c1(mut self, v: f64) -> Self { self.c1 = Some(v); self }
    pub fn with_cmu(mut self, v: f64) -> Self { self.cmu = Some(v); self }
}
```

---

### `src/engines/cma/engine.rs` (engine, new)

**Primary analog:** `src/engines/de/engine.rs` — struct layout, `new()`, `run()` loop structure, `is_better()`, `reached_target()`, `find_best()` helpers, `DeResult`→`CmaResult` shape

**Secondary analog:** `src/engines/gp/engine.rs` — observer field, `with_observer()`, `notify()`, WASM-gated `Instant`, `TerminationCause`, `GenerationStats`, `all_stats` accumulation

**Imports pattern** (composite of both analogs):
```rust
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;

use crate::configuration::ProblemSolving;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::rng::make_rng;
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, LinearChromosome};
use crate::traits::RealGene;

use super::configuration::CmaConfiguration;
```

**Result struct pattern** (from `src/engines/de/engine.rs` lines 16–25):
```rust
pub struct CmaResult<U: LinearChromosome> {
    /// Final population (all individuals evaluated).
    pub population: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
}
```

**Engine struct pattern** (DE lines 50–57 + GP lines 96–102 for observer):
```rust
pub struct CmaEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: CmaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}
```

**`new()` pattern** (from `src/engines/de/engine.rs` lines 69–79):
```rust
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
```

**`with_observer()` + `notify()` pattern** (from `src/engines/gp/engine.rs` lines 145–160):
```rust
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

**WASM-gated `Instant` pattern** (from `src/engines/gp/engine.rs` lines 250–262):
```rust
let t_fit: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
// ... work ...
if let Some(t) = t_fit {
    let count = pop.len();
    self.notify(|obs| obs.on_fitness_evaluation_complete(gen, t.elapsed(), count));
}
```

**`run()` skeleton pattern** (composite of DE lines 82–217 and GP lines 221–462):
```rust
pub fn run(&mut self) -> CmaResult<U> {
    let mut rng = make_rng();

    self.notify(|obs| obs.on_run_start());

    // ── Initialise ────────────────────────────────────────────────────────
    let pop_size = /* auto-compute λ from n if population_size == 0 */;
    let mut pop: Vec<U> = (self.init_fn)(pop_size);
    for ind in &mut pop {
        let f = (self.fitness_fn)(ind.dna());
        ind.set_fitness(f);
    }

    // ── Best tracking ─────────────────────────────────────────────────────
    let (mut best_idx, mut best_fitness) = self.find_best(&pop);
    let mut best = pop[best_idx].clone();
    let mut termination_cause = TerminationCause::GenerationLimitReached;
    let mut all_stats: Vec<GenerationStats> = Vec::with_capacity(self.config.max_generations);

    // ── CmaState init (problem dimension n from pop[0].dna().len()) ───────
    let n = pop[0].dna().len();
    let mut state = CmaState::new(n, pop_size, &self.config, /* mean from pop */);

    // ── Main loop ─────────────────────────────────────────────────────────
    for gen in 0..self.config.max_generations {
        self.notify(|obs| obs.on_generation_start(gen));

        // Sample λ offspring from N(m, σ²·C), evaluate fitness
        // ... (CMA-ES sample + evaluate, no crossover/mutation operators used)

        // Update best
        let (bi, bf) = self.find_best(&pop);
        if self.is_better(bf, best_fitness) {
            best_fitness = bf;
            best_idx = bi;
            best = pop[bi].clone();
            let best_clone = best.clone();
            self.notify(|obs| obs.on_new_best(gen, best_clone));
        }

        // Stats (GenerationStats::from_fitness_values — matches GP lines 422-430)
        let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
        let stats = GenerationStats::from_fitness_values(
            gen,
            &fitness_values,
            matches!(self.config.problem_solving, ProblemSolving::Maximization),
        );
        all_stats.push(stats.clone());
        self.notify(|obs| obs.on_generation_end(&stats));

        // Early stopping (matches DE lines 204-208)
        if let Some(target) = self.config.fitness_target {
            if self.reached_target(best_fitness, target) {
                termination_cause = TerminationCause::FitnessTargetReached;
                break;
            }
        }
    }

    self.notify(|obs| obs.on_run_end(termination_cause, &all_stats));

    CmaResult { population: pop, best, best_fitness, generations: all_stats.len() }
}
```

**Helper methods pattern** (from `src/engines/de/engine.rs` lines 221–274):
```rust
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

fn reached_target(&self, fitness: f64, target: f64) -> bool {
    match self.config.problem_solving {
        ProblemSolving::Minimization => fitness <= target,
        ProblemSolving::Maximization => fitness >= target,
        ProblemSolving::FixedFitness => (fitness - target).abs() < 1e-6,
    }
}
```

---

### `src/lib.rs` (modified)

**Pattern** — add CMA module using exact `#[path]` alias pattern (lines 293–328):
```rust
// Add after the gp module declaration (line 327-328):
#[path = "engines/cma/mod.rs"]
pub mod cma;
```

Also add `RealGene` to the traits re-exports (line 359 area):
```rust
pub use traits::{LinearChromosome, OperatorCompat, RealGene, Strategy, VectorFitness};
```

And update the engine count in the top-level doc comment (line 4 — "12 optimization engines" → "13 optimization engines").

---

### `tests/test_engines.rs` (modified)

**Pattern** — add CMA entry following the exact `de` module block (lines 12–14):
```rust
// Current de block (lines 12-14):
mod de {
    mod test_de;
}
// Add after scatter block:
mod cma {
    mod test_cma;
}
```

---

### `tests/engines/cma/test_cma.rs` (test, new)

**Analog:** `tests/engines/de/test_de.rs` — identical structure: helper functions, fitness functions, test-per-requirement pattern

**Imports pattern** (from `tests/engines/de/test_de.rs` lines 1–13):
```rust
use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::LinearChromosome;
```

**Helper function pattern** (from `tests/engines/de/test_de.rs` lines 18–39):
```rust
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

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
```

**Test structure pattern** (from `tests/engines/de/test_de.rs` lines 61–71):
```rust
#[test]
fn test_cma_sphere_converges() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_fitness_target(1.0)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    );
    let result = engine.run();
    assert!(
        result.best_fitness < 5.0,
        "CMA-ES should reduce sphere fitness; got {}",
        result.best_fitness
    );
    assert!(result.generations > 0);
    assert!(!result.population.is_empty());
}
```

**Required tests per requirements map (CMA-01 through CMA-11):** Each maps to a test function following the pattern above. `test_cma_early_stopping` mirrors `test_de_early_stopping` (lines 178–190); `test_cma_result_fields` mirrors `test_de_result_fields` (lines 167–175); `test_cma_maximization` mirrors `test_de_maximization` (lines 149–163).

---

## Shared Patterns

### `is_better()` helper
**Source:** `src/engines/de/engine.rs` lines 254–266
**Apply to:** `CmaEngine` (copy verbatim, field path changes to `self.config.problem_solving`)
```rust
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
```

### Observer notification
**Source:** `src/engines/gp/engine.rs` lines 155–160
**Apply to:** `CmaEngine` (copy verbatim, substitute generic type)
```rust
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### `TerminationCause` tracking
**Source:** `src/engines/gp/engine.rs` lines 240–241, 439–441, 453
**Apply to:** `CmaEngine::run()`
```rust
let mut termination_cause = TerminationCause::GenerationLimitReached;
// ... in early-stop branch:
termination_cause = TerminationCause::FitnessTargetReached;
// ... after loop:
self.notify(|obs| obs.on_run_end(termination_cause, &all_stats));
```

### `GenerationStats` collection
**Source:** `src/engines/gp/engine.rs` lines 422–430
**Apply to:** `CmaEngine::run()` — collect per-generation, pass to `on_generation_end`
```rust
let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
let stats = GenerationStats::from_fitness_values(
    gen,
    &fitness_values,
    /* is_maximization = */ matches!(self.config.problem_solving, ProblemSolving::Maximization),
);
all_stats.push(stats.clone());
self.notify(|obs| obs.on_generation_end(&stats));
```

### WASM-gated timing
**Source:** `src/engines/gp/engine.rs` lines 250–262
**Apply to:** Any `Instant::now()` / `elapsed()` call in `CmaEngine`
```rust
let t_fit: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
```

### DNA extraction helper (CMA-ES specific)
**Source:** Research CONTEXT.md, derived from `src/engines/de/engine.rs` DNA access pattern
**Apply to:** `CmaEngine::run()` — extract f64 coords for mean/covariance arithmetic
```rust
// Helper to extract f64 coordinates from a chromosome
fn extract_coords(chr: &U) -> Vec<f64> {
    chr.dna().iter().map(|g| g.real_value()).collect()
}

// Reconstruct chromosome from f64 coordinates (uses template's gene metadata)
// template.dna()[j] preserves id/bounds; only value changes
let new_dna: Vec<U::Gene> = x_k.iter().enumerate()
    .map(|(j, &v)| template.dna()[j].with_real_value(v))
    .collect();
template.set_dna(Cow::Owned(new_dna));
```

### `#[path]` module alias in lib.rs
**Source:** `src/lib.rs` lines 293–328
**Apply to:** New `pub mod cma` declaration
```rust
#[path = "engines/cma/mod.rs"]
pub mod cma;
```

### Test RNG seeding
**Source:** `tests/engines/de/test_de.rs` lines 23–25
**Apply to:** `tests/engines/cma/test_cma.rs` — all tests that need reproducibility
```rust
rng::set_seed(Some(seed));
let mut r = rng::make_rng();
```

---

## No Analog Found

All files have close analogs. No greenfield patterns required.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/engines/cma/engine.rs` (CmaState internal struct) | internal state | batch | No prior eigendecomposition or CMA state struct in codebase; use RESEARCH.md Pattern 1 (Hansen arXiv:1604.00772) for `CmaState` fields |

---

## Metadata

**Analog search scope:** `src/engines/de/`, `src/engines/gp/`, `src/engines/scatter/`, `src/traits/`, `tests/engines/de/`, `tests/test_engines.rs`, `src/lib.rs`
**Files scanned:** 13
**Pattern extraction date:** 2026-06-01
