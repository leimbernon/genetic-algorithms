# Survivor Selection Operators

> Strategies for choosing which individuals survive to the next generation.

## Overview

Survivor selection operators determine which individuals from the current population are retained for the next generation in a genetic algorithm. This process is crucial for controlling population diversity, convergence speed, and overall algorithm performance. Survivor selection is typically applied after offspring have been generated through crossover and mutation, ensuring that the population size remains constant and that only the most promising individuals continue.

The library provides several survivor selection strategies: fitness-based, age-based, (mu+lambda), (mu,lambda), and deterministic crowding. Fitness-based selection retains the most fit individuals, driving the population towards optimal solutions more aggressively. Age-based selection removes the oldest individuals, promoting generational turnover and diversity. The (mu+lambda) and (mu,lambda) strategies offer different parent-offspring competition models. Deterministic crowding forces competition between genetically similar individuals, preserving niches and diversity. The choice of survivor selection method can significantly affect the behavior and results of your genetic algorithm.

Survivor selection is a modular component of the library, allowing users to choose or implement custom strategies as needed. It integrates seamlessly with the population management and configuration systems, ensuring flexibility and ease of use.

## Available Operators

The following `Survivor` enum variants are available:

| Variant | Description |
|---------|-------------|
| `Fitness` | Retains individuals with highest fitness |
| `Age` | Removes oldest individuals to reach target size |
| `MuPlusLambda` | (mu+lambda) strategy: parents and offspring compete together for survival |
| `MuCommaLambda` | (mu,lambda) strategy: only offspring (age == 0) are eligible for survival |
| `DeterministicCrowding` | Each offspring competes against its most similar parent |

### Trait Requirements

| Trait | Required Methods | Description |
|-------|-----------------|-------------|
| `ChromosomeT` | `fn fitness(&self) -> f64`, `fn age(&self) -> u64` | Required for all survivor selection methods |

## Usage

### Basic Example

```rust
use genetic_algorithms::chromosomes::Range;
use genetic_algorithms::operations::{Survivor, Selection, Crossover, Mutation};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, SurvivorConfig,
    SelectionConfig, CrossoverConfig, MutationConfig, StoppingConfig};

let mut ga: Ga<Range<f64>> = Ga::new()
    .with_survivor_method(Survivor::Fitness)
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    .with_initialization_fn(range_random_initialization)
    .with_fitness_fn(|dna| dna.iter().map(|g| g.value).sum())
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("valid config");
```

## Operator Details

### Fitness

Retains the individuals with the highest fitness scores up to the target population size. The most aggressive convergence strategy — best individuals always survive.

**Variant:** `Survivor::Fitness`

**When to use:** Default survivor method for most single-objective problems. Fast convergence but may lose diversity.

---

### Age

Removes the oldest individuals from the population until the desired population size is reached. Promotes generational turnover and diversity at the cost of slower convergence.

**Variant:** `Survivor::Age`

**When to use:** When maintaining high diversity is important, or when combined with elitism to preserve the best solutions.

---

### MuPlusLambda (mu+lambda)

Parents and offspring compete together in a combined pool for survival. The best individuals from the parent+offspring pool are selected, regardless of whether they are parents or offspring.

**Variant:** `Survivor::MuPlusLambda`

**When to use:** Standard evolutionary strategy approach. Preserves high-fitness parents while allowing offspring to compete.

---

### MuCommaLambda (mu,lambda)

Only offspring (age == 0) are eligible for survival. All parent individuals are replaced each generation, regardless of their fitness.

**Variant:** `Survivor::MuCommaLambda`

**When to use:** Strong generational turnover. Prevents stagnation by completely replacing the parent generation each cycle.

---

### DeterministicCrowding

Pairs each offspring (identified by `age() == 0`) with its most similar parent (lowest Hamming distance on gene IDs), and the fitter of each pair survives. Ensures offspring replace genetically similar individuals, preserving population niches and diversity.

**Algorithm:**
1. Partition the population into offspring (age == 0) and parents (age > 0).
2. For each offspring, find the most similar available parent using Hamming distance on gene IDs (positions where `gene_a.id() != gene_b.id()`).
3. Keep the fitter of the (offspring, most-similar-parent) pair; discard the other.
4. Offspring that have no available parent to pair with survive unconditionally.

**Configuration:** No additional parameters.

**Variant:** `Survivor::DeterministicCrowding`

**When to use:** Multimodal problems, niching, steady-state replacement, preserving multiple fitness peaks.

**Added in:** v2.4.0

---

### Parsimony Pressure

Parsimony pressure is not a standalone survivor strategy — it is a fitness adjustment that wraps any of the above strategies. When configured, each chromosome's effective fitness during survivor selection is temporarily adjusted by `±(length_penalty × dna_length)`. The stored `fitness()` value is **never** permanently mutated.

**Sign convention (auto-adjusted per optimization direction):**
- **Maximization** — `adjusted = fitness - (penalty × length)` (longer chromosomes appear worse)
- **Minimization** — `adjusted = fitness + (penalty × length)` (longer chromosomes appear worse)

```rust
// Enable parsimony pressure: penalize chromosomes with more genes.
let mut ga = Ga::new()
    .with_survivor_method(Survivor::Fitness)    // any survivor strategy
    .with_length_penalty(0.01)                   // parsimony coefficient
    // ... rest of configuration
    .build()
    .expect("valid config");
```

**Builder:** `.with_length_penalty(coefficient: f64)`

**When to use:** Variable-length chromosomes where you want to bias toward shorter, more parsimonious solutions — avoids bloat when using `Mutation::Insertion`.

**Added in:** v3.0.0

---

### `factory`

Dispatches survivor selection according to the configured method.

**Signature:**
```rust
pub fn factory<U: ChromosomeT>(chromosomes: &mut Vec<U>, config: &SurvivorConfiguration) -> Result<(), GaError>
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `chromosomes` | `&mut Vec<U>` | Population to trim |
| `config` | `&SurvivorConfiguration` | Survivor selection configuration |

## Related

- [operators/selection.md](selection.md) — Selection operators for parent selection
- [configuration.md](../configuration.md) — Survivor selection configuration options
- [chromosomes.md](../chromosomes.md) — Chromosome types supported
- [src/operations/survivor/](../../src/operations/survivor/)
