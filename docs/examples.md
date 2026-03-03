# End-to-End Examples

> Complete, runnable examples demonstrating genetic algorithms for common optimization problems.

## Overview

This module provides comprehensive, real-world examples using the genetic algorithm (GA) library to solve classic combinatorial optimization problems. These examples are designed to guide new users through the process of modeling a problem, configuring the GA, and interpreting results. Each example demonstrates the full workflow: defining chromosomes and fitness functions, configuring operators, initializing populations, running the algorithm, and handling results and errors.

The examples cover two canonical problems: the **Knapsack Problem** (using binary chromosomes) and the **N-Queens Problem** (using range chromosomes). Both showcase how to leverage the library's flexible configuration system, custom initialization and fitness functions, and logging options. Error handling and termination causes are explicitly demonstrated to help users build robust applications.

These examples serve as a practical starting point for adapting the library to your own optimization tasks, illustrating how core abstractions fit together and how advanced features (custom operators, logging, callbacks) can be integrated.

## Key Concepts

The following concepts are central to the examples:

| Concept                | Type/Module            | Description                                              |
|------------------------|------------------------|----------------------------------------------------------|
| Chromosome             | `BinaryChromosome`, `RangeChromosome` | Represents a candidate solution (bit vector or integer vector) |
| Genotype               | `Genotype`             | Defines the chromosome type and allele range             |
| Genetic Algorithm      | `Ga`                   | Main orchestrator for GA runs                            |
| Fitness Function       | `FitnessFn`            | Evaluates solution quality                               |
| Initialization Function| `InitializationFn`     | Customizes population initialization                     |
| Configuration          | `GaConfiguration`      | Controls operators, limits, logging, and more            |
| Population             | `Population`           | Collection of candidate solutions                        |
| Termination Cause      | `TerminationCause`     | Indicates why a GA run stopped                           |
| Logging                | `LogLevel`             | Controls verbosity of GA output                          |
| Error Handling         | `GaError`              | Error type for GA operations                             |

### Example Configuration Fields

| Field                   | Type                  | Description                                              |
|-------------------------|-----------------------|----------------------------------------------------------|
| `population_size`       | `usize`               | Number of chromosomes in each generation                 |
| `generation_limit`      | `usize`               | Maximum number of generations to run                     |
| `fitness_target`        | `f64`                 | Stop when best fitness reaches this value                |
| `log_level`             | `LogLevel`            | Controls logging verbosity                               |
| `alleles`               | `Vec<Gene>`           | Possible gene values for initialization                  |
| `initialization_fn`     | `InitializationFn`    | Custom population initialization logic                   |
| `fitness_fn`            | `FitnessFn`           | Function to evaluate chromosome fitness                  |

## Usage

### Basic Example

**Knapsack Problem (Binary Chromosome)**

This example solves the classic knapsack problem: select items to maximize value without exceeding weight.

```rust
use ga::{
    Ga, GaConfiguration, Population, BinaryChromosome, TerminationCause, LogLevel,
};

const ITEM_WEIGHTS: [u32; 5] = [2, 3, 4, 5, 9];
const ITEM_VALUES: [u32; 5] = [3, 4, 8, 8, 10];
const KNAPSACK_CAPACITY: u32 = 15;

fn knapsack_fitness(chromosome: &BinaryChromosome) -> f64 {
    let mut total_weight = 0;
    let mut total_value = 0;
    for (gene, (&weight, &value)) in chromosome.genes().iter().zip(ITEM_WEIGHTS.iter().zip(ITEM_VALUES.iter())) {
        if *gene {
            total_weight += weight;
            total_value += value;
        }
    }
    if total_weight > KNAPSACK_CAPACITY {
        0.0
    } else {
        total_value as f64
    }
}

fn main() -> Result<(), ga::GaError> {
    let mut ga = Ga::<BinaryChromosome>::default();
    ga.configuration.population_size = 50;
    ga.configuration.generation_limit = 100;
    ga.configuration.log_level = LogLevel::Info;
    ga.with_fitness_fn(knapsack_fitness);

    ga.initialization()?; // Randomly initialize population
    let population = ga.run()?;

    let best = population.best_chromosome().unwrap();
    println!("Best solution: {:?}", best.genes());
    println!("Best value: {}", knapsack_fitness(best));
    println!("Termination cause: {:?}", ga.termination_cause);

    Ok(())
}
```

### Advanced Example

**N-Queens Problem (Range Chromosome, Custom Initialization, Logging, Error Handling)**

This example finds a solution to the N-Queens problem using integer chromosomes, custom initialization, and advanced configuration.

