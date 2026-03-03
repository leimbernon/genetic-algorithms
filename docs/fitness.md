# Fitness Evaluation

> Mechanisms for evaluating and customizing fitness functions in genetic algorithms.

## Overview

Fitness evaluation is a core component of any genetic algorithm, determining how well each candidate solution (chromosome) performs with respect to the problem being solved. The fitness function assigns a numerical value to each chromosome, guiding the selection and evolution process toward optimal solutions.

This module provides both built-in fitness functions and flexible abstractions for defining custom fitness logic. Whether you're working with binary chromosomes or more complex gene types, you can easily plug in your own fitness criteria or use the provided helpers for common cases.

Fitness evaluation integrates tightly with the overall library architecture, as every population member is scored using a fitness function. These scores influence selection, crossover, and survivor operations, making fitness functions a central customization point for adapting the genetic algorithm to your specific problem domain.

## Key Concepts

| Item                      | Type/Signature                           | Description                                                 |
|---------------------------|------------------------------------------|-------------------------------------------------------------|
| `count_true`              | `fn(&[Binary]) -> f64`                   | Counts the number of `true` bits in a binary chromosome.    |
| `FitnessFnWrapper<G>`     | `struct`                                 | Wraps a fitness function for any gene type.                 |
| `FitnessFnWrapper::new`   | `fn<F>(func: F) -> Self`                 | Constructs a wrapper from a user-provided function.         |
| `FitnessFnWrapper::call`  | `fn(&self, dna: &[G]) -> f64`            | Invokes the wrapped fitness function on a chromosome.       |

### Fitness Function Abstraction

| Field / Method   | Type / Signature                       | Description                                  |
|------------------|----------------------------------------|----------------------------------------------|
| `func`           | `Fn(&[G]) -> f64 + Send + Sync + 'static` | The user-defined fitness function.           |
| `new`            | Constructor                            | Creates a new wrapper from a function.       |
| `call`           | Method                                 | Evaluates fitness for a given chromosome.    |

## Usage

### Basic Example

```rust
use ga_lib::fitness::{count_true, FitnessFnWrapper};
use ga_lib::chromosomes::Binary;

// Using the built-in count_true fitness function for a binary chromosome
let chromosome: Vec<Binary> = vec![true, false, true, true, false];
let fitness = count_true(&chromosome);
println!("Fitness: {}", fitness); // Output: Fitness: 3
```

### Advanced Example

```rust
use ga_lib::fitness::FitnessFnWrapper;

// Define a custom fitness function for floating-point genes
fn sum_squares(dna: &[f64]) -> f64 {
    dna.iter().map(|x| x * x).sum()
}

// Wrap the custom function for use in the genetic algorithm
let fitness_fn = FitnessFnWrapper::new(sum_squares);

let chromosome = vec![1.0, -2.0, 3.0];
let fitness = fitness_fn.call(&chromosome);
println!("Fitness: {}", fitness); // Output: Fitness: 14.0
```

## API Reference

### `count_true`

Counts the number of `true` values in a binary chromosome.

| Name        | Signature                      | Description                                   |
|-------------|-------------------------------|-----------------------------------------------|
| `count_true`| `fn(dna: &[Binary]) -> f64`   | Returns the count of `true` bits as fitness.  |

### `FitnessFnWrapper<G>`

A thread-safe wrapper for user-defined fitness functions for any gene type.

**Traits Implemented:** `Clone`, `Default`, `Debug`, `PartialEq`

| Name        | Signature                                                                 | Description                                         |
|-------------|--------------------------------------------------------------------------|-----------------------------------------------------|
| `new`       | `fn<F>(func: F) -> Self`<br>where F: Fn(&[G]) -> f64 + Send + Sync + 'static | Constructs a wrapper from a user-provided function. |
| `call`      | `fn(&self, dna: &[G]) -> f64`                                            | Evaluates the fitness function on the chromosome.   |

## Related

- [chromosomes.md](chromosomes.md) — Chromosome types and representations
- [traits.md](traits.md) — Core traits for genetic algorithms
- [src/fitness.rs](../src/fitness.rs) — Fitness module source code
- [src/fitness/count_true.rs](../src/fitness/count_true.rs) — Built-in binary fitness function
- [src/fitness/fitness_fn_wrapper.rs](../src/fitness/fitness_fn_wrapper.rs) — Fitness function wrapper implementation
