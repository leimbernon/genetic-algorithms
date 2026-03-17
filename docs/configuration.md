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

### Mutation Configuration

Mutation configuration controls how genetic diversity is introduced into the population. It includes settings for mutation probability, mutation method, and method-specific parameters. The latest version introduces **dynamic mutation probability adjustment** to help maintain population diversity.

#### Fields in `MutationConfiguration`

| Field                  | Description                                                                                   | Typical Values              |
|------------------------|-----------------------------------------------------------------------------------------------|-----------------------------|
| `probability_max`      | Maximum mutation probability (for adaptive GA).                                               | `Some(0.2)`                 |
| `probability_min`      | Minimum mutation probability (for adaptive GA).                                               | `Some(0.01)`                |
| `method`               | Mutation operator (e.g., Creep, Gaussian, Polynomial, NonUniform).                            | `Mutation::Creep`           |
| `step`                 | Step size for Creep mutation.                                                                | `Some(1.0)`                 |
| `sigma`                | Standard deviation for Gaussian mutation.                                                    | `Some(1.0)`                 |
| `polynomial_eta`       | Distribution index for Polynomial mutation. Higher values produce smaller perturbations.      | `Some(20.0)`                |
| `non_uniform_b`        | Decay parameter for NonUniform mutation. Controls mutation magnitude decrease.                | `Some(2.0)`                 |
| `dynamic_mutation`     | **Enable dynamic mutation probability adjustment.**                                           | `true` or `false`           |
| `target_cardinality`   | **Target ratio of unique fitness values to population size.** Guides mutation probability.    | `Some(0.5)`                 |
| `probability_step`     | **Step size for mutation probability adjustment each generation.**                            | `Some(0.01)`                |

#### Dynamic Mutation Probability

When `dynamic_mutation` is enabled, the mutation probability is automatically adjusted each generation based on population diversity:

- If the ratio of unique fitness values (`cardinality`) is **below** `target_cardinality`, mutation probability is **increased** by `probability_step`.
- If the ratio is **above** the target, mutation probability is **decreased** by `probability_step`.
- This helps maintain diversity and avoid premature convergence.

Dynamic mutation interacts with `probability_max` and `probability_min` by bounding the adjusted probability within these limits. It can be used alongside any mutation method.

## Usage

### Example: Configuring Dynamic Mutation Probability

```rust
use genetic_algorithms::prelude::*;
use genetic_algorithms::mutation::Mutation;

let ga = Ga::new()
    .with_population_size(100)
    .with_genes_per_chromosome(8)
    .with_mutation_method(Mutation::Creep)
    .with_mutation_probability_max(0.2)
    .with_mutation_probability_min(0.01)
    .with_mutation_step(1.0)
    // Enable dynamic mutation probability adjustment
    .with_dynamic_mutation(true)
    // Target 50% unique fitness values in the population
    .with_mutation_target_cardinality(0.5)
    // Adjust mutation probability by 0.01 per generation
    .with_mutation_probability_step(0.01)
    .with_fitness_fn(|dna: &[u8]| { /* compute fitness */ 0.0 })
    .with_initialization_fn(generic_random_initialization)
    .build()
    .expect("Failed to build GA");

let result = ga.run().expect("GA run failed");
println!("Best chromosome: {:?}", result.best_chromosome);
```

### Typical Values

- `dynamic_mutation`: Use `true` to enable, `false` to disable.
- `target_cardinality`: A value between `0.3` and `0.7` is common; adjust based on desired diversity.
- `probability_step`: Small values (`0.005` to `0.02`) are recommended for gradual adjustment.

## API Reference

### MutationConfiguration

```rust
pub struct MutationConfiguration {
    pub probability_max: Option<f64>,
    pub probability_min: Option<f64>,
    pub method: Mutation,
    pub step: Option<f64>,
    pub sigma: Option<f64>,
    pub polynomial_eta: Option<f64>,
    pub non_uniform_b: Option<f64>,
    pub dynamic_mutation: bool,
    pub target_cardinality: Option<f64>,
    pub probability_step: Option<f64>,
}
```

#### Builder Methods

- `with_mutation_probability_max(probability_max: f64)`
- `with_mutation_probability_min(probability_min: f64)`
- `with_mutation_method(method: Mutation)`
- `with_mutation_step(step: f64)`
- `with_mutation_sigma(sigma: f64)`
- `with_dynamic_mutation(enabled: bool)`
- `with_mutation_target_cardinality(target: f64)`
- `with_mutation_probability_step(step: f64)`

See [`MutationConfig`] trait for the full builder API.

---

For more details on other configuration types, see their respective sections in this document or refer to the API documentation.
