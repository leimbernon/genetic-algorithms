# API Reference

> Comprehensive summary of all public types, traits, functions, and configuration options in the genetic algorithms library.

## Overview

This API reference provides a detailed listing of every public item in the genetic algorithms library, including modules, types, traits, enums, and functions. It serves as a central resource for developers to understand the available abstractions and how to use them when building custom genetic algorithm solutions.

The library is designed to be modular and extensible, with clear separation between core abstractions (genes, chromosomes, configuration), concrete implementations (binary/range genes and chromosomes), and composable operators (selection, crossover, mutation, survivor). The orchestrator (`Ga`) coordinates the entire lifecycle of a genetic algorithm, while utility modules provide helpers for initialization, fitness evaluation, and population management.

This document is intended for Rust developers who want to leverage genetic algorithms for optimization or search problems. It provides both high-level guidance and low-level details, ensuring users can quickly find and understand every public API surface.

## Key Concepts

The following table summarizes the core modules and their primary types:

| Module            | Type/Enum/Trait         | Description                                                      |
|-------------------|------------------------|------------------------------------------------------------------|
| `traits`          | `GeneT`, `ChromosomeT`, `ConfigurationT` | Core traits for genes, chromosomes, and configuration            |
| `chromosomes`     | `Binary`, `Range`      | Concrete chromosome types                                        |
| `genotypes`       | `Binary`, `Range`      | Concrete gene types                                              |
| `ga`              | `Ga`                   | Genetic algorithm orchestrator                                   |
| `configuration`   | `ProblemSolving`       | Problem-solving mode (Minimization/Maximization)                 |
| `operations`      | `Selection`, `Crossover`, `Mutation`, `Survivor`, `Extension` | Operator enums for GA phases                          |
| `extension`       | `ExtensionConfiguration` | Extension strategy configuration for diversity control           |
| `population`      | `Population`           | Population management and best tracking                          |
| `fitness`         | `FitnessFn`            | Fitness evaluation helpers                                       |
| `initializers`    | `range_random_initialization` | Utilities for initial DNA generation                             |
| `stats`           | `Stats`                | Statistics tracking for GA runs                                  |
| `error`           | `Error`                | Error types for GA operations                                    |

### Core Traits

| Trait             | Description                                                      |
|-------------------|------------------------------------------------------------------|
| `GeneT`           | Abstraction for a gene; provides methods for mutation and value access. |
| `ChromosomeT`     | Abstraction for a chromosome; provides DNA access, fitness, age, and mutation. |
| `ConfigurationT`  | Trait for configuring genetic algorithm parameters.              |

### Operator Enums

| Enum        | Variants (examples)                    | Description                        |
|-------------|----------------------------------------|------------------------------------|
| `Selection` | `Tournament`, `Roulette`, `Random`     | Parent selection strategies        |
| `Crossover` | `Uniform`, `Multipoint`, `Cycle`       | DNA recombination strategies       |
| `Mutation`  | `Swap`, `Scramble`, `Inversion`, `Value` | DNA mutation strategies            |
| `Survivor`  | `Age`, `Fitness`                       | Survivor selection strategies      |
| `Extension` | `Noop`, `MassExtinction`, `MassGenesis`, `MassDegeneration`, `MassDeduplication` | Population diversity control strategies |

### Configuration Options

| Field                | Type              | Description                                 |
|----------------------|-------------------|---------------------------------------------|
| `genes_per_chromosome` | `usize`         | Number of genes in each chromosome          |
| `population_size`      | `usize`         | Number of chromosomes in the population     |
| `initialization_fn`    | `Fn`            | Function to initialize population DNA       |
| `fitness_fn`           | `Fn`            | Function to evaluate chromosome fitness     |
| `selection_method`     | `Selection`     | Parent selection operator                   |
| `crossover_method`     | `Crossover`     | Crossover operator                          |
| `mutation_method`      | `Mutation`      | Mutation operator                           |
| `survivor_method`      | `Survivor`      | Survivor selection operator                 |
| `problem_solving`      | `ProblemSolving`| Minimization or maximization mode           |
| `max_generations`      | `usize`         | Maximum number of generations               |
| `fitness_target`       | `f64`           | Target fitness value to terminate           |

