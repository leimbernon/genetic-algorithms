# Phase 35: NSGA-III for Many-Objective Optimization - Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 14 (new/modified)
**Analogs found:** 13 / 14

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/engines/multi_objective/mod.rs` | module-root | extract/re-export | `src/engines/nsga2/mod.rs` (module pattern) | role-match |
| `src/engines/multi_objective/non_dominated_sort.rs` | utility | transform | `src/engines/nsga2/non_dominated_sort.rs` | exact (file moves verbatim) |
| `src/engines/multi_objective/pareto.rs` | model | CRUD | `src/engines/nsga2/pareto.rs` | exact (file moves verbatim) |
| `src/engines/nsga2/mod.rs` (modify) | engine | request-response | itself | n/a (add pub use re-exports) |
| `src/engines/nsga3/mod.rs` | engine | request-response | `src/engines/nsga2/mod.rs` | exact |
| `src/engines/nsga3/configuration.rs` | config | CRUD | `src/engines/nsga2/configuration.rs` | exact |
| `src/engines/nsga3/das_dennis.rs` | utility | transform | none (pure math, no codebase analog) | no-analog |
| `src/observe/observer/mod.rs` (modify) | observer | event-driven | itself — `Nsga2Observer<U>` trait (lines 154–167) | exact |
| `src/lib.rs` (modify) | config | re-export | itself (lines 109–110) | exact |
| `src/error.rs` (modify) | error | n/a | itself — `InvalidNsga2Configuration` variant (lines 35–36) | exact |
| `tests/engines/nsga3/test_nsga3.rs` | test | request-response | `tests/engines/nsga2/test_nsga2.rs` | exact |
| `tests/engines/nsga3/test_nsga3_configuration.rs` | test | CRUD | `tests/engines/nsga2/test_nsga2_configuration.rs` | exact |
| `tests/engines/nsga3/test_das_dennis.rs` | test | transform | `tests/engines/nsga2/test_non_dominated_sort.rs` | role-match |
| `examples/nsga3_dtlz2.rs` | example | request-response | `examples/nsga2_zdt1.rs` | exact |

---

## Pattern Assignments

### `src/engines/multi_objective/mod.rs` (module-root, re-export)

**Analog:** `src/lib.rs` lines 109–110 (`#[path]` pattern) + `src/engines/nsga2/mod.rs` lines 30–33 (sub-module declarations)

**Module declaration pattern** (`src/engines/nsga2/mod.rs` lines 30–33):
```rust
pub mod configuration;
pub mod crowding_distance;
pub mod non_dominated_sort;
pub mod pareto;
```

**`multi_objective/mod.rs` should declare:**
```rust
pub mod non_dominated_sort;
pub mod pareto;

// Shared type alias used by both nsga2 and nsga3 engines
pub type ObjectiveFn<G> = dyn Fn(&[G]) -> f64 + Send + Sync;
```

Note: `ObjectiveFn<G>` moves here from `nsga2/mod.rs` (see CONTEXT.md §code_context and RESEARCH.md Pitfall 4).

---

### `src/engines/multi_objective/non_dominated_sort.rs` (utility, transform)

**Analog:** `src/engines/nsga2/non_dominated_sort.rs` — **move verbatim**

**Only change required:** Update the import at line 1 from:
```rust
use super::pareto::{constrained_dominates, dominates, dominates_with_directions};
use crate::nsga2::configuration::ObjectiveDirection;
```
to:
```rust
use super::pareto::{constrained_dominates, dominates, dominates_with_directions};
use crate::multi_objective::... // or keep ObjectiveDirection re-exported from nsga2 via multi_objective
```

Since `ObjectiveDirection` lives in `nsga2::configuration` and is also needed by `nsga3::configuration`, the cleanest approach is to keep importing it from wherever it ends up (likely `nsga2::configuration` for backward compat, or a shared location). The planner should decide whether to move `ObjectiveDirection` to `multi_objective` — if left in `nsga2::configuration`, use `use crate::nsga2::configuration::ObjectiveDirection;`.

