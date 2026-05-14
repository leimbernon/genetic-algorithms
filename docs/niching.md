# Niching and Fitness Sharing

> Reduces the fitness of similar individuals to maintain population diversity and prevent a single peak from dominating.

## Overview

Fitness sharing (also called niching) is a diversity-preserving mechanism that penalizes individuals that are too similar to others in the population. The adjusted fitness of each individual is reduced by a factor proportional to the number of other individuals within its "niche" — a region of the genotype or fitness space defined by the sharing radius.

The key idea: instead of converging on a single elite solution, the population maintains multiple distinct solutions across different peaks of the fitness landscape.

## Key Concepts

### How Fitness Sharing Works

1. For each individual `i`, compute the **niche count** `m_i` — the sum of sharing function values with all other individuals.
2. Compute **adjusted fitness** as `f'_i = f_i / m_i`.
3. The sharing function is defined as:
   ```
   sh(d_ij) = 1 - (d_ij / sigma_share)^alpha   if d_ij < sigma_share
   sh(d_ij) = 0                                  otherwise
   ```
   where `d_ij` is the distance between individuals i and j, `sigma_share` is the sharing radius, and `alpha` controls the shape of the sharing function.

### NichingConfiguration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `enabled` | `bool` | `false` | Whether fitness sharing is enabled. |
| `sigma_share` | `f64` | `1.0` | Sharing radius. Individuals closer than this share fitness. |
| `alpha` | `f64` | `1.0` | Shape parameter for the sharing function. Higher values make it steeper. |

### Guiding Sigma Share Selection

The sharing radius `sigma_share` is the most important parameter. Guidelines:

- **Known number of peaks (k):** Set `sigma_share` so that each peak's radius covers approximately `pop_size / k` individuals.
- **Unknown landscape:** Start with `sigma_share = 0.1 * (max_bound - min_bound)` for a single dimension, scaled by `sqrt(dimensions)`.
- **Too small:** Niches are too narrow; individuals within a peak don't share, leading to single-peak convergence.
- **Too large:** Fitness sharing becomes effectively uniform; diversity pressure is lost.

## Usage Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::niching::configuration::NichingConfiguration;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};

type MyChromosome = RangeChromosome<f64>;

let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();

// Configure niching
let niching_config = NichingConfiguration::new()
    .with_enabled(true)
    .with_sigma_share(2.0)
    .with_alpha(1.0);

let mut ga = Ga::new()
    .with_genes_per_chromosome(1_usize)
    .with_population_size(200)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(|dna: &[RangeGenotype<f64>]| -> f64 {
        // Multimodal function with two peaks
        let x = dna[0].value;
        (-(x - 2.0).powi(2) / 0.5).exp() + (-(x + 2.0).powi(2) / 0.5).exp()
    })
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Maximization)
    .with_max_generations(500)
    .with_niching_config(niching_config)
    .build()
    .expect("Valid configuration");

ga.run().expect("GA run failed");
```

## Configuration

Use `Ga`'s builder method to attach niching:

```rust,ignore
use genetic_algorithms::niching::configuration::NichingConfiguration;

// Enable with custom parameters
let config = NichingConfiguration::new()
    .with_enabled(true)
    .with_sigma_share(1.5)
    .with_alpha(2.0);

let mut ga = Ga::new()
    // ... other configuration
    .with_niching_config(config)
    .build()?;
```

## Performance Considerations

- Fitness sharing distance computation is O(N^2) in the worst case (pairwise distances). The library uses a single-pass O(N) computation with pre-computed distances where possible.
- Sigma share is in **fitness space** by default (distance measured as fitness value difference). Genotypic distance is not yet supported.
- For large populations (> 1000), consider disabling niching or increasing sigma_share to reduce effective niche comparisons.
- Niching is applied during fitness evaluation, so it does NOT add an extra pass over the population.

## See Also

- [Operations Overview](operations.md) — All operator categories
- [docs.rs/genetic_algorithms::niching](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/niching/index.html) — Module API reference
- [niching example](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/examples/niching.rs)
