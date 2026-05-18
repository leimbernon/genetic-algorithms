<!-- generated-by: gsd-doc-writer -->
# Configuration

> All tunable parameters for the `genetic_algorithms` library, with defaults and builder API.

## Overview

The configuration system controls every aspect of a GA run: population size, operator choice,
stopping criteria, logging, checkpointing, niching, and diversity control. Every setting has a
sensible default; start with those and override only what your problem requires.

The primary entry point is `GaConfiguration`, which implements the `ConfigurationT` supertrait
and all focused sub-traits (`SelectionConfig`, `CrossoverConfig`, `MutationConfig`, etc.).
You typically build configuration through the `Ga::new()` fluent API rather than constructing
structs directly.

### Feature flags

Some configuration options require optional Cargo features:

| Feature            | Enables |
|--------------------|---------|
| `serde`            | Checkpoint serialization (`SaveProgressConfiguration`), all config types derive `Serialize`/`Deserialize` |
| `observer-tracing` | `tracing` crate integration for structured spans |
| `observer-metrics` | `metrics` crate integration for counters and gauges |
| `visualization`    | `plotters`-backed chart output |

Enable features in `Cargo.toml`:

```toml
genetic_algorithms = { version = "2.4.0", features = ["serde"] }
```

---

## Environment Variables

This library does not read environment variables at runtime. All configuration is passed
programmatically through the builder API or configuration structs.

The `env_logger` dependency is included, so standard `RUST_LOG` filtering applies to the
library's `log`-crate output when the host application initializes `env_logger`.

---

## Config File Format

There is no external config file format. Configuration is expressed entirely in Rust via the
builder API or by constructing `GaConfiguration` directly. When the `serde` feature is enabled,
a `GaConfiguration` can be serialized to/from JSON for checkpoint purposes.

---

## Core Types

### `ProblemSolving`

Optimization direction. Set via `with_problem_solving()`.

| Variant        | Meaning |
|----------------|---------|
| `Minimization` | Lower fitness values are better (default) |
| `Maximization` | Higher fitness values are better |
| `FixedFitness` | Stop when fitness reaches the value set by `with_fitness_target()` |

### `LogLevel`

Controls verbosity of the library's internal `log`-crate output. Set via `with_logs()`.
Default is `Off`.

| Variant | Output |
|---------|--------|
| `Off`   | No output (default) |
| `Error` | Errors only |
| `Warn`  | Warnings and above |
| `Info`  | Informational messages and above |
| `Debug` | Debug-level messages and above |
| `Trace` | All messages including fine-grained traces |

---

## `GaConfiguration` Reference

`GaConfiguration` is the top-level configuration struct for the single-population GA engine (`Ga<U>`).

### Defaults

```rust
GaConfiguration {
    adaptive_ga: false,
    number_of_threads: 1,
    survivor: Survivor::Fitness,
    log_level: LogLevel::Off,
    elitism_count: 0,
    rng_seed: None,
    limit_configuration: LimitConfiguration {
        problem_solving: ProblemSolving::Minimization,
        max_generations: 100,
        fitness_target: None,
        population_size: 0,          // must be set
        genes_per_chromosome: 0,     // must be set
        needs_unique_ids: false,
        alleles_can_be_repeated: false,
    },
    selection_configuration: SelectionConfiguration {
        number_of_couples: 0,        // must be set
        method: Selection::Tournament,
        boltzmann_temperature: 1.0,
    },
    crossover_configuration: CrossoverConfiguration {
        number_of_points: None,
        probability_max: None,
        probability_min: None,
        method: Crossover::Uniform,
        sbx_eta: None,
        blend_alpha: None,
        arithmetic_alpha: None,
    },
    mutation_configuration: MutationConfiguration {
        probability_max: None,
        probability_min: None,
        method: Mutation::Swap,
        step: None,
        sigma: None,
        polynomial_eta: None,
        non_uniform_b: None,
        dynamic_mutation: false,
        target_cardinality: None,
        probability_step: None,
    },
    save_progress_configuration: SaveProgressConfiguration {
        save_progress: false,
        save_progress_interval: 0,
        save_progress_path: String::new(),
    },
    stopping_criteria: StoppingCriteria {
        stagnation_generations: None,
        convergence_threshold: None,
        max_duration_secs: None,
    },
    niching_configuration: None,
    extension_configuration: None,
}
```

