# Phase 50: Lexicase Selection - Pattern Map

**Mapped:** 2026-05-23
**Files analyzed:** 10 (2 new, 8 modified)
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/traits/multi_case_fitness.rs` | trait | request-response | `src/traits/chromosome.rs` | role-match |
| `src/operations/selection/lexicase.rs` | operator (selection) | transform | `src/operations/selection/clearing.rs` | exact |
| `src/operations/selection.rs` | operator registry | request-response | `src/operations/selection.rs` (self, extend) | exact |
| `src/operations.rs` | enum definition | config | `src/operations.rs` (self, extend) | exact |
| `src/configuration.rs` | config struct | config | `src/configuration.rs` (self, `boltzmann_temperature` / `niche_radius` fields) | exact |
| `src/traits/configuration.rs` | config trait | config | `src/traits/configuration.rs` (self, `with_niche_radius` builder) | exact |
| `src/engines/ga.rs` | engine dispatch | request-response | `src/engines/ga.rs` (self, lines 1474-1478 + `SelectionConfig` impl) | exact |
| `src/lib.rs` | crate re-export | config | `src/lib.rs` (self, lines 333-335 `pub use traits::*`) | exact |
| `src/traits.rs` | module re-export | config | `src/traits.rs` (self, lines 47-62 `pub use` block) | exact |
| `tests/lexicase.rs` | integration test | CRUD | `tests/operations/test_selection_clearing.rs` | exact |

---

## Pattern Assignments

### `src/traits/multi_case_fitness.rs` (trait, request-response)

**Analog:** `src/traits/chromosome.rs`

**Trait definition pattern** (`src/traits/chromosome.rs` lines 27-62 — the full file):
```rust
// Module-level doc comment in //! style
// No derive macros — trait definitions carry no data
// Supertrait bound on the first line of the trait declaration
// All methods documented with /// doc comments
// Default method implementations permitted (fitness_distance is an example)

pub trait ChromosomeT: Clone + Default + Send + Sync + 'static {
    type Gene: GeneT;
    fn new() -> Self { Default::default() }
    fn calculate_fitness(&mut self);
    fn fitness(&self) -> f64;
    fn set_fitness(&mut self, fitness: f64) -> &mut Self;
    fn set_age(&mut self, age: usize) -> &mut Self;
    fn age(&self) -> usize;
    fn fitness_distance(&self, fitness_target: &f64) -> f64 {
        (fitness_target - self.fitness()).abs()
    }
}
```

**MultiCaseFitness pattern to follow** — file should contain exactly:
```rust
//! Multi-case fitness trait for lexicase selection.
// (module-level doc in //! style)

use crate::traits::ChromosomeT;

/// Opt-in trait enabling [`Selection::Lexicase`] and [`Selection::EpsilonLexicase`].
///
/// Implement alongside [`ChromosomeT`]. Call `set_case_fitness` inside your
/// `calculate_fitness()` implementation.
pub trait MultiCaseFitness: ChromosomeT {
    /// Returns the per-case fitness scores set during `calculate_fitness`.
    fn case_fitness(&self) -> &[f64];

    /// Sets the per-case fitness scores. Called inside `calculate_fitness`.
    fn set_case_fitness(&mut self, scores: Vec<f64>);
}
```

Key points:
- Supertrait is `ChromosomeT`, declared on same line as `pub trait MultiCaseFitness`
- No `type Gene` associated type needed (inherited from supertrait)
- No `Clone + Default + Send + Sync + 'static` bounds needed (inherited via `ChromosomeT`)
- Two methods only: getter (`&[f64]`) and setter (`Vec<f64>`) — D-01

---

### `src/operations/selection/lexicase.rs` (operator, transform)

**Analog:** `src/operations/selection/clearing.rs` (full file, 103 lines)

**Module-level doc pattern** (`clearing.rs` lines 1-15):
```rust
//! Clearing selection operator.
//!
//! Implements a diversity-promoting selection strategy...
//! 1. Step one description.
//! 2. Step two description.
//! 3. Step three description.
//!
//! Brief note on the distance metric / key characteristic.
```

