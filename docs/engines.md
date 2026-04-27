# Engines

> Overview of all available GA engines: when to use each, configuration reference, and usage examples.

## Overview

The library provides six engines for different problem structures and evolutionary strategies:

| Engine             | Module                  | Output type           | Use case                                          |
|--------------------|-------------------------|-----------------------|---------------------------------------------------|
| `Ga<U>`            | `ga`                    | Single best chromosome | General-purpose single-objective optimisation     |
| `IslandGa<U>`      | `engines::island`       | Multi-island best     | Diversity via parallel sub-populations + migration |
| `Nsga2Ga<U>`       | `engines::nsga2`        | Pareto front          | Multi-objective optimisation                      |
| `AlpsEngine<U>`    | `engines::alps`         | Best + layer results  | Prevent premature convergence via age layers      |
| `CellularEngine<U>` | `engines::cellular`   | Best + final grid     | Spatial locality — neighbourhood-only competition  |
| `DeEngine`         | `engines::de`           | Best `f64` vector     | Continuous parameter optimisation                 |
| `ScatterEngine<U>` | `engines::scatter`      | Reference set + best  | Reference-set search with linear combination      |

---

## `Ga<U>` — Standard Genetic Algorithm

The core single-population engine. Implements the classic GA cycle:
selection → crossover + mutation (rayon parallel) → survivor selection → elitism → statistics.

**Entry point:** `src/engines/ga.rs`

```rust
use genetic_algorithms::ga::Ga;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::chromosomes::Binary;

let mut ga: Ga<Binary> = Ga::new()
    .with_population_size(100)
    .with_genes_per_chromosome(32)
    .with_initialization_fn(binary_random_initialization)
    .with_fitness_fn(|dna| dna.iter().filter(|g| g.value).count() as f64)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::BitFlip)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Maximization)
    .with_max_generations(500)
    .build()
    .expect("valid config");

ga.run().expect("GA should succeed");
```

See [getting-started.md](getting-started.md) for full builder options and
[configuration.md](configuration.md) for all parameters.

---

## `IslandGa<U>` — Island Model

Runs multiple independent sub-populations (islands) in parallel using rayon. After every
`migration_interval` generations, a fraction of the best individuals migrate between islands
according to the configured topology.

**Entry point:** `src/engines/island/`

### Key configuration fields

| Field                  | Type                | Default     | Description                                          |
|------------------------|---------------------|-------------|------------------------------------------------------|
| `number_of_islands`    | `usize`             | —           | Number of parallel sub-populations                   |
| `migration_interval`   | `usize`             | `10`        | Generations between migration events                 |
| `migration_size`       | `usize`             | `2`         | Individuals migrating per island per event           |
| `topology`             | `IslandTopology`    | `Ring`      | `Ring`, `FullyConnected`, `Random`                   |

Each island uses the same `GaConfiguration` as a standard `Ga<U>`. Set the inner config
via `IslandConfiguration::with_ga_configuration(...)`.

---

## `Nsga2Ga<U>` — NSGA-II

Multi-objective optimisation using Non-dominated Sorting Genetic Algorithm II. The population
is ranked into Pareto fronts; crowding distance breaks ties within a front. Returns a full
Pareto front rather than a single best chromosome.

**Entry point:** `src/engines/nsga2/`

Chromosomes must return a `Vec<f64>` objective vector from `get_fitness_objectives()`. The
engine does not use the scalar `get_fitness()` value.

---

## `AlpsEngine<U>` — Age-Layered Population Structure

ALPS prevents premature convergence by maintaining `n_layers` sub-populations. Each layer
has an age limit; individuals exceeding the limit are promoted to the next layer. Layer 0 is
periodically reseeded with fresh random individuals.

**Entry point:** `src/engines/alps/`

### Configuration

```rust
use genetic_algorithms::engines::alps::{AlpsConfiguration, AlpsAgeScheme};
use genetic_algorithms::operations::{Crossover, Mutation};
use genetic_algorithms::configuration::ProblemSolving;

let config = AlpsConfiguration::default()
    .with_n_layers(6)
    .with_layer_size(20)
    .with_age_scheme(AlpsAgeScheme::Fibonacci)
    .with_age_gap(5)
    .with_injection_interval(10)
    .with_max_generations(1000)
    .with_crossover(Crossover::Uniform)
    .with_mutation(Mutation::Gaussian)
    .with_mutation_sigma(0.1)
    .with_problem_solving(ProblemSolving::Minimization);
```

### `AlpsAgeScheme` variants

| Variant      | Age limit formula                         | Description                                      |
|--------------|-------------------------------------------|--------------------------------------------------|
| `Linear`     | `max_age[i] = (i + 1) * age_gap`         | Even spacing; good default                       |
| `Fibonacci`  | `max_age[i] = fib(i + 2) * age_gap`      | Rapidly-cycling lower layers; very stable upper  |
| `Polynomial` | `max_age[i] = (i + 1)^2 * age_gap`       | Exponentially expanding windows                  |

