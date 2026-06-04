# Chromosome Types

> Defines the core chromosome structures for genetic algorithms, including binary and range-based representations.

## Overview

Chromosomes are fundamental building blocks in genetic algorithms, representing candidate solutions as sequences of genes. This module provides two primary chromosome types: `Binary` and `Range`. Each type is tailored for different problem domains and supports essential operations such as fitness evaluation, mutation, and phenotype extraction.

The `Binary` chromosome is suitable for problems where solutions can be encoded as sequences of bits (true/false values), such as feature selection or classic optimization tasks. The `Range` chromosome, on the other hand, is designed for scenarios where genes represent values within specified ranges, making it ideal for parameter optimization and numerical problems.

Both chromosome types implement the `ChromosomeT` trait, ensuring compatibility with the library's genetic algorithm framework. Users can customize fitness functions, manipulate DNA, and integrate these chromosomes into their own evolutionary workflows.

## Key Concepts

### Chromosome Types

| Type      | Description                                            | Typical Use Cases             |
|-----------|--------------------------------------------------------|-------------------------------|
| `Binary`  | Encodes DNA as a vector of binary genes (`true`/`false`)| Feature selection, bitstring optimization |
| `Range<T>`| Encodes DNA as a vector of ranged genes (`T` values)    | Parameter optimization, numeric GA        |

### Fields

#### Binary

| Field         | Type                                 | Description                                 |
|---------------|--------------------------------------|---------------------------------------------|
| `dna`         | `Vec<BinaryGenotype>`                | Sequence of binary genes                    |
| `fitness`     | `f64`                                | Fitness score of the chromosome             |
| `age`         | `i32`                                | Age of the chromosome                       |
| `fitness_fn`  | `FitnessFnWrapper<BinaryGenotype>`   | Fitness function used for evaluation        |

#### Range

| Field         | Type                                        | Description                                 |
|---------------|---------------------------------------------|---------------------------------------------|
| `dna`         | `Vec<RangeGenotype<T>>`                     | Sequence of ranged genes                    |
| `fitness`     | `f64`                                       | Fitness score of the chromosome             |
| `age`         | `i32`                                       | Age of the chromosome                       |
| `fitness_fn`  | `FitnessFnWrapper<RangeGenotype<T>>`        | Fitness function used for evaluation        |

### Generic Parameter `T` in `Range<T>`

- `T` must implement `Sync`, `Send`, `Clone`, `Default`, and `Debug`.
- Typical choices: numeric types such as `i32`, `f64`.
- Allows chromosomes to represent genes with arbitrary value types within specified ranges.

### Mutation Traits

| Chromosome | Mutation Trait         | Supported Mutation Methods      |
|------------|-----------------------|---------------------------------|
| `Binary`   | `ValueMutable`        | `bit_flip_mutate`               |
| `Range`    | _(Not implemented)_   | _(Mutation must be implemented separately)_ |

## Usage

### Basic Example

```rust
use genetic_algorithms::chromosomes::{Binary, Range};
use genetic_algorithms::genotypes::{Binary as BinaryGenotype, Range as RangeGenotype};
use genetic_algorithms::traits::ChromosomeT;

// Binary chromosome example
let mut binary = Binary::new();
binary.dna = vec![BinaryGenotype::True, BinaryGenotype::False, BinaryGenotype::True];
binary.set_fitness_fn(|dna| dna.iter().filter(|g| **g == BinaryGenotype::True).count() as f64);
binary.calculate_fitness();
println!("Binary fitness: {}", binary.get_fitness());

// Range chromosome example
let mut range = Range::<i32>::new();
range.dna.push(RangeGenotype::new(0, vec![(0, 10)], 5));
range.set_fitness_fn(|dna| dna.iter().map(|g| g.value as f64).sum());
range.calculate_fitness();
println!("Range fitness: {}", range.get_fitness());
```

### Advanced Example