**Imports pattern** (`clearing.rs` lines 16-17):
```rust
use crate::traits::ChromosomeT;
use log::{debug, trace};
```
For lexicase, add:
```rust
use crate::traits::{ChromosomeT, MultiCaseFitness};
use log::{debug, trace};
use rand::Rng;
```

**Function signature pattern** (`clearing.rs` lines 29-33):
```rust
pub fn clearing_selection<U: ChromosomeT>(
    chromosomes: &[U],
    niche_radius: f64,
    number_of_couples: usize,
) -> Vec<(usize, usize)> {
```
Lexicase analogs:
```rust
pub fn lexicase_selection<U>(
    chromosomes: &[U],
    number_of_couples: usize,
) -> Vec<(usize, usize)>
where
    U: ChromosomeT + MultiCaseFitness,
{

pub fn epsilon_lexicase_selection<U>(
    chromosomes: &[U],
    number_of_couples: usize,
    epsilon: Option<f64>,   // None = compute MAD; Some(v) = fixed epsilon
) -> Vec<(usize, usize)>
where
    U: ChromosomeT + MultiCaseFitness,
{
```

**Debug log entry pattern** (`clearing.rs` line 34):
```rust
debug!(target="selection_events", method="clearing"; "Starting clearing selection with niche_radius={} number_of_couples={}", niche_radius, number_of_couples);
```
Lexicase analog:
```rust
debug!(target="selection_events", method="lexicase"; "Starting lexicase selection, pop={} cases={} couples={}", chromosomes.len(), num_cases, number_of_couples);
```

**RNG + mating Vec pattern** (`clearing.rs` lines 86-98):
```rust
let mut rng = crate::rng::make_rng();
let mut mating = Vec::with_capacity(number_of_couples);

use rand::Rng;
while mating.len() < number_of_couples {
    let i1 = rng.random_range(0..eligible.len());
    let i2_raw = rng.random_range(0..(eligible.len() - 1));
    let i2 = if i2_raw >= i1 { i2_raw + 1 } else { i2_raw };
    let idx1 = eligible[i1];
    let idx2 = eligible[i2];
    mating.push((idx1, idx2));
    trace!(target="selection_events", method="clearing"; "Mating index {} with index {}", idx1, idx2);
}
```
For lexicase, the pairing uses two independent filter-loop runs per couple (not the `eligible[i1]/eligible[i2]` approach). The `while mating.len() < number_of_couples` loop wrapper is the same. Call the single-winner filter as a helper (`fn filter_one_winner`) twice per iteration.

**Early-exit guard pattern** (`clearing.rs` lines 77-80):
```rust
if eligible.len() < 2 {
    debug!(...);
    return Vec::new();
}
```
Lexicase analog — at top of function:
```rust
if chromosomes.len() < 2 || chromosomes[0].case_fitness().is_empty() {
    debug!(target="selection_events", method="lexicase"; "Lexicase selection skipped: pop={} cases={}", chromosomes.len(), chromosomes.first().map(|c| c.case_fitness().len()).unwrap_or(0));
    return Vec::new();
}
```

**Fisher-Yates shuffle pattern** (same as used in `src/initializers/unique_initializer.rs`):
```rust
let mut case_order: Vec<usize> = (0..num_cases).collect();
for i in (1..case_order.len()).rev() {
    let j = rng.random_range(0..=i);
    case_order.swap(i, j);
}
```

**WASM comment** (mandatory per CLAUDE.md):
```rust
// WASM: intentionally not par_iter — lexicase inner loop is sequential by design
// (shrinking pool state cannot be parallelized)
```

**Debug log exit pattern** (`clearing.rs` line 101):
```rust
debug!(target="selection_events", method="clearing"; "Clearing selection finished: {} pairs", mating.len());
mating
```

---

### `src/operations/selection.rs` (operator registry, extend)

**Analog:** `src/operations/selection.rs` (self — extend the existing file)

**Existing pub use block** (lines 13-20):
```rust
pub use self::boltzmann::boltzmann_selection;
pub use self::clearing::clearing_selection;
pub use self::fitness_proportionate::roulette_wheel_selection;
pub use self::fitness_proportionate::stochastic_universal_sampling;
pub use self::random::random;
pub use self::rank::rank_selection;
pub use self::tournament::tournament;
pub use self::truncation::truncation_selection;
```
Add after the existing block:
```rust
pub use self::lexicase::epsilon_lexicase_selection;
pub use self::lexicase::lexicase_selection;
```

