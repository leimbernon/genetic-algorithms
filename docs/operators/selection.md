# Selection Operators

> Parent selection strategies for genetic algorithms, including tournament, roulette, rank, and random methods.

## Overview

Selection operators determine which individuals from a population are chosen as parents for crossover in a genetic algorithm. The choice of selection method can significantly affect the convergence speed and diversity of solutions. This module provides several widely-used selection strategies: tournament selection, fitness-proportionate (roulette wheel and stochastic universal sampling), rank-based selection, and random selection.

Each selection operator has its own configuration options and trade-offs. For example, tournament selection allows you to control selection pressure via the tournament size, while fitness-proportionate methods rely on the fitness values assigned to each chromosome. Selection is a critical step in the genetic algorithm cycle, as it balances exploration and exploitation of the search space.

These operators are designed to work with any chromosome type that implements the required trait (`ChromosomeT`). Fitness values must be accessible via the chromosome interface, and the population must be large enough to form the requested number of parent pairs. Error handling is provided for cases where selection cannot be performed due to insufficient population size.

Selection operators are typically configured as part of the genetic algorithm setup and invoked automatically during each generation. Advanced users may also call them directly for custom workflows.

## Key Concepts

Selection operators work on slices of chromosomes and return pairs of parent indices. Chromosomes must implement the `ChromosomeT` trait, which provides access to fitness values.

| Operator                        | Description                                                      | Key Parameters           |
|----------------------------------|------------------------------------------------------------------|--------------------------|
| `tournament`                    | Selects parents via tournaments of random individuals            | `tournament_size: usize` |
| `roulette_wheel_selection`       | Probability proportional to fitness                              | —                        |
| `stochastic_universal_sampling`  | Evenly spaced sampling proportional to fitness                   | —                        |
| `rank_selection`                 | Probability proportional to rank in sorted population            | —                        |
| `random`                        | Selects parents uniformly at random                              | —                        |
| `factory`                       | Dispatches selection based on configuration                      | method, parameters       |

### Chromosome Trait Requirements

| Trait         | Required Methods                | Description                                |
|---------------|-------------------------------|--------------------------------------------|
| `ChromosomeT` | `fn fitness(&self) -> f64`     | Provides fitness value for selection        |

### Error Handling

| Error Type               | Description                                         |
|--------------------------|-----------------------------------------------------|
| `GaError::SelectionError`| Returned if population is too small for selection   |

## Usage

### Basic Example

```rust
use ga_lib::chromosomes::BinaryChromosome;
use ga_lib::operators::selection::tournament;

// Assume BinaryChromosome implements ChromosomeT and has fitness values set
let population: Vec<BinaryChromosome> = /* ... */;
let tournament_size = 3;
let parent_pairs = tournament(&population, tournament_size);
// parent_pairs: Vec<(usize, usize)> of selected parent indices
```

### Advanced Example

```rust
use ga_lib::chromosomes::RangeChromosome;
use ga_lib::operators::selection::{factory, SelectionMethod};
use ga_lib::error::GaError;

// Prepare a population with fitness values
let population: Vec<RangeChromosome> = /* ... */;

// Configure selection method and parameters
let selection_method = SelectionMethod::Tournament { size: 4 };

// Use the factory to select parents, handling possible errors
match factory(&population, selection_method) {
    Ok(parent_pairs) => {
        // Proceed with crossover using parent_pairs
    }
    Err(GaError::SelectionError) => {
        eprintln!("Population too small for selection");
    }
    _ => unreachable!(),
}
```

## API Reference

### `tournament`

Selects parent pairs using tournament selection.

**Signature:**
```rust
pub fn tournament<U: ChromosomeT>(chromosomes: &[U], tournament_size: usize) -> Vec<(usize, usize)>
```

**Parameters:**

| Name             | Type           | Description                                      |
|------------------|----------------|--------------------------------------------------|
| `chromosomes`    | `&[U]`         | Slice of chromosomes implementing `ChromosomeT`  |
| `tournament_size`| `usize`        | Number of individuals per tournament             |

**Returns:**  
`Vec<(usize, usize)>` — Indices of selected parent pairs.