### General settings

| Builder method | Field | Default | Description |
|----------------|-------|---------|-------------|
| `with_adaptive_ga(bool)` | `adaptive_ga` | `false` | Enable adaptive crossover/mutation probabilities |
| `with_threads(usize)` | `number_of_threads` | `1` | Parallel fitness evaluation threads (rayon) |
| `with_logs(LogLevel)` | `log_level` | `LogLevel::Off` | Logging verbosity |
| `with_survivor_method(Survivor)` | `survivor` | `Survivor::Fitness` | Survivor-selection strategy |
| `with_elitism(usize)` | `elitism_count` | `0` | Best individuals preserved unchanged |
| `with_rng_seed(u64)` | `rng_seed` | `None` | Seed for reproducible runs |

### Limit / problem settings

| Builder method | Field | Default | Description |
|----------------|-------|---------|-------------|
| `with_problem_solving(ProblemSolving)` | `limit_configuration.problem_solving` | `Minimization` | Optimization direction |
| `with_population_size(usize)` | `limit_configuration.population_size` | `0` | Number of individuals per generation |
| `with_genes_per_chromosome(usize)` | `limit_configuration.genes_per_chromosome` | `0` | Chromosome length |
| `with_max_generations(usize)` | `limit_configuration.max_generations` | `100` | Generation cap |
| `with_fitness_target(f64)` | `limit_configuration.fitness_target` | `None` | Target fitness for `FixedFitness` mode |
| `with_needs_unique_ids(bool)` | `limit_configuration.needs_unique_ids` | `false` | Assign unique IDs to chromosomes |
| `with_alleles_can_be_repeated(bool)` | `limit_configuration.alleles_can_be_repeated` | `false` | Allow duplicate allele values |

### Required settings

`population_size`, `genes_per_chromosome`, and `selection_configuration.number_of_couples` all
default to `0`. The `Ga::build()` validator rejects configurations where these remain at zero.
Always set them explicitly:

```rust
use genetic_algorithms::ga::Ga;
use genetic_algorithms::traits::{ConfigurationT, SelectionConfig, StoppingConfig};
use genetic_algorithms::configuration::ProblemSolving;

let mut ga = Ga::new()
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    .with_number_of_couples(50)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(200)
    // ... fitness function and initialization ...
    .build()
    .expect("invalid configuration");
```

---

## Selection Configuration

Set the selection strategy with `with_selection_method()` and the number of parent pairs with
`with_number_of_couples()`.

| Variant | Description |
|---------|-------------|
| `Selection::Random` | Uniform random — every individual has equal probability |
| `Selection::RouletteWheel` | Fitness-proportionate selection |
| `Selection::StochasticUniversalSampling` | Evenly spaced pointers (lower variance than roulette wheel) |
| `Selection::Tournament` | Pairwise tournament (default) |
| `Selection::Rank` | Rank-based selection — avoids dominance by very fit individuals |
| `Selection::Boltzmann` | Temperature-controlled selective pressure |
| `Selection::Truncation` | Only the top fraction of the population reproduces |

**Boltzmann temperature:** Controls selective pressure for `Selection::Boltzmann`. High values
(e.g., `10.0`) approach uniform selection (exploration); low values (e.g., `0.1`) apply strong
selective pressure (exploitation). Default is `1.0`. Set via the `boltzmann_temperature` field
on `SelectionConfiguration` directly (no builder method on `GaConfiguration`).

---

## Crossover Configuration

