# Crossover Operators

> Genetic algorithm operators for combining parent solutions to produce offspring.

## Overview

Crossover operators are a fundamental component of genetic algorithms, responsible for combining genetic material from two or more parent chromosomes to generate new offspring. The choice of crossover strategy can significantly affect the diversity and convergence properties of the population, making it crucial for effective evolutionary search.

This module provides several crossover techniques, each tailored to different chromosome representations and problem domains. For example, uniform and single-point crossover are commonly used for binary or fixed-length chromosomes, while cycle and order crossover are designed for permutation-based problems. Advanced operators like blend-alpha and simulated binary crossover (SBX) support real-valued chromosomes, expanding the library's applicability.

Crossover operators are typically invoked during the reproduction phase of a genetic algorithm, after parent selection and before mutation. Users can select and configure the appropriate crossover operator based on their problem requirements, and even compose custom strategies using the provided factory function.

## Available Operators

The following `Crossover` enum variants are available:

| Variant | Chromosome Type | Description |
|---------|----------------|-------------|
| `Uniform` | Any (`ChromosomeT`) | Randomly selects genes from either parent for each locus |
| `SinglePoint` | Any | Splits parents at a random point and swaps segments |
| `MultiPoint` | Any | Uses multiple crossover points for segment swapping |
| `Cycle` | Permutation | Preserves absolute positions from parents (cycle method) |
| `Order` | Permutation | Maintains relative order from parents (order method) |
| `Pmx` | Permutation | Partially Mapped Crossover — preserves absolute positions within a segment and relative order outside |
| `Sbx` | Real-valued (`Range<T>`) | Simulated binary crossover for continuous genes |
| `BlendAlpha` | Real-valued (`Range<T>`) | Blends gene values using an alpha parameter |
| `Arithmetic` | Real-valued (`Range<T>`) | Child = alpha * parent1 + (1 - alpha) * parent2 |
| `Clone` | Any | Copies parents directly as offspring |
| `Rejuvenate` | Any | Clones parents as offspring and resets their ages to zero |
| `EdgeRecombination` | Permutation | Preserves adjacency relationships from both parents for TSP/scheduling |
| `VariableLength(AlignmentStrategy)` | Variable-length | Single-point crossover after aligning parents of different lengths |

### Common Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `parent_1` | `&U` | First parent chromosome |
| `parent_2` | `&U` | Second parent chromosome |
| `points` | `usize` | Number of crossover points (for multipoint) |
| `alpha` | `f64` | Blending factor (for blend-alpha) |
| `probability` | `f64` | Crossover probability (for factory, aga_probability) |

## Usage

### Basic Example

```rust
use genetic_algorithms::chromosomes::Range;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::Crossover;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, CrossoverConfig,
    SelectionConfig, MutationConfig, StoppingConfig};

let mut ga: Ga<Range<f64>> = Ga::new()
    .with_crossover_method(Crossover::BlendAlpha)
    .with_blend_alpha(0.5)
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    .with_fitness_fn(|dna| dna.iter().map(|g| g.value).sum())
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("valid config");
```

### Configuration via Builder

```rust
use genetic_algorithms::operations::Crossover;
use genetic_algorithms::chromosomes::Range;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Mutation, Selection, Survivor};
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, CrossoverConfig,
    MutationConfig, SelectionConfig, StoppingConfig};

let mut ga: Ga<Range<f64>> = Ga::new()
    .with_crossover_method(Crossover::BlendAlpha)
    .with_blend_alpha(0.5)
    .with_crossover_rate(0.9)
    // ... other configuration
    .build()
    .expect("valid config");
```

## Operator Details

### Uniform

Performs uniform crossover between two parent chromosomes. Each gene is randomly selected from either parent.

**Signature:**
```rust
pub fn uniform<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

**Variant:** `Crossover::Uniform`

---

### SinglePoint

Performs single-point crossover, splitting parents at a random point.

**Signature:**
```rust
pub fn single_point<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

**Variant:** `Crossover::SinglePoint`

---

### MultiPoint

Performs multipoint crossover using a specified number of crossover points.

**Signature:**
```rust
pub fn multipoint<U: ChromosomeT>(parent_1: &U, parent_2: &U, points: usize) -> Result<Vec<U>, GaError>
```

**Variant:** `Crossover::MultiPoint`

---

### Cycle

Implements cycle crossover for permutation chromosomes, preserving absolute positions.

**Signature:**
```rust
pub fn cycle<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

**Variant:** `Crossover::Cycle`

---

### Order (OX)

Implements order crossover for permutation chromosomes, preserving relative order.

**Signature:**
```rust
pub fn order<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

**Variant:** `Crossover::Order`

---

### PMX (Partially Mapped Crossover)

Implements partially mapped crossover for permutation chromosomes. Preserves absolute positions within a segment and relative order outside it.

**Signature:**
```rust
pub fn pmx<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError>
```

**Variant:** `Crossover::Pmx`

---

### SBX (Simulated Binary Crossover)

