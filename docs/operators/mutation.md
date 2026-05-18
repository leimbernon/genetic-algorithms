# Mutation Operators

> Mutation operators introduce genetic diversity by altering chromosomes during evolution.

## Overview

Mutation is a fundamental mechanism in genetic algorithms, responsible for introducing random changes to individuals in a population. By modifying genes within chromosomes, mutation helps prevent premature convergence and enables exploration of the solution space. In this library, mutation operators are modular and support a variety of strategies tailored to different chromosome types, such as binary, range, and permutation-based representations.

Users typically configure mutation operators to balance exploration and exploitation in their genetic algorithm. The choice of mutation strategy depends on the problem domain and chromosome encoding. For example, swap, scramble, and inversion are well-suited for permutation problems, while value, creep, and gaussian mutations are designed for numeric range chromosomes. The mutation module provides a unified API for applying these operators, including adaptive mutation probability for advanced scenarios.

Mutation integrates seamlessly with the overall genetic algorithm workflow, working alongside selection and crossover operators. The library's mutation system is extensible, allowing users to implement custom mutation logic if needed.

## Available Operators

The following `Mutation` enum variants are available:

| Variant | Applicable Chromosome | Description |
|---------|----------------------|-------------|
| `Swap` | Permutation | Swaps two randomly chosen genes |
| `Scramble` | Permutation | Randomly shuffles a subsequence of genes |
| `Inversion` | Permutation | Reverses the order of a subsequence of genes |
| `Insertion` | Permutation | Removes a gene and reinserts it at a different position |
| `Value` | Range | Assigns a new random value to a gene within its range |
| `BitFlip` | Binary | Flips a random bit in the chromosome |
| `Creep` | Range | Small uniform perturbation to a gene |
| `Gaussian` | Range | Gaussian (normal distribution) perturbation |
| `Polynomial` | Range | Polynomial mutation (NSGA-II style) |
| `NonUniform` | Range | Non-uniform mutation — magnitude decreases over generations |
| `ListValue` | List | Replaces a single gene's value with a different allele from its allele set |
| `Cauchy` | Range | Heavy-tailed perturbation using Cauchy distribution |
| `LevyFlight` | Range | Long-range jumps via Lévy alpha-stable distribution |
| `Uniform` | Range | Random gene reset to uniform value within gene's valid range |
| `Differential` | Range | DE-style mutation using three random population members |

### Trait: `ValueMutable`

| Method | Signature | Description |
|--------|-----------|-------------|
| `value_mutate` | `fn value_mutate(&mut self)` | Performs value mutation |
| `bit_flip_mutate` | `fn bit_flip_mutate(&mut self)` | Flips a random bit (for binary chromosomes) |
| `creep_mutate` | `fn creep_mutate(&mut self, step: f64)` | Applies creep mutation |
| `gaussian_mutate` | `fn gaussian_mutate(&mut self, sigma: f64)` | Applies gaussian mutation |

Implementations: `RangeChromosome<i32>`, `RangeChromosome<i64>`, `RangeChromosome<f32>`, `RangeChromosome<f64>`.

## Usage

### Basic Example

```rust
use genetic_algorithms::chromosomes::Range;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig,
    SelectionConfig, StoppingConfig,
};

let mut ga: Ga<Range<f64>> = Ga::new()
    .with_mutation_method(Mutation::Gaussian)
    .with_mutation_sigma(0.1)
    .with_mutation_rate(0.05)
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    .with_initialization_fn(range_random_initialization)
    .with_fitness_fn(|dna| dna.iter().map(|g| g.value).sum())
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("valid config");
```

## Operator Details

### Swap

Swaps two randomly chosen genes in a permutation chromosome.

**Variant:** `Mutation::Swap`

---

### Scramble

Randomly shuffles a subsequence of genes in a permutation chromosome.

**Variant:** `Mutation::Scramble`

---

### Inversion

Reverses the order of a subsequence of genes in a permutation chromosome.

**Variant:** `Mutation::Inversion`

---

### Insertion

Removes a gene and reinserts it at a different position in a permutation chromosome.

**Variant:** `Mutation::Insertion`

---

### Value

Assigns a new random value to a gene within its allowed range.

**Variant:** `Mutation::Value`

---

### BitFlip

Flips each bit (gene) in a binary chromosome with a given probability.

**Variant:** `Mutation::BitFlip`

---

### Creep

Applies a small uniform perturbation to each gene in a `Range<T>` chromosome. Requires a step size configured via `with_mutation_step()`.

**Variant:** `Mutation::Creep`

| Builder Method | Default | Description |
|---------------|---------|-------------|
| `with_mutation_step(f64)` | `0.1` | Step size for Creep mutation |

---

