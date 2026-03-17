# Configuration

> Genetic algorithm configuration options and the builder API.

## Overview

The configuration system defines all tunable parameters for running a genetic algorithm (GA) with this library. It encapsulates settings for population size, selection, crossover, mutation, survivor selection, logging, stopping criteria, and more. A well-structured configuration is essential for tailoring the GA to your problem and controlling its behavior and performance.

This module provides both direct struct-based configuration and a flexible builder API via the `ConfigurationT` trait. The builder pattern allows you to incrementally set options in a readable, chainable style. All configuration types have sensible defaults, so you can start with the defaults and override only the parameters you care about.

Configuration is a central part of the library's architecture: it is consumed by the main `Ga` orchestrator and governs every aspect of the evolutionary process. Understanding and customizing configuration is the key to effective use of the library.

## Key Concepts

### Main Configuration Types

| Type                        | Description                                                                 |
|-----------------------------|-----------------------------------------------------------------------------|
| `GaConfiguration`           | Top-level GA configuration struct. Contains all operator and control settings. |
| `SelectionConfiguration`    | Selection operator and number of parent pairs per generation.                |
| `CrossoverConfiguration`    | Crossover operator, points, probabilities, and method-specific parameters.   |
| `MutationConfiguration`     | Mutation operator, probabilities, and method-specific parameters.            |
| `LimitConfiguration`        | Problem type, population size, chromosome length, uniqueness, and repeats.   |
| `SaveProgressConfiguration` | Controls periodic saving of progress to disk.                                |
| `StoppingCriteria`          | Compound stopping criteria (stagnation, convergence, time).                  |
| `LogLevel`                  | Controls verbosity of logging output.                                        |
| `ProblemSolving`            | Indicates if the problem is minimization, maximization, or fixed fitness.    |

### GaConfiguration Fields

| Field                         | Type                        | Description                                                                                       |
|-------------------------------|-----------------------------|---------------------------------------------------------------------------------------------------|
| `adaptive_ga`                 | `bool`                      | Enables adaptive parameter control (default: false).                                              |
| `number_of_threads`           | `i32`                       | Number of threads for parallelism (default: 1).                                                   |
| `limit_configuration`         | `LimitConfiguration`         | Population, chromosome, and problem constraints.                                                  |
| `selection_configuration`     | `SelectionConfiguration`     | Selection operator and parent pairing.                                                            |
| `crossover_configuration`     | `CrossoverConfiguration`     | Crossover operator and parameters.                                                                |
| `mutation_configuration`      | `MutationConfiguration`      | Mutation operator and parameters.                                                                 |
| `survivor`                    | `Survivor`                  | Survivor selection method.                                                                        |
| `log_level`                   | `LogLevel`                  | Logging verbosity.                                                                                |
| `save_progress_configuration` | `SaveProgressConfiguration`  | Controls progress saving.                                                                         |
| `elitism_count`               | `usize`                     | Number of elite individuals preserved each generation (default: 0).                               |
| `stopping_criteria`           | `StoppingCriteria`           | Additional stopping criteria.                                                                     |
| `extension_configuration`     | `Option<ExtensionConfiguration>` | Optional extension strategy for population diversity control (default: None).                |

#### ExtensionConfiguration Fields

| Field                  | Type         | Description                                                                                   |
|------------------------|--------------|-----------------------------------------------------------------------------------------------|
| `method`               | `Extension`  | Extension strategy (`Noop`, `MassExtinction`, `MassGenesis`, `MassDegeneration`, `MassDeduplication`). |
| `diversity_threshold`  | `f64`        | Fitness std dev threshold that triggers the extension (default: 0.01).                        |
| `survival_rate`        | `f64`        | Fraction of population surviving MassExtinction (default: 0.1).                               |
| `mutation_rounds`      | `usize`      | Number of mutation rounds for MassDegeneration (default: 3).                                  |
| `elite_count`          | `usize`      | Number of elite individuals protected (default: 1).                                           |

#### LimitConfiguration Fields

| Field                    | Type         | Description                                                                                   |
|--------------------------|--------------|-----------------------------------------------------------------------------------------------|
| `problem_solving`        | `ProblemSolving` | Minimization, Maximization, or FixedFitness.                                                  |
| `max_generations`        | `i32`        | Maximum number of generations (default: 100).                                                 |
| `fitness_target`         | `Option<f64>`| Target fitness value to stop (None disables).                                                 |
| `population_size`        | `i32`        | Number of individuals in the population (default: 100).                                       |
| `genes_per_chromosome`   | `i32`        | Number of genes per chromosome.                                                               |
| `needs_unique_ids`       | `bool`       | If true, chromosomes must have unique IDs (default: false).                                   |
| `alleles_can_be_repeated`| `bool`       | If true, alleles can be repeated in a chromosome (default: true).                             |