**Existing pub mod block** (lines 24-31):
```rust
pub mod boltzmann;
pub mod clearing;
pub mod fitness_proportionate;
pub mod random;
pub mod rank;
pub mod tournament;
pub mod truncation;
```
Add:
```rust
pub mod lexicase;
```

**SelectionOperator trait match arms** (lines 42-65) — the `Clearing` arm is the exact panic/warn pattern to follow for Lexicase/EpsilonLexicase (lines 57-63):
```rust
Selection::Clearing => {
    log::warn!(target: "selection_events",
        "Selection::Clearing called through SelectionOperator trait: \
         niche_radius defaults to 0.1 (configured value ignored). \
         Use selection::factory for the full configuration.");
    clearing_selection(chromosomes, 0.1, number_of_couples)
}
```
Lexicase arms use `panic!` (D-08):
```rust
Selection::Lexicase | Selection::EpsilonLexicase => {
    panic!(
        "Selection::Lexicase/EpsilonLexicase cannot be called through SelectionOperator \
         trait: use selection::factory_lexicase for Lexicase/EpsilonLexicase operators. \
         Island-model and NSGA-II paths do not support MultiCaseFitness."
    );
}
```

**factory() function pattern** (lines 74-118) — `factory_lexicase` mirrors this exactly:
```rust
pub fn factory<U>(
    chromosomes: &[U],
    configuration: SelectionConfiguration,
    number_of_threads: usize,
) -> Result<Vec<(usize, usize)>, GaError>
where
    U: ChromosomeT + Sync + Send + 'static + Clone,
{
    if chromosomes.len() < 2 {
        return Err(GaError::SelectionError(format!(
            "Population size {} is too small for selection (minimum 2)",
            chromosomes.len()
        )));
    }

    // Guard: reject NaN fitness values which corrupt selection logic
    for (i, chromosome) in chromosomes.iter().enumerate() {
        if chromosome.fitness().is_nan() {
            return Err(GaError::SelectionError(format!(
                "Chromosome at index {} has NaN fitness. ...",
                i
            )));
        }
    }

    let pairs = match configuration.method {
        Selection::Boltzmann => boltzmann_selection(...),
        Selection::Clearing => clearing_selection(...),
        _ => configuration.method.select(...),
    };

    Ok(pairs)
}
```
`factory_lexicase` signature and structure:
```rust
pub fn factory_lexicase<U>(
    chromosomes: &mut [U],   // mut needed for set_fitness sync (D-04)
    configuration: SelectionConfiguration,
    number_of_threads: usize,
) -> Result<Vec<(usize, usize)>, GaError>
where
    U: ChromosomeT + MultiCaseFitness + Sync + Send + 'static + Clone,
{
    // Guard: min population size
    if chromosomes.len() < 2 { ... }

    // Guard: case fitness populated
    if chromosomes[0].case_fitness().is_empty() {
        return Err(GaError::SelectionError(
            "case_fitness() is empty — implement MultiCaseFitness::set_case_fitness \
             in calculate_fitness()".into()
        ));
    }

    // Guard: NaN in case scores (follow same NaN guard as factory())
    for (i, c) in chromosomes.iter().enumerate() {
        for (j, &score) in c.case_fitness().iter().enumerate() {
            if score.is_nan() {
                return Err(GaError::SelectionError(format!(
                    "Chromosome {} has NaN at case_fitness[{}]", i, j
                )));
            }
        }
    }

    let pairs = match configuration.method {
        Selection::Lexicase => lexicase_selection(chromosomes, configuration.number_of_couples),
        Selection::EpsilonLexicase => {
            let eps = if configuration.epsilon == 0.0 { None } else { Some(configuration.epsilon) };
            epsilon_lexicase_selection(chromosomes, configuration.number_of_couples, eps)
        }
        _ => return Err(GaError::ConfigurationError(
            "factory_lexicase called with non-lexicase selection method".into()
        )),
    };

    // D-04: sync scalar fitness to mean case score for all individuals
    for c in chromosomes.iter_mut() {
        let scores = c.case_fitness();
        if !scores.is_empty() {
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            c.set_fitness(mean);
        }
    }

    Ok(pairs)
}
```
Also add error return in `factory()` for Lexicase/EpsilonLexicase arms (D-06):
```rust
Selection::Lexicase | Selection::EpsilonLexicase => {
    return Err(GaError::ConfigurationError(
        "Use selection::factory_lexicase for Lexicase/EpsilonLexicase; \
         standard factory() does not support MultiCaseFitness bound".into()
    ));
}
```
This arm goes before the `_ =>` arm in the `match configuration.method` block inside `factory()`.