| Builder method | Default | Description |
|----------------|---------|-------------|
| `with_crossover_method(Crossover)` | `Crossover::Uniform` | Crossover strategy |
| `with_crossover_number_of_points(usize)` | `None` | Cut points for `MultiPoint` |
| `with_crossover_probability_max(f64)` | `None` | Static probability (or upper bound with adaptive GA) |
| `with_crossover_probability_min(f64)` | `None` | Lower bound for adaptive GA |
| `with_sbx_eta(f64)` | `None` | Distribution index for `Sbx` (typical: 2–20) |
| `with_blend_alpha(f64)` | `None` | Alpha for `BlendAlpha` (typical: 0.5) |

**`arithmetic_alpha`** (for `Crossover::Arithmetic`) is set directly on `CrossoverConfiguration`
with no dedicated builder method on `GaConfiguration`. Default is `None`; when not set, the
operator uses an internal fallback of `0.5`.

### Crossover variants

| Variant | Chromosome type | Notes |
|---------|----------------|-------|
| `Cycle` | Permutation | Preserves gene positions from each parent |
| `MultiPoint` | Any | Alternates parent segments at N random cut points |
| `Uniform` | Any | Each gene independently chosen from either parent (default) |
| `SinglePoint` | Any | One cut point splits both parents |
| `Order` | Permutation | Preserves relative ordering (OX) |
| `Pmx` | Permutation | Partially Mapped Crossover |
| `Sbx` | `Range<T>` | Simulated Binary Crossover; requires `sbx_eta` |
| `BlendAlpha` | `Range<T>` | BLX-α; requires `blend_alpha` |
| `Arithmetic` | `Range<T>` | Weighted midpoint; uses `arithmetic_alpha` (default `None`) |
| `Clone` | Any | Copies parents directly (mutation-only strategies) |
| `Rejuvenate` | Any | Copies parents and resets their ages to zero |

---

## Mutation Configuration

| Builder method | Default | Description |
|----------------|---------|-------------|
| `with_mutation_method(Mutation)` | `Mutation::Swap` | Mutation strategy |
| `with_mutation_probability_max(f64)` | `None` | Static probability (or upper bound with adaptive GA) |
| `with_mutation_probability_min(f64)` | `None` | Lower bound for adaptive GA |
| `with_mutation_step(f64)` | `None` | Step size for `Creep` mutation |
| `with_mutation_sigma(f64)` | `None` | Sigma for `Gaussian` mutation |
| `with_dynamic_mutation(bool)` | `false` | Adjust probability each generation by cardinality |
| `with_mutation_target_cardinality(f64)` | `None` | Target cardinality ratio (0.0–1.0) for dynamic mutation |
| `with_mutation_probability_step(f64)` | `None` | Adjustment step for dynamic mutation |

`polynomial_eta` (for `Mutation::Polynomial`, typical: 20–100, doc comment default 20.0) and `non_uniform_b`
(for `Mutation::NonUniform`, typical: 2–5, doc comment default 2.0) are set directly on
`MutationConfiguration` with no dedicated builder methods on `GaConfiguration`. Both default to `None`.

### Mutation variants

| Variant | Chromosome type | Notes |
|---------|----------------|-------|
| `Swap` | Any | Two random genes exchange positions (default) |
| `Inversion` | Any | Random sub-sequence reversed |
| `Scramble` | Any | Random sub-sequence shuffled |
| `Value` | Any | Single gene replaced with a random allele |
| `BitFlip` | Binary | Each bit flipped with per-gene probability |
| `Creep` | `Range<T>` | Small uniform perturbation; requires `step` |
| `Gaussian` | `Range<T>` | Normal distribution perturbation; requires `sigma` |
| `Polynomial` | `Range<T>` | NSGA-II style; uses `polynomial_eta` |
| `NonUniform` | `Range<T>` | Magnitude decreases over generations; uses `non_uniform_b` |
| `Insertion` | Permutation | Gene removed and reinserted at a different position |
| `ListValue` | `ListChromosome<T>` | Gene replaced with a different allele from its allele set |

---

## Survivor Configuration

Set via `with_survivor_method(Survivor)`. Default is `Survivor::Fitness`.

