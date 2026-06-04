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
| `TreeChromosome`  | Supertrait of `ChromosomeT` for tree chromosomes; provides `tree()`, `depth()`, `node_count()`. Implemented by `GpChromosome<N>`. |
| `GpNode`          | User-implemented trait defining the GP function and terminal set. |
| `ConfigurationT`  | Trait for configuring genetic algorithm parameters.              |

### Operator Enums

| Enum        | Variants (examples) | Description |
|-------------|---------------------|-------------|
| `Selection` | `Tournament`, `RouletteWheel`, `StochasticUniversalSampling`, `Rank`, `Boltzmann`, `Truncation`, `Clearing`, `Random` | Parent selection strategies |
| `Crossover` | `Uniform`, `SinglePoint`, `MultiPoint`, `Order`, `Pmx`, `Cycle`, `Sbx`, `BlendAlpha`, `Arithmetic`, `EdgeRecombination`, `Clone`, `Rejuvenate`, `VariableLength(AlignmentStrategy)` | DNA recombination strategies |
| `Mutation`  | `Swap`, `Scramble`, `Inversion`, `Value`, `BitFlip`, `Creep`, `Gaussian`, `Polynomial`, `NonUniform`, `PermutationInsert`, `Insertion`, `Deletion`, `Cauchy`, `LevyFlight`, `Uniform`, `ListValue`, `Differential` | DNA mutation strategies |
| `Survivor`  | `Fitness`, `Age`, `MuPlusLambda`, `MuCommaLambda`, `DeterministicCrowding` | Survivor selection strategies |
| `Extension` | `Noop`, `MassExtinction`, `MassGenesis`, `MassDegeneration`, `MassDeduplication` | Population diversity control strategies |
| `AlignmentStrategy` | `Trim`, `Pad` | Variable-length parent alignment used by `Crossover::VariableLength` |
| `GpCrossover` | `SubtreeCrossover` | GP-specific crossover (used by `GpGa<N>`) |
| `GpMutation` | `SubtreeMutation { mutation_max_depth }`, `PointMutation { p_per_node }`, `HoistMutation` | GP-specific mutation strategies |

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
| `chromosome_length`    | `Option<ChromosomeLength>` | `Fixed(usize)` or `Variable { min, max }` — required for `Mutation::Insertion`/`Deletion` |
| `length_penalty`       | `Option<f64>`   | Parsimony pressure coefficient applied during survivor selection |

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
| `chromosomes`       | Concrete chromosome types (`Binary`, `Range<T>`, `ListChromosome<T>`) plus `ChromosomeLength` |
| `configuration`     | Configuration options and builder API                            |
| `error`             | Error types for GA operations (`GaError`)                        |
| `fitness`           | Fitness evaluation helpers                                       |
| `ga`                | Single-population GA orchestrator (`Ga<U>`)                      |
| `alps`              | Age-Layered Population Structure engine (`AlpsEngine`)           |
| `cellular`          | Cellular GA engine on 2D toroidal grid (`CellularEngine`)        |
| `de`                | Differential Evolution engine (`DeEngine`)                       |
| `island`            | Island model multi-population engine (`IslandGa`)                |
| `gp`                | Genetic Programming engine (`GpGa<N>`, `GpChromosome<N>`, `GpNode`, `Node<N>`, `MathNode`, `BoolNode`) |
| `nsga2`             | NSGA-II multi-objective engine (`Nsga2Ga`)                       |
| `nsga3`             | NSGA-III many-objective engine (`Nsga3Ga`)                       |
| `moead`             | MOEA/D decomposition-based engine (`MoeaDGa`)                    |
| `spea2`             | SPEA2 strength-Pareto engine (`Spea2Ga`)                         |
| `sms_emoa`          | SMS-EMOA hypervolume-based engine (`SmsEmoaGa`)                  |
| `ibea`              | IBEA indicator-based engine (`IbeaGa`)                           |
| `multi_objective`   | Shared Pareto dominance, quality indicators, reference points    |
| `scatter`           | Scatter Search engine with reference set (`ScatterEngine`)       |
| `genotypes`         | Concrete gene types (`Binary`, `Range<T>`, `List<T>`)            |
| `initializers`      | Population initialization utilities                              |
| `extension`         | Extension strategy configuration for diversity control           |
| `niching`           | Fitness sharing for multimodal optimization                      |
| `observer`          | `GaObserver` trait and built-in observers (`LogObserver`, `CompositeObserver`, `AllObserver`, `NoopObserver`) |
| `reporter`          | Legacy `Reporter` trait                                          |
| `visualization`     | Visualization utilities for GA runs (feature `visualization`)    |
| `operations`        | Selection, crossover, mutation, survivor, extension operators plus `AlignmentStrategy` |
| `population`        | Population management and best tracking                          |
| `stats`             | Statistics tracking for GA runs (`GenerationStats`)              |
| `aos`               | Adaptive Operator Selection (`AosStrategy`, `AosState`)          |
| `constraints`       | Constraint handling (`ConstraintHandling`, `PenaltyStrategy`)    |
| `hall_of_fame`      | Elite solution archive (`HallOfFame`, `DistanceMetric`)          |
| `traits`            | Core traits for genes, chromosomes, and configuration            |
| `validators`        | Configuration validation                                         |
| `rng`               | RNG with optional seeding                                        |
| `checkpoint`        | Serialization for checkpoint save/load (feature `serde`)         |
| `benchmarks`        | Standard benchmark functions and quality indicators (feature `benchmarks`) |

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

