# Examples

> Complete, runnable examples demonstrating every engine and feature.

## Overview

The `examples/` directory contains **19 runnable examples** covering 12 of the 15 engines (GP, CMA-ES, and EDA runnable patterns live in `tests/`), problem
types, and framework features. Each example demonstrates the full workflow: problem definition,
chromosome configuration, operator selection, engine setup, execution, and result interpretation.

Run any example with:

```sh
cargo run --example <name>
```

Some examples require feature flags (e.g., `--features observer-metrics`). See the table below
for details.

## Examples Catalog

| Example | Engine/Feature | Problem Type | Key Concepts |
|---------|---------------|-------------|--------------|
| `rastrigin` | Ga | Continuous optimization | Range genes, Gaussian mutation, tournament selection |
| `nsga2_zdt1` | Nsga2Ga | Bi-objective | Pareto ranking, crowding distance, multi-objective fitness |
| `island_model` | IslandGa | Parallel GA | Migration, sub-populations, coarse-grained parallelism |
| `job_scheduling` | Ga | Permutation/scheduling | Custom crossover (PMX), permutation representation |
| `feature_selection` | Ga | Binary classification | Binary genes, adaptive mutation, feature subsets |
| `niching` | Ga | Multimodal | Fitness sharing, niche radius, multimodal landscape |
| `knapsack_binary` | Ga | Binary/combinatorial | Binary chromosome, weight/value constraints |
| `nqueens_range` | Ga | Constraint satisfaction | Range genes, permutation encoding, penalty fitness |
| `onemax_binary` | Ga | Binary baseline | Binary genes, simple fitness, convergence monitoring |
| `onemax_extension` | Ga | Diversity control | Extension strategies, MassExtinction, diversity triggers |
| `nsga3_dtlz2` | Nsga3Ga | Many-objective (3+) | Reference points, Das-Dennis, DTLZ2 benchmark |
| `moead_dtlz2` | MoeaDGa | Decomposition | Tchebycheff scalarization, weight vectors, DTLZ2 |
| `spea2_zdt1` | Spea2Ga | Archive-based MO | Strength Pareto, k-NN density, ZDT1 benchmark |
| `sms_emoa_zdt1` | SmsEmoaGa | Indicator-based | Hypervolume contribution, steady-state, ZDT1 |
| `ibea_zdt1` | IbeaGa | Indicator-based | I_eps+ indicator, exponential scaling, ZDT1 |
| `aos_demo` | Ga + AOS | Adaptive operators | Operator portfolio, reward accumulation, strategy switching |
| `constrained_g1` | Ga + Constraints | Constrained optimization | Deb's feasibility rules, penalty functions, G1 problem |
| `hall_of_fame_demo` | Ga + HOF | Solution archive | Top-N tracking, deduplication, diversity filtering |
| `memetic_rastrigin` | Ga + Memetic | Local search hybrid | HillClimbing, Lamarckian mode, local refinement |

### Where to find runnable patterns for v3.0.0 features

Variable-length chromosomes and Genetic Programming are not yet covered by dedicated examples in `examples/` — runnable patterns live in the integration test suites instead, and these are the canonical references until standalone examples are added:

| Feature | Reference | Description |
|---------|-----------|-------------|
| Variable-length chromosomes (`ChromosomeLength::Variable`, `Mutation::Insertion`, `Mutation::Deletion`, `Crossover::VariableLength`, parsimony pressure) | [`tests/test_variable_length.rs`](../tests/test_variable_length.rs) | End-to-end patterns for length-changing mutation, alignment strategies, and `with_length_penalty` |
| Genetic Programming (`GpGa<N>`, `GpChromosome<N>`, custom `GpNode` impls, subtree/point/hoist mutation, bloat control, serde checkpointing) | [`tests/gp.rs`](../tests/gp.rs) | Symbolic regression with `MathNode`, classification trees with `BoolNode`, custom primitive sets |

The [Genetic Programming guide](gp.md) and the variable-length sections in [Chromosomes](chromosomes.md), [Mutation operators](operators/mutation.md), [Crossover operators](operators/crossover.md), and [Survivor operators](operators/survivor.md) walk through these APIs.

## Detailed Walkthroughs

The following sections walk through four representative examples in detail, showing the
complete workflow from problem definition to result interpretation.

### Walkthrough 1: `rastrigin` — Standard GA on Continuous Optimization

**Source:** [`examples/rastrigin.rs`](../examples/rastrigin.rs)

This is the most common starting point for new users. It minimizes the Rastrigin function,
a classic continuous optimization benchmark with many local minima and a global minimum of
0.0 at the origin.