## Usage

### Basic Example

```rust
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};

const N: i32 = 8;

fn fitness_fn(dna: &[RangeGene<i32>]) -> f64 {
    // Compute fitness for a chromosome
    0.0
}

let alleles = vec![RangeGene::new(0, vec![(0, N - 1)], 0)];
let alleles_clone = alleles.clone();
let mut ga = Ga::new();
let _population = ga
    .with_genes_per_chromosome(N)
    .with_population_size(100)
    .with_initialization_fn(move |genes_per_chromosome, _, _| {
        range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(fitness_fn)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Swap)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_survivor_method(Survivor::Fitness)
    .with_max_generations(5000)
    .with_fitness_target(0.0)
    .run();
```

### Advanced Example

```rust
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};

const CHROMOSOME_LEN: usize = 32;

fn fitness_fn(dna: &[BinaryGene]) -> f64 {
    dna.iter().map(|gene| gene.value() as f64).sum()
}

let mut ga = Ga::new();
let _population = ga
    .with_genes_per_chromosome(CHROMOSOME_LEN)
    .with_population_size(200)
    .with_initialization_fn(|genes_per_chromosome, _, _| {
        binary_random_initialization(genes_per_chromosome)
    })
    .with_fitness_fn(fitness_fn)
    .with_selection_method(Selection::Roulette)
    .with_crossover_method(Crossover::Multipoint)
    .with_mutation_method(Mutation::Scramble)
    .with_problem_solving(ProblemSolving::Maximization)
    .with_survivor_method(Survivor::Age)
    .with_max_generations(1000)
    .with_fitness_target(32.0)
    .run();
```

## API Reference

### Modules

| Module              | Description                                                      |
|---------------------|------------------------------------------------------------------|
| `chromosomes`       | Concrete chromosome types (`Binary`, `Range`)                    |
| `configuration`     | Configuration options and builder API                            |
| `error`             | Error types for GA operations                                    |
| `fitness`           | Fitness evaluation helpers                                       |
| `ga`                | Genetic algorithm orchestrator (`Ga`)                            |
| `genotypes`         | Concrete gene types (`Binary`, `Range`)                          |
| `initializers`      | Population initialization utilities                              |
| `extension`         | Extension strategy configuration for diversity control           |
| `operations`        | Selection, crossover, mutation, survivor, extension operators    |
| `population`        | Population management and best tracking                          |
| `stats`             | Statistics tracking for GA runs                                  |
| `traits`            | Core traits for genes, chromosomes, and configuration            |

---

### `traits::GeneT`

Trait for gene abstraction.

| Name         | Signature                                      | Description                      |
|--------------|------------------------------------------------|----------------------------------|
| `mutate`     | `fn mutate(&mut self)`                         | Mutates the gene                 |
| `value`      | `fn value(&self) -> T`                         | Returns the gene's value         |
| `clone_box`  | `fn clone_box(&self) -> Box<dyn GeneT<T>>`     | Clones the gene as a trait object|

### `traits::ChromosomeT`

Trait for chromosome abstraction.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `set_dna`      | `fn set_dna(&mut self, dna: Cow<[Gene]>)`              | Sets the chromosome's DNA            |
| `dna`          | `fn dna(&self) -> &[Gene]`                             | Returns a reference to the DNA       |
| `fitness`      | `fn fitness(&self) -> f64`                             | Returns the fitness value            |
| `age`          | `fn age(&self) -> usize`                               | Returns the age of the chromosome    |
| `mutate`       | `fn mutate(&mut self, method: Mutation)`               | Mutates the chromosome               |

### `traits::ConfigurationT`

Trait for configuring GA parameters.

