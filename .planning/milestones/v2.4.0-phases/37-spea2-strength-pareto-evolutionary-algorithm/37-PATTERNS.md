# Phase 37: SPEA2 - Pattern Map

**Mapped:** 2026-05-10
**Files analyzed:** 9 (4 new engine files, 3 modifications to existing files, 2 test files)
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/engines/spea2/mod.rs` | engine | CRUD (generation loop) | `src/engines/moead/mod.rs` | exact (same engine pattern) |
| `src/engines/spea2/configuration.rs` | config | CRUD | `src/engines/moead/configuration.rs` | exact (same builder pattern) |
| `src/observe/observer/mod.rs` | trait | event-driven | `MoeaDObserver<U>` (lines 204-215) | exact (same observer struct pattern) |
| `src/observe/observer/log.rs` | observer | event-driven | `impl MoeaDObserver<U>` (lines 224-240) | exact (same impl block pattern) |
| `src/lib.rs` | config | N/A | `pub mod moead` (lines 116-117) | exact (same re-export pattern) |
| `src/error.rs` | utility | N/A | `InvalidMoeaDConfiguration(String)` (line 39) | exact (same variant pattern) |
| `tests/engines/spea2/test_spea2.rs` | test | CRUD | `tests/engines/moead/test_moead.rs` | exact (same engine test pattern) |
| `tests/engines/spea2/test_spea2_configuration.rs` | test | CRUD | `tests/engines/moead/test_moead_configuration.rs` | exact (same config test pattern) |
| `examples/spea2_zdt1.rs` | example | request-response | `examples/nsga2_zdt1.rs` | exact (same example structure) |

## Pattern Assignments

### `src/engines/spea2/mod.rs` (engine, CRUD generation loop)

**Analog:** `src/engines/moead/mod.rs`

**Imports pattern** (lines 17-30):
```rust
pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::moead::configuration::{MoeaDConfiguration, ScalarizationFn};
use crate::multi_objective::non_dominated_sort::{assign_ranks, non_dominated_sort_with_directions};
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::multi_objective::ObjectiveFn;
use crate::observer::MoeaDObserver;
use crate::operations::{crossover, mutation};
use crate::traits::{ChromosomeT, InitializationFn};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
```

For SPEA2, adapt to:
```rust
pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::non_dominated_sort::{assign_ranks, non_dominated_sort_with_directions};
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::multi_objective::ObjectiveFn;
use crate::observer::Spea2Observer;
use crate::operations::{crossover, mutation};
use crate::spea2::configuration::Spea2Configuration;
use crate::traits::{ChromosomeT, InitializationFn};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
```

**Engine struct pattern** (lines 37-53):
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
```

For SPEA2, use `Spea2Ga`, `spea2_config: Spea2Configuration`, `Spea2Observer`.

**Builder methods pattern** (lines 61-84):
```rust
pub fn new(moead_config: MoeaDConfiguration, ga_config: GaConfiguration) -> Self {
    MoeaDGa { moead_config, ga_config, alleles: Vec::new(),
        initialization_fn: None, objective_fns: Vec::new(), observer: None }
}

pub fn with_observer(mut self, obs: Arc<dyn MoeaDObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(obs); self
}

#[inline]
pub(crate) fn notify<F: FnOnce(&dyn MoeaDObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer { f(obs.as_ref()); }
}

pub fn with_alleles(mut self, alleles: Vec<U::Gene>) -> Self { ... }
pub fn with_initialization_fn<F>(mut self, f: F) -> Self { ... }
pub fn with_objective_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self { ... }
pub fn build(self) -> Result<Self, GaError> { self.validate()?; Ok(self) }
```

**Validate pattern** (lines 118-185):
```rust
pub fn validate(&self) -> Result<(), GaError> {
    if self.moead_config.num_objectives == 0 {
        return Err(GaError::InvalidMoeaDConfiguration("num_objectives must be > 0".to_string()));
    }
    if self.moead_config.population_size < 2 {
        return Err(GaError::InvalidMoeaDConfiguration("population_size must be >= 2".to_string()));
    }
    // ... more checks ...
    Ok(())
}
```

For SPEA2, add archive-specific validation:
- Reject `archive_size > population_size`
- Reject `archive_size == 0`

**WASM cfg-gating for Instant::now()** (lines 325-336):
```rust
let t_sort: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))] { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")] { None }
} else { None };
```

**WASM cfg-gating for par_iter()** (lines 454-471):
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

**Observer notification pattern** (lines 406-416):
```rust
if let Some(start) = t_sort {
    self.notify(|obs| {
        obs.on_non_dominated_sort_complete(gen, start.elapsed().as_secs_f64() * 1000.0)
    });
}
self.notify(|obs| {
    obs.on_pareto_front_assigned(gen, front_count, population.len())
});
```

