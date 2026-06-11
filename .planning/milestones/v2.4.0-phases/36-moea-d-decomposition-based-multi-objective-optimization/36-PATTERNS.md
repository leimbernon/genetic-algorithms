# Phase 36: MOEA/D — Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 10 new/modified files
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/engines/moead/mod.rs` | engine | request-response (batch) | `src/engines/nsga3/mod.rs` | exact |
| `src/engines/moead/configuration.rs` | config | transform | `src/engines/nsga3/configuration.rs` | exact |
| `src/observe/observer/mod.rs` | middleware (modify) | event-driven | `src/observe/observer/mod.rs` lines 180–191 | exact (in-file extension) |
| `src/observe/observer/log.rs` | middleware (modify) | event-driven | `src/observe/observer/log.rs` lines 208–220 | exact (in-file extension) |
| `src/error.rs` | utility (modify) | transform | `src/error.rs` lines 37–38, 62–64 | exact (in-file extension) |
| `src/lib.rs` | config (modify) | transform | `src/lib.rs` lines 113–114, 127 | exact (in-file extension) |
| `tests/engines/moead/test_moead.rs` | test | request-response | `tests/engines/nsga3/test_nsga3.rs` | exact |
| `tests/engines/moead/test_moead_configuration.rs` | test | transform | `tests/engines/nsga3/test_nsga3_configuration.rs` | exact |
| `tests/engines/moead/mod.rs` | test | — | `tests/engines/nsga3/` (directory layout) | role-match |
| `examples/moead_dtlz2.rs` | utility | batch | `examples/nsga3_dtlz2.rs` | exact |

---

## Pattern Assignments

### `src/engines/moead/mod.rs` (engine, batch)

**Analog:** `src/engines/nsga3/mod.rs`

**Imports pattern** (lines 1–31):
```rust
pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::non_dominated_sort::{assign_ranks, non_dominated_sort_with_directions};
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::multi_objective::ObjectiveFn;
use crate::nsga2::configuration::ObjectiveDirection;
use crate::moead::configuration::{MoeaDConfiguration, ScalarizationFn};
use crate::observer::MoeaDObserver;
use crate::operations::{crossover, mutation};
use crate::traits::{ChromosomeT, InitializationFn};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
```

**Struct and constructor pattern** (lines 37–83):
```rust
pub struct MoeaDGa<U>
where
    U: ChromosomeT,
{
    pub moead_config: MoeaDConfiguration,
    pub ga_config: GaConfiguration,
    pub alleles: Vec<U::Gene>,
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    pub observer: Option<Arc<dyn MoeaDObserver<U> + Send + Sync>>,
}

