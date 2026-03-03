# Crossover Operators

> Genetic algorithm operators for combining parent solutions to produce offspring.

## Overview

Crossover operators are a fundamental component of genetic algorithms, responsible for combining genetic material from two or more parent chromosomes to generate new offspring. The choice of crossover strategy can significantly affect the diversity and convergence properties of the population, making it crucial for effective evolutionary search.

This module provides several crossover techniques, each tailored to different chromosome representations and problem domains. For example, uniform and single-point crossover are commonly used for binary or fixed-length chromosomes, while cycle and order crossover are designed for permutation-based problems. Advanced operators like blend-alpha and simulated binary crossover (SBX) support real-valued chromosomes, expanding the library's applicability.

Crossover operators are typically invoked during the reproduction phase of a genetic algorithm, after parent selection and before mutation. Users can select and configure the appropriate crossover operator based on their problem requirements, and even compose custom strategies using the provided factory function.

## Key Concepts

The crossover module exposes several public functions and types for combining chromosomes. Each operator has specific requirements and behaviors:

| Operator         | Chromosome Type      | Description                                               |
|------------------|---------------------|-----------------------------------------------------------|
| `uniform`        | Any (`ChromosomeT`) | Randomly selects genes from either parent for each locus. |
| `single_point`   | Any                 | Splits parents at a random point and swaps segments.      |
| `multipoint`     | Any                 | Uses multiple crossover points for segment swapping.       |
| `cycle`          | Permutation         | Preserves absolute positions from parents (cycle method).  |
| `order`          | Permutation         | Maintains relative order from parents (order method).      |
| `blend_alpha`    | Real-valued         | Blends gene values using an alpha parameter.              |
| `sbx`            | Real-valued         | Simulated binary crossover for continuous genes.           |
| `factory`        | Any                 | Returns a configured crossover operator function.          |
| `aga_probability`| Any                 | Computes crossover probability for adaptive GA.            |

### Common Parameters

| Parameter        | Type                | Description                                               |
|------------------|---------------------|-----------------------------------------------------------|
| `parent_1`       | `&U`                | First parent chromosome                                   |
| `parent_2`       | `&U`                | Second parent chromosome                                  |
| `points`         | `usize`             | Number of crossover points (for multipoint)               |
| `alpha`          | `f64`               | Blending factor (for blend-alpha)                         |
| `probability`    | `f64`               | Crossover probability (for factory, aga_probability)      |

## Usage

### Basic Example

```rust
use ga_lib::operations::crossover::uniform;
use ga_lib::chromosomes::BinaryChromosome;

fn main() -> Result<(), ga_lib::GaError> {
    let parent1 = BinaryChromosome::from_bits(vec![true, false, true, false]);
    let parent2 = BinaryChromosome::from_bits(vec![false, true, false, true]);
    let offspring = uniform(&parent1, &parent2)?;
    println!("Offspring: {:?}", offspring);
    Ok(())
}
```

### Advanced Example

```rust
use ga_lib::operations::crossover::{cycle, blend_alpha, factory};
use ga_lib::chromosomes::{PermutationChromosome, RealChromosome};

fn main() -> Result<(), ga_lib::GaError> {
    // Permutation crossover
    let parent1 = PermutationChromosome::new(vec![1, 2, 3, 4, 5]);
    let parent2 = PermutationChromosome::new(vec![5, 4, 3, 2, 1]);
    let offspring_perm = cycle(&parent1, &parent2)?;
    println!("Cycle Crossover Offspring: {:?}", offspring_perm);

    // Real-valued crossover
    let parent1 = RealChromosome::new(vec![0.1, 0.5, 0.9]);
    let parent2 = RealChromosome::new(vec![0.9, 0.5, 0.1]);
    let offspring_real = blend_alpha(&parent1, &parent2, 0.7)?;
    println!("Blend-Alpha Offspring: {:?}", offspring_real);

    // Using factory to configure a crossover operator
    let crossover_fn = factory::<PermutationChromosome>("cycle", None)?;
    let offspring = crossover_fn(&parent1, &parent2)?;
    println!("Factory Offspring: {:?}", offspring);

    Ok(())
}
```

## API Reference

### `uniform`

Performs uniform crossover between two parent chromosomes. Each gene is randomly selected from either parent.

**Signature:**
```rust
pub fn uniform<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

| Parameter   | Type           | Description                      |
|-------------|----------------|----------------------------------|
| `parent_1`  | `&U`           | First parent chromosome          |
| `parent_2`  | `&U`           | Second parent chromosome         |

Returns: `Result<Vec<U>, GaError>` — Vector of offspring chromosomes.

---

### `single_point`

Performs single-point crossover, splitting parents at a random point.

**Signature:**
```rust
pub fn single_point<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

| Parameter   | Type           | Description                      |
|-------------|----------------|----------------------------------|
| `parent_1`  | `&U`           | First parent chromosome          |
| `parent_2`  | `&U`           | Second parent chromosome         |

