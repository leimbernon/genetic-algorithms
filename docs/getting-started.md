# Getting Started

> Quick-start guide for using the genetic algorithms library.

## Overview

This library provides a modular and efficient framework for building and running Genetic Algorithms (GAs) in Rust. It is designed to be flexible, supporting a variety of gene and chromosome types, and composable operator pipelines for selection, crossover, mutation, and survivor selection. The library is suitable for both beginners and advanced users who want to solve optimization and search problems using evolutionary computation.

Genetic Algorithms are a class of optimization algorithms inspired by natural selection. They operate on a population of candidate solutions, iteratively applying selection, crossover, and mutation to evolve better solutions over generations. This crate abstracts the core GA concepts into reusable traits and types, making it easy to configure and extend for a wide range of problems.

The library emphasizes performance through efficient data handling (using `Cow` to avoid unnecessary cloning), parallel fitness evaluation, and pre-allocated operator pipelines. It is suitable for problems such as the knapsack problem, n-queens, scheduling, and more.

## Key Concepts

The following table summarizes the core abstractions and configuration options relevant to getting started:

| Concept / Field                | Type / Enum                       | Description                                                                                 |
|-------------------------------|-----------------------------------|---------------------------------------------------------------------------------------------|
| `GeneT`                       | Trait                             | Trait for gene types (e.g., binary, range).                                                 |
| `ChromosomeT`                 | Trait                             | Trait for chromosomes (sequences of genes, with fitness and age).                           |
| `Ga<U>`                       | Struct                            | Main orchestrator for running a genetic algorithm.                                          |
| `ProblemSolving`              | Enum (`Minimization`, `Maximization`, `FixedFitness`) | Defines the optimization goal.                                                              |
| `Selection`                   | Enum                              | Selection operator (e.g., tournament, roulette, random).                                    |
| `Crossover`                   | Enum                              | Crossover operator (e.g., uniform, multipoint, cycle, SBX, BLX-α).                          |
| `Mutation`                    | Enum                              | Mutation operator (e.g., swap, scramble, inversion, value, creep, gaussian).                |
| `Survivor`                    | Enum                              | Survivor selection method (e.g., fitness, age-based).                                       |
| `GaConfiguration`             | Struct                            | Compound configuration for all GA parameters.                                               |
| `StoppingCriteria`            | Struct                            | Compound stopping criteria (stagnation, convergence, time limit).                           |
| `initialization_fn`           | Fn                                | Function to generate initial population DNA.                                                |
| `fitness_fn`                  | Fn                                | User-defined function to evaluate chromosome fitness.                                       |
| `elitism_count`               | usize                             | Number of top individuals preserved between generations.                                    |
| `alleles`                     | Vec<Gene>                         | Template for gene values used in initialization.                                            |

### Example: Main Configuration Fields

| Field                      | Type                | Description                                              |
|----------------------------|---------------------|----------------------------------------------------------|
| `population_size`          | `i32`               | Number of chromosomes in the population.                 |
| `genes_per_chromosome`     | `i32`               | Number of genes in each chromosome.                      |
| `max_generations`          | `i32`               | Maximum number of generations to run.                    |
| `fitness_target`           | `Option<f64>`       | Target fitness value to stop early (if reached).         |
| `selection_method`         | `Selection`         | Parent selection operator.                               |
| `crossover_method`         | `Crossover`         | Crossover operator.                                      |
| `mutation_method`          | `Mutation`          | Mutation operator.                                       |
| `survivor_method`          | `Survivor`          | Survivor selection operator.                             |
| `elitism_count`            | `usize`             | Number of elite chromosomes preserved each generation.   |
| `adaptive_ga`              | `bool`              | Enable adaptive parameter tuning.                        |
| `number_of_threads`        | `i32`               | Number of threads for parallel evaluation.               |
| `log_level`                | `LogLevel`          | Logging verbosity.                                       |
| `stopping_criteria`        | `StoppingCriteria`  | Additional stopping conditions.                          |

### Advanced Operator Parameters

| Field                | Type         | Description                                                      |
|----------------------|--------------|------------------------------------------------------------------|
| `sbx_eta`            | `Option<f64>`| Distribution index for SBX crossover (typical: 2–20, default: 2) |
| `blend_alpha`        | `Option<f64>`| Alpha parameter for BLX-α crossover (typical: 0.5)               |
| `mutation_step`      | `Option<f64>`| Step size for Creep mutation (default: 1.0)                      |
| `mutation_sigma`     | `Option<f64>`| Std. deviation for Gaussian mutation (default: 1.0)              |