| Name                   | Signature                                      | Description                          |
|------------------------|------------------------------------------------|--------------------------------------|
| `with_genes_per_chromosome` | `fn with_genes_per_chromosome(self, usize) -> Self` | Sets genes per chromosome            |
| `with_population_size`      | `fn with_population_size(self, usize) -> Self`      | Sets population size                 |
| `with_initialization_fn`    | `fn with_initialization_fn(self, fn) -> Self`       | Sets initialization function         |
| `with_fitness_fn`           | `fn with_fitness_fn(self, fn) -> Self`              | Sets fitness function                |
| `with_selection_method`     | `fn with_selection_method(self, Selection) -> Self` | Sets selection operator              |
| `with_crossover_method`     | `fn with_crossover_method(self, Crossover) -> Self` | Sets crossover operator              |
| `with_mutation_method`      | `fn with_mutation_method(self, Mutation) -> Self`   | Sets mutation operator               |
| `with_survivor_method`      | `fn with_survivor_method(self, Survivor) -> Self`   | Sets survivor operator               |
| `with_problem_solving`      | `fn with_problem_solving(self, ProblemSolving) -> Self` | Sets problem-solving mode        |
| `with_max_generations`      | `fn with_max_generations(self, usize) -> Self`      | Sets max generations                 |
| `with_fitness_target`       | `fn with_fitness_target(self, f64) -> Self`         | Sets fitness target                  |
| `run`                      | `fn run(self) -> Population`                         | Runs the genetic algorithm           |

---

### `chromosomes::Binary`, `chromosomes::Range`

Concrete chromosome types.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `new`          | `fn new(...) -> Self`                                  | Creates a new chromosome             |
| `set_dna`      | `fn set_dna(&mut self, dna: Cow<[Gene]>)`              | Sets the chromosome's DNA            |
| `dna`          | `fn dna(&self) -> &[Gene]`                             | Returns a reference to the DNA       |
| `fitness`      | `fn fitness(&self) -> f64`                             | Returns the fitness value            |
| `age`          | `fn age(&self) -> usize`                               | Returns the age of the chromosome    |
| `mutate`       | `fn mutate(&mut self, method: Mutation)`               | Mutates the chromosome               |

---

### `genotypes::Binary`, `genotypes::Range`

Concrete gene types.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `new`          | `fn new(...) -> Self`                                  | Creates a new gene                   |
| `mutate`       | `fn mutate(&mut self)`                                 | Mutates the gene                     |
| `value`        | `fn value(&self) -> T`                                 | Returns the gene's value             |

---

### `ga::Ga`

Genetic algorithm orchestrator.

| Name                   | Signature                                      | Description                          |
|------------------------|------------------------------------------------|--------------------------------------|
| `new`                  | `fn new() -> Self`                             | Creates a new GA instance            |
| `with_genes_per_chromosome` | `fn with_genes_per_chromosome(self, usize) -> Self` | Sets genes per chromosome    |
| `with_population_size`      | `fn with_population_size(self, usize) -> Self`      | Sets population size         |
| `with_initialization_fn`    | `fn with_initialization_fn(self, fn) -> Self`       | Sets initialization function |
| `with_fitness_fn`           | `fn with_fitness_fn(self, fn) -> Self`              | Sets fitness function        |
| `with_selection_method`     | `fn with_selection_method(self, Selection) -> Self` | Sets selection operator      |
| `with_crossover_method`     | `fn with_crossover_method(self, Crossover) -> Self` | Sets crossover operator      |
| `with_mutation_method`      | `fn with_mutation_method(self, Mutation) -> Self`   | Sets mutation operator       |
| `with_survivor_method`      | `fn with_survivor_method(self, Survivor) -> Self`   | Sets survivor operator       |
| `with_problem_solving`      | `fn with_problem_solving(self, ProblemSolving) -> Self` | Sets problem-solving mode|
| `with_max_generations`      | `fn with_max_generations(self, usize) -> Self`      | Sets max generations         |
| `with_fitness_target`       | `fn with_fitness_target(self, f64) -> Self`         | Sets fitness target          |
| `run`                      | `fn run(self) -> Population`                         | Runs the genetic algorithm   |

---

### `configuration::ProblemSolving`

Enum for problem-solving mode.

| Variant         | Description                          |
|-----------------|--------------------------------------|
| `Minimization`  | Minimize the fitness function        |
| `Maximization`  | Maximize the fitness function        |

---

### `operations::Selection`

Enum for selection operators.

| Variant         | Description                          |
|-----------------|--------------------------------------|
| `Tournament`    | Tournament selection                 |
| `Roulette`      | Roulette wheel selection             |
| `Random`        | Random selection                     |