#### SelectionConfiguration Fields

| Field               | Type         | Description                                    |
|---------------------|--------------|------------------------------------------------|
| `number_of_couples` | `i32`        | Number of parent pairs selected per generation. |
| `method`            | `Selection`  | Selection operator (e.g., Tournament, Roulette).|

#### CrossoverConfiguration Fields

| Field              | Type           | Description                                                                                  |
|--------------------|----------------|----------------------------------------------------------------------------------------------|
| `number_of_points` | `Option<i32>`  | Number of crossover points (None for operator default).                                      |
| `probability_max`  | `Option<f64>`  | Maximum crossover probability.                                                               |
| `probability_min`  | `Option<f64>`  | Minimum crossover probability.                                                               |
| `method`           | `Crossover`    | Crossover operator (e.g., Uniform, Multipoint, SBX).                                         |
| `sbx_eta`          | `Option<f64>`  | Distribution index for SBX crossover (higher = children closer to parents, default: 2.0).    |
| `blend_alpha`      | `Option<f64>`  | Alpha parameter for BLX-α crossover (controls range, default: 0.5).                          |

#### MutationConfiguration Fields

| Field              | Type           | Description                                                                              |
|--------------------|----------------|------------------------------------------------------------------------------------------|
| `probability_max`  | `Option<f64>`  | Maximum mutation probability.                                                            |
| `probability_min`  | `Option<f64>`  | Minimum mutation probability.                                                            |
| `method`           | `Mutation`     | Mutation operator (e.g., Swap, Scramble, Creep, Gaussian).                              |
| `step`             | `Option<f64>`  | Step size for Creep mutation (default: 1.0).                                            |
| `sigma`            | `Option<f64>`  | Standard deviation for Gaussian mutation (default: 1.0).                                 |

#### SaveProgressConfiguration Fields

| Field                   | Type      | Description                                              |
|-------------------------|-----------|----------------------------------------------------------|
| `save_progress`         | `bool`    | If true, enables periodic saving of GA progress.         |
| `save_progress_interval`| `i32`     | Interval (in generations) between saves.                 |
| `save_progress_path`    | `String`  | File path for saving progress.                           |

#### StoppingCriteria Fields

| Field                   | Type         | Description                                                                                   |
|-------------------------|--------------|-----------------------------------------------------------------------------------------------|
| `stagnation_generations`| `Option<i32>`| Stop if no fitness improvement for N generations (None disables).                             |
| `convergence_threshold` | `Option<f64>`| Stop if fitness standard deviation drops below threshold (None disables).                     |
| `max_duration_secs`     | `Option<f64>`| Stop if elapsed time exceeds this value in seconds (None disables).                           |

#### LogLevel

| Variant    | Description                         |
|------------|-------------------------------------|
| `Off`      | No logging.                         |
| `Error`    | Error messages only.                |
| `Warn`     | Warnings and errors.                |
| `Info`     | Informational messages.             |
| `Debug`    | Debug-level output.                 |
| `Trace`    | Most verbose, for tracing.          |

#### ProblemSolving

| Variant         | Description                                             |
|-----------------|--------------------------------------------------------|
| `Minimization`  | Lower fitness is better.                               |
| `Maximization`  | Higher fitness is better.                              |
| `FixedFitness`  | Run until a fixed fitness value is reached.            |

## Usage

### Basic Example

This example demonstrates how to construct a `GaConfiguration` using both direct struct initialization and the builder API.