| Variant | Description |
|---------|-------------|
| `Random` | Equal probability for every individual |
| `RouletteWheel` | Fitness-proportionate selection |
| `StochasticUniversalSampling` | Roulette with evenly-spaced pointers |
| `Tournament` | Pairwise tournament competition |
| `Rank` | Probability proportional to rank |
| `Boltzmann` | Temperature-controlled selection pressure |
| `Truncation` | Only top portion eligible |
| `Clearing` | Niche winners — promotes diversity |

### `operations::AlignmentStrategy`

Alignment strategy used by `Crossover::VariableLength(strategy)` to reconcile parents with different DNA lengths.

| Variant | Description |
|---------|-------------|
| `Trim` | Truncate both parents to `min(len_a, len_b)` |
| `Pad` | Pad the shorter parent (from its own alleles) to `max(len_a, len_b)` |

### `operations::Crossover`

Enum for crossover operators.

| Variant | Description |
|---------|-------------|
| `Uniform` | Each gene chosen from either parent |
| `SinglePoint` | One cut point, halves swapped |
| `MultiPoint` | N cut points, alternating segments swapped |
| `Order` | Permutation-preserving order crossover (OX) |
| `Pmx` | Partially Mapped Crossover for permutations |
| `Cycle` | Position-preserving cycle crossover |
| `Sbx` | Simulated Binary Crossover for `Range<T>` |
| `BlendAlpha` | BLX-α blending for `Range<T>` |
| `Arithmetic` | Weighted average for `Range<T>` |
| `EdgeRecombination` | Adjacency-preserving for TSP / permutations |
| `Clone` | Direct parent copy (mutation-only strategies) |
| `Rejuvenate` | Clone + age reset to zero |
| `VariableLength(AlignmentStrategy)` | Crossover for variable-length parents |

### `operations::Mutation`

Enum for mutation operators.

| Variant | Description |
|---------|-------------|
| `Swap` | Swap two random genes |
| `Scramble` | Shuffle a random subsequence |
| `Inversion` | Reverse a random subsequence |
| `Value` | Replace a gene with a random allele |
| `BitFlip` | Flip random bits (binary) |
| `Creep` | Small uniform perturbation (`Range<T>`) |
| `Gaussian` | Normal-distribution perturbation (`Range<T>`) |
| `Polynomial` | Polynomial mutation, NSGA-II style (`Range<T>`) |
| `NonUniform` | Magnitude decreases over generations (`Range<T>`) |
| `PermutationInsert` | Move a gene to a different position (permutation) |
| `Insertion` | Grow chromosome by 1 gene (variable-length) |
| `Deletion` | Shrink chromosome by 1 gene (variable-length) |
| `Cauchy` | Heavy-tailed Cauchy perturbation (`Range<T>`) |
| `LevyFlight` | Lévy-stable distribution perturbation (`Range<T>`) |
| `Uniform` | Random gene reset to uniform value (`Range<T>`) |
| `ListValue` | Replace gene from finite allele set (`List<T>`) |
| `Differential` | DE-style difference-vector mutation (`Range<T>`) |

