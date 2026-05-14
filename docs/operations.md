# Operations Overview

> All operator categories — selection, crossover, mutation, survivor, and extension — dispatched via the enum + factory function pattern at runtime.

## Overview

The operations module provides five categories of operators, each dispatched via runtime enum variants with factory functions. This pattern allows users to select operators at configuration time without coupling to specific implementations.

Each operator category is defined as an enum in `src/operations.rs`, with individual implementations in sub-modules under `src/operations/<category>/`. The factory functions select the correct implementation based on the enum variant at runtime.

## Selection Operators

Determines how individuals are chosen from the current population to become parents:

| Variant | Description | Use Case |
|---------|-------------|----------|
| `Random` | Equal probability for every individual. | Baseline; no selection pressure. |
| `RouletteWheel` | Fitness-proportionate probability. | Standard proportional selection. |
| `StochasticUniversalSampling` | Evenly spaced pointers for lower variance. | Lower variance than roulette. |
| `Tournament` | Pairwise competition: fitter individual wins. | Balanced pressure + diversity. |
| `Rank` | Probability proportional to rank, not raw fitness. | Avoids domination by outliers. |
| `Boltzmann` | Temperature-controlled selection pressure. | Adjustable exploration/exploitation. |
| `Truncation` | Only top portion eligible for reproduction. | High selective pressure. |
| `Clearing` | Niche winners only; uses `niche_radius`. | Promotes population diversity. |

## Crossover Operators

Combines two parent chromosomes to produce offspring:

| Variant | Description | Chromosome Type |
|---------|-------------|------------------|
| `Uniform` | Each gene from either parent with equal probability. | Any |
| `SinglePoint` | One cut point splits both parents, halves swapped. | Any |
| `MultiPoint` | Alternating segments at N random cut points. | Any |
| `Order` | Preserves relative ordering (permutation). | Permutation / List<T> |
| `Pmx` | Partially Mapped Crossover for permutations. | Permutation |
| `Cycle` | Preserves position of each gene from one parent. | Permutation |
| `Sbx` | Simulated Binary Crossover with distribution index eta. | Range<T> |
| `BlendAlpha` | BLX-alpha: child in [p1-a, p2+a] extended interval. | Range<T> |
| `Arithmetic` | Weighted average: child = a*p1 + (1-a)*p2. | Range<T> |
| `EdgeRecombination` | Adjacency-preserving for TSP-style problems. | Permutation |
| `Clone` | Copy parents directly without exchange. | Any (mutation-only) |
| `Rejuvenate` | Clone + reset age to zero. | Any (age-aware) |

## Mutation Operators

Randomly alters offspring to maintain diversity:

| Variant | Description | Chromosome Type |
|---------|-------------|------------------|
| `Swap` | Two random genes exchange positions. | Permutation / Any |
| `Inversion` | Random subsequence reversed. | Permutation |
| `Scramble` | Random subsequence shuffled in place. | Permutation |
| `Value` | Single gene replaced with random allele. | Binary / Discrete |
| `BitFlip` | Each bit flipped with a given probability. | Binary |
| `Creep` | Small uniform perturbation by step size. | Range<T> |
| `Gaussian` | Normal distribution perturbation by sigma. | Range<T> |
| `Polynomial` | Polynomial distribution (NSGA-II style) with eta_m. | Range<T> |
| `NonUniform` | Decreasing perturbation over generations. | Range<T> |
| `Insertion` | Move random gene to random position. | Permutation |
| `Cauchy` | Cauchy distribution perturbation by scale. | Range<T> |
| `LevyFlight` | Levy-stable distribution perturbation by alpha. | Range<T> |
| `Uniform` | Replace gene with uniform random value. | Range<T> |
| `ListValue` | Replace gene from finite allele set. | List<T> |
| `Differential` | DE-style difference vector mutation. | Range<T> |

## Survivor Operators

Determines which individuals survive to the next generation:

| Variant | Description | Use Case |
|---------|-------------|----------|
| `Fitness` | Keep top N by fitness. | Standard truncation. |
| `Age` | Keep youngest N individuals. | Novelty / diversity. |
| `MuPlusLambda` | (mu+lambda): combine parents and offspring, select best. | Standard GA pattern. |
| `MuCommaLambda` | (mu,lambda): select only from offspring, discard parents. | Generational replacement. |
| `DeterministicCrowding` | Replace similar individuals, preserving diversity. | Niching / multimodal. |

## Extension Operators

Triggered when population diversity drops below a threshold:

| Variant | Description |
|---------|-------------|
| `Noop` | No action (default). |
| `MassExtinction` | Random cull protecting N elite individuals. |
| `MassGenesis` | Trim to 2 best, regrow population. |
| `MassDegeneration` | Multiple mutation rounds on non-elite. |
| `MassDeduplication` | Remove duplicate chromosomes. |

## Configuration Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::operations::{
    Selection, Crossover, Mutation, Survivor, Extension,
};
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;

let mut ga = Ga::new()
    .with_population_size(100)
    .with_genes_per_chromosome(32)
    .with_initialization_fn(binary_random_initialization)
    .with_fitness_fn(|dna| dna.iter().filter(|g| g.value).count() as f64)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::BitFlip)
    .with_survivor_method(Survivor::Fitness)
    .with_extension_method(Extension::MassExtinction)
    .with_diversity_threshold(0.05)
    .with_problem_solving(ProblemSolving::Maximization)
    .with_max_generations(500)
    .build()
    .expect("Valid configuration");
```

## See Also

- [docs/operators/selection.md](operators/selection.md) — Detailed selection operator reference
- [docs/operators/crossover.md](operators/crossover.md) — Detailed crossover operator reference
- [docs/operators/mutation.md](operators/mutation.md) — Detailed mutation operator reference
- [docs/operators/survivor.md](operators/survivor.md) — Detailed survivor operator reference
- [docs/operators/extension.md](operators/extension.md) — Detailed extension operator reference
- [docs.rs/genetic_algorithms::operations](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/operations/index.html) — Module API reference