### `AlpsConfiguration` fields

| Field                | Type              | Default            | Description                                       |
|----------------------|-------------------|--------------------|---------------------------------------------------|
| `n_layers`           | `usize`           | `6`                | Number of age layers (minimum 2)                  |
| `layer_size`         | `usize`           | `20`               | Target individuals per layer                      |
| `age_scheme`         | `AlpsAgeScheme`   | `Fibonacci`        | Formula for per-layer age limits                  |
| `age_gap`            | `usize`           | `5`                | Base age unit for scheme calculations             |
| `injection_interval` | `usize`           | `10`               | Generations between layer-0 reseedings (0 = off)  |
| `max_generations`    | `usize`           | `1000`             | Maximum generations before stopping               |
| `crossover`          | `Crossover`       | `Uniform`          | Crossover operator                                |
| `mutation`           | `Mutation`        | `Gaussian`         | Mutation operator                                 |
| `mutation_step`      | `Option<f64>`     | `None`             | Step size for Creep mutation                      |
| `mutation_sigma`     | `Option<f64>`     | `Some(0.1)`        | Std deviation for Gaussian mutation               |
| `problem_solving`    | `ProblemSolving`  | `Minimization`     | Minimise or maximise                              |
| `fitness_target`     | `Option<f64>`     | `None`             | Early-stop fitness target                         |

---

## `CellularEngine<U>` — Cellular Genetic Algorithm

Individuals are placed on a 2D toroidal grid. Each cell competes only with its local
neighbourhood (Von Neumann, Moore, etc.), which naturally preserves diversity through
spatial isolation.

**Entry point:** `src/engines/cellular/`

### Configuration

```rust
use genetic_algorithms::engines::cellular::{CellularConfiguration, Neighborhood, UpdateMode};
use genetic_algorithms::operations::{Crossover, Mutation, Selection};
use genetic_algorithms::configuration::ProblemSolving;

let config = CellularConfiguration::default()
    .with_rows(10)
    .with_cols(10)
    .with_neighborhood(Neighborhood::Moore)
    .with_update_mode(UpdateMode::Asynchronous)
    .with_selection(Selection::Tournament)
    .with_crossover(Crossover::Uniform)
    .with_mutation(Mutation::BitFlip)
    .with_max_generations(500)
    .with_problem_solving(ProblemSolving::Maximization);
```

### `Neighborhood` variants

| Variant       | Cells | Description                                          |
|---------------|-------|------------------------------------------------------|
| `VonNeumann`  | 4     | N/S/E/W — L1 distance ≤ 1                           |
| `Moore`       | 8     | 3×3 square minus center — L∞ distance ≤ 1            |
| `CompactR2`   | 24    | 5×5 square minus center — L∞ distance ≤ 2            |
| `Linear`      | 2     | Left and right neighbours in row-major order (ring)  |

### `UpdateMode` variants

| Variant         | Description                                                                    |
|-----------------|--------------------------------------------------------------------------------|
| `Synchronous`   | All cells read from previous generation; writes applied after full sweep       |
| `Asynchronous`  | Cells updated in-place; later cells may see offspring from the same generation |

### `CellularConfiguration` fields

| Field                | Type              | Default          | Description                                       |
|----------------------|-------------------|------------------|---------------------------------------------------|
| `rows`               | `usize`           | `10`             | Grid rows                                         |
| `cols`               | `usize`           | `10`             | Grid columns                                      |
| `neighborhood`       | `Neighborhood`    | `Moore`          | Neighbour topology                                |
| `update_mode`        | `UpdateMode`      | `Asynchronous`   | Update strategy                                   |
| `max_generations`    | `usize`           | `1000`           | Maximum generations before stopping               |
| `selection`          | `Selection`       | `Tournament`     | Selection method to pick mate from neighbourhood  |
| `crossover`          | `Crossover`       | `Uniform`        | Crossover operator                                |
| `mutation`           | `Mutation`        | `Gaussian`       | Mutation operator                                 |
| `mutation_step`      | `Option<f64>`     | `None`           | Step size for Creep mutation                      |
| `mutation_sigma`     | `Option<f64>`     | `Some(0.1)`      | Std deviation for Gaussian mutation               |
| `problem_solving`    | `ProblemSolving`  | `Minimization`   | Minimise or maximise                              |
| `fitness_target`     | `Option<f64>`     | `None`           | Early-stop fitness target                         |

---

## `DeEngine` — Differential Evolution

A continuous optimisation engine that maintains a population of `f64` vectors. Each
candidate is mutated by combining difference vectors from randomly selected individuals,
then crossed with the original candidate.

**Entry point:** `src/engines/de/`

Uses the `DeGene` type (a `f64` value with bounds) rather than the standard chromosome system.

### Configuration