---

### `src/operations.rs` (enum definition, extend)

**Analog:** `src/operations.rs` (self — add two variants to `Selection` enum)

**Existing enum pattern** (lines 43-70):
```rust
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Selection {
    Random,
    RouletteWheel,
    StochasticUniversalSampling,
    Tournament,
    Rank,
    Boltzmann,
    Truncation,
    /// Clearing selection: identifies niche winners ...
    /// Configure `niche_radius` via the selection configuration.
    Clearing,
}
```
Add after `Clearing`:
```rust
    /// Lexicase selection: shuffles test cases randomly per selection event
    /// and filters candidates case-by-case to the elite subset. Requires
    /// chromosomes implementing [`MultiCaseFitness`].
    /// Scalar `fitness()` is synced to mean case score after each selection call.
    Lexicase,
    /// Epsilon-lexicase selection: extends lexicase with a tolerance band so
    /// candidates within epsilon of the best on each case remain eligible.
    /// Set `epsilon` in `SelectionConfiguration` (0.0 = dynamic MAD default).
    /// Requires chromosomes implementing [`MultiCaseFitness`].
    EpsilonLexicase,
```
No data in either variant — enum stays `#[derive(Copy, Clone)]`.

---

### `src/configuration.rs` (config struct, extend)

**Analog:** `src/configuration.rs` existing `boltzmann_temperature` and `niche_radius` fields (lines 96-120)

**Existing field pattern** (lines 101-109):
```rust
/// Temperature parameter for Boltzmann selection. Controls selective pressure:
/// high values → uniform selection, low values → strong selective pressure.
/// Only used when `method` is `Selection::Boltzmann`. Default is `1.0`.
pub boltzmann_temperature: f64,
/// Niche radius for Clearing selection, measured in fitness space (`|f_a - f_b|`).
/// Within each niche (defined by the best individual in that radius), all other
/// individuals are cleared from the selection pool. Default is `0.1`.
/// Only used when `method` is `Selection::Clearing`.
pub niche_radius: f64,
```
Add after `niche_radius`:
```rust
/// Tolerance band for Epsilon-Lexicase selection.
/// Candidates within `epsilon` of the best on each test case remain eligible.
/// Set to `0.0` (default) to use the dynamic MAD (Median Absolute Deviation)
/// computed per case from the population at each generation.
/// Only used when `method` is `Selection::EpsilonLexicase`.
pub epsilon: f64,
```
In the `Default` impl (lines 111-119), add `epsilon: 0.0` alongside the existing defaults:
```rust
impl Default for SelectionConfiguration {
    fn default() -> Self {
        SelectionConfiguration {
            number_of_couples: 0,
            method: Selection::Tournament,
            boltzmann_temperature: 1.0,
            niche_radius: 0.1,
            epsilon: 0.0,    // 0.0 = dynamic MAD for EpsilonLexicase
        }
    }
}
```
The struct derives `#[derive(Copy, Clone, Debug, PartialEq)]` — `f64` is `Copy`, so this is non-breaking.

---

### `src/traits/configuration.rs` (config trait, extend)

**Analog:** `src/traits/configuration.rs` `SelectionConfig` trait (lines 14-24), specifically `with_niche_radius`

