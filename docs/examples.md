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
| Extension Strategy     | `MassDeduplication`    | Prevents premature convergence by deduplicating population |
| Callback               | `Callback`             | Allows custom logic during GA execution                  |

## Usage

### OneMax with Extension Strategies

The `onemax_extension.rs` example demonstrates how to solve the classic OneMax problem using extension strategies to improve genetic algorithm robustness. The OneMax problem seeks to maximize the number of ones in a binary chromosome.

#### Preventing Premature Convergence with MassDeduplication

Premature convergence occurs when the population loses diversity too quickly, often resulting in suboptimal solutions. The `MassDeduplication` extension strategy addresses this by deduplicating individuals in the population, ensuring that similar solutions do not dominate and allowing the algorithm to explore a broader search space.

#### Configuration and Callback Usage

The example configures the GA to use the `MassDeduplication` extension strategy. It also demonstrates how to use callbacks for monitoring progress or custom logic during the run.

#### Running the Example

To run the example, use the following command from the project root:

```bash
cargo run --example onemax_extension
```

#### Example Code

Below is a simplified excerpt illustrating the key setup:

```rust
use ga_lib::{
    BinaryChromosome, Genotype, Ga, ExtensionStrategy, MassDeduplication, Callback,
};

fn main() {
    // Define the genotype for a 100-bit chromosome
    let genotype = Genotype::binary(100);

    // Fitness function: count the number of ones
    let fitness_fn = |chromosome: &BinaryChromosome| chromosome.iter().filter(|&&bit| bit).count();

    // Configure MassDeduplication extension strategy
    let extension = ExtensionStrategy::MassDeduplication(MassDeduplication::default());

    // Optional: define a callback to monitor progress
    let callback = Callback::new(|generation, best| {
        println!("Generation {}: best fitness {}", generation, best.fitness);
    });

    // Set up and run the GA
    let mut ga = Ga::builder()
        .genotype(genotype)
        .fitness_fn(fitness_fn)
        .extension_strategy(extension)
        .callback(callback)
        .build();

    let result = ga.run();
    println!("Best solution found: {:?}", result.best_chromosome());
}
```

### Other Examples

- **Knapsack Problem**: Demonstrates binary chromosome optimization for item selection under weight constraints.
- **N-Queens Problem**: Uses range chromosomes to solve the classic chessboard placement problem.

## API Reference

| Component              | Description                                                                 |
|------------------------|-----------------------------------------------------------------------------|
| `BinaryChromosome`     | Represents a binary vector solution                                         |
| `Genotype`            | Specifies chromosome structure and allele range                             |
| `Ga`                  | Main genetic algorithm runner                                               |
| `ExtensionStrategy`   | Configures population extension behavior, e.g., deduplication               |
| `MassDeduplication`   | Deduplicates similar individuals to maintain diversity                      |
| `Callback`            | Allows custom logic during GA execution                                     |

For detailed API documentation, refer to the [API Reference](./api.md).
