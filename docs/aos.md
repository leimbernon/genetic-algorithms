# Adaptive Operator Selection (AOS)

> Dynamically selects crossover/mutation operators from a portfolio based on recent fitness-improvement credit.

## Overview

AOS is the runtime module that selects which operator to apply at each generation based on recent reward history. The GA may be configured with a portfolio of multiple operators (e.g., three crossover strategies and four mutation strategies), and AOS decides which one to use based on their recent performance.

Three strategies are available, each with different exploration-exploitation trade-offs:

- **Probability Matching (PM)**: Maintains selection probabilities updated toward observed rewards using a learning rate.
- **Adaptive Pursuit (AP)**: Like PM but more aggressive — pursues the current best arm with higher probability.
- **Multi-Armed Bandit (MAB)**: Uses UCB1 with epsilon-greedy exploration.

This module is pure data arithmetic — no `Instant`, no `rayon`, no `std::sync::Mutex`. It compiles for `wasm32-unknown-unknown`.

## Key Concepts

### AosStrategy

| Variant | Parameters | Description |
|---------|-----------|-------------|
| `ProbabilityMatching { alpha, learning_rate }` | alpha=0.8, lr=0.3 (default) | Updates probabilities toward observed rewards. PM is the standard baseline. |
| `AdaptivePursuit { beta, c }` | beta=0.5, c=1.5 (default) | Aggressively pursues the best arm with higher probability. Faster convergence than PM. |
| `MultiArmedBandit { c, epsilon }` | c=1.0, epsilon=0.1 (default) | UCB1 with epsilon-greedy. Balances exploration via confidence bounds. |

Convenience constructors: `AosStrategy::pm_default()`, `AosStrategy::ap_default()`, `AosStrategy::mab_default()`.

### AosState

The runtime state machine for one operator portfolio:

| Method | Description |
|--------|-------------|
| `new(num_arms, strategy, window_size)` | Creates AOS state for `num_arms` operators. |
| `select_operator(&mut self, rng, generation)` | Selects an operator index from the portfolio. |
| `record_reward(&mut self, arm, reward)` | Records a fitness-improvement reward for the selected arm. |
| `reset(&mut self)` | Clears all state for a fresh start. |

### Reward Model

The reward is typically the fitness improvement of the offspring produced by the selected operator, normalized relative to the parent's fitness:

```rust,ignore
let reward = (parent_fitness - offspring_fitness).abs() / parent_fitness.abs().max(f64::EPSILON);
```

Rewards are stored in per-arm sliding window ring buffers of configurable size (`window_size`).

### Selection Process

1. **Exploration phase**: For the first `window_size / 2` generations, operators are selected uniformly at random (accumulating initial reward data).
2. **Steady-state phase**: After exploration, the strategy dispatches:
   - **PM/AP**: Roulette-wheel selection over arm probabilities.
   - **MAB**: Epsilon-greedy with UCB1 confidence bound.

## Usage Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::aos::{AosStrategy, AosState};
use std::sync::{Arc, Mutex};

type MyChromosome = RangeChromosome<f64>;

// Configure AOS with three crossover variants
let aos_strategy = AosStrategy::mab_default();
let mut aos_state = AosState::new(3, aos_strategy, 30); // 3 arms, window 30

let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut ga = Ga::new()
    .with_genes_per_chromosome(5_usize)
    .with_population_size(100)
    .with_initialization_fn(move |genes, _, _| {
        range_random_initialization(genes, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(|dna: &[RangeGenotype<f64>]| -> f64 {
        // Rastrigin function
        let a = 10.0;
        let n = dna.len() as f64;
        a * n + dna.iter()
            .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
            .sum::<f64>()
    })
    .with_selection_method(Selection::Tournament)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("Valid configuration");

ga.run().expect("GA run failed");

// After the run, inspect AOS state
// (In production, AOS is integrated via GaConfiguration crossover/mutation portfolio)
```

## Configuration

AOS is configured via the `GaConfiguration` builder. The GA engine tracks operator selection internally — `AosState` can also be used standalone for custom operator portfolios.

### Standalone AOS Example

```rust,ignore
use genetic_algorithms::aos::{AosStrategy, AosState};
use rand::Rng;

let mut rng = rand::rng();
let mut aos = AosState::new(4, AosStrategy::pm_default(), 20);

for gen in 0..100 {
    let arm = aos.select_operator(&mut rng, gen);
    // Apply operator `arm` and measure fitness improvement
    let reward = /* compute reward */ 0.5;
    aos.record_reward(arm, reward);
}
```

## Performance Considerations

- All AOS operations are O(num_arms) — suitable for portfolios of up to ~20 operators.
- The module compiles for `wasm32-unknown-unknown` (no threading or time-dependency).
- Sliding window rewards use a ring buffer — O(1) per reward insertion.

## See Also

- [Operations Overview](operations.md) — All operator categories and dispatch pattern
- [docs.rs/genetic_algorithms::aos](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/aos/index.html) — Module API reference
- [aos_demo example](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/examples/aos_demo.rs)