**Existing builder method pattern** (lines 19-23):
```rust
/// Sets the niche radius for [`Selection::Clearing`] (fitness-space distance).
///
/// Individuals within `niche_radius` of a niche winner are cleared from
/// the mating pool each generation. Default is `0.1`.
fn with_niche_radius(self, niche_radius: f64) -> Self;
```
Add to `SelectionConfig` trait after `with_niche_radius`:
```rust
/// Sets the epsilon tolerance for [`Selection::EpsilonLexicase`].
///
/// Candidates within `epsilon` of the best on each test case remain eligible.
/// Pass `0.0` (or omit) to use the dynamic MAD default. Default is `0.0`.
fn with_epsilon_lexicase(self, epsilon: f64) -> Self;
```
The `impl<U> SelectionConfig for Ga<U>` block in `src/engines/ga.rs` (lines 375-391) also needs the corresponding implementation — see ga.rs section below.

---

### `src/engines/ga.rs` (engine dispatch, extend)

**Analog:** `src/engines/ga.rs` (self — two locations: `SelectionConfig` impl and `run()` dispatch)

**Location 1: `SelectionConfig` impl** (lines 375-391):
```rust
impl<U> SelectionConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_number_of_couples(mut self, number_of_couples: usize) -> Self {
        self.configuration.selection_configuration.number_of_couples = number_of_couples;
        self
    }
    fn with_selection_method(mut self, selection_method: crate::operations::Selection) -> Self {
        self.configuration.selection_configuration.method = selection_method;
        self
    }
    fn with_niche_radius(mut self, niche_radius: f64) -> Self {
        self.configuration.selection_configuration.niche_radius = niche_radius;
        self
    }
}
```
Add `with_epsilon_lexicase` inside this same `impl` block (same pattern as `with_niche_radius`):
```rust
fn with_epsilon_lexicase(mut self, epsilon: f64) -> Self {
    self.configuration.selection_configuration.epsilon = epsilon;
    self
}
```

**Location 2: `run()` selection dispatch** (lines 1474-1478):
```rust
let parents = selection::factory(
    &self.population.chromosomes,
    self.configuration.selection_configuration,
    self.configuration.number_of_threads,
)?;
```
This becomes an if/else (D-07). The type-system challenge (Pitfall 1 from RESEARCH.md) means `factory_lexicase` requires `U: MultiCaseFitness` which `Ga<U: LinearChromosome>` does not provide. The resolution per D-07 + RESEARCH.md Option D is a separate `impl<U> Ga<U> where U: LinearChromosome + MultiCaseFitness` block providing a `run_lexicase()` helper, OR using a runtime `if/else` where only the non-lexicase branch is in the main `run()` and a `run()` override exists in the `MultiCaseFitness` impl block.

**Concrete approach:** Add a new impl block for the lexicase dispatch:
```rust
// In ga.rs, new impl block added below the existing `impl<U> Ga<U> where U: LinearChromosome + ...` block

impl<U> Ga<U>
where
    U: LinearChromosome
        + MultiCaseFitness
        + Send
        + Sync
        + 'static
        + Clone
        + Debug
        + mutation::ValueMutable
        + MaybeSerialize
        + MaybeDeserialize
        + OperatorCompat,
    U::Gene: 'static + Debug,
{
    /// Calls factory_lexicase instead of factory for Lexicase/EpsilonLexicase methods.
    /// Called from run() when the selection method is Lexicase or EpsilonLexicase.
    fn select_parents_lexicase(
        &mut self,
        config: SelectionConfiguration,
        threads: usize,
    ) -> Result<Vec<(usize, usize)>, GaError> {
        selection::factory_lexicase(
            &mut self.population.chromosomes,
            config,
            threads,
        )
    }
}
```
And in the existing `run()` method, the selection call becomes:
```rust
let parents = match self.configuration.selection_configuration.method {
    Selection::Lexicase | Selection::EpsilonLexicase => {
        // This branch is only reachable when U: MultiCaseFitness
        // (enforced by the separate impl block above)
        self.select_parents_lexicase(
            self.configuration.selection_configuration,
            self.configuration.number_of_threads,
        )?
    }
    _ => selection::factory(
        &self.population.chromosomes,
        self.configuration.selection_configuration,
        self.configuration.number_of_threads,
    )?,
};
```
**Note for planner:** The type system constraint (RESEARCH.md Pitfall 1) means this approach only compiles if `Ga<U>` is used with `U: MultiCaseFitness`. The standard `run()` in the non-`MultiCaseFitness` impl block must NOT call `factory_lexicase` — it must instead return an error or panic for Lexicase variants. The Lexicase arms in the standard `run()` `match` block should delegate to `factory()` which now returns `GaError::ConfigurationError` for those variants (D-06).