#### Problem Definition

The Rastrigin function is defined as:

```
f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))   where A = 10
```

We optimize in 5 dimensions with each variable in `[-5.12, 5.12]`.

#### Configuration Walkthrough

```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, CrossoverConfig,
    MutationConfig, SelectionConfig, StoppingConfig};
use genetic_algorithms::{CompositeObserver, LogObserver};
use std::sync::Arc;

// 5-dimensional continuous optimization in [-5.12, 5.12]
const DIMENSIONS: usize = 5;
const POP_SIZE: usize = 100;
const MAX_GENERATIONS: usize = 500;

// Fitness: Rastrigin function (minimize toward 0.0)
let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
    let a = 10.0;
    let n = dna.len() as f64;
    a * n + dna.iter()
        .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
        .sum::<f64>()
};

// Allele range for each dimension
let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut ga = Ga::new()
    .with_genes_per_chromosome(DIMENSIONS)
    .with_population_size(POP_SIZE)
    .with_initialization_fn(move |genes_per_chromosome, _, _| {
        range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(fitness_fn)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(MAX_GENERATIONS)
    .with_observer(Arc::new(CompositeObserver::new().add(Arc::new(LogObserver))))
    .build()
    .expect("Failed to build GA configuration");
```

#### Key Lines Explained

- **Chromosome type:** `RangeChromosome<f64>` — a chromosome with `f64` genes, each bounded
  within an interval.
- **Initialization:** `range_random_initialization` randomly places each gene within its
  allele bounds `[-5.12, 5.12]`.
- **Tournament selection:** Standard pressure-based selection — each tournament pits 2 random
  individuals against each other, and the fitter wins.
- **Gaussian mutation:** Adds random Gaussian noise to each gene, appropriate for continuous
  landscapes. Configure sigma via `with_mutation_sigma(...)`.
- **Observer:** `CompositeObserver` fans out to `LogObserver` (console logging) and optionally
  `MetricsObserver` (behind the `observer-metrics` feature flag).

#### Running

```sh
cargo run --example rastrigin
```

#### Expected Output

The GA prints progress every 50 generations, showing best and average fitness:

```
== Rastrigin Continuous Optimization ==
Chromosome: 5 dimensions, Population: 100, Max generations: 500
Operators: Selection=Tournament, Crossover=Uniform, Mutation=Gaussian
-------------------------------------------------------
Generation   50: best =  14.9245, avg =  35.9282
Generation  100: best =   5.9701, avg =  19.3028
...
Generation  500: best =   0.0001, avg =   2.3400
-------------------------------------------------------
Finished. Best fitness: 0.000099
Near-optimal solution found!
```

---

### Walkthrough 2: `nsga2_zdt1` — Multi-Objective GA on ZDT1

**Source:** [`examples/nsga2_zdt1.rs`](../examples/nsga2_zdt1.rs)

This example demonstrates bi-objective optimization using NSGA-II on the ZDT1 benchmark
problem. ZDT1 has two conflicting minimization objectives defined over 30 continuous variables
in `[0, 1]`. The Pareto-optimal front is a convex curve where minimizing f1 forces f2 upward.

#### Problem Definition

```
Variables: x = (x_1, ..., x_30) in [0, 1]
f1(x) = x_1
f2(x) = g(x) * (1 - sqrt(x_1 / g(x)))
g(x)  = 1 + (9 / 29) * sum(x_2, ..., x_30)
```

#### Configuration Walkthrough

```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::{LogObserver, Nsga2Observer};
use std::sync::Arc;

const N_VARS: usize = 30;
const POP_SIZE: usize = 100;
const MAX_GENERATIONS: usize = 250;

let nsga2_config = Nsga2Configuration::new()
    .with_num_objectives(2)
    .with_population_size(POP_SIZE)
    .with_max_generations(MAX_GENERATIONS)
    .with_objective_directions(vec![
        ObjectiveDirection::Minimize,
        ObjectiveDirection::Minimize,
    ]);

let mut ga_config = GaConfiguration::default();
ga_config.limit_configuration.genes_per_chromosome = N_VARS;

let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
let alleles_clone = alleles.clone();

let obj_f1 = |dna: &[RangeGenotype<f64>]| -> f64 { dna[0].value };
let obj_f2 = |dna: &[RangeGenotype<f64>]| -> f64 {
    let n = dna.len();
    let g = 1.0 + (9.0 / (n - 1) as f64)
        * dna[1..].iter().map(|gene| gene.value).sum::<f64>();
    g * (1.0 - (dna[0].value / g).sqrt())
};

let mut nsga2 = Nsga2Ga::<RangeChromosome<f64>>::new(nsga2_config, ga_config)
    .with_alleles(alleles)
    .with_initialization_fn(move |n, _, _| {
        range_random_initialization(n, Some(&alleles_clone), Some(true))
    })
    .with_objective_fns(vec![Box::new(obj_f1), Box::new(obj_f2)])
    .with_observer(
        Arc::new(LogObserver)
            as Arc<dyn Nsga2Observer<RangeChromosome<f64>> + Send + Sync>
    )
    .build()
    .expect("Failed to build NSGA-II");
```