### `operations::Crossover`

Enum for crossover operators.

| Variant         | Description                          |
|-----------------|--------------------------------------|
| `Uniform`       | Uniform crossover                    |
| `Multipoint`    | Multipoint crossover                 |
| `Cycle`         | Cycle crossover                      |

### `operations::Mutation`

Enum for mutation operators.

| Variant         | Description                          |
|-----------------|--------------------------------------|
| `Swap`          | Swap mutation                        |
| `Scramble`      | Scramble mutation                    |
| `Inversion`     | Inversion mutation                   |
| `Value`         | Value mutation                       |

### `operations::Survivor`

Enum for survivor selection operators.

| Variant         | Description                          |
|-----------------|--------------------------------------|
| `Age`           | Age-based survivor selection         |
| `Fitness`       | Fitness-based survivor selection     |

### `operations::Extension`

Enum for extension strategies (population diversity control).

| Variant              | Description                                                                    |
|----------------------|--------------------------------------------------------------------------------|
| `Noop`               | No extension — diversity drops are ignored (default).                         |
| `MassExtinction`     | Random cull to a survival rate, protecting elite individuals.                 |
| `MassGenesis`        | Trim to 2 best chromosomes, regrow population.                               |
| `MassDegeneration`   | Apply N mutation rounds to non-elite chromosomes.                             |
| `MassDeduplication`  | Remove duplicate chromosomes by gene ID comparison.                           |

---

### `population::Population`

Population management.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `new`          | `fn new(...) -> Self`                                  | Creates a new population             |
| `best`         | `fn best(&self) -> &Chromosome`                        | Returns the best chromosome          |
| `size`         | `fn size(&self) -> usize`                              | Returns population size              |

---

### `fitness::FitnessFn`

Fitness evaluation helper.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `wrap`         | `fn wrap(fn) -> FitnessFn`                             | Wraps a user fitness function        |

---

### `initializers::range_random_initialization`

Population initialization utility.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `range_random_initialization` | `fn range_random_initialization(genes_per_chromosome: usize, alleles: Option<&[RangeGene<T>]>, unique: Option<bool>) -> Vec<RangeGene<T>>` | Initializes DNA for range chromosomes |

---

### `stats::Stats`

Statistics tracking for GA runs.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `new`          | `fn new() -> Self`                                     | Creates a new stats tracker          |
| `update`       | `fn update(&mut self, population: &Population)`        | Updates stats from population        |

---

### `error::Error`

Error type for GA operations.

| Name           | Signature                                              | Description                          |
|----------------|--------------------------------------------------------|--------------------------------------|
| `new`          | `fn new(msg: &str) -> Self`                            | Creates a new error                  |

## Related

- [traits.md](traits.md) — Core traits documentation
- [chromosomes.md](chromosomes.md) — Chromosome types
- [genotypes.md](genotypes.md) — Genotype definitions
- [operators/selection.md](operators/selection.md) — Selection operators
- [operators/crossover.md](operators/crossover.md) — Crossover operators
- [operators/mutation.md](operators/mutation.md) — Mutation operators
- [operators/survivor.md](operators/survivor.md) — Survivor selection
- [operators/extension.md](operators/extension.md) — Extension strategies
- [configuration.md](configuration.md) — Configuration options
- [fitness.md](fitness.md) — Fitness evaluation
- [population.md](population.md) — Population management
- [examples.md](examples.md) — End-to-end examples
- [Source: src/lib.rs](../src/lib.rs)
- [Source: src/traits.rs](../src/traits.rs)
- [Source: src/ga.rs](../src/ga.rs)
- [Source: src/operations.rs](../src/operations.rs)
- [Source: src/chromosomes.rs](../src/chromosomes.rs)
- [Source: src/genotypes.rs](../src/genotypes.rs)
- [Source: src/population.rs](../src/population.rs)
- [Source: src/fitness.rs](../src/fitness.rs)
- [Source: src/initializers.rs](../src/initializers.rs)
- [Source: src/stats.rs](../src/stats.rs)
- [Source: src/error.rs](../src/error.rs)