```rust
use genetic_algorithms::chromosomes::{Binary, Range};
use genetic_algorithms::genotypes::{Binary as BinaryGenotype, Range as RangeGenotype};
use genetic_algorithms::traits::ChromosomeT;

// Binary chromosome with mutation and phenotype
let mut binary = Binary::new();
binary.dna_from_string("10101").unwrap();
binary.set_fitness_fn(|dna| dna.iter().filter(|g| **g == BinaryGenotype::True).count() as f64);
binary.calculate_fitness();
println!("Binary phenotype: {}", binary.phenotype());
binary.bit_flip_mutate();
println!("Mutated DNA: {:?}", binary.dna);

// Range chromosome with custom fitness and phenotype
let mut range = Range::<i32>::new();
range.dna.push(RangeGenotype::new(0, vec![(0, 10)], 7));
range.dna.push(RangeGenotype::new(1, vec![(0, 5)], 3));
range.set_fitness_fn(|dna| dna.iter().map(|g| g.value as f64).sum());
range.calculate_fitness();
println!("Range phenotype: {}", range.phenotype());
```

## API Reference

### `Binary`

A chromosome using a binary genotype (bitstring).

**Fields:**

| Name         | Type                        | Description                        |
|--------------|-----------------------------|------------------------------------|
| `dna`        | `Vec<BinaryGenotype>`       | Sequence of binary genes           |
| `fitness`    | `f64`                       | Fitness score                      |
| `age`        | `i32`                       | Age of the chromosome              |
| `fitness_fn` | `FitnessFnWrapper<BinaryGenotype>` | Fitness function wrapper   |

**Methods:**

| Name                | Signature                                                                                      | Description                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------|
| `new`               | `fn new() -> Self`                                                                            | Creates a new `Binary` chromosome with default values.                                       |
| `phenotype`         | `fn phenotype(&self) -> String`                                                               | Returns the phenotype (string representation) of the chromosome.                             |
| `dna_from_string`   | `fn dna_from_string(&mut self, s: &str) -> Result<(), GaError>`                               | Sets DNA from a string of '1'/'0'. Returns error if invalid input.                           |
| `get_dna`           | `fn get_dna(&self) -> &[BinaryGenotype]`                                                      | Returns a reference to the DNA sequence.                                                     |
| `set_dna`           | `fn set_dna<'a>(&mut self, dna: Cow<'a, [BinaryGenotype]>) -> &mut Self`                      | Sets the DNA sequence (clones or moves depending on ownership).                              |
| `set_fitness_fn`    | `fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self`                                 | Sets the fitness function.                                                                   |
| `calculate_fitness` | `fn calculate_fitness(&mut self)`                                                             | Calculates and updates the fitness score using the fitness function.                         |
| `get_fitness`       | `fn get_fitness(&self) -> f64`                                                                | Returns the current fitness score.                                                           |
| `set_fitness`       | `fn set_fitness(&mut self, fitness: f64) -> &mut Self`                                        | Sets the fitness score manually.                                                             |
| `set_age`           | `fn set_age(&mut self, age: i32) -> &mut Self`                                                | Sets the chromosome's age.                                                                   |
| `get_age`           | `fn get_age(&self) -> i32`                                                                    | Returns the chromosome's age.                                                                |
| `bit_flip_mutate`   | `fn bit_flip_mutate(&mut self)`                                                               | Applies bit-flip mutation to the DNA.                                                        |

**Trait Implementations:**

- `ChromosomeT` (core chromosome trait)
- `ValueMutable` (mutation trait, supports `bit_flip_mutate`)

**Error Handling:**

- `dna_from_string` returns `Err(GaError::ValidationError)` if input contains characters other than '1' or '0'.

### `Range<T>`

A chromosome using a range genotype. Generic over `T`.

**Fields:**

| Name         | Type                                    | Description                        |
|--------------|-----------------------------------------|------------------------------------|
| `dna`        | `Vec<RangeGenotype<T>>`                 | Sequence of ranged genes           |
| `fitness`    | `f64`                                   | Fitness score                      |
| `age`        | `i32`                                   | Age of the chromosome              |
| `fitness_fn` | `FitnessFnWrapper<RangeGenotype<T>>`    | Fitness function wrapper           |

**Methods:**