impl<U> MoeaDGa<U>
where
    U: ChromosomeT,
{
    pub fn new(moead_config: MoeaDConfiguration, ga_config: GaConfiguration) -> Self {
        MoeaDGa {
            moead_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
            observer: None,
        }
    }

    pub fn with_observer(mut self, obs: Arc<dyn MoeaDObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    #[inline]
    fn notify<F: FnOnce(&dyn MoeaDObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    pub fn with_alleles(mut self, alleles: Vec<U::Gene>) -> Self { self.alleles = alleles; self }

    pub fn with_initialization_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, Option<&[U::Gene]>, Option<bool>) -> Vec<U::Gene> + Send + Sync + 'static,
    {
        self.initialization_fn = Some(Arc::new(f));
        self
    }

    pub fn with_objective_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self {
        self.objective_fns = fns.into_iter().map(Arc::from).collect();
        self
    }

    pub fn build(self) -> Result<Self, GaError> {
        self.validate()?;
        Ok(self)
    }
}
```

**Validate + validate_and_get_weight_vectors pattern** (lines 117–256 of analog; port to moead):
```rust
// validate() — mirrors nsga3/mod.rs lines 117–184; change InvalidNsga3Configuration
// to InvalidMoeaDConfiguration and reference_points to weight_vectors.
pub fn validate(&self) -> Result<(), GaError> {
    if self.moead_config.num_objectives == 0 {
        return Err(GaError::InvalidMoeaDConfiguration("num_objectives must be > 0".to_string()));
    }
    if self.moead_config.population_size < 2 {
        return Err(GaError::InvalidMoeaDConfiguration("population_size must be >= 2".to_string()));
    }
    if self.initialization_fn.is_none() {
        return Err(GaError::InvalidMoeaDConfiguration("initialization_fn is required".to_string()));
    }
    if self.objective_fns.len() != self.moead_config.num_objectives {
        return Err(GaError::InvalidMoeaDConfiguration(format!(
            "Expected {} objective functions, got {}",
            self.moead_config.num_objectives, self.objective_fns.len()
        )));
    }
    // weight vector checks (D-06) — see validate_and_get_weight_vectors()
    let _ = self.moead_config.effective_weight_vectors().ok_or_else(|| {
        GaError::InvalidMoeaDConfiguration(
            "weight vectors must be configured via with_weight_vectors_auto(p) or with_weight_vectors(vecs)".to_string()
        )
    })?;
    Ok(())
}

// validate_and_get_weight_vectors() — mirrors nsga3/mod.rs lines 191–256
fn validate_and_get_weight_vectors(&self) -> Result<Vec<Vec<f64>>, GaError> {
    // ... same checks as validate() ...
    let wvs = self.moead_config.effective_weight_vectors()
        .ok_or_else(|| GaError::InvalidMoeaDConfiguration(
            "weight vectors must be configured via with_weight_vectors_auto(p) or with_weight_vectors(vecs)".to_string()
        ))?;
    if wvs.is_empty() {
        return Err(GaError::InvalidMoeaDConfiguration("weight vector list must not be empty".to_string()));
    }
    for (i, wv) in wvs.iter().enumerate() {
        if wv.len() != self.moead_config.num_objectives {
            return Err(GaError::InvalidMoeaDConfiguration(format!(
                "weight vector {} has dimension {}, expected {}",
                i, wv.len(), self.moead_config.num_objectives
            )));
        }
    }
    Ok(wvs)
}
```

**WASM-gated Instant pattern** (lines 295–306 of analog):
```rust
let t_sort: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
// ... work ...
if let Some(start) = t_sort {
    self.notify(|obs| obs.on_non_dominated_sort_complete(gen, start.elapsed().as_secs_f64() * 1000.0));
}
```

**WASM-gated par_iter pattern** (lines 395–413 of analog):
```rust
#[cfg(not(target_arch = "wasm32"))]
let population: Vec<ParetoIndividual<U>> = chromosomes
    .into_par_iter()
    .map(|chrom| {
        let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
        ParetoIndividual::new(chrom, objectives)
    })
    .collect();
#[cfg(target_arch = "wasm32")]
let population: Vec<ParetoIndividual<U>> = chromosomes
    .into_iter()
    .map(|chrom| {
        let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
        ParetoIndividual::new(chrom, objectives)
    })
    .collect();
```

**Post-hoc Pareto front extraction at end of run()** (lines 363–365 of analog):
```rust
let front_individuals: Vec<ParetoIndividual<U>> =
    population.into_iter().filter(|ind| ind.rank == 0).collect();
Ok(ParetoFront::new(front_individuals))
```

**Observer notify pattern** (lines 355–360 of analog):
```rust
self.notify(|obs| obs.on_pareto_front_assigned(gen, front_count, population.len()));
```

**RNG initialisation pattern** (line 285 of analog):
```rust
crate::rng::set_seed(self.ga_config.rng_seed);
// ...
let mut rng = crate::rng::make_rng();
```

**Mutation dispatch pattern** (lines 454–490 of analog — copy and adapt; MOEA/D uses same crossover/mutation factories):
```rust
crossover::factory(
    &population[parent_a_idx].chromosome,
    &population[parent_b_idx].chromosome,
    crossover_config,
)?

mutation::factory_with_params(
    mutation_config.method,
    &mut child,
    mutation_config.step,
    mutation_config.sigma,
)?
```

---

### `src/engines/moead/configuration.rs` (config, transform)

**Analog:** `src/engines/nsga3/configuration.rs`

**Struct definition pattern** (lines 29–48 of analog):
```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoeaDConfiguration {
    pub num_objectives: usize,
    pub population_size: usize,
    pub max_generations: usize,
    pub objective_directions: Vec<ObjectiveDirection>,
    pub scalarization: ScalarizationFn,          // NEW (no analog field)
    pub neighborhood_size: usize,                // NEW (D-08, default 20)
    pub max_neighbor_replacements: usize,        // NEW (D-09, default 2)
    weight_vectors_auto_p: Option<usize>,        // mirrors reference_points_auto_p
    weight_vectors_custom: Option<Vec<Vec<f64>>>, // mirrors reference_points_custom
}
```

**Default impl pattern** (lines 50–61 of analog):
```rust
impl Default for MoeaDConfiguration {
    fn default() -> Self {
        MoeaDConfiguration {
            num_objectives: 3,
            population_size: 100,
            max_generations: 200,
            objective_directions: Vec::new(),
            scalarization: ScalarizationFn::default(), // Tchebycheff
            neighborhood_size: 20,
            max_neighbor_replacements: 2,
            weight_vectors_auto_p: None,
            weight_vectors_custom: None,
        }
    }
}
```

**Builder methods pattern** (lines 70–113 of analog; substitute reference_points → weight_vectors):
```rust
pub fn with_num_objectives(mut self, n: usize) -> Self { self.num_objectives = n; self }
pub fn with_population_size(mut self, size: usize) -> Self { self.population_size = size; self }
pub fn with_max_generations(mut self, gens: usize) -> Self { self.max_generations = gens; self }
pub fn with_objective_directions(mut self, dirs: Vec<ObjectiveDirection>) -> Self {
    self.objective_directions = dirs; self
}
pub fn with_scalarization(mut self, s: ScalarizationFn) -> Self { self.scalarization = s; self }
pub fn with_neighborhood_size(mut self, t: usize) -> Self { self.neighborhood_size = t; self }
pub fn with_max_neighbor_replacements(mut self, nr: usize) -> Self {
    self.max_neighbor_replacements = nr; self
}

// D-04: auto weight vectors (last-call-wins; mirrors with_reference_points_auto)
pub fn with_weight_vectors_auto(mut self, p: usize) -> Self {
    self.weight_vectors_auto_p = Some(p);
    self.weight_vectors_custom = None;
    self
}

// D-05: custom weight vectors (last-call-wins; mirrors with_reference_points)
pub fn with_weight_vectors(mut self, vecs: Vec<Vec<f64>>) -> Self {
    self.weight_vectors_custom = Some(vecs);
    self.weight_vectors_auto_p = None;
    self
}
```

**effective_weight_vectors pattern** (lines 120–129 of analog; substitute generate_das_dennis call):
```rust
// Mirrors Nsga3Configuration::effective_reference_points() lines 120–129
pub fn effective_weight_vectors(&self) -> Option<Vec<Vec<f64>>> {
    if let Some(p) = self.weight_vectors_auto_p {
        Some(crate::nsga3::das_dennis::generate_das_dennis(self.num_objectives, p))
    } else {
        self.weight_vectors_custom.clone()
    }
}

pub(crate) fn weight_vectors_auto_p(&self) -> Option<usize> {
    self.weight_vectors_auto_p
}
```

**effective_directions helper pattern** (lines 140–146 of analog):
```rust
pub fn effective_directions(&self) -> Vec<ObjectiveDirection> {
    if self.objective_directions.is_empty() {
        vec![ObjectiveDirection::Minimize; self.num_objectives]
    } else {
        self.objective_directions.clone()
    }
}
```

**ScalarizationFn enum** (mirrors `ObjectiveDirection` style from `src/engines/nsga2/configuration.rs` line 1 — the enum pattern — D-02):
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarizationFn {
    /// Classic Tchebycheff: g = max_i { w_i * |f_i - z*_i| }
    Tchebycheff,
    /// Penalty-based boundary intersection: g = d1 + theta * d2
    /// theta = 5.0 per Zhang & Li 2007.
    Pbi { theta: f64 },
}

impl Default for ScalarizationFn {
    fn default() -> Self { ScalarizationFn::Tchebycheff }
}
```

---

### `src/observe/observer/mod.rs` (in-file modification, event-driven)

**Analog:** Lines 180–191 of `src/observe/observer/mod.rs` (`Nsga3Observer<U>` definition)

**Exact insertion point:** After line 191 (after the closing `}` of `Nsga3Observer<U>`) and before line 193 (`/// Combined observer bound`).

**New trait block to insert:**
```rust
/// Observer for [`MoeaDGa<U>`](crate::moead::MoeaDGa) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// # Note: not in `AllObserver`
///
/// `AllObserver<U>` does NOT include `MoeaDObserver<U>` in Phase 36 — adding it
/// would be a breaking change for existing `AllObserver` implementors. Use
/// [`MoeaDGa::with_observer`](crate::moead::MoeaDGa::with_observer) to attach
/// a `MoeaDObserver` independently.
pub trait MoeaDObserver<U: ChromosomeT>: Send + Sync {
    /// Called after Pareto fronts are assigned for the current generation.
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
    }
    /// Called after non-dominated sorting completes for the current generation.
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
}
```

Mirror of `Nsga3Observer<U>` at lines 180–191:
```rust
// Current Nsga3Observer pattern (lines 180–191) to replicate exactly for MoeaDObserver:
pub trait Nsga3Observer<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
    }
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
}
```

---

### `src/observe/observer/log.rs` (in-file modification, event-driven)

**Analog:** Lines 208–220 of `src/observe/observer/log.rs` (`impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver`)

**Import line change** (line 27 of log.rs):
```rust
// Current (after Phase 35):
use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer, Nsga3Observer};
// After Phase 36 — add MoeaDObserver:
use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, MoeaDObserver, Nsga2Observer, Nsga3Observer};
```

**New impl block to add after line 220** (exact mirror of Nsga3Observer impl at lines 208–220):
```rust
impl<U: ChromosomeT> MoeaDObserver<U> for LogObserver {
    fn on_pareto_front_assigned(
        &self,
        generation: usize,
        front_count: usize,
        population_size: usize,
    ) {
        log::debug!(target: "moead_events",
            "Generation {} complete, population size = {}, fronts = {}",
            generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "moead_events",
            "Non-dominated sort complete at generation {} ({:.2}ms)", generation, duration_ms);
    }
}
```

Mirror (Nsga3Observer impl from lines 208–220):
```rust
impl<U: ChromosomeT> Nsga3Observer<U> for LogObserver {
    fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize) {
        log::debug!(target: "nsga3_events", "Generation {} complete, population size = {}, fronts = {}", generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "nsga3_events", "Non-dominated sort complete at generation {} ({:.2}ms)", generation, duration_ms);
    }
}
```

---

### `src/error.rs` (in-file modification, transform)

**Analog:** Lines 37–38 and 62–64 of `src/error.rs` (`InvalidNsga3Configuration` variant)

**Enum variant to add after line 37** (`InvalidNsga3Configuration`):
```rust
/// An NSGA-III configuration parameter is invalid.
InvalidNsga3Configuration(String),
/// A MOEA/D configuration parameter is invalid.
InvalidMoeaDConfiguration(String),
```

**Display arm to add after the `InvalidNsga3Configuration` arm** (lines 62–64):
```rust
GaError::InvalidNsga3Configuration(msg) => {
    write!(f, "Invalid NSGA-III configuration: {}", msg)
}
GaError::InvalidMoeaDConfiguration(msg) => {
    write!(f, "Invalid MOEA/D configuration: {}", msg)
}
```

---

### `src/lib.rs` (in-file modification, config)

**Analog:** Lines 113–114 and 127 of `src/lib.rs`

**Module re-export to add after line 114**:
```rust
#[path = "engines/nsga3/mod.rs"]
pub mod nsga3;
// ADD:
#[path = "engines/moead/mod.rs"]
pub mod moead;
```

**Observer pub use to add after line 127**:
```rust
pub use observer::Nsga3Observer;
// ADD:
pub use observer::MoeaDObserver;
```

---

### `tests/engines/moead/test_moead.rs` (test, request-response)

**Analog:** `tests/engines/nsga3/test_nsga3.rs`

**Imports pattern** (lines 1–15 of analog):
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ScalarizationFn};
use genetic_algorithms::moead::MoeaDGa;
use genetic_algorithms::MoeaDObserver;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
```

**validate() test pattern** (lines 17–26 of analog — each GaError variant assertion):
```rust
#[test]
fn test_moead_validate_missing_weight_vectors() {
    let config = MoeaDConfiguration::new().with_num_objectives(3);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config)
        .with_initialization_fn(|_, _, _| vec![])
        .with_objective_fns(vec![Box::new(|_| 0.0), Box::new(|_| 0.0), Box::new(|_| 0.0)]);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(ref msg)) if msg.contains("weight vectors")));
}
```

**build_test_moead helper pattern** (lines 157–183 of analog):
```rust
fn build_test_moead(
    population_size: usize,
    max_generations: usize,
    scalarization: ScalarizationFn,
) -> MoeaDGa<RangeChromosome<f64>> {
    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(population_size)
        .with_max_generations(max_generations)
        .with_weight_vectors_auto(4)   // 15 weight vectors for M=3, p=4
        .with_scalarization(scalarization);

    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = 4;
    ga_config.limit_configuration.alleles_can_be_repeated = true;
    ga_config.rng_seed = Some(42);

    let alleles = vec![RangeGenotype::<f64>::new(0, vec![(0.0, 1.0)], 0.0)];
    let alleles_clone = alleles.clone();

    MoeaDGa::<RangeChromosome<f64>>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(dtlz2_objectives())
        .build()
        .expect("MoeaD build should succeed")
}
```

**run() integration test pattern** (lines 185–197 of analog):
```rust
#[test]
fn test_moead_run_produces_pareto_front() {
    let mut moead = build_test_moead(15, 5, ScalarizationFn::Tchebycheff);
    let result = moead.run();
    assert!(result.is_ok(), "MoeaD run should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "Pareto front should contain at least one rank-0 individual");
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0);
        assert_eq!(ind.objectives.len(), 3);
    }
}
```

**Observer hook test pattern** (uses AtomicUsize counter, same as nsga3 test lines 15):
```rust
// Uses std::sync::atomic::{AtomicUsize, Ordering} — count calls to on_pareto_front_assigned
// and assert == max_generations at end of run().
```

---

### `tests/engines/moead/test_moead_configuration.rs` (test, transform)

**Analog:** `tests/engines/nsga3/test_nsga3_configuration.rs`

**Imports and default test pattern** (lines 1–11 of analog):
```rust
use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ScalarizationFn};

