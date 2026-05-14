# Benchmarks

> Standard benchmark functions for evaluating optimization algorithms — requires the `benchmarks` feature flag.

## Overview

This module provides implementations of well-known single-objective, bi-objective (ZDT), and many-objective (DTLZ) benchmark functions behind the `benchmarks` feature flag. All functions implement the `BenchmarkFn` trait which provides a unified `evaluate(&[f64]) -> Vec<f64>` interface along with metadata (name, bounds, known optimum).

**Feature flag:** Add `benchmarks` to your `Cargo.toml`:

```toml
genetic_algorithms = { version = "2.4", features = ["benchmarks"] }
```

## Key Concepts

### BenchmarkFn Trait

All benchmark functions implement the `BenchmarkFn` trait:

| Method | Returns | Description |
|--------|---------|-------------|
| `name(&self)` | `&'static str` | Human-readable function name. |
| `bounds(&self)` | `&[(f64, f64)]` | Per-dimension `(lower, upper)` bounds. |
| `optimum_value(&self)` | `Vec<f64>` | Known optimum objective vector. |
| `evaluate(&self, x: &[f64])` | `Vec<f64>` | Evaluate at point x. Returns `vec![f]` for single-objective, `vec![f1, f2, ...]` for multi-objective. |

### Single-Objective Benchmarks

| Function | Type | Dimensions | Bounds | Optimum |
|----------|------|------------|--------|---------|
| `Sphere` | Unimodal, convex | Any | [-100, 100] | f(0) = 0 |
| `Rastrigin` | Multimodal | Any | [-5.12, 5.12] | f(0) = 0 |
| `Ackley` | Multimodal | Any | [-32, 32] | f(0) = 0 |

### ZDT Bi-Objective Benchmarks

| Function | Type | Dimensions | Features |
|----------|------|------------|----------|
| `ZDT1` | Convex | 30 | Convex Pareto front |
| `ZDT2` | Non-convex | 30 | Non-convex Pareto front |
| `ZDT3` | Disconnected | 30 | Disconnected Pareto front (5 segments) |
| `ZDT4` | Many local fronts | 10 | 21^9 local Pareto fronts |
| `ZDT5` | Deceptive | 11 | Binary-representation biased |
| `ZDT6` | Non-uniform density | 10 | Variable density along the front |

### DTLZ Many-Objective Benchmarks

| Function | Type | Features |
|----------|------|----------|
| `DTLZ1` | Linear | Linear Pareto front (hyperplane f1+...+fM = 0.5) |
| `DTLZ2` | Concave | Spherical Pareto front (quarter-sphere) |
| `DTLZ3` | Many local fronts | DTLZ2 with 10^K local fronts |
| `DTLZ4` | Biased density | DTLZ2 with variable density |
| `DTLZ5` | Degenerate | Reduced-dimension Pareto front |
| `DTLZ6` | Degenerate, biased | DTLZ5 with biased mapping |
| `DTLZ7` | Disconnected | Disconnected Pareto front |

## Usage Example

```rust,ignore
use genetic_algorithms::benchmarks::{Sphere, Rastrigin, BenchmarkFn};

// Evaluate Sphere at the optimum
let sphere = Sphere;
let x = vec![0.0, 0.0, 0.0, 0.0, 0.0];
let result = sphere.evaluate(&x);
println!("Sphere({:?}) = {}", x, result[0]);
assert!(result[0].abs() < 1e-10);

// Evaluate Rastrigin at a non-optimum point
let rastrigin = Rastrigin;
let x = vec![1.0, 2.0, 3.0];
let result = rastrigin.evaluate(&x);
println!("Rastrigin({:?}) = {}", x, result[0]);
```

### Using Benchmarks with a GA

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::benchmarks::{Rastrigin, BenchmarkFn};
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};

type MyChromosome = RangeChromosome<f64>;

let benchmark = Rastrigin;
let bounds = benchmark.bounds().to_vec();
let alleles = vec![RangeGenotype::new(0, bounds, 0.0_f64)];
let alleles_clone = alleles.clone();
let n = bounds.len() as f64;

let mut ga = Ga::new()
    .with_genes_per_chromosome(bounds.len())
    .with_population_size(100)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(move |dna: &[RangeGenotype<f64>]| -> f64 {
        let x: Vec<f64> = dna.iter().map(|g| g.value).collect();
        benchmark.evaluate(&x)[0]
    })
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("Valid configuration");

ga.run().expect("GA run failed");
```

### Multi-Objective Benchmarks

```rust,ignore
use genetic_algorithms::benchmarks::{ZDT1, DTLZ2, BenchmarkFn};

// ZDT1: bi-objective convex front
let zdt1 = ZDT1;
let x = vec![0.5; 30];  // 30-dimensional input
let result = zdt1.evaluate(&x);
println!("ZDT1 f1={:.4}, f2={:.4}", result[0], result[1]);

// DTLZ2: many-objective spherical front
let dtlz2 = DTLZ2;
let x = vec![0.5; 12];  // 12-dimensional input
let result = dtlz2.evaluate(&x);
println!("DTLZ2: {:?}", result);
```

## See Also

- [Multi-Objective Concepts](multi_objective.md) — Quality indicators for benchmark evaluation
- [docs.rs/genetic_algorithms::benchmarks](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/benchmarks/index.html) — Module API reference
- [Cargo.toml feature flags](https://github.com/leimbernon/rust_genetic_algorithms/blob/main/Cargo.toml)
