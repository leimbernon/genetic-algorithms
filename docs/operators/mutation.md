# Mutation Operators

> Mutation operators introduce genetic diversity by altering chromosomes during evolution.

## Overview

Mutation is a fundamental mechanism in genetic algorithms, responsible for introducing random changes to individuals in a population. By modifying genes within chromosomes, mutation helps prevent premature convergence and enables exploration of the solution space. In this library, mutation operators are modular and support a variety of strategies tailored to different chromosome types, such as binary, range, and permutation-based representations.

Users typically configure mutation operators to balance exploration and exploitation in their genetic algorithm. The choice of mutation strategy depends on the problem domain and chromosome encoding. For example, swap, scramble, and inversion are well-suited for permutation problems, while value, creep, and gaussian mutations are designed for numeric range chromosomes. The mutation module provides a unified API for applying these operators, including adaptive mutation probability for advanced scenarios.

Mutation integrates seamlessly with the overall genetic algorithm workflow, working alongside selection and crossover operators. The library's mutation system is extensible, allowing users to implement custom mutation logic if needed.

## Key Concepts

Mutation operators are implemented as functions and traits, each targeting specific chromosome types. The core abstractions are summarized below:

| Operator        | Applicable Chromosome | Description                                               |
|-----------------|----------------------|-----------------------------------------------------------|
| `swap`          | Permutation          | Swaps two randomly chosen genes                           |
| `scramble`      | Permutation          | Randomly shuffles a subsequence of genes                  |
| `inversion`     | Permutation          | Reverses the order of a subsequence of genes              |
| `value_mutation`| Range                | Assigns a new random value to a gene within its range     |
| `creep_mutation`| Range                | Applies small uniform perturbation to a gene              |
| `gaussian_mutation`| Range             | Applies gaussian noise to a gene                          |
| `bit_flip_mutate`| Binary              | Flips a random bit in the chromosome                      |

### Trait: `ValueMutable`

| Method              | Signature                                       | Description                                              |
|---------------------|------------------------------------------------|----------------------------------------------------------|
| `value_mutate`      | `fn value_mutate(&mut self)`                    | Performs value mutation                                  |
| `bit_flip_mutate`   | `fn bit_flip_mutate(&mut self)`                 | Flips a random bit (for binary chromosomes)              |
| `creep_mutate`      | `fn creep_mutate(&mut self, step: f64)`         | Applies creep mutation                                   |
| `gaussian_mutate`   | `fn gaussian_mutate(&mut self, sigma: f64)`     | Applies gaussian mutation                                |

### Mutation Factory Functions

| Function                | Signature                                                                 | Description                                             |
|-------------------------|--------------------------------------------------------------------------|---------------------------------------------------------|
| `factory`               | `fn factory<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>` | Applies the specified mutation operator                 |
| `factory_with_params`   | `fn factory_with_params<U>(...)`                                         | Applies mutation with optional parameters               |
| `factory_non_value`     | `fn factory_non_value<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>` | Applies non-value mutation operators                    |
| `aga_probability`       | `fn aga_probability<U: ChromosomeT>(...) -> f64`                         | Calculates adaptive mutation probability                |

## Usage

### Basic Example

```rust
use ga_lib::chromosomes::PermutationChromosome;
use ga_lib::operators::mutation::{swap, scramble, inversion, factory, Mutation};

fn main() {
    // Create a permutation chromosome (e.g., for TSP)
    let mut chromosome = PermutationChromosome::new(vec![1, 2, 3, 4, 5]);

    // Apply swap mutation
    swap(&mut chromosome);

    // Apply scramble mutation
    scramble(&mut chromosome);

    // Apply inversion mutation
    inversion(&mut chromosome);

    // Use the mutation factory to apply a mutation by enum
    factory(Mutation::Swap, &mut chromosome).unwrap();
}
```

### Advanced Example