#[test]
fn test_moead_configuration_default() {
    let config = MoeaDConfiguration::default();
    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.max_generations, 200);
    assert!(config.objective_directions.is_empty());
    assert!(config.effective_weight_vectors().is_none());
}
```

**Last-call-wins test pattern** (lines 55–77 of analog):
```rust
#[test]
fn test_moead_last_call_wins_auto_then_custom() {
    let custom = vec![vec![0.5, 0.3, 0.2]];
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4)
        .with_weight_vectors(custom.clone());
    let wvs = config.effective_weight_vectors().unwrap();
    assert_eq!(wvs, custom);
}
```

**ScalarizationFn default test**:
```rust
#[test]
fn test_scalarization_default() {
    // D-03: default is Tchebycheff
    assert!(matches!(ScalarizationFn::default(), ScalarizationFn::Tchebycheff));
    let config = MoeaDConfiguration::default();
    assert!(matches!(config.scalarization, ScalarizationFn::Tchebycheff));
}
```

---

### `examples/moead_dtlz2.rs` (utility, batch)

**Analog:** `examples/nsga3_dtlz2.rs`

**Module doc comment pattern** (lines 1–29 of analog — adapt for MOEA/D):
```rust
/*!
# MOEA/D Many-Objective Optimization (DTLZ2 3-objective Benchmark)

This example demonstrates many-objective optimization using MOEA/D on the
DTLZ2 benchmark problem with three objectives.
...
*/
```

**Constants pattern** (lines 41–44 of analog):
```rust
const N_VARS: usize = 12;
const POP_SIZE: usize = 91;       // C(14,2) = 91 with p=12 for M=3
const MAX_GENERATIONS: usize = 300;
const DAS_DENNIS_P: usize = 12;   // C(14,2) = 91 weight vectors for M=3
// NOTE: C(p+M-1, M-1) for M=3 is (p+2)(p+1)/2; p=12 gives 91 (not C(12,2)=66)
```

**Config builder pattern** (lines 47–57 of analog):
```rust
let moead_config = MoeaDConfiguration::new()
    .with_num_objectives(3)
    .with_population_size(POP_SIZE)
    .with_max_generations(MAX_GENERATIONS)
    .with_objective_directions(vec![
        ObjectiveDirection::Minimize,
        ObjectiveDirection::Minimize,
        ObjectiveDirection::Minimize,
    ])
    .with_weight_vectors_auto(DAS_DENNIS_P)
    .with_scalarization(ScalarizationFn::Tchebycheff);
