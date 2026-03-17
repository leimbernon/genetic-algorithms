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
| `StoppingCriteria`          | Compound stopping criteria for GA termination.                               |
| `NichingConfiguration`      | Optional fitness sharing (niching) configuration.                            |
| `ExtensionConfiguration`    | **New:** Optional population diversity control via extension strategies.      |

### Extension Strategies for Diversity Control

Extension strategies are mechanisms to restore population diversity when the GA risks premature convergence. They monitor the fitness standard deviation and trigger corrective actions when diversity drops below a threshold. This helps maintain exploration and avoid getting stuck in local optima.

#### Extension Configuration Fields

- `method`: The extension strategy to use (e.g., `MassExtinction`, `MassDegeneration`).
- `diversity_threshold`: Fitness standard deviation threshold. Extension triggers when diversity falls below this value.
- `survival_rate`: For `MassExtinction`, the fraction of the population that survives the cull.
- `mutation_rounds`: For `MassDegeneration`, the number of mutation rounds applied to non-elite chromosomes.
- `elite_count`: Number of elite individuals protected from extension events.

#### How Extension Configuration Interacts with Other GA Parameters

- **Population Size:** Extension strategies operate on the current population. The number of elites and survivors is relative to the configured population size.
- **Elitism:** Extension events can protect a configurable number of elite individuals, in addition to the standard elitism mechanism.
- **Niching:** Extension and niching can be enabled independently. Both aim to maintain diversity but use different mechanisms.
- **Stopping Criteria:** Extension events are triggered before checking stopping criteria. If diversity is restored, the GA continues; otherwise, stopping criteria may halt the run.

## Usage

### Configuring Extension Strategies

You can enable and customize extension strategies using builder methods on `GaConfiguration` or directly via `ExtensionConfiguration`. Extension configuration is optional and only applied if set.

#### Example: Enabling Mass Extinction Extension

```rust
use genetic_algorithms::ga::GaConfiguration;
use genetic_algorithms::operations::Extension;

let config = GaConfiguration::new()
    .with_population_size(100)
    .with_extension_method(Extension::MassExtinction)
    .with_extension_diversity_threshold(0.05)
    .with_extension_survival_rate(0.2)
    .with_extension_elite_count(2);

// This configuration will trigger a mass extinction event whenever the fitness
// standard deviation drops below 0.05, preserving 20% of the population and 2 elites.
```

#### Example: Using Mass Degeneration

```rust
use genetic_algorithms::ga::GaConfiguration;
use genetic_algorithms::operations::Extension;

let config = GaConfiguration::new()
    .with_extension_method(Extension::MassDegeneration)
    .with_extension_diversity_threshold(0.1)
    .with_extension_mutation_rounds(3)
    .with_extension_elite_count(1);

// When diversity drops below 0.1, all non-elite chromosomes will be mutated 3 times.
```

### Builder Methods for Extension Strategies

- `with_extension_method(method: Extension)`: Sets the extension strategy.
- `with_extension_diversity_threshold(threshold: f64)`: Sets the diversity threshold.
- `with_extension_survival_rate(rate: f64)`: Sets survival rate for `MassExtinction`.
- `with_extension_mutation_rounds(rounds: usize)`: Sets mutation rounds for `MassDegeneration`.
- `with_extension_elite_count(count: usize)`: Sets number of elites protected from extension events.

## API Reference

### `ExtensionConfiguration`

```rust
pub struct ExtensionConfiguration {
    pub method: Extension,
    pub diversity_threshold: f64,
    pub survival_rate: f64,
    pub mutation_rounds: usize,
    pub elite_count: usize,
}
```

#### Methods

| Method                                 | Description                                                        |
|-----------------------------------------|--------------------------------------------------------------------|
| `new()`                                | Creates a new `ExtensionConfiguration` with default values.        |
| `with_method(method: Extension)`        | Sets the extension strategy.                                       |
| `with_diversity_threshold(threshold)`   | Sets the fitness std dev threshold for triggering extension.        |
| `with_survival_rate(rate)`              | Sets survival rate for `MassExtinction`.                           |
| `with_mutation_rounds(rounds)`          | Sets mutation rounds for `MassDegeneration`.                       |
| `with_elite_count(count)`               | Sets number of elites protected from extension events.             |

### `GaConfiguration` Extension Builder Methods

```rust
impl ExtensionConfig for GaConfiguration {
    fn with_extension_method(self, method: Extension) -> Self;
    fn with_extension_diversity_threshold(self, threshold: f64) -> Self;
    fn with_extension_survival_rate(self, rate: f64) -> Self;
    fn with_extension_mutation_rounds(self, rounds: usize) -> Self;
    fn with_extension_elite_count(self, count: usize) -> Self;
}
```

## Summary Table: Extension Configuration Fields

| Field                | Applies To         | Description                                                       |
|----------------------|-------------------|-------------------------------------------------------------------|
| `method`             | All strategies    | Extension strategy (`MassExtinction`, `MassDegeneration`, etc.)   |
| `diversity_threshold`| All strategies    | Fitness std dev threshold for triggering extension                 |
| `survival_rate`      | MassExtinction    | Fraction of population surviving the cull                         |
| `mutation_rounds`    | MassDegeneration  | Number of mutation rounds applied to non-elites                   |
| `elite_count`        | All strategies    | Number of elite individuals protected from extension events        |

Extension strategies provide robust diversity control and can be tuned independently of other GA parameters. For advanced use, combine extension configuration with niching and elitism for maximum population diversity and exploration.