---

### `src/lib.rs` (crate re-export, extend)

**Analog:** `src/lib.rs` lines 333-335 (existing `pub use traits::*` pattern):
```rust
pub use traits::LinearChromosome;
pub use traits::OperatorCompat;
pub use traits::Strategy;
```
Add after these lines:
```rust
pub use traits::MultiCaseFitness;
```

---

### `src/traits.rs` (module re-export, extend)

**Analog:** `src/traits.rs` lines 38-62 (existing `pub mod` + `pub use` pattern):
```rust
pub mod chromosome;
// ... other mods ...
pub mod strategy;
pub use strategy::Strategy;

pub use chromosome::ChromosomeT;
// ... other pub uses ...
```
Add `pub mod multi_case_fitness;` to the `pub mod` block (alphabetically after `linear_chromosome`):
```rust
pub mod multi_case_fitness;
```
Add to the `pub use` block:
```rust
pub use multi_case_fitness::MultiCaseFitness;
```

---

### `tests/lexicase.rs` (integration test, CRUD)

**Note from RESEARCH.md:** Test files should go in `tests/operations/test_selection_lexicase.rs` (not a root-level `tests/lexicase.rs`). See RESEARCH.md Wave 0 Gaps and project convention (CLAUDE.md: "All unit tests must be in `tests/`"). The CONTEXT.md listed `tests/lexicase.rs` but the established project structure places operation tests under `tests/operations/`.

**Analog:** `tests/operations/test_selection_clearing.rs` (full file, 162 lines)

**File header and import pattern** (lines 1-6):
```rust
//! Tests for the Clearing selection operator.

#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{fitness::FitnessFnWrapper, operations::selection::clearing::clearing_selection};
```
Lexicase analog:
```rust
//! Tests for the Lexicase and Epsilon-Lexicase selection operators.

#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper,
    operations::selection::{lexicase::lexicase_selection, lexicase::epsilon_lexicase_selection},
    traits::MultiCaseFitness,
};
```

**Test fixture pattern** (`clearing.rs` lines 7-26):
```rust
fn make_chromosome(fitness: f64, dna: Vec<Gene>) -> Chromosome { ... }
fn gene(id: i32) -> Gene { Gene { id } }
fn pop_distinct(fitnesses: &[f64]) -> Vec<Chromosome> {
    fitnesses.iter().map(|&f| make_chromosome(f, vec![gene(0)])).collect()
}
```
Lexicase needs a `MultiCaseChromosome` fixture — extends `Chromosome` with `case_scores: Vec<f64>` and `impl MultiCaseFitness`. This fixture is new (no existing analog in `tests/structures.rs`) — add it inline in the test file or add to `tests/structures.rs`.

**Test case pattern** (`clearing.rs` lines 30-42):
```rust
#[test]
fn test_clearing_returns_pairs_of_valid_indices() {
    let pop = pop_distinct(&[10.0, 9.0, 5.0, 4.5, 0.0, 0.3]);
    let pairs = clearing_selection(&pop, 0.5, 3);

    for (a, b) in &pairs {
        assert!(*a < pop.len(), "Index {} out of bounds", a);
        assert!(*b < pop.len(), "Index {} out of bounds", b);
        assert_ne!(a, b, "Self-pairing not allowed");
    }
}
```

**Factory dispatch test pattern** (`clearing.rs` lines 119-150):
```rust
#[test]
fn test_clearing_via_factory_respects_niche_radius() {
    use genetic_algorithms::{
        configuration::SelectionConfiguration, operations::selection, operations::Selection,
    };
    let config = SelectionConfiguration {
        method: Selection::Clearing,
        number_of_couples: 3,
        niche_radius: 0.1,
        ..Default::default()
    };
    let result = selection::factory(&pop, config, 1);
    assert!(result.is_ok());
    ...
}
```
Lexicase analog uses `selection::factory_lexicase` and `SelectionConfiguration { method: Selection::Lexicase, epsilon: 0.0, ..Default::default() }`.