```rust
use my_ga_lib::configuration::*;
use my_ga_lib::operators::{Selection, Crossover, Mutation, Survivor};
use my_ga_lib::traits::ConfigurationT;

// Direct struct construction with defaults and overrides
let mut config = GaConfiguration {
    limit_configuration: LimitConfiguration {
        population_size: 200,
        genes_per_chromosome: 20,
        problem_solving: ProblemSolving::Maximization,
        ..Default::default()
    },
    selection_configuration: SelectionConfiguration {
        number_of_couples: 100,
        method: Selection::Tournament(3),
    },
    crossover_configuration: CrossoverConfiguration {
        method: Crossover::Uniform,
        probability_max: Some(0.9),
        ..Default::default()
    },
    mutation_configuration: MutationConfiguration {
        method: Mutation::Swap,
        probability_max: Some(0.05),
        ..Default::default()
    },
    survivor: Survivor::FitnessBased,
    log_level: LogLevel::Info,
    ..Default::default()
};

// Using the builder API
let mut config2 = GaConfiguration::new()
    .with_population_size(200)
    .with_genes_per_chromosome(20)
    .with_problem_solving(ProblemSolving::Maximization)
    .with_selection_method(Selection::Tournament(3))
    .with_number_of_couples(100)
    .with_crossover_method(Crossover::Uniform)
    .with_crossover_probability_max(0.9)
    .with_mutation_method(Mutation::Swap)
    .with_mutation_probability_max(0.05)
    .with_survivor_method(Survivor::FitnessBased)
    .with_logs(LogLevel::Info);
```

### Advanced Example

This example shows a more complete configuration, including elitism, stopping criteria, multi-threading, and progress saving.

```rust
use my_ga_lib::configuration::*;
use my_ga_lib::operators::{Selection, Crossover, Mutation, Survivor};
use my_ga_lib::traits::ConfigurationT;

let stopping = StoppingCriteria {
    stagnation_generations: Some(50),
    convergence_threshold: Some(0.001),
    max_duration_secs: Some(600.0),
};

let mut config = GaConfiguration::new()
    .with_adaptive_ga(true)
    .with_threads(4)
    .with_logs(LogLevel::Debug)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_population_size(300)
    .with_genes_per_chromosome(30)
    .with_selection_method(Selection::Roulette)
    .with_number_of_couples(150)
    .with_crossover_method(Crossover::SBX)
    .with_crossover_number_of_points(2)
    .with_crossover_probability_max(0.85)
    .with_sbx_eta(10.0)
    .with_mutation_method(Mutation::Gaussian)
    .with_mutation_probability_max(0.02)
    .with_mutation_sigma(0.5)
    .with_survivor_method(Survivor::AgeBased)
    .with_elitism(5)
    .with_stopping_criteria(stopping)
    .with_save_progress(true)
    .with_save_progress_interval(10)
    .with_save_progress_path("checkpoints/ga_progress.json".to_string());
```

## Builder API

The `ConfigurationT` trait provides a builder-style API for constructing and customizing GA configurations. All methods return `&mut Self` for chaining.

### Main Methods

| Method                                | Description                                                                                   |
|----------------------------------------|-----------------------------------------------------------------------------------------------|
| `new()`                               | Creates a new configuration with defaults.                                                    |
| `with_adaptive_ga(bool)`               | Enables or disables adaptive parameter control.                                               |
| `with_threads(i32)`                    | Sets the number of threads for parallel execution.                                            |
| `with_logs(LogLevel)`                  | Sets the logging verbosity.                                                                   |
| `with_survivor_method(Survivor)`       | Sets the survivor selection operator.                                                         |
| `with_problem_solving(ProblemSolving)` | Sets the problem type (min/max/fixed).                                                        |
| `with_max_generations(i32)`            | Sets the maximum number of generations.                                                       |
| `with_fitness_target(f64)`             | Sets the fitness value to stop at.                                                            |
| `with_population_size(i32)`            | Sets the population size.                                                                     |
| `with_genes_per_chromosome(i32)`       | Sets the number of genes per chromosome.                                                      |
| `with_needs_unique_ids(bool)`          | Enforces unique chromosome IDs.                                                               |
| `with_alleles_can_be_repeated(bool)`   | Allows or disallows repeated alleles in chromosomes.                                          |
| `with_number_of_couples(i32)`          | Sets the number of parent pairs per generation.                                               |
| `with_selection_method(Selection)`     | Sets the selection operator.                                                                  |
| `with_crossover_number_of_points(i32)` | Sets the number of crossover points.                                                          |
| `with_crossover_probability_max(f64)`  | Sets the maximum crossover probability.                                                       |
| `with_crossover_probability_min(f64)`  | Sets the minimum crossover probability.                                                       |
| `with_crossover_method(Crossover)`     | Sets the crossover operator.                                                                  |
| `with_sbx_eta(f64)`                    | Sets the SBX crossover distribution index (eta).                                              |
| `with_blend_alpha(f64)`                | Sets the BLX-α crossover alpha parameter.                                                     |
| `with_mutation_probability_max(f64)`   | Sets the maximum mutation probability.                                                        |
| `with_mutation_probability_min(f64)`   | Sets the minimum mutation probability.                                                        |
| `with_mutation_method(Mutation)`       | Sets the mutation operator.                                                                   |
| `with_mutation_step(f64)`              | Sets the step size for Creep mutation.                                                        |
| `with_mutation_sigma(f64)`             | Sets the sigma for Gaussian mutation.                                                         |
| `with_save_progress(bool)`             | Enables or disables periodic progress saving.                                                 |
| `with_save_progress_interval(i32)`     | Sets the interval (in generations) for saving progress.                                       |
| `with_save_progress_path(String)`      | Sets the file path for saving progress.                                                       |
| `with_elitism(usize)`                  | Sets the number of elite individuals preserved each generation.                               |
| `with_stopping_criteria(StoppingCriteria)` | Sets compound stopping criteria in addition to max_generations and fitness_target.        |
| `with_extension_method(Extension)`     | Sets the extension strategy for population diversity control.                                 |
| `with_extension_diversity_threshold(f64)` | Sets the fitness std dev threshold that triggers the extension.                            |
| `with_extension_survival_rate(f64)`    | Sets the survival rate for MassExtinction (0.0..1.0).                                        |
| `with_extension_mutation_rounds(usize)` | Sets the number of mutation rounds for MassDegeneration.                                    |
| `with_extension_elite_count(usize)`    | Sets the number of elite individuals protected from extension events.                         |