---

### `roulette_wheel_selection`

Selects parent pairs with probability proportional to fitness.

**Signature:**
```rust
pub fn roulette_wheel_selection<U: ChromosomeT>(chromosomes: &[U]) -> Vec<(usize, usize)>
```

**Parameters:**

| Name          | Type           | Description                                      |
|---------------|----------------|--------------------------------------------------|
| `chromosomes` | `&[U]`         | Slice of chromosomes implementing `ChromosomeT`  |

**Returns:**  
`Vec<(usize, usize)>` — Indices of selected parent pairs.

---

### `stochastic_universal_sampling`

Selects parent pairs using evenly spaced sampling proportional to fitness.

**Signature:**
```rust
pub fn stochastic_universal_sampling<U: ChromosomeT>(chromosomes: &[U]) -> Vec<(usize, usize)>
```

**Parameters:**

| Name          | Type           | Description                                      |
|---------------|----------------|--------------------------------------------------|
| `chromosomes` | `&[U]`         | Slice of chromosomes implementing `ChromosomeT`  |

**Returns:**  
`Vec<(usize, usize)>` — Indices of selected parent pairs.

---

### `rank_selection`

Selects parent pairs with probability proportional to rank in the population.

**Signature:**
```rust
pub fn rank_selection<U: ChromosomeT>(chromosomes: &[U]) -> Vec<(usize, usize)>
```

**Parameters:**

| Name          | Type           | Description                                      |
|---------------|----------------|--------------------------------------------------|
| `chromosomes` | `&[U]`         | Slice of chromosomes implementing `ChromosomeT`  |

**Returns:**  
`Vec<(usize, usize)>` — Indices of selected parent pairs.

---

### `random`

Selects parent pairs uniformly at random.

**Signature:**
```rust
pub fn random<U: ChromosomeT>(chromosomes: &[U]) -> Vec<(usize, usize)>
```

**Parameters:**

| Name          | Type           | Description                                      |
|---------------|----------------|--------------------------------------------------|
| `chromosomes` | `&[U]`         | Slice of chromosomes implementing `ChromosomeT`  |

**Returns:**  
`Vec<(usize, usize)>` — Indices of selected parent pairs.

---

### `factory`

Dispatches parent selection according to the configured method.

**Signature:**
```rust
pub fn factory<U: ChromosomeT>(
    chromosomes: &[U],
    method: SelectionMethod
) -> Result<Vec<(usize, usize)>, GaError>
```

**Parameters:**

| Name          | Type                | Description                                      |
|---------------|---------------------|--------------------------------------------------|
| `chromosomes` | `&[U]`              | Slice of chromosomes implementing `ChromosomeT`  |
| `method`      | `SelectionMethod`   | Enum specifying the selection strategy and options|

**Returns:**  
`Result<Vec<(usize, usize)>, GaError>` — Ok with parent pairs, or Err if selection fails.

**Errors:**

- `GaError::SelectionError` if the population is too small to form parent pairs.

---

### `SelectionMethod` (enum)

Specifies the selection strategy and configuration.

| Variant                      | Parameters           | Description                                  |
|------------------------------|---------------------|----------------------------------------------|
| `Tournament { size }`        | `size: usize`       | Tournament selection with given size         |
| `RouletteWheel`              | —                   | Fitness-proportionate selection              |
| `StochasticUniversal`        | —                   | Stochastic universal sampling                |
| `Rank`                       | —                   | Rank-based selection                         |
| `Random`                     | —                   | Uniform random selection                     |

## Related

- [operators/crossover.md](./crossover.md)
- [operators/mutation.md](./mutation.md)
- [chromosomes.md](../chromosomes.md)
- [traits.md](../traits.md)
- [src/operations/selection.rs](../../src/operations/selection.rs)
- [src/operations/selection/tournament.rs](../../src/operations/selection/tournament.rs)
- [src/operations/selection/fitness_proportionate.rs](../../src/operations/selection/fitness_proportionate.rs)
- [src/operations/selection/rank.rs](../../src/operations/selection/rank.rs)
- [src/operations/selection/random.rs](../../src/operations/selection/random.rs)