---

### `src/engines/multi_objective/pareto.rs` (model, CRUD)

**Analog:** `src/engines/nsga2/pareto.rs` — **move verbatim**

**Only change required:** Update line 1 from:
```rust
use crate::nsga2::configuration::ObjectiveDirection;
```
to the appropriate import after `ObjectiveDirection` is resolved (see above).

**Full struct and function inventory to preserve** (all from `src/engines/nsga2/pareto.rs`):
- `ParetoIndividual<U>` — struct with `chromosome`, `objectives`, `rank`, `crowding_distance`, `constraint_violation`
- `ParetoFront<U>` — wrapper struct
- `dominates()` — all-minimize predicate
- `dominates_with_directions()` — direction-aware predicate
- `constrained_dominates()` — constrained-domination predicate

**Critical constraint (RESEARCH.md Pitfall 3):** Do NOT add a `niche_distance` or `reference_point_idx` field to `ParetoIndividual`. NSGA-III tracks these as local variables inside `nsga3_environmental_selection()` only.

---

### `src/engines/nsga2/mod.rs` (modify — add re-exports)

**Change:** Remove `pub mod non_dominated_sort;` and `pub mod pareto;` declarations, replace with `pub use` re-exports. Also remove the `ObjectiveFn<G>` type alias (moved to `multi_objective`).

**Pattern** (RESEARCH.md Pattern 1):
```rust
// After extraction — add at top of nsga2/mod.rs module declarations section
pub use crate::multi_objective::pareto::*;
pub use crate::multi_objective::non_dominated_sort::*;
pub use crate::multi_objective::ObjectiveFn;
```

**Existing import to update** (`src/engines/nsga2/mod.rs` lines 39–42):
```rust
// Before:
use crate::nsga2::non_dominated_sort::{
    assign_ranks, non_dominated_sort_constrained, non_dominated_sort_with_directions,
};
use crate::nsga2::pareto::{ParetoFront, ParetoIndividual};

// After: these still resolve via the pub use re-exports above, so internal use can become:
use crate::multi_objective::non_dominated_sort::{
    assign_ranks, non_dominated_sort_constrained, non_dominated_sort_with_directions,
};
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
```

---

### `src/engines/nsga3/mod.rs` (engine, request-response)

**Analog:** `src/engines/nsga2/mod.rs` — copy structure verbatim, swap crowding-distance step for reference-point association.

**Imports pattern** (from `src/engines/nsga2/mod.rs` lines 35–50, adapted for nsga3):
```rust
pub mod configuration;
pub mod das_dennis;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::nsga3::configuration::Nsga3Configuration;
use crate::multi_objective::non_dominated_sort::{assign_ranks, non_dominated_sort_with_directions};
use crate::multi_objective::pareto::{ParetoFront, ParetoIndividual};
use crate::multi_objective::ObjectiveFn;
use crate::observer::Nsga3Observer;
use crate::operations::mutation;
use crate::traits::{ChromosomeT, InitializationFn};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;
```

**Struct pattern** (`src/engines/nsga2/mod.rs` lines 60–79, adapted):
```rust
pub struct Nsga3Ga<U>
where
    U: ChromosomeT,
{
    pub nsga3_config: Nsga3Configuration,
    pub ga_config: GaConfiguration,
    pub alleles: Vec<U::Gene>,
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    // No constraint_fns in Phase 35 (deferred per CONTEXT.md)
    pub observer: Option<Arc<dyn Nsga3Observer<U> + Send + Sync>>,
}
```