```

**Observer attachment pattern** (lines 92–95 of analog):
```rust
.with_observer(
    Arc::new(LogObserver) as Arc<dyn MoeaDObserver<RangeChromosome<f64>> + Send + Sync>
)
```

**run() output pattern** (lines 110–137 of analog — adapt for MOEA/D output):
```rust
match moead.run() {
    Ok(mut front) => {
        println!("\nPareto front: {} non-dominated solutions", front.individuals.len());
        front.individuals.sort_by(|a, b| {
            a.objectives[0].partial_cmp(&b.objectives[0]).unwrap_or(std::cmp::Ordering::Equal)
        });
        // print first 10, same column format as nsga3_dtlz2.rs
    }
    Err(e) => { eprintln!("MOEA/D failed: {:?}", e); std::process::exit(1); }
}
```

---

## Shared Patterns

### Authentication / Authorization
Not applicable — library code with no auth layer.

### Error Handling (GaError + ? operator)
**Source:** `src/error.rs` lines 17–68 and `src/engines/nsga3/mod.rs` (pervasive `?` usage)
**Apply to:** `src/engines/moead/mod.rs`, `src/engines/moead/configuration.rs`
```rust
// New variant at src/error.rs (add after InvalidNsga3Configuration line 37):
/// A MOEA/D configuration parameter is invalid.
InvalidMoeaDConfiguration(String),