| Name                | Signature                                                                                      | Description                                                                                  |
|---------------------|-----------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------|
| `new`               | `fn new() -> Self`                                                                            | Creates a new `Range` chromosome with default values.                                        |
| `phenotype`         | `fn phenotype(&self) -> String`                                                               | Returns the phenotype (string representation) of the chromosome.                             |
| `get_dna`           | `fn get_dna(&self) -> &[RangeGenotype<T>]`                                                    | Returns a reference to the DNA sequence.                                                     |
| `set_dna`           | `fn set_dna<'a>(&mut self, dna: Cow<'a, [RangeGenotype<T>]>) -> &mut Self`                    | Sets the DNA sequence (clones or moves depending on ownership).                              |
| `set_fitness_fn`    | `fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self`                                 | Sets the fitness function.                                                                   |
| `calculate_fitness` | `fn calculate_fitness(&mut self)`                                                             | Calculates and updates the fitness score using the fitness function.                         |
| `get_fitness`       | `fn get_fitness(&self) -> f64`                                                                | Returns the current fitness score.                                                           |
| `set_fitness`       | `fn set_fitness(&mut self, fitness: f64) -> &mut Self`                                        | Sets the fitness score manually.                                                             |
| `set_age`           | `fn set_age(&mut self, age: i32) -> &mut Self`                                                | Sets the chromosome's age.                                                                   |
| `get_age`           | `fn get_age(&self) -> i32`                                                                    | Returns the chromosome's age.                                                                |

**Trait Implementations:**

- `ChromosomeT` (core chromosome trait)
- _(Mutation trait not implemented; mutation must be handled externally or via custom operator.)_

**Generic Parameter `T`:**

- Must implement: `Sync`, `Send`, `Clone`, `Default`, `Debug`
- Typical values: numeric types (`i32`, `f64`)
- Used for gene values within specified ranges

**Error Handling:**

- Methods do not return errors directly; DNA manipulation and fitness calculation are assumed to be infallible unless custom fitness functions introduce errors.

## ChromosomeLength

`ChromosomeLength` controls whether a chromosome's DNA length is fixed or allowed to vary during evolution. It is used by `Mutation::Insertion` and `Mutation::Deletion` to enforce length bounds, and by `Crossover::VariableLength` to describe alignment intent.

```rust
use genetic_algorithms::chromosomes::ChromosomeLength;

// Fixed: length-changing mutations return GaError::MutationError.
let fixed = ChromosomeLength::Fixed(10);

// Variable: Insertion grows up to max, Deletion shrinks down to min.
let variable = ChromosomeLength::Variable { min: 2, max: 20 };
```

**Variants:**

| Variant | Description |
|---------|-------------|
| `Fixed(usize)` | Chromosome length is constant. `Insertion` and `Deletion` return `GaError::MutationError`. |
| `Variable { min, max }` | Chromosome length may vary between `min` and `max` (inclusive). |

**Builder integration:**

```rust
use genetic_algorithms::chromosomes::ChromosomeLength;
use genetic_algorithms::operations::Mutation;
use genetic_algorithms::traits::MutationConfig;

let mut ga = Ga::new()
    .with_mutation_method(Mutation::Insertion)
    .with_chromosome_length(ChromosomeLength::Variable { min: 2, max: 20 })
    .with_length_penalty(0.005)   // optional parsimony pressure
    // ...
    .build()
    .unwrap();
```

**Added in:** v3.0.0

## GpChromosome

`GpChromosome<N>` is the library-provided tree chromosome for Genetic Programming. It implements `ChromosomeT` but **not** `LinearChromosome` — `dna()`, `dna_mut()`, and `set_dna()` all panic. Use `GpGa<N>`, not `Ga<U>`, for tree chromosomes.

See [Genetic Programming guide](gp.md) for complete documentation.

**Key types in `gp` module:**

| Type | Description |
|------|-------------|
| `GpChromosome<N>` | Concrete tree chromosome wrapping a `Node<N>` |
| `GpGene` | Marker gene type (zero-sized; satisfies `ChromosomeT::Gene` bound) |
| `TreeChromosome` | Supertrait of `ChromosomeT` for tree chromosomes — provides `tree()`, `tree_mut()`, `depth()`, `node_count()` |
| `Node<N>` | Recursive expression tree (`Function { value, children }` or `Terminal(N)`) |
| `GpNode` | Trait to implement on your primitive-set enum |

**Added in:** v3.0.0

## Related

- [Genotypes Documentation](genotypes.md)
- [Traits Documentation](traits.md)
- [Genetic Programming](gp.md)
- [Mutation Operators](operators/mutation.md)
- [Selection Operators](operators/selection.md)
- [Source: `src/types/chromosomes/mod.rs`](../src/types/chromosomes/mod.rs)
- [Source: `src/engines/gp/chromosome.rs`](../src/engines/gp/chromosome.rs)
- [Examples](examples.md)