### `operations::Survivor`

Enum for survivor selection operators.

| Variant | Description |
|---------|-------------|
| `Fitness` | Keep top N by fitness |
| `Age` | Keep youngest N individuals |
| `MuPlusLambda` | (µ+λ): parents and offspring compete together |
| `MuCommaLambda` | (µ,λ): only offspring eligible |
| `DeterministicCrowding` | Replace genetically similar individuals |

> Parsimony pressure is applied via `.with_length_penalty(coefficient)` and wraps any of the variants above.

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

---

### Alternative Engines

The library ships several engine alternatives to the standard `Ga<U>` for specialized use cases:

#### `engines::alps::AlpsEngine`

Age-Layered Population Structure. Maintains `n_layers` sub-populations; individuals are
promoted to higher layers as they age, preventing premature convergence.

| Type                  | Description                                                           |
|-----------------------|-----------------------------------------------------------------------|
| `AlpsEngine`          | Engine entry point. Call `.run()` to execute.                         |
| `AlpsResult`          | Return type containing the best chromosome and per-layer populations. |
| `AlpsConfiguration`   | Builder for ALPS parameters.                                          |
| `AlpsAgeScheme`       | `Linear`, `Fibonacci`, `Polynomial` — controls per-layer age limits.  |

#### `engines::cellular::CellularEngine`

Cellular GA. Individuals are placed on a 2D toroidal grid; each cell evolves by
competing with its neighbourhood only, promoting diversity through spatial structure.

| Type                     | Description                                                          |
|--------------------------|----------------------------------------------------------------------|
| `CellularEngine`         | Engine entry point. Call `.run()` to execute.                        |
| `CellularResult`         | Return type containing the best chromosome and final grid state.     |
| `CellularConfiguration`  | Builder for grid size, neighbourhood type, and update mode.          |
| `Neighborhood`           | `VonNeumann` (4 cells) or `Moore` (8 cells).                         |
| `UpdateMode`             | `Synchronous` (all cells at once) or `Asynchronous` (random order).  |

#### `engines::de::DeEngine`

Differential Evolution. Continuous optimisation engine with 5 mutation strategies and 2
crossover modes. Supports JADE and L-SHADE adaptive parameter control.

| Type                  | Description                                                           |
|-----------------------|-----------------------------------------------------------------------|
| `DeEngine`            | Engine entry point. Call `.run()` to execute.                         |
| `DeResult`            | Return type containing the best solution vector and fitness.          |
| `DeConfiguration`     | Builder for population size, `F`, `CR`, strategy, and adaptation.    |
| `DeMutationStrategy`  | `Rand1`, `Best1`, `CurrentToBest1`, `Rand2`, `Best2`.                 |
| `DeCrossoverMode`     | `Binomial` or `Exponential`.                                          |
| `DeAdaptive`          | `None`, `Jade`, or `LShade` adaptive parameter control.               |
| `DeGene`              | Gene type carrying a continuous `f64` value for DE chromosomes.       |

#### `engines::island::IslandGa`

Island model. Runs multiple independent sub-populations (islands) in parallel, periodically
exchanging migrants according to a configurable topology and migration policy.

See `IslandConfiguration` in `configuration.md` for full parameter documentation.

#### `engines::nsga2::Nsga2Ga`

NSGA-II multi-objective optimisation. Ranks the population into Pareto fronts, uses
crowding-distance tie-breaking, and returns a Pareto-front set rather than a single best.

See `Nsga2Configuration` in `configuration.md` for full parameter documentation.

#### `engines::scatter::ScatterEngine`

Scatter Search. Maintains a small reference set of high-quality and diverse solutions.
New candidates are generated by linear combination of reference-set members, with
optional local search refinement.

| Type                   | Description                                                          |
|------------------------|----------------------------------------------------------------------|
| `ScatterEngine`        | Engine entry point. Call `.run()` to execute.                        |
| `ScatterResult`        | Return type containing the final reference set and best chromosome.  |
| `ScatterConfiguration` | Builder for reference-set size, diversification rate, and local search. |

#### `engines::gp::GpGa`

