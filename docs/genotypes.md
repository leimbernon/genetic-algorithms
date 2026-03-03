# Genotypes

> Definitions and usage of gene types for representing genetic information.

## Overview

The `genotypes` module provides core types for representing genetic information in a genetic algorithm. Genotypes are the building blocks of chromosomes, which in turn form individuals in a population. This module offers two standard gene types: `Binary` and `Range<T>`, each suited to different problem domains. 

The `Binary` gene represents a simple on/off or true/false value, commonly used in problems like the classic binary knapsack or bitstring optimization. The `Range<T>` gene supports values within specified ranges, making it suitable for problems where genes can take on a range of numeric values, such as parameter optimization or scheduling.

Both gene types implement the `GeneT` trait, which defines the interface required for integration with the genetic algorithm framework. Users can import and use these genotypes directly, or define their own custom gene types by implementing the `GeneT` trait.

## Key Concepts

The genotypes module exports the following core types:

| Type           | Description                                                    |
|----------------|---------------------------------------------------------------|
| `Binary`       | Represents a gene with a boolean value (`true`/`false`).      |
| `Range<T>`     | Represents a gene with a value in one or more ranges.         |

### Binary Gene

| Field     | Type    | Description                         |
|-----------|---------|-------------------------------------|
| `id`      | `i32`   | Unique identifier for the gene.     |
| `value`   | `bool`  | The binary value of the gene.       |

### Range Gene

| Field     | Type            | Description                                  |
|-----------|----------------|----------------------------------------------|
| `id`      | `i32`           | Unique identifier for the gene.              |
| `ranges`  | `Vec<(T, T)>`   | List of allowed value ranges for the gene.   |
| `value`   | `T`             | The value of the gene within its ranges.     |

### Trait Bounds for `Range<T>`

- `Default` is required for construction.
- `Sync + Send + Clone + Default` are required for trait implementations.
- `Clone + Default` are required for inherent methods.

### The Genotypes Module

The top-level `genotypes` module re-exports `Binary` and `Range`, making them available for use:

```rust
pub use genetic_algorithms::genotypes::{Binary, Range};
```

## Usage

### Basic Example

```rust
use genetic_algorithms::genotypes::{Binary, Range};

// Create a Binary gene
let mut binary_gene = Binary { id: 1, value: false };
binary_gene.set_id(2).set_value(true);
assert_eq!(binary_gene.get_id(), 2);
assert_eq!(binary_gene.get_value(), true);

// Create a Range gene for integer values
let ranges = vec![(0, 10), (20, 30)];
let mut range_gene = Range::new(1, ranges.clone(), 5);
range_gene.set_id(2).set_value(25);
assert_eq!(range_gene.get_id(), 2);
assert_eq!(range_gene.get_value(), 25);
```

### Advanced Example

```rust
use genetic_algorithms::genotypes::{Binary, Range};
use genetic_algorithms::traits::GeneT;

// Example: Custom mutation function for a population of mixed genes
fn mutate_binary_gene(gene: &mut Binary) {
    let current = gene.get_value();
    gene.set_value(!current);
}

fn mutate_range_gene<T: Clone + Default>(gene: &mut Range<T>, new_value: T) {
    gene.set_value(new_value);
}

fn main() {
    let mut bin = Binary { id: 0, value: false };
    mutate_binary_gene(&mut bin);
    assert_eq!(bin.get_value(), true);

    let mut rng = Range::new(0, vec![(1, 10)], 5);
    mutate_range_gene(&mut rng, 7);
    assert_eq!(rng.get_value(), 7);
}
```

## API Reference

### Module: `genotypes`

Exports the following types:

| Name      | Type                | Description                                 |
|-----------|---------------------|---------------------------------------------|
| `Binary`  | struct              | Boolean gene type.                          |
| `Range`   | struct (generic)    | Range-based gene type.                      |

---

### `Binary`

Represents a gene with a boolean value.

**Fields:**

| Name     | Type    | Description                         |
|----------|---------|-------------------------------------|
| `id`     | `i32`   | Unique identifier for the gene.     |
| `value`  | `bool`  | The binary value of the gene.       |

**Methods:**

| Name         | Signature                                   | Description                                  |
|--------------|---------------------------------------------|----------------------------------------------|
| `new`        | `fn new(&mut self, id: i32, value: bool) -> &mut Self` | Creates a new `Binary` gene with given id and value. |
| `get_id`     | `fn get_id(&self) -> i32`                   | Returns the gene's identifier.               |
| `set_id`     | `fn set_id(&mut self, id: i32) -> &mut Self`| Sets the gene's identifier.                  |
| `get_value`  | `fn get_value(&self) -> bool`               | Returns the binary value.                    |
| `set_value`  | `fn set_value(&mut self, value: bool) -> &mut Self` | Sets the binary value.                       |

**Trait Implementations:**

- Implements `GeneT`.

---

### `Range<T>`

Represents a gene with a value in one or more ranges.

**Fields:**

| Name     | Type            | Description                                  |
|----------|-----------------|----------------------------------------------|
| `id`     | `i32`           | Unique identifier for the gene.              |
| `ranges` | `Vec<(T, T)>`   | List of allowed value ranges.                |
| `value`  | `T`             | The value of the gene within its ranges.     |

**Methods:**

| Name         | Signature                                                        | Description                                  |
|--------------|------------------------------------------------------------------|----------------------------------------------|
| `new`        | `fn new(id: i32, ranges: Vec<(T, T)>, value: T) -> Self`         | Creates a new `Range` gene.                  |
| `get_id`     | `fn get_id(&self) -> i32`                                        | Returns the gene's identifier.               |
| `set_id`     | `fn set_id(&mut self, id: i32) -> &mut Self`                     | Sets the gene's identifier.                  |
| `get_value`  | `fn get_value(&self) -> T`                                       | Returns the value of the gene.               |
| `set_value`  | `fn set_value(&mut self, value: T) -> &mut Self`                 | Sets the value of the gene.                  |

**Trait Implementations:**

- Implements `GeneT` for `T: Sync + Send + Clone + Default`.
- Implements `Default` for `T: Default`.

**Trait Bounds:**

- Construction: `T: Default`
- Trait implementation: `T: Sync + Send + Clone + Default`
- Inherent methods: `T: Clone + Default`

---

### `GeneT` Trait (for custom genotypes)

To define your own genotype type, implement the `GeneT` trait:

```rust
pub trait GeneT {
    fn get_id(&self) -> i32;
    fn set_id(&mut self, id: i32) -> &mut Self;
}
```

You may also add custom fields and methods as needed.

## Related

- [traits.md](traits.md) — Core traits, including `GeneT`
- [chromosomes.md](chromosomes.md) — Chromosome types and composition
- [src/genotypes.rs](../src/genotypes.rs)
- [src/genotypes/binary.rs](../src/genotypes/binary.rs)
- [src/genotypes/range.rs](../src/genotypes/range.rs)
- [examples.md](examples.md) — End-to-end usage examples

**Note:** For custom genotype types, implement the `GeneT` trait and ensure your type satisfies any trait bounds required by the genetic algorithm framework.
