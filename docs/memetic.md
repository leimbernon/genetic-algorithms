# Memetic Algorithms

> Local search integration with GA — combines global evolutionary exploration with local refinement.

> **Note:** This guide reflects the API as of Phase 45. The memetic algorithm framework is an active feature — see Phase 45 for current status and implementation details.

## Overview

Memetic algorithms (also called hybrid or cultural GAs) embed a local search step into the GA loop. After offspring are created via crossover and mutation, a local search operator refines promising individuals before they enter the survivor pool. This hybrid approach combines the global exploration of evolutionary search with the local exploitation of hill-climbing or gradient-based methods.

The library provides:
- A `LocalSearchOperator` trait for defining custom local search strategies.
- A built-in `HillClimbing` strategy for continuous optimization.
- Application strategies to control when and how local search is applied.
- Two modes: **Lamarckian** (offspring replaced with improved version) and **Baldwinian** (fitness updated, DNA preserved).

## Key Concepts

### LocalSearch Trait

Defined in `src/operations/local_search/`:

| Method | Description |
|--------|-------------|
| `search(&self, individual, generation)` | Apply local search to an individual. Returns the improved chromosome. |

### HillClimbingConfig

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `step_size` | `f64` | `0.1` | Perturbation magnitude for each dimension. |
| `max_steps` | `usize` | `20` | Maximum local search steps per application. |
| `bounds` | `Vec<(f64, f64)>` | — | Per-dimension bounds for the search space. |

### LocalSearchMode

| Variant | Description |
|---------|-------------|
| `Lamarckian` | The improved chromosome replaces the original. The local search result is inherited by offspring. |
| `Baldwinian` | Only the fitness is updated; the original chromosome DNA is preserved. The experience is not inherited. |

### LocalSearchApplicationStrategy

Controls which offspring receive local search:

| Variant | Description |
|---------|-------------|
| `AllOffspring` | Apply local search to every new offspring. |
| `BestN(usize)` | Apply to the top N offspring by fitness. |
| `Probabilistic(f64)` | Apply with probability p to each offspring. |
| `EveryNGenerations(usize)` | Apply only every N generations. |

## Usage Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::operations::{
    Crossover, Mutation, Selection, Survivor,
    local_search::{HillClimbingConfig, LocalSearchMode, LocalSearchApplicationStrategy},
};
use std::sync::Arc;

type MyChromosome = RangeChromosome<f64>;

let bounds = vec![(-5.12, 5.12); 5];
let alleles = vec![RangeGenotype::new(0, bounds.clone(), 0.0_f64)];
let alleles_clone = alleles.clone();
let bounds_clone = bounds.clone();

let mut ga = Ga::new()
    .with_genes_per_chromosome(5_usize)
    .with_population_size(100)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(move |dna: &[RangeGenotype<f64>]| -> f64 {
        // Rastrigin function
        let a = 10.0;
        let n = dna.len() as f64;
        a * n + dna.iter()
            .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
            .sum::<f64>()
    })
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian)
    .with_mutation_sigma(0.1)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("Valid configuration");

ga.run().expect("GA run failed");
```

Note: The actual API for attaching local search to `Ga` may vary based on Phase 45 implementation. The `LocalSearch` type and `factory` functions are available in `genetic_algorithms::operations::local_search`.

## Configuration Tips

- Use Lamarckian mode for problems where the local search result can be exploited by the GA (most continuous optimization problems).
- Use Baldwinian mode when you want the GA to evolve its own solution structure while still benefiting from local evaluation.
- `BestN(5)` with `EveryNGenerations(10)` is a good starting point — frequent enough to guide the search, rare enough to keep computational cost manageable.
- Hill climbing is most effective on smooth, continuous landscapes. For combinatorial or discrete problems, implement a custom `LocalSearchOperator`.

## Performance Considerations

- Local search is the most expensive operation in the GA loop. Apply it selectively using `EveryNGenerations` and `BestN` strategies.
- Hill climbing does one evaluation per step per selected individual. For `max_steps=20` applied to `BestN(5)`, each local search cycle costs 100 extra fitness evaluations.
- The local search is sequential within each generation — it does not benefit from rayon parallelism (the GA loop's crossover and mutation are already parallelized).

## See Also

- [Operations Overview](operations.md) — All operator categories including local search
- [docs.rs/genetic_algorithms::operations::local_search](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/operations/local_search/index.html) — Module API reference
- [memetic_rastrigin example](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/examples/memetic_rastrigin.rs)
