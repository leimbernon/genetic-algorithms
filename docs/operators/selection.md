# Selection Operators

> Parent selection strategies for genetic algorithms, including tournament, roulette, rank, and random methods.

## Overview

Selection operators determine which individuals from a population are chosen as parents for crossover in a genetic algorithm. The choice of selection method can significantly affect the convergence speed and diversity of solutions. This module provides several widely-used selection strategies: tournament selection, fitness-proportionate (roulette wheel and stochastic universal sampling), rank-based selection, Boltzmann selection, truncation selection, and clearing selection.

Each selection operator has its own configuration options and trade-offs. For example, tournament selection allows you to control selection pressure via the tournament size, while fitness-proportionate methods rely on the fitness values assigned to each chromosome. Selection is a critical step in the genetic algorithm cycle, as it balances exploration and exploitation of the search space.

These operators are designed to work with any chromosome type that implements the required trait (`ChromosomeT`). Fitness values must be accessible via the chromosome interface, and the population must be large enough to form the requested number of parent pairs. Error handling is provided for cases where selection cannot be performed due to insufficient population size.

Selection operators are typically configured as part of the genetic algorithm setup and invoked automatically during each generation. Advanced users may also call them directly for custom workflows.

## Available Operators

The following `Selection` enum variants are available, dispatched via the `factory` function:

| Variant | Description | Key Parameters |
|---------|-------------|----------------|
| `Random` | Uniform random selection — every individual has equal probability | — |
| `RouletteWheel` | Fitness-proportionate selection | — |
| `StochasticUniversalSampling` | Evenly spaced sampling proportional to fitness | — |
| `Tournament` | Pairwise tournament — two or more individuals compete and the fitter wins | `tournament_size: usize` |
| `Rank` | Probability proportional to rank in sorted population | — |
| `Boltzmann` | Temperature-controlled selection pressure | Temperature parameter |
| `Truncation` | Only the top portion of the population is eligible | Truncation threshold |
| `Clearing` | Niche-based diversity preservation | `niche_radius: f64` |

### Chromosome Trait Requirements

| Trait | Required Methods | Description |
|-------|-----------------|-------------|
| `ChromosomeT` | `fn fitness(&self) -> f64` | Provides fitness value for selection |

### Error Handling

| Error Type | Description |
|------------|-------------|
| `GaError::SelectionError` | Returned if population is too small for selection |

## Usage

### Basic Example

```rust
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::operations::Selection;
use genetic_algorithms::traits::ChromosomeT;

// Selection is configured via the Selection enum, dispatched at runtime
let selection_method = Selection::Tournament;
// The GA builder applies selection automatically:
//     .with_selection_method(selection_method)
```

### Configuration via Builder

```rust
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, SelectionConfig, CrossoverConfig,
    MutationConfig, StoppingConfig,
};

let mut ga: Ga<Range<f64>> = Ga::new()
    .with_selection_method(Selection::Tournament)
    .with_tournament_size(4)
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    .with_initialization_fn(range_random_initialization)
    .with_fitness_fn(|dna| dna.iter().map(|g| g.value).sum())
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("valid config");
```

## Operator Details

### Tournament

Selects parent pairs via tournament competition. The fitter individual in each tournament wins, and tournament winners are then randomly paired.

Configure via `Selection::Tournament` and the `with_tournament_size(size)` builder method. Larger tournament sizes increase selection pressure (faster convergence, lower diversity).

**Signature (via factory):**
```rust
Selection::Tournament  // dispatches to tournament(chromosomes, tournament_size, threads)
```

**Configuration:**
| Method | Description |
|--------|-------------|
| `with_tournament_size(usize)` | Number of individuals per tournament (default: 3) |

---

### Roulette Wheel

Fitness-proportionate selection. Each individual's selection probability is proportional to its fitness. Handles negative fitness values through shift-normalization.

Configure via `Selection::RouletteWheel`. No additional parameters.

---

### Stochastic Universal Sampling

Like roulette wheel but uses evenly spaced pointers for lower variance. All parents are selected in a single pass with equally spaced selection points.

Configure via `Selection::StochasticUniversalSampling`. No additional parameters.

---

### Rank

Probability is proportional to rank rather than absolute fitness. Reduces the dominance of very fit individuals and maintains selection pressure when the population converges.

Configure via `Selection::Rank`. No additional parameters.

---

### Boltzmann

Temperature-controlled selection pressure. High temperature produces near-uniform selection (exploration); low temperature produces high selective pressure (exploitation). The temperature parameter controls the transition.

Configure via `Selection::Boltzmann`. Requires a temperature parameter in the selection configuration.

---

### Truncation

Only the top portion of the population (by fitness) is eligible for reproduction. The threshold determines what fraction of the population is kept.

Configure via `Selection::Truncation`. Requires a truncation threshold parameter.

---

### Clearing

Promotes population diversity by grouping individuals by niche radius. The best individual in each niche is preserved as the niche winner; all other individuals within that niche's radius are cleared (removed from the selection pool). Remaining eligible individuals are then paired randomly.

This prevents a single high-fitness region from dominating the mating pool, encouraging exploration of multiple fitness peaks.

**Configuration:**
- `niche_radius: f64` — Individuals within this distance in fitness space are considered part of the same niche. Smaller radii preserve more niches.

**Enum variant:** `Selection::Clearing`

**Builder configuration:**
```rust
.with_selection_method(Selection::Clearing)
.with_niche_radius(0.1)
```

**When to use:** Multimodal problems, niching, maintaining population diversity across multiple fitness peaks.

**Added in:** v2.4.0

---

### `factory`

Dispatches parent selection according to the configured method.

**Signature:**
```rust
pub fn factory<U: ChromosomeT>(
    chromosomes: &[U],
    method: &SelectionConfiguration,
) -> Result<Vec<(usize, usize)>, GaError>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `chromosomes` | `&[U]` | Slice of chromosomes implementing `ChromosomeT` |
| `method` | `&SelectionConfiguration` | Configuration specifying the selection strategy and options |

**Returns:**
`Result<Vec<(usize, usize)>, GaError>` — Ok with parent pairs, or Err if selection fails.

**Errors:**

- `GaError::SelectionError` if the population is too small to form parent pairs.

---

### `Selection` (enum)

Specifies the selection strategy and configuration.

| Variant | Parameters | Description |
|---------|-----------|-------------|
| `Random` | — | Uniform random selection |
| `RouletteWheel` | — | Fitness-proportionate selection |
| `StochasticUniversalSampling` | — | Stochastic universal sampling |
| `Tournament` | — | Tournament selection (size via `SelectionConfiguration`) |
| `Rank` | — | Rank-based selection |
| `Boltzmann` | — | Boltzmann selection (temperature via `SelectionConfiguration`) |
| `Truncation` | — | Truncation selection (threshold via `SelectionConfiguration`) |
| `Clearing` | — | Clearing selection (niche_radius via `SelectionConfiguration`) |

## Related

- [operators/crossover.md](./crossover.md)
- [operators/mutation.md](./mutation.md)
- [chromosomes.md](../chromosomes.md)
- [traits.md](../traits.md)
- [engines.md](../engines.md)
- [src/operations/selection/](../../src/operations/selection/)