// Display arm (add after InvalidNsga3Configuration arm lines 62–64):
GaError::InvalidMoeaDConfiguration(msg) => write!(f, "Invalid MOEA/D configuration: {}", msg),
```

### WASM Cfg-Gating (mandatory)
**Source:** `src/engines/nsga3/mod.rs` lines 27–28, 295–306, 395–413, 501–519
**Apply to:** `src/engines/moead/mod.rs` — every `Instant::now()` call site and every `par_iter` vs `iter` branch
```rust
// Instant gate (copy from nsga3/mod.rs lines 295–306):
let t_sort: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else { None };

// par_iter gate (copy from nsga3/mod.rs lines 395–413):
#[cfg(not(target_arch = "wasm32"))]
let population: Vec<ParetoIndividual<U>> = chromosomes.into_par_iter().map(|chrom| { ... }).collect();
#[cfg(target_arch = "wasm32")]
let population: Vec<ParetoIndividual<U>> = chromosomes.into_iter().map(|chrom| { ... }).collect();
```

### Observer Dispatch (zero-cost notify)
**Source:** `src/engines/nsga3/mod.rs` lines 77–83
**Apply to:** `src/engines/moead/mod.rs`
```rust
#[inline]
fn notify<F: FnOnce(&dyn MoeaDObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### Serde Feature-Gating
**Source:** `src/engines/nsga3/configuration.rs` lines 29–30 and `src/error.rs` lines 15–16
**Apply to:** `src/engines/moead/configuration.rs` (struct and enum), `src/error.rs` (GaError derive already covers new variant)
```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
```

### Das-Dennis Reuse
**Source:** `src/engines/nsga3/configuration.rs` lines 121–128 (`effective_reference_points`)
**Apply to:** `src/engines/moead/configuration.rs` (`effective_weight_vectors`) — call `crate::nsga3::das_dennis::generate_das_dennis(self.num_objectives, p)` directly. Do NOT duplicate the generator.

### Post-hoc Pareto Sort
**Source:** `src/engines/nsga3/mod.rs` lines 363–365 and `src/engines/multi_objective/non_dominated_sort.rs` (`non_dominated_sort_with_directions`, `assign_ranks`)
**Apply to:** `src/engines/moead/mod.rs` final block of `run()` and the per-generation front_count derivation
```rust
let obj_slices: Vec<&[f64]> = population.iter().map(|i| i.objectives.as_slice()).collect();
let fronts = non_dominated_sort_with_directions(&obj_slices, &directions);
let mut ranks = vec![0usize; population.len()];
assign_ranks(&mut ranks, &fronts);
for (i, &r) in ranks.iter().enumerate() { population[i].rank = r; }
let front_individuals: Vec<ParetoIndividual<U>> =
    population.into_iter().filter(|ind| ind.rank == 0).collect();
Ok(ParetoFront::new(front_individuals))
```

### Vec::with_capacity Pre-allocation
**Source:** `src/engines/nsga3/mod.rs` line 431, `nsga3_environmental_selection` line 563
**Apply to:** `src/engines/moead/mod.rs` — neighbourhoods Vec, offspring Vec, ideal_point Vec
```rust
let mut neighbourhoods: Vec<Vec<usize>> = Vec::with_capacity(n);
let mut raw_offspring: Vec<U> = Vec::with_capacity(pop_size);
let mut ideal_point: Vec<f64> = Vec::with_capacity(num_objectives);
```

---

## No Analog Found

All files have close analogs. No entries.

---

## Metadata

**Analog search scope:** `src/engines/`, `src/observe/`, `src/error.rs`, `src/lib.rs`, `tests/engines/nsga3/`, `examples/`
**Files scanned:** 10 source files read in full
**Pattern extraction date:** 2026-05-09