**with_observer + notify pattern** (`src/engines/nsga2/mod.rs` lines 104–115):
```rust
pub fn with_observer(mut self, obs: Arc<dyn Nsga3Observer<U> + Send + Sync>) -> Self {
    self.observer = Some(obs);
    self
}

#[inline]
fn notify<F: FnOnce(&dyn Nsga3Observer<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

**WASM-gated Instant pattern** (`src/engines/nsga2/mod.rs` lines 234–241):
```rust
let t_sort: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
// ...
if let Some(start) = t_sort {
    self.notify(|obs| {
        obs.on_non_dominated_sort_complete(gen, start.elapsed().as_secs_f64() * 1000.0)
    });
}
```

**WASM-gated par_iter pattern** (`src/engines/nsga2/mod.rs` lines 385–407):
```rust
#[cfg(not(target_arch = "wasm32"))]
let population = chromosomes
    .into_par_iter()
    .map(|chrom| {
        let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
        let mut ind = ParetoIndividual::new(chrom, objectives);
        ind
    })
    .collect();
#[cfg(target_arch = "wasm32")]
let population = chromosomes
    .into_iter()
    .map(|chrom| {
        let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
        let mut ind = ParetoIndividual::new(chrom, objectives);
        ind
    })
    .collect();
```

**initialize_population() pattern** (`src/engines/nsga2/mod.rs` lines 356–408): copy verbatim, remove `constraint_fns` usage and `evaluate_constraints` call.

**create_offspring() pattern** (`src/engines/nsga2/mod.rs` lines 411–523): copy verbatim, remove `constraint_fns` usage.

**binary_tournament() pattern** (`src/engines/nsga2/mod.rs` lines 534–570): copy verbatim. In NSGA-III, `crowding_distance` is always 0.0, so the tie-breaker will effectively be random — this is acceptable for Phase 35.

**run() structure** (`src/engines/nsga2/mod.rs` lines 218–330):
```rust
pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
    self.validate()?;
    crate::rng::set_seed(self.ga_config.rng_seed);

    let pop_size = self.nsga3_config.population_size;
    let max_gens = self.nsga3_config.max_generations;
    let directions = self.nsga3_config.effective_directions();
    let reference_points = self.nsga3_config.effective_reference_points()
        .ok_or_else(|| GaError::InvalidNsga3Configuration("...".to_string()))?;

    let mut population = self.initialize_population()?;

    for gen in 0..max_gens {
        // non-dominated sort + assign ranks (copy from nsga2)
        // observer: on_non_dominated_sort_complete (WASM-gated Instant)
        // create_offspring (copy from nsga2)
        // combine parent + offspring
        // non-dominated sort on combined
        // assign ranks on combined
        // nsga3_environmental_selection(&combined, pop_size, &reference_points, &directions)
        // observer: on_pareto_front_assigned
        // track best on obj-0 in rank-0 (on_new_best hook through Nsga3Observer)
    }

    let front_individuals: Vec<ParetoIndividual<U>> =
        population.into_iter().filter(|ind| ind.rank == 0).collect();
    Ok(ParetoFront::new(front_individuals))
}
```

**validate() pattern** (`src/engines/nsga2/mod.rs` lines 169–202):
```rust
pub fn validate(&self) -> Result<(), GaError> {
    if self.nsga3_config.num_objectives == 0 {
        return Err(GaError::InvalidNsga3Configuration(
            "num_objectives must be > 0".to_string(),
        ));
    }
    if self.nsga3_config.population_size < 2 {
        return Err(GaError::InvalidNsga3Configuration(
            "population_size must be >= 2".to_string(),
        ));
    }
    if self.initialization_fn.is_none() {
        return Err(GaError::InvalidNsga3Configuration(
            "initialization_fn is required".to_string(),
        ));
    }
    if self.objective_fns.len() != self.nsga3_config.num_objectives {
        return Err(GaError::InvalidNsga3Configuration(format!(
            "Expected {} objective functions, got {}",
            self.nsga3_config.num_objectives,
            self.objective_fns.len()
        )));
    }
    // Additional: validate reference points configured
    if self.nsga3_config.effective_reference_points().is_none() {
        return Err(GaError::InvalidNsga3Configuration(
            "reference points must be configured via with_reference_points_auto or with_reference_points".to_string(),
        ));
    }
    Ok(())
}
```

---

### `src/engines/nsga3/configuration.rs` (config, CRUD)

**Analog:** `src/engines/nsga2/configuration.rs` — mirror structure exactly.

**Full pattern** (`src/engines/nsga2/configuration.rs` lines 1–99):
```rust
/// Direction of optimization — shared with nsga2::configuration (or re-exported from there).
// Option A: import from nsga2 and re-export:
pub use crate::nsga2::configuration::ObjectiveDirection;
// Option B: move ObjectiveDirection to multi_objective — either works, pick one.

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nsga3Configuration {
    pub num_objectives: usize,
    pub population_size: usize,
    pub max_generations: usize,
    pub objective_directions: Vec<ObjectiveDirection>,
    // Phase 35 additions:
    reference_points_auto_p: Option<usize>,
    reference_points_custom: Option<Vec<Vec<f64>>>,
}