For SPEA2, the two hooks are `on_fitness_assigned` and `on_archive_updated`.

**Binary tournament selection** (from nsga3/mod.rs lines 525-542):
```rust
fn binary_tournament(&self, population: &[ParetoIndividual<U>], rng: &mut impl Rng) -> usize {
    let n = population.len();
    let i = rng.random_range(0..n);
    let j = rng.random_range(0..n);
    if population[i].rank < population[j].rank { i }
    else if population[j].rank < population[i].rank { j }
    else if rng.random::<bool>() { i }
    else { j }
}
```

SPEA2 tournament selects from the **archive** (not population), and uses SPEA2 fitness (lower is better) instead of rank. Early-generation fallback: when archive has < 2 individuals, fall back to population.

**Return type pattern** (from nsga3/mod.rs lines 363-365):
```rust
let front_individuals: Vec<ParetoIndividual<U>> =
    population.into_iter().filter(|ind| ind.rank == 0).collect();
Ok(ParetoFront::new(front_individuals))
```

SPEA2 extracts the Pareto front from the **final archive** (not population).

---

### `src/engines/spea2/configuration.rs` (config, CRUD)

**Analog:** `src/engines/moead/configuration.rs`

**Struct + Default pattern** (lines 53-91):
```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoeaDConfiguration {
    pub num_objectives: usize,
    pub population_size: usize,
    pub max_generations: usize,
    pub objective_directions: Vec<ObjectiveDirection>,
    // ... extra fields ...
}

impl Default for MoeaDConfiguration {
    fn default() -> Self {
        MoeaDConfiguration {
            num_objectives: 3,
            population_size: 100,
            max_generations: 200,
            objective_directions: Vec::new(),
            // ...
        }
    }
}
```

For SPEA2, the config struct is simpler — no weight vectors, no scalarization, no neighbourhoods. Key fields:
- `num_objectives`, `population_size`, `max_generations`, `objective_directions` (same as all engines)
- `archive_size: usize` (new — D-01, default equals population_size)
- No `effective_weight_vectors()` — no weight vectors needed
- `effective_directions()` — same as all engines (lines 184-190 of moead/configuration.rs)

**Builder methods pattern** (lines 98-139):
```rust
pub fn with_num_objectives(mut self, n: usize) -> Self { self.num_objectives = n; self }
pub fn with_population_size(mut self, size: usize) -> Self { self.population_size = size; self }
pub fn with_max_generations(mut self, gens: usize) -> Self { self.max_generations = gens; self }
pub fn with_objective_directions(mut self, directions: Vec<ObjectiveDirection>) -> Self { ... }
```

Add for SPEA2: `pub fn with_archive_size(mut self, size: usize) -> Self { self.archive_size = size; self }`

**effective_directions pattern** (lines 184-190):
```rust
pub fn effective_directions(&self) -> Vec<ObjectiveDirection> {
    if self.objective_directions.is_empty() {
        vec![ObjectiveDirection::Minimize; self.num_objectives]
    } else { self.objective_directions.clone() }
}
```

**Re-export ObjectiveDirection** (line 3):
```rust
pub use crate::nsga2::configuration::ObjectiveDirection;
```

---

### `src/observe/observer/mod.rs` (trait definition, event-driven)

**Analog:** `MoeaDObserver<U>` at lines 204-215

**Existing trait pattern** (lines 204-215):
```rust
pub trait MoeaDObserver<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(&self, _generation: usize, _front_count: usize, _population_size: usize) {}
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
}
```

**Spea2Observer pattern (add after MoeaDObserver block, before AllObserver)** (per D-04):
```rust
pub trait Spea2Observer<U: ChromosomeT>: Send + Sync {
    fn on_fitness_assigned(
        &self, _generation: usize, _duration_ms: f64,
        _pop_size: usize, _archive_size: usize,
    ) {}
    fn on_archive_updated(
        &self, _generation: usize, _archive_size: usize, _non_dominated_count: usize,
    ) {}
}
```

**Placement:** Insert `Spea2Observer<U>` trait definition AFTER `MoeaDObserver<U>` (after line 215) and BEFORE `AllObserver` (line 225). Do NOT add `Spea2Observer<U>` to the `AllObserver` supertrait bounds (per D-07).

---

### `src/observe/observer/log.rs` (observer impl, event-driven)

**Analog:** `impl MoeaDObserver<U>` at lines 224-240

