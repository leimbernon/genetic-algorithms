# Constraints

> Constraint handling system for single-objective GA with three penalty strategies and Deb's feasibility rules.

## Overview

The constraint handling system provides strategies for enforcing constraints on single-objective optimization problems. When a solution violates constraints, its fitness is penalized based on the configured strategy. During comparison operations (selection, survivor, elite), **Deb's feasibility rules** determine which individual wins.

The library provides:
- **Three penalty strategies**: Static, Dynamic, Adaptive — plus a "Death" option via `PenaltyStrategy::None` with feasibility rules (discard all infeasible).
- **Feasibility rules** for tournament selection and survivor comparison.
- A `total_violation()` helper function to aggregate per-constraint violations.
- The `RepairOperator` trait for custom constraint repair (not yet integrated into the main GA loop).

## Key Concepts

### PenaltyStrategy

| Variant | Formula | Description |
|---------|---------|-------------|
| `None` | No penalty | Infeasible solutions are handled only by feasibility rules |
| `Static { coefficient }` | `fitness + R * sum(violations)` | Fixed penalty coefficient R |
| `Dynamic { c, alpha, beta }` | `fitness + (C*g)^a * sum(v^b)` | Joines & Houck (1994) — penalty increases with generation |
| `Adaptive { initial_coefficient, window_size }` | Adjusts coefficient every window_size gens | Bean & Hadj-Alouane (1997) — coefficient increases if best is feasible, decreases if infeasible |

### ConstraintHandling

| Variant | Description |
|---------|-------------|
| `FeasibilityRules` | Deb's feasibility rules (Deb, 2000): (1) Between two feasible individuals, the one with better fitness wins. (2) Feasible always beats infeasible. (3) Between two infeasible individuals, the one with lower total violation wins. |

### Constraint Violation Format

Constraints should return individual violation values, where 0 means satisfied and any positive value means violated. Use `total_violation(&[f64])` to sum all violations for penalty calculation:

```rust,ignore
let violations = vec![
    (2.0 - x).max(0.0),  // x >= 2
    (x - 5.0).max(0.0),  // x <= 5
];
let total = genetic_algorithms::constraints::total_violation(&violations);
```

## Usage Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::constraints::PenaltyStrategy;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};

type MyChromosome = RangeChromosome<f64>;

// Constraint: x must be >= 2 and <= 5
let constraint_fn = |dna: &[RangeGenotype<f64>]| -> Vec<f64> {
    let x = dna[0].value;
    vec![
        (2.0 - x).max(0.0),  // x >= 2
        (x - 5.0).max(0.0),  // x <= 5
    ]
};

let alleles = vec![RangeGenotype::new(0, vec![(0.0, 10.0)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut ga = Ga::new()
    .with_genes_per_chromosome(1_usize)
    .with_population_size(100)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(|dna: &[RangeGenotype<f64>]| -> f64 {
        // Minimize (x - 3)^2 subject to x >= 2 and x <= 5
        let x = dna[0].value;
        (x - 3.0).powi(2)
    })
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_constraints(constraint_fn)
    .with_penalty_strategy(PenaltyStrategy::Static { coefficient: 100.0 })
    .with_max_generations(500)
    .build()
    .expect("Valid configuration");

ga.run().expect("GA run failed");
```

## Configuration

Configure constraints on `Ga` using:

```rust,ignore
// Static penalty
.with_penalty_strategy(PenaltyStrategy::Static { coefficient: 100.0 })

// Dynamic penalty (Joines & Houck 1994)
.with_penalty_strategy(PenaltyStrategy::Dynamic {
    c: 0.5,
    alpha: 1.0,
    beta: 2.0,
})

// Adaptive penalty (Bean & Hadj-Alouane 1997)
.with_penalty_strategy(PenaltyStrategy::Adaptive {
    initial_coefficient: 1.0,
    window_size: 10,
})
```

The constraint function itself is provided via `.with_constraints(fn)`:

```rust,ignore
.with_constraints(|dna: &[RangeGenotype<f64>]| -> Vec<f64> {
    vec![
        (2.0 - dna[0].value).max(0.0),
        (dna[1].value - 5.0).max(0.0),
    ]
})
```

## See Also

- [Error Handling](error.md) — GaError enum variants for constraint configuration errors
- [Hall of Fame](hall_of_fame.md) — Elite solution archive
- [docs.rs/genetic_algorithms::constraints](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/constraints/index.html) — Module API reference
- [constrained_g1 example](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/examples/constrained_g1.rs)