```rust
use ga_lib::chromosomes::{RangeChromosome, PermutationChromosome};
use ga_lib::operators::mutation::{factory_with_params, Mutation, ValueMutable};

fn main() {
    // Numeric chromosome for a continuous optimization problem
    let mut range_chromosome = RangeChromosome::<f64>::new(vec![(0.0, 10.0); 5]);

    // Apply value mutation
    range_chromosome.value_mutate();

    // Apply creep mutation with step size
    range_chromosome.creep_mutate(0.1);

    // Apply gaussian mutation with sigma
    range_chromosome.gaussian_mutate(0.5);

    // Use factory_with_params to apply gaussian mutation via unified API
    factory_with_params(
        Mutation::Gaussian,
        &mut range_chromosome,
        None,    // step (not used for gaussian)
        Some(0.5) // sigma
    ).unwrap();

    // Permutation chromosome for combinatorial optimization
    let mut perm_chromosome = PermutationChromosome::new(vec![1, 2, 3, 4, 5]);

    // Use factory to apply inversion mutation
    factory_with_params(
        Mutation::Inversion,
        &mut perm_chromosome,
        None,
        None
    ).unwrap();
}
```

## API Reference

### `ValueMutable` (Trait)

Trait for chromosomes that support value mutation. Automatically implemented for range chromosomes.

| Method              | Signature                                       | Description                                              |
|---------------------|------------------------------------------------|----------------------------------------------------------|
| `value_mutate`      | `fn value_mutate(&mut self)`                    | Performs value mutation                                  |
| `bit_flip_mutate`   | `fn bit_flip_mutate(&mut self)`                 | Flips a random bit (for binary chromosomes)              |
| `creep_mutate`      | `fn creep_mutate(&mut self, step: f64)`         | Applies creep mutation                                   |
| `gaussian_mutate`   | `fn gaussian_mutate(&mut self, sigma: f64)`     | Applies gaussian mutation                                |

#### Implementations

- Implemented for `RangeChromosome<i32>`, `RangeChromosome<i64>`, `RangeChromosome<f32>`, `RangeChromosome<f64>`

### `swap`

Swaps two randomly chosen genes in a permutation chromosome.

| Name   | Signature                                         | Description                  |
|--------|---------------------------------------------------|------------------------------|
| `swap` | `fn swap<U: ChromosomeT>(chromosome: &mut U)`     | Swaps two genes              |

### `scramble`

Randomly shuffles a subsequence of genes in a permutation chromosome.

| Name      | Signature                                         | Description                  |
|-----------|---------------------------------------------------|------------------------------|
| `scramble`| `fn scramble<U: ChromosomeT>(chromosome: &mut U)` | Scrambles a gene subsequence |

### `inversion`

Reverses the order of a subsequence of genes in a permutation chromosome.

| Name      | Signature                                             | Description                  |
|-----------|-------------------------------------------------------|------------------------------|
| `inversion`| `fn inversion<U: ChromosomeT>(individual: &mut U)`   | Inverts a gene subsequence   |

### `value_mutation`

Assigns a new random value to a gene within its allowed range.

| Name            | Signature                                               | Description                  |
|-----------------|--------------------------------------------------------|------------------------------|
| `value_mutation`| `fn value_mutation<T>(individual: &mut RangeChromosome<T>)` | Value mutation for range chromosomes |

### `factory`

Applies the specified mutation operator to a chromosome.

| Name     | Signature                                                           | Description                  |
|----------|---------------------------------------------------------------------|------------------------------|
| `factory`| `fn factory<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>` | Unified mutation dispatcher  |

### `factory_with_params`

Applies mutation with optional parameters (step for creep, sigma for gaussian).

| Name                | Signature                                      | Description                  |
|---------------------|------------------------------------------------|------------------------------|
| `factory_with_params`| `fn factory_with_params<U>(...)`               | Mutation with extra params   |

### `factory_non_value`

Applies non-value mutation operators (swap, inversion, scramble).

| Name               | Signature                                                           | Description                  |
|--------------------|---------------------------------------------------------------------|------------------------------|
| `factory_non_value`| `fn factory_non_value<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>` | Non-value mutation dispatcher|

### `aga_probability`

Calculates adaptive mutation probability for AGA.

| Name            | Signature                                      | Description                  |
|-----------------|------------------------------------------------|------------------------------|
| `aga_probability`| `fn aga_probability<U: ChromosomeT>(...) -> f64` | Adaptive mutation probability|

## Related

- [operators/selection.md](selection.md)
- [operators/crossover.md](crossover.md)
- [chromosomes.md](../chromosomes.md)
- [traits.md](../traits.md)
- [Source: mutation.rs](../../src/operations/mutation.rs)
- [Source: swap.rs](../../src/operations/mutation/swap.rs)
- [Source: scramble.rs](../../src/operations/mutation/scramble.rs)
- [Source: inversion.rs](../../src/operations/mutation/inversion.rs)
- [Source: value.rs](../../src/operations/mutation/value.rs)