**Existing impl pattern** (lines 224-240):
```rust
impl<U: ChromosomeT> MoeaDObserver<U> for LogObserver {
    fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize) {
        log::debug!(target: "moead_events",
            "Generation {} complete, population size = {}, fronts = {}",
            generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "moead_events",
            "Non-dominated sort complete at generation {} ({:.2}ms)",
            generation, duration_ms);
    }
}
```

**Spea2Observer impl pattern (add after MoeaDObserver impl, before end of file)** (per D-06):
```rust
impl<U: ChromosomeT> Spea2Observer<U> for LogObserver {
    fn on_fitness_assigned(&self, generation: usize, duration_ms: f64, pop_size: usize, archive_size: usize) {
        log::debug!(target: "spea2_events",
            "Strength+density fitness assigned at generation {} ({:.2}ms) — pop={}, archive={}",
            generation, duration_ms, pop_size, archive_size);
    }
    fn on_archive_updated(&self, generation: usize, archive_size: usize, non_dominated_count: usize) {
        log::debug!(target: "spea2_events",
            "Archive updated at generation {} — size={}, non-dominated={}",
            generation, archive_size, non_dominated_count);
    }
}
```

**Import update:** Add `Spea2Observer` to the existing `use` import from `crate::observer` (line 28):
```rust
use crate::observer::{
    ExtensionEvent, GaObserver, IslandGaObserver, MoeaDObserver, Nsga2Observer, Nsga3Observer,
    Spea2Observer,  // <-- ADD
};
```

---

### `src/lib.rs` (module re-export, N/A)

**Analog:** `pub mod moead` at lines 116-117

**Existing re-export pattern** (lines 116-117):
```rust
#[path = "engines/moead/mod.rs"]
pub mod moead;
```

**For SPEA2** (add after moead block):
```rust
#[path = "engines/spea2/mod.rs"]
pub mod spea2;
```

**Public re-exports** (after line 126):
```rust
pub use observer::Spea2Observer;  // <-- ADD after MoeaDObserver
```

---

### `src/error.rs` (error variant, N/A)

**Analog:** `InvalidMoeaDConfiguration(String)` at line 39

**Existing variant pattern** (lines 38-39):
```rust
    /// A MOEA/D configuration parameter is invalid.
    InvalidMoeaDConfiguration(String),
```

**For SPEA2** (add after `InvalidMoeaDConfiguration` at line 39):
```rust
    /// A SPEA2 configuration parameter is invalid.
    InvalidSpea2Configuration(String),
```

**Display impl** (after `InvalidMoeaDConfiguration` match arm at lines 67-69):
```rust
    GaError::InvalidSpea2Configuration(msg) => {
        write!(f, "Invalid SPEA2 configuration: {}", msg)
    }
```

---

### `tests/engines/spea2/test_spea2.rs` (test, CRUD)

**Analog:** `tests/engines/moead/test_moead.rs`

**Imports pattern** (lines 7-17):
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ObjectiveDirection, ScalarizationFn};
use genetic_algorithms::moead::MoeaDGa;
use genetic_algorithms::LogObserver;
use genetic_algorithms::MoeaDObserver;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
```

For SPEA2:
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};
use genetic_algorithms::spea2::Spea2Ga;
use genetic_algorithms::LogObserver;
use genetic_algorithms::Spea2Observer;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
```

**Validate test pattern** (lines 19-28):
```rust
#[test]
fn test_moead_validate_no_init_fn() {
    let config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_weight_vectors_auto(4);
    let ga_config = GaConfiguration::default();
    let moead = MoeaDGa::<RangeChromosome<f64>>::new(config, ga_config);
    let result = moead.validate();
    assert!(matches!(result, Err(GaError::InvalidMoeaDConfiguration(_))));
}
```

For SPEA2, mirror each validate test with `Spea2Configuration`, `Spea2Ga`, `InvalidSpea2Configuration`. Add specific tests for:
- `archive_size > population_size` -> error
- `archive_size == 0` -> error
- Valid config with `with_archive_size(N)` -> ok

**Build helper function pattern** (lines 170-200):
```rust
fn build_test_moead(...) -> MoeaDGa<RangeChromosome<f64>> {
    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(population_size)
        .with_max_generations(max_generations)
        .with_weight_vectors_auto(4)
        .with_scalarization(scalarization)
        .with_neighborhood_size(5)
        .with_max_neighbor_replacements(2);

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
        .with_objective_fns(zdt1_objectives())  // <-- ZDT1 for SPEA2
        .build()
        .expect("build should succeed with all required builders called")
}
```