### Gaussian

Applies Gaussian (normal distribution) noise to each gene in a `Range<T>` chromosome. Requires sigma configured via `with_mutation_sigma()`.

**Variant:** `Mutation::Gaussian`

| Builder Method | Default | Description |
|---------------|---------|-------------|
| `with_mutation_sigma(f64)` | `0.1` | Standard deviation for Gaussian mutation |

---

### Polynomial

Polynomial mutation for `Range<T>` chromosomes (NSGA-II style). Uses a distribution index (eta_m) configured via `with_polynomial_eta()`.

**Variant:** `Mutation::Polynomial`

| Builder Method | Default | Description |
|---------------|---------|-------------|
| `with_polynomial_eta(f64)` | `20.0` | Distribution index — higher values produce offspring closer to parents |

---

### NonUniform

Non-uniform mutation for `Range<T>` chromosomes. Mutation magnitude decreases over generations, providing exploration early and exploitation later.

**Variant:** `Mutation::NonUniform`

---

### ListValue

Replaces a single gene's value with a different allele from that gene's allele set. Requires a `ListChromosome<T>`.

**Variant:** `Mutation::ListValue`

---

### Cauchy

Heavy-tailed perturbation using the Cauchy (Lorentzian) distribution. Uses the inverse-CDF method: `noise = scale * tan(pi * (u - 0.5))`, where `u ~ Uniform(0, 1)`. Produces occasional large jumps beyond what Gaussian can achieve.

**Variant:** `Mutation::Cauchy`

| Configuration | Type | Default | Description |
|--------------|------|---------|-------------|
| `cauchy_scale` | `f64` | `1.0` | Spread parameter — larger values produce more extreme jumps |

**Builder:** `.with_cauchy_scale(1.0)`

**Returns:** `GaError::MutationError` for non-`Range<T>` chromosomes (Binary, List).

**When to use:** Rugged landscapes where large jumps can escape local optima.

**Added in:** v2.4.0

---

### LevyFlight

Long-range jumps via Mantegna's algorithm for Lévy alpha-stable distribution. Generates heavy-tailed steps via `step = sigma_u * u / |v|^(1/alpha)`.

**Variant:** `Mutation::LevyFlight`

| Configuration | Type | Default | Description |
|--------------|------|---------|-------------|
| `levy_alpha` | `f64` | `1.5` | Stability index (0.0, 2.0) — lower values produce heavier tails |

**Builder:** `.with_levy_alpha(1.5)`

**Returns:** `GaError::MutationError` for non-`Range<T>` chromosomes.

**When to use:** Very rugged landscapes where both local and global exploration are needed.

**Added in:** v2.4.0

---

### Uniform

Random gene reset to a uniform value within the gene's valid range. Equivalent to random re-initialization of selected genes, providing maximum perturbation.

**Variant:** `Mutation::Uniform`

**Returns:** `GaError::MutationError` for non-`Range<T>` chromosomes.

**When to use:** When a complete gene rest is needed to escape local optima.

**Added in:** v2.4.0

---

### Differential

DE-style mutation using three distinct random population members to generate a mutant vector. Formula: `v = x_r1 + F * (x_r2 - x_r3)`, where F is the scale factor. Only applicable to real-valued (`Range<T>`) chromosomes.

**Variant:** `Mutation::Differential`

| Configuration | Type | Default | Description |
|--------------|------|---------|-------------|
| `differential_f` | `f64` | `0.5` | Scale factor F — differential weight |

**Builder:** `.with_differential_f(0.5)`

**Requires:** `population_size >= 4`.

**Note:** Applied automatically by the standard GA engine — do not call `factory_with_params` for this variant.

**When to use:** Continuous optimization where DE-style exploration is beneficial.

**Added in:** v2.4.0

---

### Factory Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `factory` | `fn factory<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>` | Applies the specified mutation operator |
| `factory_with_params` | `fn factory_with_params<U>(...)` | Applies mutation with optional parameters |
| `factory_non_value` | `fn factory_non_value<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>` | Applies non-value mutation operators |
| `aga_probability` | `fn aga_probability<U: ChromosomeT>(...) -> f64` | Calculates adaptive mutation probability |

## Related

- [operators/selection.md](selection.md)
- [operators/crossover.md](crossover.md)
- [chromosomes.md](../chromosomes.md)
- [traits.md](../traits.md)
- [Source: mutation.rs](../../src/operations/mutation.rs)
- [Source: swap.rs](../../src/operations/mutation/swap.rs)
- [Source: scramble.rs](../../src/operations/mutation/scramble.rs)
- [Source: inversion.rs](../../src/operations/mutation/inversion.rs)
- [Source: value.rs](../../src/operations/mutation/value.rs)