## Defaults

A summary of default values for each configuration struct:

### GaConfiguration

- `adaptive_ga`: false
- `number_of_threads`: 1
- `limit_configuration`: see below
- `selection_configuration`: see below
- `crossover_configuration`: see below
- `mutation_configuration`: see below
- `survivor`: library default (e.g., FitnessBased)
- `log_level`: LogLevel::Info
- `save_progress_configuration`: see below
- `elitism_count`: 0
- `stopping_criteria`: all fields None
- `extension_configuration`: None

### ExtensionConfiguration

- `method`: Extension::Noop
- `diversity_threshold`: 0.01
- `survival_rate`: 0.1
- `mutation_rounds`: 3
- `elite_count`: 1

### LimitConfiguration

- `problem_solving`: ProblemSolving::Maximization
- `max_generations`: 100
- `fitness_target`: None
- `population_size`: 100
- `genes_per_chromosome`: 10
- `needs_unique_ids`: false
- `alleles_can_be_repeated`: true

### SelectionConfiguration

- `number_of_couples`: 50
- `method`: Selection::Tournament(2)

### CrossoverConfiguration

- `number_of_points`: None
- `probability_max`: None
- `probability_min`: None
- `method`: Crossover::Uniform
- `sbx_eta`: Some(2.0)
- `blend_alpha`: Some(0.5)

### MutationConfiguration

- `probability_max`: None
- `probability_min`: None
- `method`: Mutation::Swap
- `step`: Some(1.0)
- `sigma`: Some(1.0)

### SaveProgressConfiguration

- `save_progress`: false
- `save_progress_interval`: 50
- `save_progress_path`: "progress.json"

### StoppingCriteria

- `stagnation_generations`: None
- `convergence_threshold`: None
- `max_duration_secs`: None

## API Reference

### `GaConfiguration`

Top-level struct for configuring all aspects of the genetic algorithm.

**Fields:**

| Name                         | Type                        | Description                                                  |
|------------------------------|-----------------------------|--------------------------------------------------------------|
| `adaptive_ga`                | `bool`                      | Enables adaptive parameter control.                          |
| `number_of_threads`          | `i32`                       | Number of threads for parallelism.                           |
| `limit_configuration`        | `LimitConfiguration`         | Problem and population constraints.                          |
| `selection_configuration`    | `SelectionConfiguration`     | Selection operator settings.                                 |
| `crossover_configuration`    | `CrossoverConfiguration`     | Crossover operator settings.                                 |
| `mutation_configuration`     | `MutationConfiguration`      | Mutation operator settings.                                  |
| `survivor`                   | `Survivor`                  | Survivor selection operator.                                 |
| `log_level`                  | `LogLevel`                  | Logging verbosity.                                           |
| `save_progress_configuration`| `SaveProgressConfiguration`  | Progress saving options.                                     |
| `elitism_count`              | `usize`                     | Number of elite individuals preserved.                       |
| `stopping_criteria`          | `StoppingCriteria`           | Additional stopping criteria.                                |
| `extension_configuration`    | `Option<ExtensionConfiguration>` | Optional extension strategy for diversity control.      |