Returns: `Result<Vec<U>, GaError>`

---

### `multipoint`

Performs multipoint crossover using a specified number of crossover points.

**Signature:**
```rust
pub fn multipoint<U: ChromosomeT>(parent_1: &U, parent_2: &U, points: usize) -> Result<Vec<U>, GaError>
```

| Parameter   | Type           | Description                      |
|-------------|----------------|----------------------------------|
| `parent_1`  | `&U`           | First parent chromosome          |
| `parent_2`  | `&U`           | Second parent chromosome         |
| `points`    | `usize`        | Number of crossover points       |

Returns: `Result<Vec<U>, GaError>`

---

### `cycle`

Implements cycle crossover for permutation chromosomes, preserving absolute positions.

**Signature:**
```rust
pub fn cycle<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

| Parameter   | Type           | Description                      |
|-------------|----------------|----------------------------------|
| `parent_1`  | `&U`           | First parent chromosome          |
| `parent_2`  | `&U`           | Second parent chromosome         |

Returns: `Result<Vec<U>, GaError>`

---

### `order`

Implements order crossover for permutation chromosomes, preserving relative order.

**Signature:**
```rust
pub fn order<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

| Parameter   | Type           | Description                      |
|-------------|----------------|----------------------------------|
| `parent_1`  | `&U`           | First parent chromosome          |
| `parent_2`  | `&U`           | Second parent chromosome         |

Returns: `Result<Vec<U>, GaError>`

---

### `blend_alpha`

Performs blend-alpha crossover for real-valued chromosomes, blending gene values using an alpha parameter.

**Signature:**
```rust
pub fn blend_alpha<U: ChromosomeT>(parent_1: &U, parent_2: &U, alpha: f64) -> Result<Vec<U>, GaError>
```

| Parameter   | Type           | Description                      |
|-------------|----------------|----------------------------------|
| `parent_1`  | `&U`           | First parent chromosome          |
| `parent_2`  | `&U`           | Second parent chromosome         |
| `alpha`     | `f64`          | Blending factor (0.0 to 1.0)     |

Returns: `Result<Vec<U>, GaError>`

---

### `sbx`

Implements simulated binary crossover (SBX) for real-valued chromosomes.

**Signature:**
```rust
pub fn sbx<U: ChromosomeT>(parent_1: &U, parent_2: &U, distribution_index: f64) -> Result<Vec<U>, GaError>
```

| Parameter           | Type           | Description                          |
|---------------------|----------------|--------------------------------------|
| `parent_1`          | `&U`           | First parent chromosome              |
| `parent_2`          | `&U`           | Second parent chromosome             |
| `distribution_index`| `f64`          | SBX distribution index parameter     |

Returns: `Result<Vec<U>, GaError>`

---

### `factory`

Returns a configured crossover operator function based on a string identifier and optional parameters.

**Signature:**
```rust
pub fn factory<U: ChromosomeT>(name: &str, config: Option<CrossoverConfig>) -> Result<fn(&U, &U) -> Result<Vec<U>, GaError>, GaError>
```

| Parameter   | Type                       | Description                                      |
|-------------|----------------------------|--------------------------------------------------|
| `name`      | `&str`                     | Name of the crossover operator ("uniform", etc.) |
| `config`    | `Option<CrossoverConfig>`  | Optional configuration parameters                |

Returns: `Result<fn(&U, &U) -> Result<Vec<U>, GaError>, GaError>`

---

### `aga_probability`

Computes adaptive crossover probability for a chromosome.

**Signature:**
```rust
pub fn aga_probability<U: ChromosomeT>(chromosome: &U, base_probability: f64) -> f64
```

| Parameter         | Type           | Description                          |
|-------------------|----------------|--------------------------------------|
| `chromosome`      | `&U`           | Chromosome to evaluate               |
| `base_probability`| `f64`          | Base crossover probability           |

Returns: `f64` — Adjusted crossover probability.

## Related

- [Selection Operators](selection.md)
- [Mutation Operators](mutation.md)
- [Chromosome Types](../chromosomes.md)
- [Genotype Definitions](../genotypes.md)
- [Fitness Evaluation](../fitness.md)
- [Source: `src/operations/crossover.rs`](../../src/operations/crossover.rs)
- [Source: `src/operations/crossover/uniform_crossover.rs`](../../src/operations/crossover/uniform_crossover.rs)
- [Source: `src/operations/crossover/cycle.rs`](../../src/operations/crossover/cycle.rs)
- [Source: `src/operations/crossover/multipoint.rs`](../../src/operations/crossover/multipoint.rs)
- [Source: `src/operations/crossover/blend_alpha.rs`](../../src/operations/crossover/blend_alpha.rs)
- [Source: `src/operations/crossover/sbx.rs`](../../src/operations/crossover/sbx.rs)
- [Examples](../examples.md)
