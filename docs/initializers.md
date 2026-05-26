# Population Initialization

> Functions and strategies for creating initial populations — random initialization, seeded warm-start initialization, and checkpoint resumption.

## Overview

Population initialization creates the initial DNA for each chromosome before the first generation. The GA accepts any function with the signature `Fn(usize, Option<&[Gene]>, Option<bool>) -> Vec<Gene>`, but the library provides convenient defaults for common chromosome types.

The initialization module provides functions for:
- **Binary (bit-string)** chromosomes
- **Range (numeric bounded)** chromosomes
- **List (finite symbolic alphabet)** chromosomes
- **Generic** random initialization from any allele set

## Key Concepts

### Initialization Function Signature

All initialization functions follow the same signature:
```rust,ignore
Fn(genes_per_chromosome: usize, alleles: Option<&[Gene]>, can_repeat: Option<bool>) -> Vec<Gene>
```

| Parameter | Description |
|-----------|-------------|
| `genes_per_chromosome` | Number of genes to create for this chromosome. |
| `alleles` | Optional template of allowed gene values. |
| `can_repeat` | If Some(false), disallows duplicate gene IDs. Default: Some(true). |

### Available Initialization Functions

| Function | Chromosome Type | Description |
|----------|-----------------|-------------|
| `binary_random_initialization` | `chromosomes::Binary` | Random `true`/`false` genes. |
| `range_random_initialization` | `chromosomes::Range<T>` | Random numeric genes within allele bounds. |
| `list_random_initialization` | `chromosomes::ListChromosome<T>` | Random allele-index genes from a finite allele set (with repetition). |
| `list_random_initialization_without_repetitions` | `chromosomes::ListChromosome<T>` | Random permutation of the allele set (no repetition). |
| `generic_random_initialization` | Any `ChromosomeT` | Random selection from a provided alleles list. |
| `generic_random_initialization_without_repetitions` | Any `ChromosomeT` | Random selection without repetition. |
| `gp::ramped_half_and_half` | `gp::GpChromosome<N>` | Standard GP initializer: combines the `full` and `grow` tree-generation methods across depths `2..=init_max_depth` to maximize structural diversity. Used by `GpGa<N>` by default. |

### GP initialization

Tree chromosomes use a different signature — `Fn(pop_size, init_max_depth, &mut Rng) -> Vec<GpChromosome<N>>` — because there are no linear "genes per chromosome" to configure. The default `GpGa::with_ramped_half_and_half(config, fitness_fn)` constructor uses the built-in `gp::ramped_half_and_half` function automatically. To install a custom initializer, use `GpGa::new(config, fitness_fn, my_init_fn)`. See [Genetic Programming](gp.md) for details.

## Usage Examples

### Binary Initialization

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::configuration::ProblemSolving;

let mut ga = Ga::new()
    .with_genes_per_chromosome(32)
    .with_population_size(100)
    .with_initialization_fn(binary_random_initialization)
    .with_fitness_fn(|dna: &[BinaryGene]| -> f64 {
        dna.iter().filter(|g| g.value).count() as f64
    })
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::BitFlip)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Maximization)
    .with_max_generations(500)
    .build()
    .expect("Valid configuration");
```

### Range Initialization

```rust,ignore
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;

type MyChromosome = RangeChromosome<f64>;

let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut ga = Ga::new()
    .with_genes_per_chromosome(10_usize)
    .with_population_size(100)
    .with_initialization_fn(move |n, _, _| {
        range_random_initialization(n, Some(&alleles_clone), Some(false))
    })
    // ... rest of configuration
    .build()
    .expect("Valid configuration");
```

### List Initialization

```rust,ignore
use genetic_algorithms::chromosomes::ListChromosome;
use genetic_algorithms::genotypes::List as ListGene;
use genetic_algorithms::initializers::list_random_initialization;

// Alleles for a permutation problem (TSP-style)
let alleles = vec![
    ListGene::new(0, String::from("A")),
    ListGene::new(1, String::from("B")),
    ListGene::new(2, String::from("C")),
    ListGene::new(3, String::from("D")),
];
let alleles_clone = alleles.clone();

let mut ga = Ga::new()
    .with_genes_per_chromosome(4_usize)
    .with_population_size(50)
    .with_initialization_fn(move |n, _, _| {
        list_random_initialization(n, Some(&alleles_clone), Some(false))
    })
    // ... rest of configuration
    .build()
    .expect("Valid configuration");
```

### Seeded Initialization (Warm Start)

To resume from a previous solution or seed the population with known good candidates, pass a custom initialization function:

```rust,ignore
use genetic_algorithms::genotypes::Binary as BinaryGene;

// Seed with a known good chromosome, then random
let known_dna: Vec<BinaryGene> = vec![
    BinaryGene::new(true), BinaryGene::new(true),
    BinaryGene::new(false), BinaryGene::new(true),
];

let mut ga = Ga::new()
    .with_genes_per_chromosome(32)
    .with_population_size(100)
    .with_initialization_fn(move |n, _, _| {
        known_dna.clone()  // All individuals start from the same seed
    })
    // ... rest of configuration
    .build()
    .expect("Valid configuration");
```

## Configuration

Initialization is always set via the `.with_initialization_fn()` builder method. The signature must match:

```rust,ignore
.with_initialization_fn(|genes_per_chromosome: usize,
                         alleles: Option<&[U::Gene]>,
                         can_repeat: Option<bool>| -> Vec<U::Gene> { ... })
```

For multi-objective engines, the same pattern applies:

```rust,ignore
let mut nsga3 = Nsga3Ga::new(nsga3_config, ga_config)
    .with_initialization_fn(move |n, _, _| {
        range_random_initialization(n, Some(&alleles_clone), Some(false))
    })
    // ... rest of configuration
    .build()?;
```

## Performance Considerations

- Initialization runs once, before the generation loop. The cost is proportional to `pop_size * genes_per_chromosome`.
- For `Range<T>` chromosomes, `range_random_initialization` creates uniformly random values within bounds.
- For `List<T>` without repetition, the initialization is O(n) per chromosome using Fisher-Yates shuffle.
- The initialization function is moved into the GA struct — closures that capture large data should use `Arc` or `move` to avoid copying.

## See Also

- [docs.rs/genetic_algorithms::initializers](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/initializers/index.html) — Module API reference