#### Key Lines Explained

- **Nsga2Configuration:** Defines the multi-objective engine parameters — number of objectives,
  population size, and per-objective minimization vs. maximization.
- **GaConfiguration:** The base GA configuration sets chromosome-level parameters (gene count).
  The NSGA-II engine reads its operators from this config.
- **Objective functions:** Each objective is passed as a `Box<dyn Fn(&[Gene]) -> f64>`. The
  engine does not use the scalar `ChromosomeT::fitness()` value — only these objective
  values.
- **Observer:** `LogObserver` implements `Nsga2Observer`, logging Pareto front and crowding
  events.

#### Running

```sh
cargo run --example nsga2_zdt1
```

#### Expected Output

```
== NSGA-II ZDT1 Multi-Objective Optimization ==
Variables: 30, Population: 100, Generations: 250
...
Completed. Pareto front: 100 non-dominated solutions
Pareto front (10 points sampled from 100 non-dominated solutions):
  f1=0.0000, f2=0.3119
  f1=0.1135, f2=0.6590
  f1=0.2148, f2=0.5377
  ...
  f1=0.8932, f2=0.0456
  f1=0.9984, f2=0.0063
```

---

### Walkthrough 3: `constrained_g1` — Constraint Handling with Feasibility Rules

**Source:** [`examples/constrained_g1.rs`](../examples/constrained_g1.rs)

This example solves a constrained optimization problem (simplified G1 benchmark) using a GA
with static penalty functions. It demonstrates the constraint handling framework.

#### Problem Definition

13 real-valued variables in `[0.0, 1.0]` with three inequality constraints:

```
g1: x[0..5] sum <= 4.0
g2: x[5..10] sum <= 3.0
g3: x[10..13] sum <= 2.0
```

Objective: minimize `sum(x)` subject to the three constraints.

#### Configuration Walkthrough

```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::constraints::PenaltyStrategy;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig,
    SelectionConfig, StoppingConfig,
};

const N_VARS: usize = 13;
const POP_SIZE: usize = 200;
const MAX_GEN: usize = 200;

let alleles = vec![RangeGene::new(0, vec![(0.0_f64, 1.0_f64)], 0.0)];
let alleles_clone = alleles.clone();

// Define inequality constraints as closure functions
let constraints = vec![
    |dna: &[RangeGene<f64>]| {
        let sum: f64 = dna[0..5].iter().map(|g| g.value).sum();
        (sum - 4.0).max(0.0)
    },
    |dna: &[RangeGene<f64>]| {
        let sum: f64 = dna[5..10].iter().map(|g| g.value).sum();
        (sum - 3.0).max(0.0)
    },
    |dna: &[RangeGene<f64>]| {
        let sum: f64 = dna[10..13].iter().map(|g| g.value).sum();
        (sum - 2.0).max(0.0)
    },
];

let fitness_fn = |dna: &[RangeGene<f64>]| -> f64 {
    dna.iter().map(|g| g.value).sum()
};

let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
    .with_genes_per_chromosome(N_VARS)
    .with_population_size(POP_SIZE)
    .with_initialization_fn(move |genes_per_chromosome, _, _| {
        range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(fitness_fn)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::BlendAlpha)
    .with_mutation_method(Mutation::Gaussian)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_survivor_method(Survivor::Fitness)
    .with_max_generations(MAX_GEN)
    .with_constraint_fns(constraints)
    .with_penalty_strategy(PenaltyStrategy::Static { coefficient: 100.0 })
    .build()
    .expect("Failed to build GA");
```

#### Key Lines Explained

- **constraint_fns:** A `Vec` of closures, each returning a violation magnitude (0.0 if
  satisfied, positive otherwise). The GA sums all violations internally.
- **PenaltyStrategy::Static:** Applies a fixed penalty coefficient to the sum of violations,
  added to the raw fitness. Alternatives include `Dynamic` (increasing over generations),
  `Adaptive`, and `Death` (infeasible individuals are discarded).