**Methods (via `ConfigurationT`):**

See [Builder API](#builder-api) above for the full list.

### `SelectionConfiguration`

Settings for parent selection.

| Name               | Type         | Description                                    |
|--------------------|--------------|------------------------------------------------|
| `number_of_couples`| `i32`        | Number of parent pairs per generation.          |
| `method`           | `Selection`  | Selection operator.                            |

### `CrossoverConfiguration`

Settings for crossover operator.

| Name               | Type           | Description                                                    |
|--------------------|----------------|----------------------------------------------------------------|
| `number_of_points` | `Option<i32>`  | Number of crossover points.                                    |
| `probability_max`  | `Option<f64>`  | Maximum crossover probability.                                 |
| `probability_min`  | `Option<f64>`  | Minimum crossover probability.                                 |
| `method`           | `Crossover`    | Crossover operator.                                            |
| `sbx_eta`          | `Option<f64>`  | SBX distribution index (default: 2.0).                         |
| `blend_alpha`      | `Option<f64>`  | BLX-α alpha parameter (default: 0.5).                          |

### `MutationConfiguration`

Settings for mutation operator.

| Name               | Type           | Description                                                    |
|--------------------|----------------|----------------------------------------------------------------|
| `probability_max`  | `Option<f64>`  | Maximum mutation probability.                                  |
| `probability_min`  | `Option<f64>`  | Minimum mutation probability.                                  |
| `method`           | `Mutation`     | Mutation operator.                                             |
| `step`             | `Option<f64>`  | Step size for Creep mutation (default: 1.0).                   |
| `sigma`            | `Option<f64>`  | Standard deviation for Gaussian mutation (default: 1.0).        |

### `LimitConfiguration`

Problem and population constraints.

| Name                    | Type          | Description                                                    |
|-------------------------|---------------|----------------------------------------------------------------|
| `problem_solving`       | `ProblemSolving` | Minimization, Maximization, or FixedFitness.                |
| `max_generations`       | `i32`         | Maximum number of generations.                                 |
| `fitness_target`        | `Option<f64>` | Target fitness value to stop.                                  |
| `population_size`       | `i32`         | Number of individuals in the population.                       |
| `genes_per_chromosome`  | `i32`         | Number of genes per chromosome.                                |
| `needs_unique_ids`      | `bool`        | If true, chromosomes must have unique IDs.                     |
| `alleles_can_be_repeated`| `bool`       | If true, alleles can be repeated in a chromosome.              |

### `SaveProgressConfiguration`

Periodic progress saving options.

| Name                    | Type      | Description                                    |
|-------------------------|-----------|------------------------------------------------|
| `save_progress`         | `bool`    | Enables progress saving.                       |
| `save_progress_interval`| `i32`     | Generations between saves.                     |
| `save_progress_path`    | `String`  | File path for saving progress.                 |

### `StoppingCriteria`

Compound stopping criteria.

| Name                    | Type         | Description                                    |
|-------------------------|--------------|------------------------------------------------|
| `stagnation_generations`| `Option<i32>`| Stop if no fitness improvement for N generations.|
| `convergence_threshold` | `Option<f64>`| Stop if fitness stddev drops below threshold.   |
| `max_duration_secs`     | `Option<f64>`| Stop if elapsed time exceeds this value.        |

### `LogLevel`

Controls logging verbosity.

Variants: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`.

### `ProblemSolving`

Indicates optimization direction.

Variants: `Minimization`, `Maximization`, `FixedFitness`.

## Related

- [traits.md](traits.md) — Core traits including `ConfigurationT`
- [operators/selection.md](operators/selection.md) — Selection operators
- [operators/crossover.md](operators/crossover.md) — Crossover operators
- [operators/mutation.md](operators/mutation.md) — Mutation operators
- [operators/survivor.md](operators/survivor.md) — Survivor selection
- [operators/extension.md](operators/extension.md) — Extension strategies for diversity control
- [fitness.md](fitness.md) — Fitness evaluation
- [population.md](population.md) — Population management
- [examples.md](examples.md) — End-to-end configuration examples

**Source code:**
- [`src/configuration.rs`](../src/configuration.rs)
- [`src/traits/configuration.rs`](../src/traits/configuration.rs)
- [`src/ga.rs`](../src/ga.rs)
