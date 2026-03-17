# Extension Operators

> Population diversity control strategies that trigger when diversity drops below a threshold.

## Overview

Extension strategies are optional diversity-rescue mechanisms integrated into the GA loop. They monitor population diversity (measured by fitness standard deviation) and trigger a corrective action when it drops below a configurable threshold. This prevents premature convergence and helps the algorithm explore new regions of the search space.

Extensions run after survivor selection and elite reinsertion, but before niching/fitness sharing. When an extension reduces the population size, the GA automatically regrows it using the configured initialization and fitness functions.

## Available Strategies

| Variant              | Description                                                                                   |
|----------------------|-----------------------------------------------------------------------------------------------|
| `Noop`               | No extension — diversity drops are ignored. This is the default.                             |
| `MassExtinction`     | Randomly culls the population to a survival rate, protecting a configurable number of elite individuals. |
| `MassGenesis`        | Trims the population to the 2 best chromosomes. The GA regrows the rest from scratch.        |
| `MassDegeneration`   | Applies N rounds of swap mutation to all non-elite chromosomes and marks them for fitness re-evaluation. |
| `MassDeduplication`  | Removes chromosomes with duplicate gene-ID sequences, keeping the best fitness in each group. |

## Configuration

Extension strategies are configured via the `ExtensionConfig` trait methods on the `Ga` builder:

| Method                                  | Description                                                        | Default |
|-----------------------------------------|--------------------------------------------------------------------|---------|
| `with_extension_method(Extension)`      | Sets the extension strategy.                                       | `Noop`  |
| `with_extension_diversity_threshold(f64)` | Fitness std dev threshold that triggers the extension.           | `0.01`  |
| `with_extension_survival_rate(f64)`     | Fraction of population surviving MassExtinction (0.0..1.0).       | `0.1`   |
| `with_extension_mutation_rounds(usize)` | Number of swap mutation rounds for MassDegeneration.               | `3`     |
| `with_extension_elite_count(usize)`     | Number of elite individuals protected from the extension.          | `1`     |

### ExtensionConfiguration

The `ExtensionConfiguration` struct (in `extension::configuration`) can also be constructed directly:

```rust
use genetic_algorithms::extension::configuration::ExtensionConfiguration;
use genetic_algorithms::operations::Extension;

let config = ExtensionConfiguration::new()
    .with_method(Extension::MassExtinction)
    .with_diversity_threshold(0.05)
    .with_survival_rate(0.2)
    .with_elite_count(2);
```

## Usage

```rust
use genetic_algorithms::ga::Ga;
use genetic_algorithms::operations::{Extension, Selection, Crossover, Mutation, Survivor};
use genetic_algorithms::traits::{ConfigurationT, ExtensionConfig};
use genetic_algorithms::configuration::ProblemSolving;

let mut ga = Ga::new()
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    // ... other configuration ...
    .with_extension_method(Extension::MassDeduplication)
    .with_extension_diversity_threshold(0.5)
    .with_extension_elite_count(2)
    .build()
    .expect("Valid configuration");

let population = ga.run();
```

## How It Works

Each generation, after survivor selection and elite reinsertion:

1. The GA computes the fitness standard deviation of the current population.
2. If `std_dev < diversity_threshold` and the configured method is not `Noop`:
   - The extension strategy is applied to the population.
   - If the population was reduced (MassGenesis, MassDeduplication, MassExtinction), new chromosomes are generated to restore the target population size.
   - If chromosomes were mutated with NaN fitness (MassDegeneration), their fitness is recalculated.
3. The GA continues with niching/fitness sharing and the rest of the generation.

## Trait

Custom extension strategies can be implemented via the `ExtensionOperator` trait:

```rust
pub trait ExtensionOperator {
    fn apply_extension<U: ChromosomeT>(
        &self,
        chromosomes: &mut Vec<U>,
        population_size: usize,
        problem_solving: ProblemSolving,
        config: &ExtensionConfiguration,
    ) -> Result<(), GaError>;
}
```

## Related

- [configuration.md](../configuration.md) — GA configuration including extension settings
- [operators/survivor.md](survivor.md) — Survivor selection (runs before extension)
- [traits.md](../traits.md) — Core traits including `ExtensionOperator`

**Source code:**
- [`src/operations/extension/mod.rs`](../../src/operations/extension/mod.rs)
- [`src/extension/configuration.rs`](../../src/extension/configuration.rs)
- [`src/traits/operators.rs`](../../src/traits/operators.rs)