## Usage

### Basic Example

The following example demonstrates how to set up and run a simple GA to solve the n-queens problem using range-based genes and chromosomes.

```rust
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::ConfigurationT;

const N: i32 = 8;

// Fitness function: counts the number of non-attacking pairs of queens
fn fitness_fn(dna: &[RangeGene<i32>]) -> f64 {
    let mut non_attacking = 0;
    for i in 0..dna.len() {
        for j in (i + 1)..dna.len() {
            let xi = dna[i].value();
            let xj = dna[j].value();
            if xi != xj && (xi - xj).abs() != (i as i32 - j as i32).abs() {
                non_attacking += 1;
            }
        }
    }
    non_attacking as f64
}

fn main() {
    let alleles = vec![RangeGene::new(0, vec![(0, N - 1)], 0)];
    let alleles_clone = alleles.clone();
    let mut ga = Ga::<RangeChromosome<i32>>::new();
    let _population = ga
        .with_genes_per_chromosome(N)
        .with_population_size(100)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(500)
        .with_fitness_target((N * (N - 1) / 2) as f64)
        .run()
        .expect("GA run failed");

    // Print best solution
    let best = _population.best().unwrap();
    println!("Best solution: {:?}", best.dna());
    println!("Fitness: {}", best.fitness());
}
```

### Advanced Example

This example demonstrates a more advanced configuration for a binary-encoded knapsack problem, including elitism, custom stopping criteria, and parallel evaluation.

```rust
use genetic_algorithms::configuration::{ProblemSolving, StoppingCriteria};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::ConfigurationT;

const N_ITEMS: i32 = 50;
const KNAPSACK_CAPACITY: i32 = 100;

fn fitness_fn(dna: &[BinaryGene]) -> f64 {
    let weights = vec![2; N_ITEMS as usize];
    let values = vec![1; N_ITEMS as usize];
    let mut total_weight = 0;
    let mut total_value = 0;
    for (gene, (&w, &v)) in dna.iter().zip(weights.iter().zip(values.iter())) {
        if gene.value() {
            total_weight += w;
            total_value += v;
        }
    }
    if total_weight > KNAPSACK_CAPACITY {
        0.0 // Penalize overweight solutions
    } else {
        total_value as f64
    }
}

fn main() {
    let alleles = vec![BinaryGene::new(false)];
    let alleles_clone = alleles.clone();
    let stopping_criteria = StoppingCriteria {
        stagnation_generations: Some(100),
        convergence_threshold: Some(0.01),
        max_duration_secs: Some(30.0),
    };
    let mut ga = Ga::<BinaryChromosome>::new();
    let _population = ga
        .with_genes_per_chromosome(N_ITEMS)
        .with_population_size(200)
        .with_initialization_fn(move |genes_per_chromosome, _, _| {
            binary_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
        })
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Roulette)
        .with_crossover_method(Crossover::Multipoint)
        .with_mutation_method(Mutation::Value)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(1000)
        .with_fitness_target(N_ITEMS as f64)
        .with_elitism(5)
        .with_threads(4)
        .with_logs(genetic_algorithms::configuration::LogLevel::Info)
        .with_stopping_criteria(stopping_criteria)
        .run()
        .expect("GA run failed");

    let best = _population.best().unwrap();
    println!("Best knapsack: {:?}", best.dna());
    println!("Fitness: {}", best.fitness());
}
```

## API Reference

### `Ga<U>`

Main orchestrator struct for running a genetic algorithm.

**Fields / Methods:**

| Name                       | Signature                                                                 | Description                                                                                      |
|----------------------------|---------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------|
| `new`                      | `fn new() -> Self`                                                        | Creates a new GA instance with default configuration.                                            |
| `with_alleles`             | `fn with_alleles(&mut self, alleles: Vec<U::Gene>) -> &mut Self`          | Sets the alleles template for initialization.                                                    |
| `with_population`          | `fn with_population(&mut self, population: Population<U>) -> &mut Self`   | Sets the initial population.                                                                     |
| `with_fitness_fn`          | `fn with_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self`            | Sets the user-defined fitness function.                                                          |
| `with_initialization_fn`   | `fn with_initialization_fn<F>(&mut self, initialization_fn: F) -> &mut Self` | Sets the initialization function for population DNA.                                             |
| `initialization`           | `fn initialization(&mut self) -> Result<&mut Self, GaError>`              | Initializes the population using the configured initialization function.                         |
| `run`                      | `fn run(&mut self) -> Result<&Population<U>, GaError>`                    | Runs the GA until a stopping condition is met.                                                   |
| `run_with_callback`        | `fn run_with_callback<F>(&mut self, callback: Option<F>, generations_to_callback: i32) -> Result<&Population<U>, GaError>` | Runs the GA, invoking a callback every N generations.                                            |