Genetic Programming. Evolves tree-structured programs (`GpChromosome<N>`) using subtree crossover, subtree/point/hoist mutation, ramped half-and-half initialization, and built-in bloat control (hard depth and node-count limits).

| Type | Description |
|------|-------------|
| `GpGa<N>` | Engine entry point. Generic over `N: GpNode` (user-defined primitive set). |
| `GpResult<N>` | Return type containing the best individual, best fitness, and final population. |
| `GpConfiguration` | Builder for population size, depth limits, operators, selection, survivor strategy. |
| `GpChromosome<N>` | Tree chromosome wrapping a `Node<N>` expression tree. Implements `ChromosomeT` + `TreeChromosome`. |
| `Node<N>` | Recursive expression-tree enum: `Function { value, children }` or `Terminal(N)`. |
| `GpNode` | Trait the user implements on a primitive-set enum (defines arity, evaluation, terminal sampling, function enumeration). |
| `TreeChromosome` | Supertrait of `ChromosomeT` exposing `tree()`, `depth()`, `node_count()`. |
| `GpCrossover` | `SubtreeCrossover` — swap random subtrees between parents with bloat retry. |
| `GpMutation` | `SubtreeMutation { mutation_max_depth }`, `PointMutation { p_per_node }`, `HoistMutation`. |
| `MathNode` | Built-in primitive set for symbolic regression (`Add`, `Sub`, `Mul`, `ProtectedDiv`, `Const`, `Var`). |
| `BoolNode` | Built-in primitive set for classification trees (`And`, `Or`, `Not`, `Gt`, `Lt`). |
| `ramped_half_and_half` | Standard GP population initializer combining the `full` and `grow` methods across multiple depths. |

See [Genetic Programming](gp.md) for the complete guide.

---

### `observe::observer::GaObserver`

Structured lifecycle observer trait for `Ga<U>`. All hook methods have default no-op
implementations — implement only the hooks you need.

```rust
pub trait GaObserver<U: ChromosomeT>: Send + Sync {
    fn on_run_start(&self) {}
    fn on_generation_start(&self, generation: usize) {}
    fn on_selection_complete(&self, generation: usize, duration: Duration, pop_size: usize) {}
    fn on_crossover_complete(&self, generation: usize, duration: Duration, offspring: usize) {}
    fn on_mutation_complete(&self, generation: usize, duration: Duration, pop_size: usize) {}
    fn on_fitness_evaluation_complete(&self, generation: usize, duration: Duration, pop_size: usize) {}
    fn on_survivor_selection_complete(&self, generation: usize, duration: Duration, pop_size: usize) {}
    fn on_new_best(&self, generation: usize, best: U) {}
    fn on_stagnation(&self, generation: usize, stagnation_count: usize) {}
    fn on_extension_triggered(&self, event: ExtensionEvent) {}
    fn on_generation_end(&self, stats: &GenerationStats) {}
    fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {}
}
```

Attach an observer via `.with_observer(Arc<dyn GaObserver<U>>)` on the `Ga` builder.

#### Built-in observers

| Type                 | Feature flag         | Description                                                             |
|----------------------|----------------------|-------------------------------------------------------------------------|
| `NoopObserver`       | always available     | Zero-sized no-op; useful as a placeholder or compile check.             |
| `LogObserver`        | always available     | Logs generation stats using the `log` crate.                            |
| `CompositeObserver`  | always available     | Fan-out: dispatches every hook to a list of inner observers.            |
| `TracingObserver`    | `observer-tracing`   | Emits structured `tracing` spans and events per hook.                   |
| `MetricsObserver`    | `observer-metrics`   | Collects numeric metrics into a thread-safe `MetricsStore`.             |

#### Related observer traits

| Trait                 | Description                                                          |
|-----------------------|----------------------------------------------------------------------|
| `IslandGaObserver<U>` | Island-specific hooks: `on_island_run_start/end`, `on_migration_triggered`. |
| `Nsga2Observer<U>`    | NSGA-II hooks: `on_pareto_front_assigned`, sort/crowding timing.     |
| `AllObserver<U>`      | Supertrait combining `GaObserver`, `IslandGaObserver`, `Nsga2Observer`. Used by `CompositeObserver`. |

---

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