- **Deb's feasibility rules:** When comparing two individuals, the GA prioritizes: (1)
  feasible over infeasible, (2) lower violation if both are infeasible, (3) better fitness
  if both are feasible.

#### Running

```sh
cargo run --example constrained_g1
```

#### Expected Output

```
Constrained G1 benchmark with static penalty
Variables: 13, Population: 200, Generations: 200

Constraints:
  g1: x[0..5] sum <= 4.0
  g2: x[5..10] sum <= 3.0
  g3: x[10..13] sum <= 2.0

...
Feasible solution found!
Best fitness: 6.0213
Constraint violations: g1=0.000, g2=0.000, g3=0.000
```

---

### Walkthrough 4: `aos_demo` — Adaptive Operator Selection

**Source:** [`examples/aos_demo.rs`](../examples/aos_demo.rs)

This example demonstrates Adaptive Operator Selection (AOS) using a crossover portfolio
with Probability Matching. The GA dynamically selects among 3 crossover operators based on
their recent performance.

#### Problem Definition

Minimize the sum of 8 integer gene values (each in `[0, 100]`). This is a deliberately
simple problem where different crossover operators may perform differently.

#### Configuration Walkthrough

```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, MutationConfig, SelectionConfig, StoppingConfig,
};

let n: i32 = 8;
let alleles = vec![RangeGene::new(0, vec![(0_i32, 100_i32)], 0)];
let alleles_clone = alleles.clone();

let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
    .with_genes_per_chromosome(n.try_into().unwrap())
    .with_population_size(50)
    .with_max_generations(100)
    .with_initialization_fn(move |genes_per_chromosome, _, _| {
        range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
    })
    .with_fitness_fn(|dna: &[RangeGene<i32>]| {
        dna.iter().map(|g| g.value() as f64).sum()
    })
    .with_selection_method(Selection::Tournament)
    // AOS crossover portfolio with 3 operators
    .with_crossover_portfolio(vec![
        Crossover::Uniform,
        Crossover::SinglePoint,
        Crossover::BlendAlpha,
    ])
    .with_mutation_method(Mutation::Swap)
    .with_mutation_probability_max(0.2)
    .with_aos_strategy(genetic_algorithms::aos::AosStrategy::pm_default())
    .with_reward_window(50)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_survivor_method(Survivor::Fitness)
    .with_alleles(alleles)
    .build()
    .expect("Failed to build GA with AOS configuration");
```

#### Key Lines Explained

- **crossover_portfolio:** Instead of a single crossover operator, this provides a `Vec` of
  3 candidates. The AOS system selects among them each generation.
- **AosStrategy::pm_default():** Probability Matching with default parameters (credit-based
  selection). The GA tracks recent fitness improvements from each operator and adjusts
  selection probabilities accordingly.
- **reward_window:** Number of generations used for the exploration-exploitation trade-off.
  During the first half of the window, all operators are explored equally.
- **Mutation::Swap:** Simple swap mutation on the integer gene values, appropriate for the
  permutation-like representation of the gene order.

#### Running

```sh
cargo run --example aos_demo
```

#### Expected Output

```
=== AOS Demo: Crossover Portfolio with Probability Matching ===

Optimization complete!
Best fitness: 246.0000
Best chromosome (first 4 of 8 genes): [23, 17, 35, 42]

=== AOS Demo Complete ===
The GA dynamically selected among Uniform, SinglePoint, and BlendAlpha crossover
operators based on recent fitness improvement (Probability Matching).
```

## Running an Example

All examples are run via `cargo run --example <name>`. For example:

```sh
# Standard GA on continuous optimization
cargo run --example rastrigin

# Multi-objective optimization with NSGA-II
cargo run --example nsga2_zdt1

# Many-objective optimization with NSGA-III (requires 3+ objectives)
cargo run --example nsga3_dtlz2

# Constrained optimization
cargo run --example constrained_g1

# Adaptive operator selection
cargo run --example aos_demo

# Memetic algorithm (GA + local search)
cargo run --example memetic_rastrigin
```

Some examples use optional feature flags:

```sh
# Observer-metrics integration
cargo run --example rastrigin --features observer-metrics

# Benchmark functions (behind feature flag)
cargo run --example nsga3_dtlz2 --features benchmarks
```

Add `--release` for better performance on longer runs:

```sh
cargo run --example rastrigin --release
```

## Related

- [Engines Overview](engines.md) — All 12 optimization engines and when to use each
- [Getting Started](getting-started.md) — Quick-start guide
- [Configuration Reference](configuration.md) — Full builder options and parameter tables
- [README Examples Table](../README.md) — Example catalog in the project README
- [Source: examples/](../examples/) — All 19 example source files