For SPEA2, use ZDT1 (2-objective) instead of DTLZ2 (3-objective):
- `num_objectives: 2`, `archive_size: N`, no weight vectors/scalarization/neighbourhood fields
- Use `ZDT1` benchmark objective functions (mirror from `examples/nsga2_zdt1.rs`)

**CountingObserver pattern** (lines 274-293):
```rust
#[derive(Default)]
struct CountingObserver {
    sort_count: AtomicUsize,
    pareto_count: AtomicUsize,
}

impl MoeaDObserver<RangeChromosome<f64>> for CountingObserver {
    fn on_pareto_front_assigned(&self, _generation: usize, _front_count: usize, _population_size: usize) {
        self.pareto_count.fetch_add(1, Ordering::Relaxed);
    }
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {
        self.sort_count.fetch_add(1, Ordering::Relaxed);
    }
}
```

For SPEA2, two hooks: `on_fitness_assigned` and `on_archive_updated`:
```rust
#[derive(Default)]
struct CountingObserver {
    fitness_count: AtomicUsize,
    archive_count: AtomicUsize,
}

impl Spea2Observer<RangeChromosome<f64>> for CountingObserver {
    fn on_fitness_assigned(&self, _gen: usize, _dur: f64, _pop: usize, _arch: usize) {
        self.fitness_count.fetch_add(1, Ordering::Relaxed);
    }
    fn on_archive_updated(&self, _gen: usize, _arch: usize, _nd: usize) {
        self.archive_count.fetch_add(1, Ordering::Relaxed);
    }
}
```

**LogObserver smoke test** (lines 346-362):
```rust
#[test]
fn test_moead_log_observer() {
    let mut moead = build_test_moead(15, 3, ScalarizationFn::Tchebycheff)
        .with_observer(Arc::new(LogObserver) as Arc<dyn MoeaDObserver<...> + Send + Sync>);
    let result = moead.run();
    assert!(result.is_ok(), "run with LogObserver should succeed: {:?}", result.err());
    let front = result.unwrap();
    assert!(!front.is_empty(), "front should be non-empty under LogObserver");
}
```

**What differs for SPEA2 tests:**
- No MOEA/D-specific weight-vector or scalarization validation tests
- Add `archive_size` validation tests
- Use ZDT1 benchmark (2 objectives, 30 variables) as the run benchmark
- Observer test checks `fitness_count` and `archive_count` instead of `sort_count` and `pareto_count`
- No Differential mutation rejection test (SPEA2 is not per-sub-problem, it batches)

---

### `tests/engines/spea2/test_spea2_configuration.rs` (test, CRUD)

**Analog:** `tests/engines/moead/test_moead_configuration.rs`

**Default test pattern** (lines 1-16):
```rust
use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ObjectiveDirection, ScalarizationFn};

#[test]
fn test_moead_configuration_default() {
    let config = MoeaDConfiguration::default();
    assert_eq!(config.num_objectives, 3);
    assert_eq!(config.population_size, 100);
    assert_eq!(config.max_generations, 200);
    // ...
}
```

For SPEA2:
```rust
use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};

#[test]
fn test_spea2_configuration_default() {
    let config = Spea2Configuration::default();
    assert_eq!(config.num_objectives, 2);      // SPEA2 default
    assert_eq!(config.population_size, 100);
    assert_eq!(config.archive_size, 100);       // archive = population size (canonical)
    assert_eq!(config.max_generations, 250);
    assert!(config.objective_directions.is_empty());
}
```

**Archive size specific tests to add:**
- Default archive size equals population size
- Builder `with_archive_size(N)` sets correctly
- Smaller archive size accepted

**effective_directions tests** (lines 97-117) — same pattern, no changes needed.

---

### `examples/spea2_zdt1.rs` (example, request-response)

**Analog:** `examples/nsga2_zdt1.rs`

**Imports pattern** (lines 46-53):
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::{LogObserver, Nsga2Observer};
use std::sync::Arc;
```

For SPEA2:
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};
use genetic_algorithms::spea2::Spea2Ga;
use genetic_algorithms::{LogObserver, Spea2Observer};
use std::sync::Arc;
```

**ZDT1 objective functions** (lines 82-91) — same as nsga2_zdt1.rs:
```rust
let obj_f1 = |dna: &[RangeGenotype<f64>]| -> f64 { dna[0].value };

let obj_f2 = |dna: &[RangeGenotype<f64>]| -> f64 {
    let n = dna.len();
    let g = 1.0 + (9.0 / (n - 1) as f64) * dna[1..].iter().map(|gene| gene.value).sum::<f64>();
    g * (1.0 - (dna[0].value / g).sqrt())
};
```