impl Default for Nsga3Configuration {
    fn default() -> Self {
        Nsga3Configuration {
            num_objectives: 3,       // NSGA-III targets 3+ objectives
            population_size: 100,
            max_generations: 200,
            objective_directions: Vec::new(),
            reference_points_auto_p: None,
            reference_points_custom: None,
        }
    }
}

impl Nsga3Configuration {
    pub fn new() -> Self { Self::default() }

    // Fluent builders (return Self — same as Nsga2Configuration):
    pub fn with_num_objectives(mut self, n: usize) -> Self { ... }
    pub fn with_population_size(mut self, size: usize) -> Self { ... }
    pub fn with_max_generations(mut self, gens: usize) -> Self { ... }
    pub fn with_objective_directions(mut self, directions: Vec<ObjectiveDirection>) -> Self { ... }

    // Phase 35 specific:
    pub fn with_reference_points_auto(mut self, p: usize) -> Self {
        self.reference_points_auto_p = Some(p);
        self.reference_points_custom = None;  // last call wins (D-07)
        self
    }
    pub fn with_reference_points(mut self, points: Vec<Vec<f64>>) -> Self {
        self.reference_points_custom = Some(points);
        self.reference_points_auto_p = None;  // last call wins (D-07)
        self
    }

    // Called at validate() time to materialise points
    pub fn effective_reference_points(&self) -> Option<Vec<Vec<f64>>> {
        if let Some(p) = self.reference_points_auto_p {
            Some(crate::nsga3::das_dennis::generate_das_dennis(self.num_objectives, p))
        } else {
            self.reference_points_custom.clone()
        }
    }

    /// effective_directions mirrors Nsga2Configuration exactly:
    pub fn effective_directions(&self) -> Vec<ObjectiveDirection> {
        if self.objective_directions.is_empty() {
            vec![ObjectiveDirection::Minimize; self.num_objectives]
        } else {
            self.objective_directions.clone()
        }
    }
}
```

**effective_directions source** (`src/engines/nsga2/configuration.rs` lines 92–98):
```rust
pub fn effective_directions(&self) -> Vec<ObjectiveDirection> {
    if self.objective_directions.is_empty() {
        vec![ObjectiveDirection::Minimize; self.num_objectives]
    } else {
        self.objective_directions.clone()
    }
}
```

---

### `src/engines/nsga3/das_dennis.rs` (utility, transform)

**Analog:** No codebase analog — pure math. Use algorithm from RESEARCH.md Pattern 2.

See "No Analog Found" section for pattern source.

---

### `src/observe/observer/mod.rs` (modify — add Nsga3Observer trait)

**Analog:** `src/observe/observer/mod.rs` lines 154–167 (`Nsga2Observer<U>` trait definition).

**Pattern to copy** (lines 150–167):
```rust
/// Observer for [`Nsga2Ga<U>`](crate::nsga2::Nsga2Ga) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
pub trait Nsga2Observer<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(
        &self,
        _generation: usize,
        _front_count: usize,
        _population_size: usize,
    ) {
    }
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
    fn on_crowding_distance_calculated(&self, _generation: usize, _duration_ms: f64) {}
}
```

**`Nsga3Observer<U>` to add immediately after line 167** (D-08):
```rust
/// Observer for [`Nsga3Ga<U>`](crate::nsga3::Nsga3Ga) engine-specific events.
///
/// All methods have default no-op implementations. The `Send + Sync`
/// supertraits are required for safe sharing across rayon threads via `Arc`.
///
/// Note: `AllObserver<U>` does not include `Nsga3Observer<U>` in Phase 35.
/// Use `Nsga3Ga::with_observer()` to attach an observer independently.
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

