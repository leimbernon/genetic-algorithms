# Mutation Operators

> Mutation operators introduce genetic diversity by altering chromosomes during evolution.

## Overview

Mutation is a fundamental mechanism in genetic algorithms, responsible for introducing random changes to individuals in a population. By modifying genes within chromosomes, mutation helps prevent premature convergence and enables exploration of the solution space. In this library, mutation operators are modular and support a variety of strategies tailored to different chromosome types, such as binary, range, and permutation-based representations.

Users typically configure mutation operators to balance exploration and exploitation in their genetic algorithm. The choice of mutation strategy depends on the problem domain and chromosome encoding. For example, swap, scramble, and inversion are well-suited for permutation problems, while value, creep, and gaussian mutations are designed for numeric range chromosomes. The mutation module provides a unified API for applying these operators, including adaptive and dynamic mutation probability for advanced scenarios.

Mutation integrates seamlessly with the overall genetic algorithm workflow, working alongside selection and crossover operators. The library's mutation system is extensible, allowing users to implement custom mutation logic if needed.

## Key Concepts

Mutation operators are implemented as functions and traits, each targeting specific chromosome types. The core abstractions are summarized below:

| Operator        | Applicable Chromosome | Description                                               |
|-----------------|----------------------|-----------------------------------------------------------|
| `swap`          | Permutation          | Swaps two randomly chosen genes                           |
| `scramble`      | Permutation          | Randomly shuffles a subsequence of genes                  |
| `inversion`     | Permutation          | Reverses the order of a subsequence of genes              |
| `value`         | Numeric              | Replaces a gene with a random value from the allowed range|
| `creep`         | Numeric              | Adds a small random value to a gene                       |
| `gaussian`      | Numeric              | Perturbs a gene using a Gaussian distribution             |
| `polynomial`    | Numeric              | Applies polynomial mutation for fine control              |
| `non_uniform`   | Numeric              | Mutation magnitude decreases over generations             |

### Dynamic Mutation Probability

The library now supports **dynamic mutation probability adjustment**. This feature adapts the mutation probability during evolution based on population diversity, specifically the cardinality ratio (unique fitness values divided by population size). When enabled, mutation probability increases if diversity drops below a target, and decreases if diversity exceeds the target, helping maintain a healthy balance between exploration and exploitation.

#### Configuration Parameters

- `dynamic_mutation`: Enables or disables dynamic mutation probability adjustment.
- `target_cardinality`: Sets the desired diversity ratio (between 0.0 and 1.0).
- `probability_step`: Specifies how much to adjust mutation probability per generation.
- `probability_min` / `probability_max`: Set lower and upper bounds for mutation probability.

## Usage

Mutation operators and their configuration are controlled via builder methods on the `Ga` struct. The dynamic mutation probability feature can be enabled and tuned as follows:

```rust
use genetic_algorithms::prelude::*;
use genetic_algorithms::mutation::Mutation;

// Example: Configure a GA with dynamic mutation probability
let mut ga = Ga::new()
    .with_population_size(100)
    .with_genes_per_chromosome(8)
    .with_max_generations(500)
    .with_mutation_method(Mutation::Creep)
    .with_mutation_probability_max(0.2) // Upper bound for mutation probability
    .with_mutation_probability_min(0.01) // Lower bound for mutation probability
    .with_dynamic_mutation(true) // Enable dynamic mutation probability
    .with_mutation_target_cardinality(0.5) // Target: 50% unique fitness values
    .with_mutation_probability_step(0.02) // Adjust probability by 0.02 per generation
    .with_fitness_fn(|dna: &[f64]| {
        // Example fitness function
        dna.iter().sum()
    })
    .with_initialization_fn(|len, alleles, _| {
        // Example initialization function
        alleles.choose_multiple(&mut rand::thread_rng(), len).cloned().collect()
    })
    .build()
    .expect("Failed to build GA");

// Run the genetic algorithm
let population = ga.run().expect("GA run failed");
println!("Best chromosome: {:?}", population.best_chromosome);
```

**How it works:**  
- `with_dynamic_mutation(true)` activates adaptive mutation probability.
- `with_mutation_target_cardinality(0.5)` sets the diversity goal (e.g., 50% of the population should have unique fitness).
- `with_mutation_probability_step(0.02)` controls how quickly mutation probability is adjusted.
- `probability_min` and `probability_max` ensure mutation probability stays within reasonable bounds.

During evolution, the library monitors the population's diversity. If diversity falls below the target, mutation probability is increased (up to `probability_max`). If diversity exceeds the target, mutation probability is decreased (down to `probability_min`). This adaptive mechanism helps maintain genetic diversity and avoid stagnation.

### Interaction with Other Mutation Parameters

- **Mutation Method:** Dynamic probability works with all mutation methods (e.g., `Creep`, `Gaussian`, `Swap`).
- **Probability Bounds:** The dynamic adjustment respects `probability_min` and `probability_max`, never exceeding these limits.
- **Step Size:** The `probability_step` determines how aggressively mutation probability changes per generation.
- **Disabling Dynamic Mutation:** Set `with_dynamic_mutation(false)` to revert to fixed mutation probability.

## API Reference

### Mutation Configuration Builder Methods

| Method                                 | Description                                                    |
|-----------------------------------------|----------------------------------------------------------------|
| `with_mutation_method(Mutation)`        | Sets the mutation operator                                     |
| `with_mutation_probability_max(f64)`    | Sets the maximum mutation probability                          |
| `with_mutation_probability_min(f64)`    | Sets the minimum mutation probability                          |
| `with_dynamic_mutation(bool)`           | Enables/disables dynamic mutation probability                  |
| `with_mutation_target_cardinality(f64)` | Sets the target cardinality ratio for diversity                |
| `with_mutation_probability_step(f64)`   | Sets the adjustment step for mutation probability              |
| `with_mutation_step(f64)`               | Sets step size for Creep mutation                             |
| `with_mutation_sigma(f64)`              | Sets sigma for Gaussian mutation                              |
| `with_mutation_polynomial_eta(f64)`     | Sets eta for Polynomial mutation                              |
| `with_mutation_non_uniform_b(f64)`      | Sets decay parameter for NonUniform mutation                   |

See [API docs](../api/ga.md) for full details.

---

Dynamic mutation probability is a powerful mechanism for maintaining genetic diversity and improving search effectiveness in evolutionary runs. Configure it via the builder methods to suit your problem's needs.
