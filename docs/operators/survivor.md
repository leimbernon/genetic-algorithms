# Survivor Selection Operators

> Strategies for choosing which individuals survive to the next generation.

## Overview

Survivor selection operators determine which individuals from the current population are retained for the next generation in a genetic algorithm. This process is crucial for controlling population diversity, convergence speed, and overall algorithm performance. Survivor selection is typically applied after offspring have been generated through crossover and mutation, ensuring that the population size remains constant and that only the most promising individuals continue.

The library provides two main survivor selection strategies: age-based and fitness-based. Age-based selection removes the oldest individuals, promoting generational turnover and diversity. Fitness-based selection retains the most fit individuals, driving the population towards optimal solutions more aggressively. The choice of survivor selection method can significantly affect the behavior and results of your genetic algorithm.

Survivor selection is a modular component of the library, allowing users to choose or implement custom strategies as needed. It integrates seamlessly with the population management and configuration systems, ensuring flexibility and ease of use.

## Key Concepts

The survivor selection module exposes the following key functions and concepts:

| Function         | Parameters                                              | Description                                  |
|------------------|--------------------------------------------------------|----------------------------------------------|
| `age_based`      | `chromosomes: &mut Vec<U>, population_size: usize`     | Removes oldest individuals to reach target size. |
| `fitness_based`  | `chromosomes: &mut Vec<U>, population_size: usize`     | Retains individuals with highest fitness.    |
| `factory`        | `chromosomes: &mut Vec<U>, config: &LimitConfiguration`| Dispatches survivor selection by configuration. |

**Traits Used:**

| Trait         | Description                                  |
|---------------|----------------------------------------------|
| `ChromosomeT` | Required for all survivor selection methods. |

## Usage

### Basic Example

```rust
use ga_lib::chromosomes::BinaryChromosome;
use ga_lib::operations::survivor::{age_based, fitness_based};

fn main() {
    // Create a population of chromosomes
    let mut population: Vec<BinaryChromosome> = (0..10)
        .map(|_| BinaryChromosome::random(64))
        .collect();

    // Age-based survivor selection: keep only 5 youngest
    age_based(&mut population, 5);
    assert_eq!(population.len(), 5);

    // Fitness-based survivor selection: keep top 3 by fitness
    fitness_based(&mut population, 3);
    assert_eq!(population.len(), 3);
}
```

### Advanced Example

```rust
use ga_lib::chromosomes::RangeChromosome;
use ga_lib::operations::survivor::{factory};
use ga_lib::configuration::{LimitConfiguration, SurvivorSelectionMethod};

fn main() {
    // Create a population of range chromosomes
    let mut population: Vec<RangeChromosome> = (0..20)
        .map(|_| RangeChromosome::random(10, 0.0..1.0))
        .collect();

    // Configure survivor selection to use fitness-based method
    let config = LimitConfiguration {
        population_size: 8,
        survivor_selection: SurvivorSelectionMethod::FitnessBased,
        ..Default::default()
    };

    // Dispatch survivor selection according to configuration
    factory(&mut population, &config).expect("Survivor selection failed");

    assert_eq!(population.len(), 8);
}
```

## API Reference

### `age_based`

Removes the oldest individuals from the population until the desired population size is reached.

**Signature:**
```rust
pub fn age_based<U: ChromosomeT>(chromosomes: &mut Vec<U>, population_size: usize)
```

| Parameter         | Type                  | Description                                 |
|-------------------|----------------------|---------------------------------------------|
| `chromosomes`     | `&mut Vec<U>`        | Population to trim                          |
| `population_size` | `usize`              | Target population size                      |

### `fitness_based`

Retains the individuals with the highest fitness scores up to the target population size.

**Signature:**
```rust
pub fn fitness_based<U: ChromosomeT>(chromosomes: &mut Vec<U>, population_size: usize)
```

| Parameter         | Type                  | Description                                 |
|-------------------|----------------------|---------------------------------------------|
| `chromosomes`     | `&mut Vec<U>`        | Population to trim                          |
| `population_size` | `usize`              | Target population size                      |

### `factory`

Dispatches survivor selection according to the configured method.

**Signature:**
```rust
pub fn factory<U: ChromosomeT>(chromosomes: &mut Vec<U>, config: &LimitConfiguration) -> Result<(), GaError>
```

| Parameter         | Type                          | Description                                 |
|-------------------|------------------------------|---------------------------------------------|
| `chromosomes`     | `&mut Vec<U>`                | Population to trim                          |
| `config`          | `&LimitConfiguration`         | Survivor selection configuration            |

## Related

- [operators/selection.md](selection.md) — Selection operators for parent selection
- [configuration.md](../configuration.md) — Survivor selection configuration options
- [chromosomes.md](../chromosomes.md) — Chromosome types supported
- [src/operations/survivor.rs](../../src/operations/survivor.rs)
- [src/operations/survivor/age.rs](../../src/operations/survivor/age.rs)
- [src/operations/survivor/fitness.rs](../../src/operations/survivor/fitness.rs)