---

## Shared Patterns

### Logging Target
**Source:** `src/operations/selection/clearing.rs` (line 34) and `src/operations/selection/tournament.rs` (line 39)
**Apply to:** `src/operations/selection/lexicase.rs` — all `debug!` and `trace!` calls
```rust
// Entry:
debug!(target="selection_events", method="lexicase"; "...", ...);
// Per-individual trace:
trace!(target="selection_events", method="lexicase"; "Selected individual {}", idx);
// Exit:
debug!(target="selection_events", method="lexicase"; "Lexicase selection finished: {} pairs", mating.len());
```

### RNG Usage
**Source:** `src/operations/selection/clearing.rs` (line 86) and `src/operations/selection/tournament.rs` (line 54)
**Apply to:** `src/operations/selection/lexicase.rs`
```rust
let mut rng = crate::rng::make_rng();
// Then:
use rand::Rng;
rng.random_range(0..n)
rng.random_range(0..=i)  // for Fisher-Yates
```

### NaN Guard
**Source:** `src/operations/selection.rs` (lines 89-97)
**Apply to:** `src/operations/selection.rs` `factory_lexicase()`
```rust
for (i, chromosome) in chromosomes.iter().enumerate() {
    if chromosome.fitness().is_nan() {
        return Err(GaError::SelectionError(format!(
            "Chromosome at index {} has NaN fitness. All chromosomes must have valid fitness before selection.",
            i
        )));
    }
}
```
Extend this pattern to check `case_fitness` scores for NaN as well.

### SelectionError Pattern
**Source:** `src/operations/selection.rs` (lines 82-87)
**Apply to:** `src/operations/selection.rs` `factory_lexicase()`
```rust
return Err(GaError::SelectionError(format!(
    "Population size {} is too small for selection (minimum 2)",
    chromosomes.len()
)));
```

### `Vec::with_capacity` Pre-allocation
**Source:** `src/operations/selection/clearing.rs` (line 87)
**Apply to:** `src/operations/selection/lexicase.rs`
```rust
let mut mating = Vec::with_capacity(number_of_couples);
```

### WASM Compatibility Gate
**Source:** `src/engines/ga.rs` (lines 1460-1473) — `Instant::now()` gate pattern
**Source:** `src/operations/selection/tournament.rs` — `into_par_iter()` (lexicase must NOT use this)
**Apply to:** `src/operations/selection/lexicase.rs`
- Never use `.par_iter()` or `.into_par_iter()` — lexicase inner loop is inherently sequential
- Add comment: `// WASM: intentionally not par_iter — lexicase inner loop is sequential by design`

### serde Derive Gate
**Source:** `src/operations.rs` (line 44) and `src/configuration.rs` (line 96-97)
**Apply to:** New `Lexicase`/`EpsilonLexicase` variants automatically via the existing `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` on `Selection` enum and `SelectionConfiguration` struct — no separate action needed.

---

## No Analog Found

All files in scope have close analogs in the codebase. No files require falling back to RESEARCH.md patterns exclusively.

| File | Note |
|------|------|
| `src/traits/multi_case_fitness.rs` | `chromosome.rs` is a role-match (same trait layer), not an exact match; the two-method shape is novel but trivial |
| `tests/lexicase.rs` / `tests/operations/test_selection_lexicase.rs` | `MultiCaseChromosome` test fixture is new — no existing analog with `case_scores: Vec<f64>` field; must be written from scratch following `structures.rs` `Chromosome` pattern |

---

## Metadata

**Analog search scope:** `src/operations/selection/`, `src/traits/`, `src/configuration.rs`, `src/operations.rs`, `src/engines/ga.rs`, `src/lib.rs`, `src/traits.rs`, `tests/operations/`, `tests/structures.rs`
**Files scanned:** 13 files read directly + grep over full codebase
**Pattern extraction date:** 2026-05-23