```rust
use ga::{
    Ga, GaConfiguration, Population, RangeChromosome, TerminationCause, LogLevel,
};

const N: usize = 8; // Number of queens

// Fitness: count non-attacking pairs of queens
fn nqueens_fitness(chromosome: &RangeChromosome) -> f64 {
    let positions = chromosome.genes();
    let mut non_attacking = 0;
    for i in 0..N {
        for j in (i + 1)..N {
            if positions[i] != positions[j] // not same column
                && (positions[i] as isize - positions[j] as isize).abs() != (i as isize - j as isize).abs() // not same diagonal
            {
                non_attacking += 1;
            }
        }
    }
    non_attacking as f64
}

// Custom initialization: random permutation (each queen in a unique column)
fn permutation_init(alleles: &[u8], rng: &mut rand::rngs::ThreadRng) -> Vec<u8> {
    let mut genes = alleles.to_vec();
    genes.shuffle(rng);
    genes
}

fn main() -> Result<(), ga::GaError> {
    let alleles: Vec<u8> = (0..N as u8).collect();

    let mut ga = Ga::<RangeChromosome>::default();
    ga.configuration.population_size = 100;
    ga.configuration.generation_limit = 500;
    ga.configuration.log_level = LogLevel::Debug;
    ga.configuration.fitness_target = ((N * (N - 1)) / 2) as f64; // All pairs non-attacking

    ga.with_alleles(alleles.clone());
    ga.with_initialization_fn(permutation_init);
    ga.with_fitness_fn(nqueens_fitness);

    ga.initialization()?; // Custom population initialization

    let population = ga.run()?;
    let best = population.best_chromosome().unwrap();

    println!("Best solution: {:?}", best.genes());
    println!("Best fitness: {}", nqueens_fitness(best));
    println!("Termination cause: {:?}", ga.termination_cause);

    // Interpret result
    if ga.termination_cause == TerminationCause::FitnessTargetReached {
        println!("Found valid N-Queens solution!");
    } else {
        println!("Did not find valid solution; best fitness: {}", nqueens_fitness(best));
    }

    Ok(())
}
```

## API Reference

### `Ga<U>`

Generic genetic algorithm orchestrator.

**Fields:**

| Name                | Type                             | Description                                              |
|---------------------|----------------------------------|----------------------------------------------------------|
| `configuration`     | `GaConfiguration`                | GA configuration options                                 |
| `alleles`           | `Vec<U::Gene>`                   | Alleles for initialization                              |
| `population`        | `Population<U>`                  | Current population                                       |
| `termination_cause` | `TerminationCause`               | Reason for termination                                   |
| `initialization_fn` | `Option<Arc<InitializationFn>>`  | Custom initialization function                           |
| `fitness_fn`        | `Option<Arc<FitnessFn>>`         | Fitness evaluation function                              |

**Methods:**

| Name                      | Signature                                                           | Description                                              |
|---------------------------|---------------------------------------------------------------------|----------------------------------------------------------|
| `with_alleles`            | `fn with_alleles(&mut self, alleles: Vec<U::Gene>) -> &mut Self`    | Sets alleles for chromosome initialization               |
| `with_population`         | `fn with_population(&mut self, population: Population<U>) -> &mut Self` | Sets initial population                              |
| `with_fitness_fn`         | `fn with_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self`      | Sets fitness function                                    |
| `with_initialization_fn`  | `fn with_initialization_fn<F>(&mut self, initialization_fn: F) -> &mut Self` | Sets custom initialization function         |
| `initialization`          | `fn initialization(&mut self) -> Result<&mut Self, GaError>`        | Initializes population                                   |
| `run`                     | `fn run(&mut self) -> Result<&Population<U>, GaError>`              | Runs GA to completion                                    |
| `run_with_callback`       | `fn run_with_callback<F>(&mut self, callback: Option<F>, generations_to_callback: usize) -> Result<&Population<U>, GaError>` | Runs GA with callback |

### `TerminationCause`

Enumerates reasons for GA termination.

| Variant                  | Description                                              |
|--------------------------|----------------------------------------------------------|
| `GenerationLimitReached` | Maximum generations reached                              |
| `FitnessTargetReached`   | Desired fitness achieved                                 |
| `StagnationReached`      | No improvement for N generations                         |
| `ConvergenceReached`     | Population converged                                     |
| `TimeLimitReached`       | Time limit exceeded                                      |
| `NotTerminated`          | Not yet terminated                                       |

### `GaConfiguration`

Controls GA behavior.

| Field             | Type         | Description                          |
|-------------------|--------------|--------------------------------------|
| `population_size` | `usize`      | Number of chromosomes per generation |
| `generation_limit`| `usize`      | Max generations                      |
| `fitness_target`  | `f64`        | Target fitness to stop               |
| `log_level`       | `LogLevel`   | Logging verbosity                    |

### `LogLevel`

Controls logging output.

| Variant   | Description                       |
|-----------|-----------------------------------|
| `Off`     | No logging                        |
| `Error`   | Errors only                       |
| `Info`    | Key events and progress           |
| `Debug`   | Detailed per-generation logging   |

### `GaError`

Error type for GA operations.

| Variant         | Description                          |
|-----------------|--------------------------------------|
| `InvalidConfig` | Configuration error                  |
| `InitFailed`    | Initialization failed                |
| `RunFailed`     | Error during GA run                  |

## Related

- [Configuration Options](configuration.md)
- [Chromosome Types](chromosomes.md)
- [Fitness Functions](fitness.md)
- [Population Management](population.md)
- [Traits](traits.md)
- [Source: examples/knapsack_binary.rs](../examples/knapsack_binary.rs)
- [Source: examples/nqueens_range.rs](../examples/nqueens_range.rs)
- [Source: src/ga.rs](../src/ga.rs)