| Variant | Description |
|---------|-------------|
| `Fitness` | Keep the fittest individuals (default) |
| `Age` | Keep the youngest individuals |
| `MuPlusLambda` | Parents and offspring compete together (μ+λ) |
| `MuCommaLambda` | Only offspring (age == 0) are eligible (μ,λ) |

---

## Stopping Criteria

The GA stops when **any** of the following criteria is met.

### Primary criteria (always checked)

| Builder method | Default | Description |
|----------------|---------|-------------|
| `with_max_generations(usize)` | `100` | Stop after this many generations |
| `with_fitness_target(f64)` | `None` | Stop when fitness reaches this value (`FixedFitness` mode) |

### Compound criteria (`StoppingCriteria`)

Set via `with_stopping_criteria(StoppingCriteria { ... })`:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `stagnation_generations` | `Option<usize>` | `None` | Stop after N generations without improvement |
| `convergence_threshold` | `Option<f64>` | `None` | Stop when fitness std dev drops below this value |
| `max_duration_secs` | `Option<f64>` | `None` | Stop after this many seconds |

```rust
use genetic_algorithms::configuration::StoppingCriteria;
use genetic_algorithms::traits::StoppingConfig;

// Stop if no improvement for 50 generations, or after 60 seconds
let criteria = StoppingCriteria {
    stagnation_generations: Some(50),
    max_duration_secs: Some(60.0),
    convergence_threshold: None,
};
// ga.with_stopping_criteria(criteria)
```

---

## Adaptive GA

Enabled with `with_adaptive_ga(true)`. When active, crossover and mutation probabilities are
adjusted each generation within the `[probability_min, probability_max]` bounds based on
population performance. Both `probability_max` and `probability_min` must be set when adaptive
GA is enabled.

```rust
use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig};

// ga
//     .with_adaptive_ga(true)
//     .with_crossover_probability_max(0.9)
//     .with_crossover_probability_min(0.6)
//     .with_mutation_probability_max(0.1)
//     .with_mutation_probability_min(0.01)
```

---

## Elitism

Set via `with_elitism(usize)`. The top N individuals (by fitness) are extracted before survivor
selection and reinserted unchanged afterwards. Default is `0` (no elitism).

---

## Niching (Fitness Sharing)

Fitness sharing reduces the effective fitness of individuals that are too similar to their
neighbors, promoting population diversity. Configure via the `NichingConfig` trait methods:

| Builder method | Default | Description |
|----------------|---------|-------------|
| `with_niching_enabled(bool)` | `false` | Enable or disable fitness sharing |
| `with_niching_sigma_share(f64)` | `1.0` | Sharing radius: individuals within this distance share fitness |
| `with_niching_alpha(f64)` | `1.0` | Shape parameter — higher values make the sharing function steeper |

`niching_configuration` is `None` by default. Calling any niching builder method initializes it
with defaults.

---

## Extension (Diversity Control)

Extension strategies trigger when population diversity (fitness standard deviation) drops below
`diversity_threshold`, applying a corrective action to restore diversity.

| Builder method | Default | Description |
|----------------|---------|-------------|
| `with_extension_method(Extension)` | `Extension::Noop` | Diversity-rescue strategy |
| `with_extension_diversity_threshold(f64)` | `0.01` | Trigger threshold (fitness std dev) |
| `with_extension_survival_rate(f64)` | `0.1` | Fraction that survives `MassExtinction` |
| `with_extension_mutation_rounds(usize)` | `3` | Mutation rounds for `MassDegeneration` |
| `with_extension_elite_count(usize)` | `1` | Elite individuals protected from the extension event |

### Extension variants

| Variant | Description |
|---------|-------------|
| `Noop` | No action (default) |
| `MassExtinction` | Random cull to `survival_rate`, protecting elite individuals |
| `MassGenesis` | Trim to the 2 best chromosomes, regrow population from scratch |
| `MassDegeneration` | Apply N mutation rounds to non-elite individuals |
| `MassDeduplication` | Remove duplicate chromosomes (by gene comparison), regrow population |

`extension_configuration` is `None` by default. Any extension builder call initializes it.