### `ProblemSolving`

Defines the optimization goal.

| Variant         | Description                                 |
|-----------------|---------------------------------------------|
| `Minimization`  | Minimize the fitness function.              |
| `Maximization`  | Maximize the fitness function.              |
| `FixedFitness`  | Stop when a fixed fitness value is reached. |

### `Selection`, `Crossover`, `Mutation`, `Survivor`

Operator enums for configuring the GA.

| Enum      | Variants (examples)                                  | Description                                  |
|-----------|-----------------------------------------------------|----------------------------------------------|
| Selection | `Tournament`, `Roulette`, `Random`                  | Parent selection strategies.                 |
| Crossover | `Uniform`, `Multipoint`, `Cycle`, `SBX`, `BLXAlpha` | Chromosome recombination strategies.         |
| Mutation  | `Swap`, `Scramble`, `Inversion`, `Value`, `Creep`, `Gaussian` | Gene mutation strategies.          |
| Survivor  | `Fitness`, `Age`, `Random`                          | Survivor selection strategies.               |

### `GaConfiguration`

Compound configuration struct for all GA parameters.

| Field                        | Type                  | Description                                            |
|------------------------------|-----------------------|--------------------------------------------------------|
| `adaptive_ga`                | `bool`                | Enable adaptive parameter tuning.                      |
| `number_of_threads`          | `i32`                 | Number of threads for parallel evaluation.             |
| `limit_configuration`        | `LimitConfiguration`  | Limits: population size, generations, fitness target.  |
| `selection_configuration`    | `SelectionConfiguration` | Selection operator and couples.                    |
| `crossover_configuration`    | `CrossoverConfiguration` | Crossover operator and parameters.                  |
| `mutation_configuration`     | `MutationConfiguration`  | Mutation operator and parameters.                   |
| `survivor`                  | `Survivor`            | Survivor selection method.                            |
| `log_level`                 | `LogLevel`            | Logging verbosity.                                    |
| `save_progress_configuration`| `SaveProgressConfiguration` | Progress saving options.                        |
| `elitism_count`             | `usize`               | Number of elite chromosomes preserved.                |
| `stopping_criteria`         | `StoppingCriteria`     | Additional stopping conditions.                       |

### `StoppingCriteria`

Compound stopping criteria struct.

| Field                   | Type           | Description                                              |
|-------------------------|----------------|----------------------------------------------------------|
| `stagnation_generations`| `Option<i32>`  | Stop after N generations without fitness improvement.     |
| `convergence_threshold` | `Option<f64>`  | Stop when fitness std. deviation drops below threshold.   |
| `max_duration_secs`     | `Option<f64>`  | Stop after specified elapsed time (seconds).              |

## Related

- [Configuration](configuration.md)
- [Chromosomes](chromosomes.md)
- [Genotypes](genotypes.md)
- [Operators: Selection](operators/selection.md)
- [Operators: Crossover](operators/crossover.md)
- [Operators: Mutation](operators/mutation.md)
- [Operators: Survivor](operators/survivor.md)
- [Fitness](fitness.md)
- [Population](population.md)
- [Traits](traits.md)
- [Validators](validators.md)
- [End-to-End Examples](examples.md)
- [API Reference](api-reference.md)
- [Source: `src/lib.rs`](https://github.com/leimbernon/rust_genetic_algorithms/blob/main/src/lib.rs)
- [Source: `src/ga.rs`](https://github.com/leimbernon/rust_genetic_algorithms/blob/main/src/ga.rs)
- [Source: `src/configuration.rs`](https://github.com/leimbernon/rust_genetic_algorithms/blob/main/src/configuration.rs)
- [Example: `examples/knapsack_binary.rs`](https://github.com/leimbernon/rust_genetic_algorithms/blob/main/examples/knapsack_binary.rs)
- [Example: `examples/nqueens_range.rs`](https://github.com/leimbernon/rust_genetic_algorithms/blob/main/examples/nqueens_range.rs)
