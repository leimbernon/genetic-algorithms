# genetic_algorithms (v2.0.0)

[![Rust Unit Tests](https://github.com/leimbernon/rust_genetic_algorithms/actions/workflows/rust-unit-tests.yml/badge.svg)](https://github.com/leimbernon/rust_genetic_algorithms/actions/workflows/rust-unit-tests.yml)

Modular and concurrent Genetic Algorithms (GA) library for Rust featuring:
- Clear abstractions (traits for genes, chromosomes, and configuration).
- Composable operators (selection, crossover, mutation, survivor).
- Multi-threaded execution (fitness evaluation, reproduction, mutation in parallel).
- Adaptive GA mode (dynamic crossover and mutation probabilities based on population performance).
- `Cow` for minimizing unnecessary DNA copies.

## Table of Contents
- [Documentation](#documentation)
- [Status & Key Changes 2.0.0](#status--key-changes-200)
- [Features](#features)
  - [Traits](#traits)
  - [Included Genotypes & Chromosomes](#included-genotypes--chromosomes)
  - [Initializers](#initializers)
  - [Operators](#operators)
  - [GA Configuration](#ga-configuration)
  - [Adaptive GA](#adaptive-ga)
  - [Multithreading & Performance](#multithreading--performance)
- [Quick Example](#quick-example)
- [Full Example (Range)](#full-example-range)
- [Usage](#usage)
- [Roadmap / Notes](#roadmap--notes)
- [License](#license)

## Documentation
Latest published docs: [docs.rs/genetic_algorithms](https://docs.rs/genetic_algorithms/latest/genetic_algorithms)

## Status & Key Changes 2.0.0
Main differences vs 1.x:
- Crate version: 2.0.0 (update your `Cargo.toml`).
- `GeneT::get_id()` now returns `i32` directly.
- `ChromosomeT::set_dna` now uses `Cow<'a, [Gene]>` to avoid redundant copies.
- Added `Range<T>` genotype alongside `Binary` (supports numeric ranges).
- Adaptive probability helpers: `aga_probability` for crossover & mutation.
- Expanded configuration: `FixedFitness`, logging levels (`Off`..`Trace`), configurable `number_of_threads`.
- Removed obsolete examples using old signatures (`set_dna(&[Gene])`).
- Internal benchmarks use Criterion 0.7 (pprof integration removed due to version conflict).

## Features
### Traits
- `GeneT`:
  - Requires: `Default + Clone + Send + Sync`.
  - Key methods: `new()`, `get_id() -> i32`, `set_id(i32)`.
- `ChromosomeT`:
  - Associated: `type Gene: GeneT`.
  - Supports: `get_dna()`, `set_dna(Cow<[Gene]>)`, `set_gene(idx, gene)`, `set_fitness_fn(...)`, `calculate_fitness()`, `get_fitness()`, `set_fitness(f64)`, `get_age()`, `set_age(i32)`.
  - Derived metric: `get_fitness_distance(&fitness_target)`.
- `ConfigurationT`: builder-style API for `Ga` / `GaConfiguration`.

### Included Genotypes & Chromosomes
- `genotypes::Binary` (boolean gene).
- `genotypes::Range<T>` (values constrained to one or more `(min,max)` intervals).
- `chromosomes::Range<T>` (chromosome built from `Range<T>` genes).
- (Custom chromosomes can be added by implementing `ChromosomeT`).

### Initializers
- `initializers::binary_random_initialization`.
- `initializers::range_random_initialization`.
- `initializers::generic_random_initialization` (takes allele slice, optional unique IDs).
- `initializers::generic_random_initialization_without_repetitions` (no allele repetition).

### Operators
- Selection: `Random`, `RouletteWheel`, `StochasticUniversalSampling`, `Tournament`.
- Crossover: `Cycle`, `MultiPoint`, `Uniform`.
- Mutation: `Swap`, `Inversion`, `Scramble`.
- Survivor: `Fitness` (keep best), `Age` (prefer younger / age-based pruning).

### GA Configuration
`GaConfiguration` (or the `Ga` builder) exposes:
- Limits: `problem_solving` (`Minimization | Maximization | FixedFitness`), `max_generations`, `fitness_target`, `population_size`, `genes_per_chromosome`, `needs_unique_ids`, `alleles_can_be_repeated`.
- Selection: `number_of_couples`, `method`.
- Crossover: `number_of_points` (MultiPoint), `probability_min` / `probability_max` (required in adaptive mode), `method`.
- Mutation: `probability_min` / `probability_max` (required in adaptive mode), `method`.
- Survivor: `survivor`.
- Infra: `adaptive_ga`, `number_of_threads`, `log_level`.
- Progress (present but not yet wired): `save_progress_configuration` (future/experimental).

### Adaptive GA
When `adaptive_ga = true`:
- Crossover & mutation probabilities are recomputed per parent pair using relative fitness (`f_max`, `f_avg`).
- You must set both `probability_min` and `probability_max` for crossover and mutation.
When `adaptive_ga = false`:
- If `probability_max` is absent, defaults to 1.0 (operator always applied).

### Multithreading & Performance
- `with_threads(n)` divides selection, crossover, and mutation across threads.
- Fitness evaluation is parallelized each generation.
- `Cow` prevents needless cloning of DNA vectors.

## Quick Example
Minimal GA using `Range<i32>` chromosomes targeting fitness 0 (minimization):
```rust
use genetic_algorithms::ga::Ga;
use genetic_algorithms::traits::ConfigurationT;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::operations::{Selection, Crossover, Mutation, Survivor};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::chromosomes::Range as RangeChromosome; // Chromosome type

fn fitness_fn(dna: &[RangeGene<i32>]) -> f64 { // Replace with domain logic
    dna.iter().map(|g| g.get_value() as f64).sum() // Simple example
}

let alleles = vec![RangeGene::new(0, vec![(0, 7)], 0)];
let alleles_clone = alleles.clone();
let mut ga = Ga::<RangeChromosome<i32>>::new();
let _population = ga
    .with_genes_per_chromosome(8)
    .with_population_size(100)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(fitness_fn)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Swap)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .with_fitness_target(0.0)
    .with_threads(4)
    .run();
```

## Full Example (Range)
Includes adaptive GA and probabilities:
```rust
use genetic_algorithms::{ga::Ga, traits::ConfigurationT};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::operations::{Selection, Crossover, Mutation, Survivor};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::initializers::range_random_initialization;

fn fitness_fn(dna: &[RangeGene<i32>]) -> f64 {
    // Penalize greater values: minimization goal
    dna.iter().map(|g| g.get_value() as f64).sum()
}

let alleles = vec![RangeGene::new(0, vec![(0, 50)], 0)];
let alleles_clone = alleles.clone();
let mut ga = Ga::<RangeChromosome<i32>>::new();
let population = ga
    .with_adaptive_ga(true)
    .with_genes_per_chromosome(16)
    .with_population_size(200)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(fitness_fn)
    .with_selection_method(Selection::StochasticUniversalSampling)
    .with_crossover_method(Crossover::MultiPoint)
    .with_crossover_number_of_points(3)
    .with_crossover_probability_min(0.4)
    .with_crossover_probability_max(0.9)
    .with_mutation_method(Mutation::Inversion)
    .with_mutation_probability_min(0.05)
    .with_mutation_probability_max(0.3)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(2000)
    .with_fitness_target(0.0)
    .with_threads(8)
    .with_logs(genetic_algorithms::configuration::LogLevel::Info)
    .run();

println!("Best fitness: {}", population.best_chromosome.as_ref().unwrap().get_fitness());
```

## Usage
Add to your `Cargo.toml`:
```toml
[dependencies]
genetic_algorithms = "2.0.0"
```

## Roadmap / Notes
- `save_progress_configuration`: fields present but not wired into the main loop yet (future: periodic population / best chromosome persistence).
- Optional flamegraph profiling integration removed to avoid Criterion version conflicts.
- For heavy fitness functions consider external profiling tools.

## License
Apache-2.0. See LICENSE file.