**AllObserver supertrait** (lines 177–187) — DO NOT modify (D-10 deferred).

---

### `src/lib.rs` (modify — add module declarations and re-exports)

**Pattern** (`src/lib.rs` lines 109–124):
```rust
// Existing:
#[path = "engines/nsga2/mod.rs"]
pub mod nsga2;

// Add after line 110:
#[path = "engines/multi_objective/mod.rs"]
pub mod multi_objective;
#[path = "engines/nsga3/mod.rs"]
pub mod nsga3;

// Existing pub use block (lines 112–124):
pub use observer::Nsga2Observer;
// Add:
pub use observer::Nsga3Observer;
```

---

### `src/error.rs` (modify — add InvalidNsga3Configuration variant)

**Pattern** (`src/error.rs` lines 35–36, 58–61):
```rust
// In enum GaError — add after InvalidNsga2Configuration:
/// An NSGA-III configuration parameter is invalid.
InvalidNsga3Configuration(String),

// In Display impl — add matching arm:
GaError::InvalidNsga3Configuration(msg) => {
    write!(f, "Invalid NSGA-III configuration: {}", msg)
}
```

---

### `tests/engines/nsga3/test_nsga3.rs` (test, request-response)

**Analog:** `tests/engines/nsga2/test_nsga2.rs`

**Imports pattern** (`tests/engines/nsga2/test_nsga2.rs` lines 1–4):
```rust
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
use genetic_algorithms::nsga2::Nsga2Ga;
```

**Adapted for nsga3:**
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::nsga3::configuration::Nsga3Configuration;
use genetic_algorithms::nsga3::Nsga3Ga;
use genetic_algorithms::initializers::range_random_initialization;
```

**Test structure pattern** (`tests/engines/nsga2/test_nsga2.rs` lines 6–50):
```rust
#[test]
fn test_nsga3_validate_no_init_fn() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let ga_config = GaConfiguration::default();
    let nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(config, ga_config);
    assert!(nsga3.validate().is_err());
}

#[test]
fn test_nsga3_run_produces_pareto_front() {
    // 3-objective DTLZ2 sphere, small population for fast test
    // ...same build + run pattern as nsga2 test
    let result = nsga3.run();
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}
```

---

### `tests/engines/nsga3/test_nsga3_configuration.rs` (test, CRUD)

**Analog:** `tests/engines/nsga2/test_nsga2_configuration.rs`

**Imports pattern** (`tests/engines/nsga2/test_nsga2_configuration.rs` lines 1–1):
```rust
use genetic_algorithms::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
```

**Adapted:**
```rust
use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};
// Or: use genetic_algorithms::nsga2::configuration::ObjectiveDirection;
//     depending on where ObjectiveDirection ends up
```

**Test structure pattern** (lines 3–57):
```rust
#[test]
fn test_nsga3_configuration_default() {
    let config = Nsga3Configuration::default();
    assert_eq!(config.num_objectives, 3);
    // ...
}