```rust
use genetic_algorithms::engines::de::{
    DeConfiguration, DeAdaptive, DeCrossoverMode, DeMutationStrategy,
};
use genetic_algorithms::configuration::ProblemSolving;

let config = DeConfiguration::default()
    .with_population_size(50)
    .with_max_generations(1000)
    .with_mutation_factor(0.8)
    .with_crossover_rate(0.9)
    .with_mutation_strategy(DeMutationStrategy::Rand1)
    .with_crossover_mode(DeCrossoverMode::Binomial)
    .with_adaptive(DeAdaptive::None)
    .with_problem_solving(ProblemSolving::Minimization);
```

### `DeMutationStrategy` variants

| Variant           | Formula                                              |
|-------------------|------------------------------------------------------|
| `Rand1`           | `v = x_r1 + F * (x_r2 - x_r3)` — DE/rand/1         |
| `Best1`           | `v = x_best + F * (x_r1 - x_r2)` — DE/best/1       |
| `CurrentToBest1`  | `v = x_i + F*(x_best - x_i) + F*(x_r1 - x_r2)` — DE/current-to-best/1 |
| `Rand2`           | `v = x_r1 + F*(x_r2 - x_r3) + F*(x_r4 - x_r5)` — DE/rand/2 |
| `Best2`           | `v = x_best + F*(x_r1 - x_r2) + F*(x_r3 - x_r4)` — DE/best/2 |

### `DeCrossoverMode` variants

| Variant       | Description                                                              |
|---------------|--------------------------------------------------------------------------|
| `Binomial`    | Independent per-gene trial at probability `CR`, plus one mandatory gene  |
| `Exponential` | Contiguous block starting at random gene, wraps if necessary             |

### `DeAdaptive` variants

| Variant         | Description                                                              |
|-----------------|--------------------------------------------------------------------------|
| `None`          | Static `F` and `CR` — no adaptation                                      |
| `Jade { p, c }` | JADE self-adaptive with external archive; `p` = top fraction, `c` = learning rate |
| `LShade { history_size }` | L-SHADE success-history adaptive with rolling memory              |

### `DeConfiguration` fields

| Field               | Type                  | Default        | Description                              |
|---------------------|-----------------------|----------------|------------------------------------------|
| `population_size`   | `usize`               | `50`           | Number of candidate vectors              |
| `max_generations`   | `usize`               | `1000`         | Maximum generations                      |
| `mutation_factor`   | `f64`                 | `0.8`          | Differential weight `F ∈ (0, 2]`        |
| `crossover_rate`    | `f64`                 | `0.9`          | Crossover probability `CR ∈ [0, 1]`     |
| `mutation_strategy` | `DeMutationStrategy`  | `Rand1`        | Mutation formula variant                 |
| `crossover_mode`    | `DeCrossoverMode`     | `Binomial`     | Crossover mode                           |
| `adaptive`          | `DeAdaptive`          | `None`         | Adaptive parameter control               |
| `problem_solving`   | `ProblemSolving`      | `Minimization` | Minimise or maximise                     |
| `fitness_target`    | `Option<f64>`         | `None`         | Early-stop fitness target                |

---

## `ScatterEngine<U>` — Scatter Search

Maintains a small reference set `b` of high-quality and diverse solutions. In each
iteration, new candidates are generated by linear combination of reference-set pairs,
optionally refined by a built-in hill-climbing local search, and the reference set is
updated with the best survivors.

**Entry point:** `src/engines/scatter/`

### Configuration

```rust
use genetic_algorithms::engines::scatter::ScatterConfiguration;
use genetic_algorithms::configuration::ProblemSolving;

let config = ScatterConfiguration::default()
    .with_population_size(50)        // diversification pool
    .with_reference_set_size(10)     // reference set b
    .with_max_iterations(100)
    .with_local_search(true)
    .with_local_search_steps(20)
    .with_local_search_step_size(0.1)
    .with_problem_solving(ProblemSolving::Minimization);
```

### `ScatterConfiguration` fields

| Field                   | Type             | Default        | Description                                          |
|-------------------------|------------------|----------------|------------------------------------------------------|
| `population_size`       | `usize`          | `50`           | Diversification pool size                            |
| `reference_set_size`    | `usize`          | `10`           | Reference set size `b` (must be < `population_size`) |
| `max_iterations`        | `usize`          | `100`          | Maximum scatter-search iterations                    |
| `local_search`          | `bool`           | `false`        | Apply hill-climbing local search after combination   |
| `local_search_steps`    | `usize`          | `20`           | Maximum steps for the local search                   |
| `local_search_step_size`| `f64`            | `0.1`          | Perturbation magnitude for local search              |
| `problem_solving`       | `ProblemSolving` | `Minimization` | Minimise or maximise                                 |
| `fitness_target`        | `Option<f64>`    | `None`         | Early-stop fitness target                            |

---

## Related

- [getting-started.md](getting-started.md) — Quick-start guide
- [configuration.md](configuration.md) — Full configuration reference for `Ga`, Island, and NSGA-II
- [api-reference.md](api-reference.md) — Public API summary including engine modules
- [traits.md](traits.md) — Core traits and observer system
- [examples.md](examples.md) — End-to-end examples