**Engine build pattern** (lines 94-105):
```rust
let mut nsga2 = Nsga2Ga::<RangeChromosome<f64>>::new(nsga2_config, ga_config)
    .with_alleles(alleles)
    .with_initialization_fn(move |n, _, _| {
        range_random_initialization(n, Some(&alleles_clone), Some(true))
    })
    .with_objective_fns(vec![Box::new(obj_f1), Box::new(obj_f2)])
    .with_observer(Arc::new(LogObserver) as Arc<dyn Nsga2Observer<RangeChromosome<f64>> + Send + Sync>)
    .build()
    .expect("Failed to build NSGA-II");
```

For SPEA2, add `.with_archive_size(ARCHIVE_SIZE)` to the Spea2Configuration builder chain.

**Config setup** (lines 62-69):
```rust
let nsga2_config = Nsga2Configuration::new()
    .with_num_objectives(2)
    .with_population_size(POP_SIZE)
    .with_max_generations(MAX_GENERATIONS)
    .with_objective_directions(vec![ObjectiveDirection::Minimize, ObjectiveDirection::Minimize]);
```

For SPEA2, add `.with_archive_size(ARCHIVE_SIZE)`.

**Result handling** (lines 120-151) — same pattern, print Pareto front sorted by f1.

---

## Shared Patterns

### Authentication / Guarding
**Not applicable** — this is a Rust library with no authentication layer.

### WASM Compatibility (mandatory)
**Source:** `CLAUDE.md` (project instructions, WASM Compatibility section) + `src/engines/moead/mod.rs` lines 325-336, 454-471
**Apply to:** `src/engines/spea2/mod.rs`

Two specific cfg-gating patterns:

**Pattern A: `Instant::now()` gating** — gate both `Some(Instant::now())` and `.elapsed()` behind `#[cfg(not(target_arch = "wasm32"))]`:
```rust
let t_fitness: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))] { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")] { None }
} else { None };
```

**Pattern B: `par_iter()` gating** — duplicate the iterator expression:
```rust
#[cfg(not(target_arch = "wasm32"))]
let population: Vec<ParetoIndividual<U>> = chromosomes
    .into_par_iter()
    .map(|chrom| { /* closure body */ })
    .collect();
#[cfg(target_arch = "wasm32")]
let population: Vec<ParetoIndividual<U>> = chromosomes
    .into_iter()
    .map(|chrom| { /* closure body */ })
    .collect();
```

### Observer Dispatch Pattern
**Source:** `src/engines/moead/mod.rs` lines 79-84 (MoeaDGa) and identical in `src/engines/nsga3/mod.rs` lines 78-83 (Nsga3Ga)
**Apply to:** `src/engines/spea2/mod.rs`

```rust
#[inline]
pub(crate) fn notify<F: FnOnce(&dyn Spea2Observer<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### Error Handling
**Source:** `src/error.rs` lines 14-73
**Apply to:** `src/error.rs` (add `InvalidSpea2Configuration(String)` variant)

Pattern for adding a new engine-specific error variant (mirror `InvalidMoeaDConfiguration`):
1. Add enum variant: `InvalidSpea2Configuration(String),`
2. Add Display arm: `GaError::InvalidSpea2Configuration(msg) => write!(f, "Invalid SPEA2 configuration: {}", msg),`

### Validation
**Source:** `src/engines/moead/mod.rs` lines 118-185
**Apply to:** `src/engines/spea2/mod.rs` (`validate()` method on `Spea2Ga`)

Pattern: shared validation for `num_objectives == 0`, `pop_size < 2`, init_fn required, objective_fns count match, directions length match. SPEA2 additions: `archive_size > population_size`, `archive_size == 0`.

### Module Re-export
**Source:** `src/lib.rs` lines 116-117 (moead) + lines 98-117 (all engines)
**Apply to:** `src/lib.rs`

All engines use the `#[path]` attribute pattern for directory-based modules. Place the `spea2` re-export after `moead`.

## No Analog Found

All files have direct analogs in the codebase. No files require novel patterns.

| File | Role | Data Flow | Reason no analog |
|---|---|---|---|
| *(none)* | | | All files have exact or role-match analogs |

## Metadata

**Analog search scope:** `src/engines/`, `src/observe/`, `src/`, `tests/engines/`, `examples/`
**Files scanned:** 16 (moead/mod.rs, moead/configuration.rs, nsga3/mod.rs, nsga3/configuration.rs, observe/observer/mod.rs, observe/observer/log.rs, error.rs, lib.rs, multi_objective/pareto.rs, tests/engines/moead/test_moead.rs, tests/engines/moead/test_moead_configuration.rs, examples/nsga2_zdt1.rs, and supporting files)
**Pattern extraction date:** 2026-05-10