#[test]
fn test_nsga3_reference_points_auto() {
    let config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_reference_points_auto(4);
    let pts = config.effective_reference_points().unwrap();
    // C(4+3-1, 3-1) = C(6,2) = 15 points
    assert_eq!(pts.len(), 15);
    // Each point sums to 1.0
    for pt in &pts {
        let sum: f64 = pt.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_nsga3_configuration_missing_reference_points_fails_validate() {
    // validate() must return Err when neither auto nor custom is set
}

#[test]
fn test_nsga3_custom_points_last_call_wins() {
    // D-07: auto then custom → custom wins
}
```

---

### `tests/engines/nsga3/test_das_dennis.rs` (test, transform)

**Analog:** `tests/engines/nsga2/test_non_dominated_sort.rs` (unit test structure for a pure utility function)

**Imports pattern** (`tests/engines/nsga2/test_non_dominated_sort.rs` lines 1–5):
```rust
use genetic_algorithms::nsga2::non_dominated_sort::{...};
```

**Adapted:**
```rust
use genetic_algorithms::nsga3::das_dennis::generate_das_dennis;
// (function must be pub to be tested; or test via nsga3::configuration::effective_reference_points)
```

**Test structure pattern** (lines 7–15):
```rust
#[test]
fn test_das_dennis_m3_p2() {
    let pts = generate_das_dennis(3, 2);
    assert_eq!(pts.len(), 6); // C(4,2)
    for pt in &pts {
        assert_eq!(pt.len(), 3);
        let sum: f64 = pt.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_das_dennis_m3_p4() {
    let pts = generate_das_dennis(3, 4);
    assert_eq!(pts.len(), 15); // C(6,2)
}

#[test]
fn test_das_dennis_m5_p6() {
    let pts = generate_das_dennis(5, 6);
    assert_eq!(pts.len(), 210); // C(10,4)
}
```

---

### `tests/test_engines.rs` (modify — add nsga3 mod block)

**Analog:** `tests/test_engines.rs` lines 1–31 (existing mod block structure)

**Pattern to extend** (lines 24–30):
```rust
// Existing:
mod nsga2 {
    mod test_crowding_distance;
    mod test_non_dominated_sort;
    mod test_nsga2;
    mod test_nsga2_configuration;
    mod test_pareto;
}

// Add after nsga2 block:
mod nsga3 {
    mod test_das_dennis;
    mod test_nsga3;
    mod test_nsga3_configuration;
}
```

---

### `examples/nsga3_dtlz2.rs` (example, request-response)

**Analog:** `examples/nsga2_zdt1.rs` — mirror structure exactly.

**Imports pattern** (`examples/nsga2_zdt1.rs` lines 46–53):
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

**Adapted for nsga3:**
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};
use genetic_algorithms::nsga3::Nsga3Ga;
use genetic_algorithms::{LogObserver, Nsga3Observer};
use std::sync::Arc;
```

**Builder pattern** (`examples/nsga2_zdt1.rs` lines 62–105):
```rust
// Nsga2Ga pattern:
let mut nsga2 = Nsga2Ga::<RangeChromosome<f64>>::new(nsga2_config, ga_config)
    .with_alleles(alleles)
    .with_initialization_fn(move |n, _, _| { ... })
    .with_objective_fns(vec![Box::new(obj_f1), Box::new(obj_f2)])
    .with_observer(
        Arc::new(LogObserver) as Arc<dyn Nsga2Observer<RangeChromosome<f64>> + Send + Sync>
    )
    .build()
    .expect("Failed to build NSGA-II");

// Nsga3 equivalent (3 objectives):
let mut nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(nsga3_config, ga_config)
    .with_alleles(alleles)
    .with_initialization_fn(move |n, _, _| { ... })
    .with_objective_fns(vec![Box::new(obj_f1), Box::new(obj_f2), Box::new(obj_f3)])
    .with_observer(
        Arc::new(LogObserver) as Arc<dyn Nsga3Observer<RangeChromosome<f64>> + Send + Sync>
    )
    .build()
    .expect("Failed to build NSGA-III");
```

**DTLZ2 objective functions** (from RESEARCH.md Code Examples):
```rust
// DTLZ2 3-objective sphere benchmark
// Variables: x = (x_1, ..., x_n) in [0, 1], standard n = 12 (M + k - 1, k=10)
// f_1 = cos(x_1 * π/2) * cos(x_2 * π/2) * (1 + g)
// f_2 = cos(x_1 * π/2) * sin(x_2 * π/2) * (1 + g)
// f_3 = sin(x_1 * π/2) * (1 + g)
// g(x) = sum_{i=M}^{n} (x_i - 0.5)^2  [standard form]
// Pareto front: g = 0 (x_3..x_n = 0.5), lies on unit sphere f1^2 + f2^2 + f3^2 = 1
```

**run() + result pattern** (`examples/nsga2_zdt1.rs` lines 120–150):
```rust
match nsga3.run() {
    Ok(mut front) => {
        println!("Pareto front: {} non-dominated solutions", front.len());
        // Sort by f1 ascending
        front.individuals.sort_by(|a, b| {
            a.objectives[0].partial_cmp(&b.objectives[0]).unwrap()
        });
        // ... print sample
    }
    Err(e) => {
        eprintln!("NSGA-III failed: {:?}", e);
        std::process::exit(1);
    }
}
```

---

## Shared Patterns

### WASM cfg-Gating: Instant::now()
**Source:** `src/engines/nsga2/mod.rs` lines 234–247
**Apply to:** `src/engines/nsga3/mod.rs` — all observer timing blocks
```rust
let t_sort: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
// use:
if let Some(start) = t_sort {
    self.notify(|obs| {
        obs.on_non_dominated_sort_complete(gen, start.elapsed().as_secs_f64() * 1000.0)
    });
}
```

### WASM cfg-Gating: par_iter()
**Source:** `src/engines/nsga2/mod.rs` lines 385–407 (initialize_population) and 500–521 (create_offspring)
**Apply to:** `src/engines/nsga3/mod.rs` — both `initialize_population()` and `create_offspring()`
```rust
#[cfg(not(target_arch = "wasm32"))]
let result: Vec<_> = items.into_par_iter().map(|chrom| { ... }).collect();
#[cfg(target_arch = "wasm32")]
let result: Vec<_> = items.into_iter().map(|chrom| { ... }).collect();
```

### serde derive on configuration structs
**Source:** `src/engines/nsga2/configuration.rs` lines 3–4 and 32–33
**Apply to:** `src/engines/nsga3/configuration.rs` (struct and enum)
```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nsga3Configuration { ... }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectiveDirection { ... }
```

### GaError variant + Display arm
**Source:** `src/error.rs` lines 35–36, 58–60
**Apply to:** `src/error.rs` — add `InvalidNsga3Configuration(String)` variant and matching Display arm
```rust
InvalidNsga3Configuration(String),
// Display:
GaError::InvalidNsga3Configuration(msg) => {
    write!(f, "Invalid NSGA-III configuration: {}", msg)
}
```

### pub use re-export for backward compat
**Source:** Established v2.3.0 pattern; spec from CONTEXT.md D-03
**Apply to:** `src/engines/nsga2/mod.rs` after extracting files
```rust
pub use crate::multi_objective::pareto::*;
pub use crate::multi_objective::non_dominated_sort::*;
pub use crate::multi_objective::ObjectiveFn;
```

### #[path] module declaration in lib.rs
**Source:** `src/lib.rs` lines 109–110
**Apply to:** `src/lib.rs` — add two new entries
```rust
#[path = "engines/multi_objective/mod.rs"]
pub mod multi_objective;
#[path = "engines/nsga3/mod.rs"]
pub mod nsga3;
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/engines/nsga3/das_dennis.rs` | utility | transform | No combinatorial enumeration functions exist in the codebase. Use algorithm from RESEARCH.md Pattern 2 (recursive `enumerate_partitions` over integer vectors summing to `p`). The function `generate_das_dennis(num_objectives: usize, p: usize) -> Vec<Vec<f64>>` is a pure function with no state or dependencies beyond `std`. |

---

## Metadata

**Analog search scope:** `src/engines/nsga2/`, `src/observe/observer/`, `src/lib.rs`, `src/error.rs`, `tests/engines/nsga2/`, `examples/`
**Files scanned:** 10 source files read directly
**Pattern extraction date:** 2026-05-07
