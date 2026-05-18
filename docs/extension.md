# Extension Strategies

> Diversity-rescue mechanisms that trigger when population diversity drops below a configurable threshold.

## Overview

Extension strategies are optional diversity-rescue mechanisms that monitor population diversity (measured by fitness standard deviation). When diversity falls below a configurable threshold, the extension fires and applies a corrective action to restore diversity and prevent premature convergence.

The library provides four diversity-restoring strategies plus a `Noop` option. Each strategy protects a configurable number of elite individuals from the corrective action.

## Key Concepts

### ExtensionConfiguration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `method` | `Extension` | `Noop` | The extension strategy to use. |
| `diversity_threshold` | `f64` | `0.01` | Fitness standard deviation threshold. Extension triggers when diversity drops below this. |
| `survival_rate` | `f64` | `0.1` | For MassExtinction: fraction of population that survives (0.0..1.0). |
| `mutation_rounds` | `usize` | `3` | For MassDegeneration: number of mutation rounds on non-elite. |
| `elite_count` | `usize` | `1` | Number of elite individuals protected from the extension event. |

### Extension Strategy Variants

| Variant | Description | Best For |
|---------|-------------|----------|
| `Noop` | No action (default). | No diversity intervention. |
| `MassExtinction` | Random cull of non-elite individuals; `survival_rate` controls how many survive. | Restarting exploration when stuck. |
| `MassGenesis` | Trim to 2 best individuals, regrow the entire population from them. | Aggressive diversity reset. |
| `MassDegeneration` | Apply multiple rounds of mutation to non-elite chromosomes, with increasing mutation strength. | Gradual diversity restoration. |
| `MassDeduplication` | Remove duplicate chromosomes (identical gene ID sequences). | Clean up redundancy. |

## Usage Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::extension::configuration::ExtensionConfiguration;
use genetic_algorithms::operations::{
    Crossover, Mutation, Selection, Survivor, Extension,
};

type MyChromosome = RangeChromosome<f64>;

let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();

// Configure MassExtinction extension
let ext_config = ExtensionConfiguration::new()
    .with_method(Extension::MassExtinction)
    .with_diversity_threshold(0.05)
    .with_survival_rate(0.2)
    .with_elite_count(2);

let mut ga = Ga::new()
    .with_genes_per_chromosome(5_usize)
    .with_population_size(100)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(|dna: &[RangeGenotype<f64>]| -> f64 {
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
    .with_max_generations(1000)
    .with_extension_method(Extension::MassExtinction)
    .with_extension_config(ext_config)
    .build()
    .expect("Valid configuration");

ga.run().expect("GA run failed");
```

## Configuration

Simpler configuration using the `Ga` builder directly:

```rust,ignore
use genetic_algorithms::operations::Extension;

// Minimal MassExtinction configuration
let mut ga = Ga::new()
    .with_extension_method(Extension::MassExtinction)
    .with_extension_config(ExtensionConfiguration::new()
        .with_diversity_threshold(0.05)
        .with_survival_rate(0.2)
        .with_elite_count(1))
    // ... other configuration
    .build()?;
```

### Deduplication

```rust,ignore
.use genetic_algorithms::operations::Extension;

let mut ga = Ga::new()
    .with_extension_method(Extension::MassDeduplication)
    .with_extension_config(ExtensionConfiguration::new()
        .with_diversity_threshold(0.01))
    // ... other configuration
    .build()?;
```

## Performance Considerations

- Extension events are rare (only when diversity drops). Most generations incur no overhead.
- MassExtinction and MassGenesis trigger a full population regrowth, which includes `pop_size` initialization calls.
- MassDegeneration applies `mutation_rounds * (pop_size - elite_count)` mutation operations.
- MassDeduplication does a full O(N^2) pairwise gene-ID comparison to detect duplicates.
- The diversity threshold is compared against `GenerationStats.diversity` (fitness standard deviation).

## See Also

- [Operations Overview](operations.md) — Extension operator variants
- [docs.rs/genetic_algorithms::extension](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/extension/index.html) — Module API reference
- [onemax_extension example](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/examples/onemax_extension.rs)