Performs simulated binary crossover for real-valued `Range<T>` chromosomes. Uses a distribution index (eta) configured via `with_sbx_eta()`.

**Variant:** `Crossover::Sbx`

| Builder Method | Default | Description |
|---------------|---------|-------------|
| `with_sbx_eta(f64)` | `20.0` | Distribution index — higher values produce offspring closer to parents |

---

### BlendAlpha (BLX-alpha)

Performs blend-alpha crossover for real-valued `Range<T>` chromosomes, blending gene values using an alpha parameter. Offspring genes are sampled uniformly from `[min - alpha * range, max + alpha * range]`.

**Variant:** `Crossover::BlendAlpha`

| Builder Method | Default | Description |
|---------------|---------|-------------|
| `with_blend_alpha(f64)` | `0.5` | Blending factor (0.0 to 1.0) |

---

### Arithmetic

Performs arithmetic (whole) crossover for `Range<T>` chromosomes. Offspring = alpha * parent1 + (1 - alpha) * parent2.

**Variant:** `Crossover::Arithmetic`

| Builder Method | Default | Description |
|---------------|---------|-------------|
| `with_arithmetic_alpha(f64)` | `0.5` | Blending weight |

---

### Clone

Copies parents directly as offspring without any genetic exchange. Useful for mutation-only strategies and baseline experiments where crossover is not desired but must be present for API compatibility.

**Variant:** `Crossover::Clone`

---

### Rejuvenate

Clones parents as offspring and resets their ages to zero. Prevents age-based survivor selection from eliminating top performers.

**Variant:** `Crossover::Rejuvenate`

---

### EdgeRecombination

Preserves parental adjacency relationships by building an edge map (adjacency list) from both parents, then constructing offspring via edge recombination. Best for permutation problems (TSP, scheduling). Produces offspring that respect the adjacency relationships present in either parent.

**Algorithm:**
1. Build a union adjacency map where each gene ID maps to the set of its neighbours in either parent (circular: first and last genes are adjacent).
2. Starting from a chosen gene, iteratively select the next gene as the unvisited neighbour with the fewest remaining unvisited neighbours.
3. On tie or exhausted neighbour list, fall back to a random unvisited gene.

**Configuration:** No additional parameters — operates on both parents' adjacency.

**Variant:** `Crossover::EdgeRecombination`

**When to use:** Permutation problems, TSP, scheduling, any problem where adjacency matters.

**Added in:** v2.4.0

---

### VariableLength

Applies single-point crossover to two chromosomes that may have different DNA lengths. Before recombination the parents are aligned according to the `AlignmentStrategy`:

- `AlignmentStrategy::Trim` — both parents are truncated to `min(len_a, len_b)`. Offspring length = `min(len_a, len_b)`.
- `AlignmentStrategy::Pad` — the shorter parent is padded with genes sampled from its own DNA until both reach `max(len_a, len_b)`. Offspring length = `max(len_a, len_b)`.

**Variant:** `Crossover::VariableLength(AlignmentStrategy)`

```rust
use genetic_algorithms::operations::{Crossover, AlignmentStrategy};

// Trim both parents to the shorter length before crossover.
.with_crossover_method(Crossover::VariableLength(AlignmentStrategy::Trim))

// Pad the shorter parent (from its own alleles) before crossover.
.with_crossover_method(Crossover::VariableLength(AlignmentStrategy::Pad))
```

**Returns:** `GaError::CrossoverError` if both parents are empty after alignment.

**When to use:** Whenever your population contains chromosomes with different DNA lengths (i.e., `ChromosomeLength::Variable` is enabled). Fixed-length operators (`SinglePoint`, `Uniform`, etc.) return an error when parent lengths differ.

**Added in:** v3.0.0

---

### `factory`

Returns a configured crossover operator function based on a string identifier and optional parameters.

**Signature:**
```rust
pub fn factory<U: ChromosomeT>(pair: (&U, &U), crossover: &CrossoverConfiguration)
    -> Result<Vec<U>, GaError>
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `pair` | `(&U, &U)` | Parent pair to recombine |
| `crossover` | `&CrossoverConfiguration` | Crossover configuration with operator and parameters |

**Returns:**
`Result<Vec<U>, GaError>`

---

### `aga_probability`

Computes adaptive crossover probability for a chromosome.

**Signature:**
```rust
pub fn aga_probability<U: ChromosomeT>(chromosome: &U, base_probability: f64) -> f64
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `chromosome` | `&U` | Chromosome to evaluate |
| `base_probability` | `f64` | Base crossover probability |

**Returns:**
`f64` — Adjusted crossover probability.

## Related

- [Selection Operators](selection.md)
- [Mutation Operators](mutation.md)
- [Chromosome Types](../chromosomes.md)
- [Genotype Definitions](../genotypes.md)
- [Fitness Evaluation](../fitness.md)
- [Source: `src/operations/crossover/`](../../src/operations/crossover/)
- [Examples](../examples.md)
