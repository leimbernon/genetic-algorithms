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
| `multipoint`     | Any                 | Uses multiple crossover points for segment swapping.      |
| `cycle`          | Permutation         | Preserves absolute positions from parent chromosomes.     |
| `order`          | Permutation         | Maintains relative ordering of genes.                     |
| `pmx`            | Permutation         | Partially mapped crossover for permutations.              |
| `sbx`            | Range               | Simulated binary crossover for real-valued chromosomes.   |
| `blend_alpha`    | Range               | Blend crossover for real-valued chromosomes.              |
| `arithmetic`     | Range               | Arithmetic combination of parent genes.                   |
| `clone`          | Any                 | Copies parents directly as offspring (no genetic exchange).|

### Clone Crossover

The **Clone crossover** operator produces offspring that are exact copies of the parent chromosomes, without any genetic recombination. This operator is useful in scenarios where crossover should be a no-op, such as mutation-only evolutionary strategies or baseline experiments to isolate the effect of crossover.

#### Use Cases

- **Mutation-only strategies:** When you want the evolutionary process to rely solely on mutation for introducing variation, Clone crossover ensures offspring are identical to parents before mutation is applied.
- **Baseline experiments:** Useful for benchmarking and isolating the impact of crossover versus mutation.

#### Expected Behavior

Given two parents, `P1` and `P2`, Clone crossover produces two children:
- `child_1` is a clone of `P1`
- `child_2` is a clone of `P2`

No genetic material is exchanged between parents.

#### Error Handling

Clone crossover returns an error if the parent chromosomes have different DNA lengths. This ensures offspring are valid clones and prevents mismatches.

## Usage

To configure and use the Clone crossover operator, set the crossover strategy to `Crossover::Clone` in your algorithm configuration. The operator can be invoked directly or via the factory function.

### Example: Using Clone Crossover

```rust
use ga_lib::operations::{Crossover, crossover};
use ga_lib::chromosome::ChromosomeT;
use ga_lib::error::GaError;

// Assume MyChromosome implements ChromosomeT and has a fixed length
let parent1 = MyChromosome::new(vec![1, 2, 3, 4]);
let parent2 = MyChromosome::new(vec![5, 6, 7, 8]);

let crossover_method = Crossover::Clone;
let offspring = crossover::factory(&crossover_method, &parent1, &parent2);

match offspring {
    Ok(children) => {
        assert_eq!(children[0], parent1); // child_1 is a clone of parent1
        assert_eq!(children[1], parent2); // child_2 is a clone of parent2
    }
    Err(GaError::CrossoverError) => {
        // Handle DNA length mismatch
    }
    Err(e) => {
        // Handle other errors
    }
}
```

### Configuring Clone Crossover in Algorithm Setup

When constructing your genetic algorithm, specify the crossover operator:

```rust
use ga_lib::operations::Crossover;

let crossover_operator = Crossover::Clone;
// Pass this operator to your GA configuration
```

## API Reference

### Enum: `Crossover`

```rust
pub enum Crossover {
    // ... other variants ...
    /// Clone crossover — copies parents directly as offspring without any genetic exchange.
    /// Useful for mutation-only strategies and baseline experiments.
    Clone,
}
```

### Function: `clone_crossover`

```rust
pub fn clone_crossover<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```
- **Description:** Produces offspring that are exact clones of the parent chromosomes.
- **Errors:** Returns `Err(GaError::CrossoverError)` if parents have different DNA lengths.

### Factory Function

```rust
pub fn factory<U: ChromosomeT>(
    crossover: &Crossover,
    parent_1: &U,
    parent_2: &U,
) -> Result<Vec<U>, GaError>
```
- **Description:** Dispatches to the correct crossover implementation based on the `Crossover` variant.

---

For further details on operator configuration and advanced usage, refer to the main genetic algorithm documentation and the `ChromosomeT` trait.