---

## Checkpointing (`SaveProgressConfiguration`)

Requires the `serde` feature. When enabled, the GA serializes its state to disk periodically so
a run can be resumed later.

| Builder method | Default | Description |
|----------------|---------|-------------|
| `with_save_progress(bool)` | `false` | Enable periodic checkpoint saving |
| `with_save_progress_interval(usize)` | `0` | How often (in generations) to write a checkpoint |
| `with_save_progress_path(String)` | `""` | Directory path for checkpoint files |

```toml
# Cargo.toml — required
genetic_algorithms = { version = "2.4.0", features = ["serde"] }
```

---

## RNG Seed (Reproducibility)

Set via `with_rng_seed(u64)`. When set, all internal random number generators are seeded
deterministically from this value. Two runs with the same seed and thread count produce identical
results.

```rust
// ga.with_rng_seed(42)
```

---

## Engine-Specific Configurations

Each alternative execution engine has its own dedicated configuration struct:

### `Nsga2Configuration` (NSGA-II multi-objective)

`src/engines/nsga2/configuration.rs`

| Field | Default | Description |
|-------|---------|-------------|
| `num_objectives` | `2` | Number of objective functions |
| `population_size` | `100` | Individuals per generation |
| `max_generations` | `200` | Generation cap |
| `objective_directions` | `vec![]` (defaults to all `Minimize`) | Per-objective `ObjectiveDirection` |

`ObjectiveDirection` variants: `Minimize`, `Maximize`.

When `objective_directions` is empty, `effective_directions()` returns `Minimize` for every
objective. When set, the length must match `num_objectives`.

### `IslandConfiguration` (island model)

`src/engines/island/configuration.rs`

| Field | Default | Description |
|-------|---------|-------------|
| `num_islands` | `4` | Number of sub-populations |
| `migration_interval` | `10` | Generations between migration events |
| `migration_count` | `1` | Individuals migrated from each island per event |
| `topology` | `MigrationTopology::Ring` | Inter-island connection topology |
| `migration_policy` | `MigrationPolicy::BestReplaceWorst` | Migrant selection and replacement policy |

**`MigrationTopology` variants** (`src/engines/island/topology.rs`): `Ring`, `FullyConnected`,
`Grid(rows, cols)`, `Hypercube`, `Custom(Vec<Vec<usize>>)`.

**`MigrationPolicy` variants:** `BestReplaceWorst` (default), `RandomReplaceWorst`,
`TournamentMigrant`, `RandomReplaceRandom`.

### `DeConfiguration` (Differential Evolution)

`src/engines/de/configuration.rs`

| Field | Default | Description |
|-------|---------|-------------|
| `population_size` | `50` | Population size |
| `max_generations` | `1000` | Generation cap |
| `mutation_factor` | `0.8` | Differential weight F ∈ (0, 2] |
| `crossover_rate` | `0.9` | Crossover probability CR ∈ [0, 1] |
| `mutation_strategy` | `DeMutationStrategy::Rand1` | DE mutation formula |
| `crossover_mode` | `DeCrossoverMode::Binomial` | Trial vector construction mode |
| `adaptive` | `DeAdaptive::None` | Self-adaptive variant |
| `problem_solving` | `ProblemSolving::Minimization` | Optimization direction |
| `fitness_target` | `None` | Optional early-stopping fitness target |

**`DeMutationStrategy` variants:** `Rand1`, `Best1`, `CurrentToBest1`, `Rand2`, `Best2`.

**`DeCrossoverMode` variants:** `Binomial`, `Exponential`.

**`DeAdaptive` variants:** `None`, `Jade { p: f64, c: f64 }`, `LShade { history_size: usize }`.

### `ScatterConfiguration` (Scatter Search)

`src/engines/scatter/configuration.rs`

