# API Reference

> Comprehensive summary of all public types, traits, functions, and configuration options in the genetic algorithms library.

## Overview

This API reference provides a detailed listing of every public item in the genetic algorithms library, including modules, types, traits, enums, and functions. It serves as a central resource for developers to understand the available abstractions and how to use them when building custom genetic algorithm solutions.

The library is designed to be modular and extensible, with clear separation between core abstractions (genes, chromosomes, configuration), concrete implementations (binary/range genes and chromosomes), and composable operators (selection, crossover, mutation, survivor). The orchestrator (`Ga`) coordinates the entire lifecycle of a genetic algorithm, while utility modules provide helpers for initialization, fitness evaluation, and population management.

This document is intended for Rust developers who want to leverage genetic algorithms for optimization or search problems. It provides both high-level guidance and low-level details, ensuring users can quickly find and understand every public API surface.

## Key Concepts

The following table summarizes the core modules and their primary types:

| Module            | Type/Enum/Trait         | Description                                                      |
|-------------------|------------------------|------------------------------------------------------------------|
| `traits`          | `GeneT`, `ChromosomeT`, `ConfigurationT` | Core traits for genes, chromosomes, and configuration            |
| `chromosomes`     | `Binary`, `Range`      | Concrete chromosome types                                        |
| `genotypes`       | `Binary`, `Range`      | Concrete gene types                                              |
| `ga`              | `Ga`                   | Genetic algorithm orchestrator                                   |
| `configuration`   | `GaConfiguration`, `SelectionConfiguration`, `CrossoverConfiguration`, `MutationConfiguration`, `LimitConfiguration`, `SaveProgressConfiguration`, `StoppingCriteria` | Configuration structs for all GA parameters                      |
| `operations`      | `Selection`, `Crossover`, `Mutation`, `Survivor`, `Extension`, `ExtensionOperator` | Operator enums and traits for selection, crossover, mutation, survivor, and extension strategies |
| `extension`       | `ExtensionConfiguration` | Configuration for population diversity control (extension strategies) |

## Usage

### Extension Strategies for Population Diversity Control

Extension strategies are mechanisms to restore population diversity when it drops below a threshold during a GA run. These are configured via the `ExtensionConfiguration` struct and the `Extension` enum.

**Example: Configuring an Extension Strategy**

```rust
use genetic_algorithms::extension::configuration::ExtensionConfiguration;
use genetic_algorithms::operations::Extension;

let extension_config = ExtensionConfiguration::new()
    .with_method(Extension::MassExtinction)
    .with_diversity_threshold(0.05)
    .with_survival_rate(0.2)
    .with_elite_count(2);

assert_eq!(extension_config.method, Extension::MassExtinction);
assert!((extension_config.diversity_threshold - 0.05).abs() < f64::EPSILON);
```

To enable extension strategies in a GA run, set the `extension_configuration` field of your `GaConfiguration`:

```rust
use genetic_algorithms::configuration::{GaConfiguration, ExtensionConfiguration};
use genetic_algorithms::operations::Extension;

let ga_config = GaConfiguration::default()
    .with_extension_method(Extension::MassDegeneration)
    .with_extension_diversity_threshold(0.1)
    .with_extension_survival_rate(0.3)
    .with_extension_mutation_rounds(5)
    .with_extension_elite_count(1);
```

### Builder Methods for Extension Configuration

The following builder methods are available for configuring extension strategies:

- `with_extension_method(method: Extension) -> Self`
- `with_extension_diversity_threshold(threshold: f64) -> Self`
- `with_extension_survival_rate(rate: f64) -> Self`
- `with_extension_mutation_rounds(rounds: usize) -> Self`
- `with_extension_elite_count(count: usize) -> Self`

These methods are available on both `GaConfiguration` and `ExtensionConfiguration`.

## API Reference

### ExtensionOperator Trait

The `ExtensionOperator` trait defines the interface for extension strategies that control population diversity:

```rust
pub trait ExtensionOperator {
    /// Applies the extension strategy to the population.
    fn apply_extension(&mut self, population: &mut Population, config: &ExtensionConfiguration);
}
```

### Extension Enum

The `Extension` enum specifies the available extension strategies:

```rust
pub enum Extension {
    /// Mass extinction: culls a fraction of the population, preserving only elites.
    MassExtinction,
    /// Mass degeneration: applies multiple mutation rounds to non-elite chromosomes.
    MassDegeneration,
    /// No extension: disables diversity control.
    None,
}
```

### ExtensionConfiguration Struct

The `ExtensionConfiguration` struct configures when and how extension strategies are triggered:

```rust
pub struct ExtensionConfiguration {
    pub method: Extension,
    pub diversity_threshold: f64,
    pub survival_rate: f64,
    pub mutation_rounds: usize,
    pub elite_count: usize,
}
```

#### Builder Methods

- `new() -> Self`: Creates a new `ExtensionConfiguration` with default values.
- `with_method(method: Extension) -> Self`: Sets the extension strategy.
- `with_diversity_threshold(threshold: f64) -> Self`: Sets the fitness standard deviation threshold.
- `with_survival_rate(rate: f64) -> Self`: Sets the survival rate for MassExtinction.
- `with_mutation_rounds(rounds: usize) -> Self`: Sets mutation rounds for MassDegeneration.
- `with_elite_count(count: usize) -> Self`: Sets the number of elite individuals protected.

### GaConfiguration Extension Methods

The `GaConfiguration` struct supports the following builder methods for extension configuration:

```rust
impl ExtensionConfig for GaConfiguration {
    fn with_extension_method(self, method: Extension) -> Self;
    fn with_extension_diversity_threshold(self, threshold: f64) -> Self;
    fn with_extension_survival_rate(self, rate: f64) -> Self;
    fn with_extension_mutation_rounds(self, rounds: usize) -> Self;
    fn with_extension_elite_count(self, count: usize) -> Self;
}
```

## Additional Types

For a complete listing of all configuration and operator types, see the [Configuration](configuration.md) and [Operators](operators.md) documentation.

---

**Note:** Extension strategies are optional and can be disabled by setting the method to `Extension::None`. Use them to maintain genetic diversity and prevent premature convergence in challenging optimization problems.
