# Population Management

> Manages the evolving set of chromosomes, tracks best solutions, and computes population statistics.

## Overview

The `Population` module is responsible for storing and managing the set of chromosomes that constitute a generation in a genetic algorithm. It provides mechanisms for tracking the best-performing chromosome, calculating aggregate statistics such as average and maximum fitness, and efficiently evaluating fitness for all members using parallel computation.

Users interact with the population to initialize generations, update members, and retrieve statistical insights necessary for adaptive genetic algorithms. The population structure is central to the evolutionary process, as it maintains the pool of candidate solutions and facilitates operations such as selection, crossover, and mutation.

Chromosome management within the population is generic, allowing users to define custom chromosome types as long as they satisfy the required trait bounds. The module also supports optional tracking of generation numbers and is designed for efficient integration with multi-threaded environments.

## Key Concepts

The core abstraction is the generic `Population<U>` struct, where `U` is the chromosome type. Chromosome types must implement several traits to support parallel fitness evaluation and best tracking.

### Trait Bounds for Chromosome Type `U`

| Trait                | Description                                                        |
|----------------------|--------------------------------------------------------------------|
| `Clone`              | Enables duplication of chromosomes for best tracking and updates.   |
| `Send` and `Sync`    | Required for parallel fitness calculation across threads.           |
| `FitnessEvaluable`   | Must provide a method to evaluate fitness.                         |

### Population Structure

| Field                | Type                  | Description                                                      |
|----------------------|-----------------------|------------------------------------------------------------------|
| `chromosomes`        | `Vec<U>`              | Current set of chromosomes in the population.                    |
| `best_chromosome`    | `U`                   | Chromosome with the highest fitness (or optimal according to objective). |
| `generation_numbers` | `Vec<i32>`            | Optional tracking of generation indices for analysis.            |
| `f_avg`              | `f64`                 | Average fitness across all chromosomes.                          |
| `f_max`              | `f64`                 | Maximum fitness value in the population.                         |

### Configuration and Customization

- **Generation Tracking:** Use `generation_numbers` to associate chromosomes with specific generations.
- **Adaptive GA Support:** Aggregate statistics (`f_avg`, `f_max`) are automatically updated and can be used for adaptive operator probabilities.
- **Parallel Fitness Evaluation:** Fitness calculation leverages Rayon for efficient multi-threaded computation.

## Usage

### Basic Example

```rust
use ga_lib::population::Population;
use ga_lib::chromosomes::BinaryChromosome;
use ga_lib::fitness::FitnessEvaluable;

// Assume BinaryChromosome implements Clone, Send, Sync, FitnessEvaluable

// Create a vector of chromosomes
let chromosomes = vec![
    BinaryChromosome::random(32),
    BinaryChromosome::random(32),
    BinaryChromosome::random(32),
];

// Initialize a population with these chromosomes
let mut population = Population::new(chromosomes);

// Calculate fitness for all chromosomes and update statistics
population.fitness_calculation();

// Access average and maximum fitness
println!("Average fitness: {}", population.f_avg);
println!("Maximum fitness: {}", population.f_max);

// Get the best chromosome
let best = &population.best_chromosome;
```

### Advanced Example

```rust
use ga_lib::population::Population;
use ga_lib::chromosomes::RangeChromosome;
use ga_lib::fitness::{FitnessEvaluable, ProblemSolving};

// Assume RangeChromosome implements required traits

// Create initial chromosomes for generation 0
let mut chromosomes = vec![
    RangeChromosome::random(10, 0.0, 1.0),
    RangeChromosome::random(10, 0.0, 1.0),
    RangeChromosome::random(10, 0.0, 1.0),
];

// Initialize an empty population and add chromosomes
let mut population = Population::new_empty();
population.add_chromosomes(&mut chromosomes);

// Track generation numbers for analysis
population.generation_numbers.push(0);

// Perform parallel fitness calculation
population.fitness_calculation();

// Adaptive GA: Recalculate average and max fitness after mutation/crossover
population.recalculate_aga();

// Update best chromosome with a new candidate (e.g., after mutation)
let candidate = RangeChromosome::random(10, 0.0, 1.0);
population.decide_best_chromosome(&candidate, ProblemSolving::Maximize);

// Access population size
println!("Population size: {}", population.size());
```

## API Reference

### `Population<U>`

Manages a collection of chromosomes, tracks the best solution, and computes aggregate statistics.

**Fields:**

| Name               | Type         | Description                                                      |
|--------------------|--------------|------------------------------------------------------------------|
| `chromosomes`      | `Vec<U>`     | Current set of chromosomes in the population.                    |
| `best_chromosome`  | `U`          | Chromosome with the highest fitness (or optimal according to objective). |
| `generation_numbers`| `Vec<i32>`  | Optional tracking of generation indices for analysis.            |
| `f_avg`            | `f64`        | Average fitness across all chromosomes.                          |
| `f_max`            | `f64`        | Maximum fitness value in the population.                         |

**Methods:**

| Name                   | Signature                                                                 | Description                                                                                  |
|------------------------|--------------------------------------------------------------------------|----------------------------------------------------------------------------------------------|
| `new_empty`            | `fn new_empty() -> Population<U>`                                         | Creates a new empty population with no chromosomes.                                          |
| `new`                  | `fn new(chromosomes: Vec<U>) -> Population<U>`                            | Creates a population initialized with the provided chromosomes.                              |
| `recalculate_aga`      | `fn recalculate_aga(&mut self)`                                           | Recomputes average (`f_avg`) and maximum (`f_max`) fitness for adaptive GA operations.       |
| `add_chromosomes`      | `fn add_chromosomes(&mut self, chromosomes: &mut Vec<U>)`                 | Appends a list of chromosomes to the current population.                                     |
| `size`                 | `fn size(&self) -> usize`                                                 | Returns the number of chromosomes in the population.                                         |
| `fitness_calculation`  | `fn fitness_calculation(&mut self)`                                       | Computes fitness for all chromosomes in parallel and updates the best chromosome.            |
| `decide_best_chromosome`| `fn decide_best_chromosome(&mut self, new_chromosome: &U, problem_solving: ProblemSolving)` | Updates the best chromosome if the candidate is superior according to the problem objective. |

**Trait Bounds for `U`:**

| Trait                | Required For                          | Description                                                        |
|----------------------|---------------------------------------|--------------------------------------------------------------------|
| `Clone`              | Best tracking, population updates      | Enables duplication of chromosomes.                                |
| `Send` and `Sync`    | Parallel fitness calculation           | Required for multi-threaded execution.                             |
| `FitnessEvaluable`   | Fitness calculation                    | Must provide a method to evaluate fitness.                         |

## Related

- [chromosomes.md](chromosomes.md) — Chromosome types and implementations
- [fitness.md](fitness.md) — Fitness evaluation and custom fitness functions
- [configuration.md](configuration.md) — GA configuration and builder API
- [traits.md](traits.md) — Core traits required for chromosomes
- [src/population.rs](../src/population.rs) — Source code for population management
- [examples.md](examples.md) — End-to-end examples using populations