| Field | Default | Description |
|-------|---------|-------------|
| `population_size` | `50` | Total diversification pool size |
| `reference_set_size` | `10` | Reference set size b (must be < `population_size`) |
| `max_iterations` | `100` | Maximum scatter-search iterations |
| `local_search` | `false` | Apply hill-climbing after each combination |
| `local_search_steps` | `20` | Maximum steps for built-in local search |
| `local_search_step_size` | `0.1` | Perturbation magnitude for local search |
| `problem_solving` | `ProblemSolving::Minimization` | Optimization direction |
| `fitness_target` | `None` | Optional early-stopping fitness target |

### `AlpsConfiguration` (Age-Layered Population Structure)

`src/engines/alps/configuration.rs`

| Field | Default | Description |
|-------|---------|-------------|
| `n_layers` | `6` | Number of age layers (minimum 2) |
| `layer_size` | `20` | Target individuals per layer |
| `age_scheme` | `AlpsAgeScheme::Fibonacci` | Age threshold scheme for each layer |
| `age_gap` | `5` | Base age unit applied to the scheme |
| `injection_interval` | `10` | Generations between layer 0 reseeds (`0` disables) |
| `max_generations` | `1000` | Generation cap |
| `crossover` | `Crossover::Uniform` | Crossover operator |
| `mutation` | `Mutation::Gaussian` | Mutation operator |
| `mutation_step` | `None` | Step size for Creep mutation |
| `mutation_sigma` | `Some(0.1)` | Sigma for Gaussian mutation |
| `problem_solving` | `ProblemSolving::Minimization` | Optimization direction |
| `fitness_target` | `None` | Optional early-stopping fitness target |

**`AlpsAgeScheme` variants:** `Linear` (`(i+1) * age_gap`), `Fibonacci`
(`fibonacci(i+2) * age_gap`), `Polynomial` (`(i+1)² * age_gap`).

### `CellularConfiguration` (Cellular GA)

`src/engines/cellular/configuration.rs`

| Field | Default | Description |
|-------|---------|-------------|
| `rows` | `10` | Grid rows (population = rows × cols) |
| `cols` | `10` | Grid columns |
| `neighborhood` | `Neighborhood::Moore` | Cell interaction topology |
| `update_mode` | `UpdateMode::Asynchronous` | Synchronous or asynchronous updates |
| `max_generations` | `1000` | Generation cap |
| `selection` | `Selection::Tournament` | Mate selection within neighborhood |
| `crossover` | `Crossover::Uniform` | Crossover operator |
| `mutation` | `Mutation::Gaussian` | Mutation operator |
| `mutation_step` | `None` | Step size for Creep mutation |
| `mutation_sigma` | `Some(0.1)` | Sigma for Gaussian mutation |
| `problem_solving` | `ProblemSolving::Minimization` | Optimization direction |
| `fitness_target` | `None` | Optional early-stopping fitness target |

**`Neighborhood` variants:** `VonNeumann` (4-cell), `Moore` (8-cell, default), `CompactR2`
(24-cell, 5×5 minus center), `Linear` (ring, 2-cell).

**`UpdateMode` variants:** `Synchronous`, `Asynchronous` (default — typically converges faster).

Grid dimensions are set via `with_grid(rows, cols)` rather than separate `with_rows`/`with_cols`
methods.

---

## Per-Environment Overrides

There is no built-in per-environment configuration file mechanism. The recommended pattern is to
read `std::env::var()` in your binary and pass the values into the builder:

```rust
let pop_size: usize = std::env::var("GA_POPULATION_SIZE")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(100);

let mut ga = Ga::new()
    .with_population_size(pop_size)
    // ...
    .build()
    .expect("invalid configuration");
```

---

## Minimal Working Example

```rust
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};

let mut ga = Ga::new()
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    .with_number_of_couples(50)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(200)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_crossover_probability_max(0.8)
    .with_mutation_method(Mutation::Swap)
    .with_mutation_probability_max(0.05)
    .with_survivor_method(Survivor::Fitness)
    .with_elitism(2)
    .with_logs(genetic_algorithms::configuration::LogLevel::Info)
    // .with_initialization_fn(...)
    // .with_fitness_fn(...)
    .build()
    .expect("invalid configuration");
```
