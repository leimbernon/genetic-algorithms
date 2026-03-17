# Extension Operators

> Population diversity control strategies that trigger when diversity drops below a threshold.

## Overview

Extension strategies are optional diversity-rescue mechanisms integrated into the GA loop. They monitor population diversity (measured by fitness standard deviation) and trigger a corrective action when it drops below a configurable threshold. This prevents premature convergence and helps the algorithm explore new regions of the search space.

Extensions run after survivor selection and elite reinsertion, but before niching/fitness sharing. When an extension reduces the population size, the GA automatically regrows it using the configured initialization and fitness functions.

## Key Concepts

### ExtensionOperator Trait and Extension Enum

The extension operator is defined by the `ExtensionOperator` trait, which is implemented for each strategy. The `Extension` enum specifies the available strategies:

- `Noop`: No extension — diversity drops are ignored. This is the default.
- `MassExtinction`: Randomly culls the population to a survival rate, protecting a configurable number of elite individuals.
- `MassGenesis`: Trims the population to the 2 best chromosomes. The GA regrows the rest from scratch.
- `MassDegeneration`: Applies N rounds of swap mutation to all non-elite chromosomes and marks them for fitness re-evaluation.
- `MassDeduplication`: Removes chromosomes with duplicate gene-ID sequences, keeping the best fitness in each group.

### Diversity Threshold and Automatic Regrowth

Each extension strategy is triggered when the population's fitness standard deviation drops below the configured diversity threshold. If the extension reduces the population size, the GA automatically regrows the population using the initialization and fitness functions specified in your configuration.

## Usage

### Configuring Extension Strategies

Extension strategies are configured via the `ExtensionConfiguration` struct. You can set the strategy, diversity threshold, survival rate, mutation rounds, and elite count using builder methods:

| Method                        | Description                                                                                       | Default Value |
|-------------------------------|---------------------------------------------------------------------------------------------------|--------------|
| `with_method(Extension)`      | Sets the extension strategy.                                                                      | `Extension::Noop` |
| `with_diversity_threshold(f64)` | Sets the fitness standard deviation threshold for triggering the extension.                     | `0.0`        |
| `with_survival_rate(f64)`     | Sets the survival rate for MassExtinction (fraction of population to keep, 0.0..1.0).             | `1.0`        |
| `with_mutation_rounds(usize)` | Sets the number of mutation rounds for MassDegeneration.                                          | `1`          |
| `with_elite_count(usize)`     | Sets the number of elite individuals protected from extension events.                             | `0`          |

#### Default Values

| Field               | Default Value | Applies To           |
|---------------------|--------------|----------------------|
| `method`            | `Extension::Noop` | All strategies      |
| `diversity_threshold` | `0.0`       | All strategies       |
| `survival_rate`     | `1.0`        | MassExtinction       |
| `mutation_rounds`   | `1`          | MassDegeneration     |
| `elite_count`       | `0`          | MassExtinction, MassDegeneration |

### Example: Configuring and Integrating Extension Strategies

Below is a complete example demonstrating how to configure an extension strategy and integrate it into your GA loop. Note that you assign the `ExtensionConfiguration` to the `extension_configuration` field of your `GaConfiguration`.

```rust
use genetic_algorithms::extension::configuration::ExtensionConfiguration;
use genetic_algorithms::operations::Extension;
use genetic_algorithms::configuration::{GaConfiguration, LimitConfiguration, SelectionConfiguration, CrossoverConfiguration, MutationConfiguration};

fn main() {
    // Create an ExtensionConfiguration for MassExtinction
    let extension_config = ExtensionConfiguration::new()
        .with_method(Extension::MassExtinction)
        .with_diversity_threshold(0.05)
        .with_survival_rate(0.2)
        .with_elite_count(2);

    // Build your GA configuration and assign the extension configuration
    let ga_config = GaConfiguration {
        limit_configuration: LimitConfiguration {
            population_size: 100,
            genes_per_chromosome: 10,
            ..Default::default()
        },
        selection_configuration: SelectionConfiguration::default(),
        crossover_configuration: CrossoverConfiguration::default(),
        mutation_configuration: MutationConfiguration::default(),
        extension_configuration: Some(extension_config),
        ..Default::default()
    };

    // Now pass ga_config to your GA instance (not shown)
}
```

## API Reference

### Extension Enum

| Variant              | Description                                                                                   |
|----------------------|----------------------------------------------------------------------------------------------|
| `Noop`               | No extension — diversity drops are ignored.                                                  |
| `MassExtinction`     | Randomly culls the population to a survival rate, protecting a configurable number of elite individuals. |
| `MassGenesis`        | Trims the population to the 2 best chromosomes. The GA regrows the rest from scratch.        |
| `MassDegeneration`   | Applies N rounds of swap mutation to all non-elite chromosomes and marks them for fitness re-evaluation. |
| `MassDeduplication`  | Removes chromosomes with duplicate gene-ID sequences, keeping the best fitness in each group. |

### ExtensionConfiguration Struct

| Field                | Type         | Description                                                                                   | Default Value |
|----------------------|--------------|-----------------------------------------------------------------------------------------------|--------------|
| `method`             | `Extension`  | The extension strategy to use.                                                                | `Extension::Noop` |
| `diversity_threshold`| `f64`        | Fitness standard deviation threshold for triggering the extension.                            | `0.0`        |
| `survival_rate`      | `f64`        | For MassExtinction: fraction of population that survives the cull (0.0..1.0).                | `1.0`        |
| `mutation_rounds`    | `usize`      | For MassDegeneration: number of mutation rounds applied to non-elite chromosomes.             | `1`          |
| `elite_count`        | `usize`      | Number of elite individuals protected from the extension event.                               | `0`          |

#### Builder Methods

- `ExtensionConfiguration::new()`: Creates a new configuration with default values.
- `with_method(method: Extension)`: Sets the extension strategy.
- `with_diversity_threshold(threshold: f64)`: Sets the diversity threshold.
- `with_survival_rate(rate: f64)`: Sets the survival rate for MassExtinction.
- `with_mutation_rounds(rounds: usize)`: Sets mutation rounds for MassDegeneration.
- `with_elite_count(count: usize)`: Sets the number of elite individuals protected.

## Summary Table: Extension Strategies

| Strategy           | Trigger Condition                      | Action Taken                                                                 | Configurable Parameters            |
|--------------------|----------------------------------------|------------------------------------------------------------------------------|------------------------------------|
| Noop               | Never                                  | No action                                                                    | None                               |
| MassExtinction     | Diversity < threshold                   | Cull population randomly to survival_rate, protect elite_count individuals    | survival_rate, elite_count         |
| MassGenesis        | Diversity < threshold                   | Keep 2 best chromosomes, regrow rest                                         | None                               |
| MassDegeneration   | Diversity < threshold                   | Apply mutation_rounds swap mutations to all non-elite chromosomes             | mutation_rounds, elite_count       |
| MassDeduplication  | Diversity < threshold                   | Remove duplicate chromosomes, keep best in each group                         | None                               |

## Integration Notes

- Set `extension_configuration` in your `GaConfiguration` to enable diversity control.
- Extensions are triggered automatically when the diversity threshold is crossed.
- Population regrowth is handled internally after extension events that reduce population size.

For further details, see the source code documentation for [`ExtensionConfiguration`](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/extension/configuration/struct.ExtensionConfiguration.html) and [`Extension`](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/operations/enum.Extension.html).
