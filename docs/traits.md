# Core Traits

> Fundamental traits for genes, chromosomes, and genetic algorithm configuration.

## Overview

The core traits define the essential abstractions for building genetic algorithms in this library. They provide the interfaces for representing genes, chromosomes, and the overall configuration of a genetic algorithm. By implementing these traits, users can customize the genetic representation, fitness evaluation, and algorithm behavior to suit a wide variety of optimization problems.

The `GeneT` trait represents the smallest unit of genetic information, allowing for unique identification and cloning. The `ChromosomeT` trait models a sequence of genes (the DNA), along with metadata such as fitness and age, and provides efficient methods for manipulating genetic material using Rust's `Cow` for zero-copy or minimal-copy operations. The `ConfigurationT` trait exposes a builder-style API for specifying all aspects of the genetic algorithm, including population size, selection and crossover methods, mutation parameters, and stopping criteria.

These traits are the foundation for extending and customizing the library. Users typically implement their own gene and chromosome types to encode problem-specific information, and use the configuration trait to tune the algorithm for their optimization task.

## Key Concepts

### GeneT

| Method         | Signature                                 | Description                                  |
|----------------|-------------------------------------------|----------------------------------------------|
| `new`          | `fn new() -> Self`                        | Creates a new gene instance.                 |
| `default`      | `fn default(mut self) -> Self`            | Resets gene to its default state.            |
| `get_id`       | `fn get_id(&self) -> i32`                 | Retrieves the unique identifier of the gene. |
| `set_id`       | `fn set_id(&mut self, id: i32) -> &mut Self` | Sets the unique identifier for the gene.     |

### ChromosomeT

| Method               | Signature                                                                 | Description                                               |
|----------------------|---------------------------------------------------------------------------|-----------------------------------------------------------|
| `new`                | `fn new() -> Self`                                                        | Creates a new chromosome instance.                        |
| `default`            | `fn default(mut self) -> Self`                                            | Resets chromosome state (fitness, age, DNA).              |
| `new_gene`           | `fn new_gene() -> Self::Gene`                                             | Creates a new default-initialized gene.                   |
| `get_dna`            | `fn get_dna(&self) -> &[Self::Gene]`                                      | Returns the chromosome's DNA as a slice.                  |
| `set_dna`            | `fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self`      | Sets DNA efficiently using `Cow` for zero/minimal copy.   |
| `set_gene`           | `fn set_gene(&mut self, gene_index: usize, gene: Self::Gene) -> &mut Self`| Replaces a gene at a specific index.                      |
| `set_fitness_fn`     | `fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self`             | Installs a fitness function.                              |
| `calculate_fitness`  | `fn calculate_fitness(&mut self)`                                         | Computes and sets fitness using the installed function.   |
| `get_fitness`        | `fn get_fitness(&self) -> f64`                                            | Retrieves the current fitness value.                      |
| `set_fitness`        | `fn set_fitness(&mut self, fitness: f64) -> &mut Self`                    | Sets the fitness value.                                   |
| `set_age`            | `fn set_age(&mut self, age: i32) -> &mut Self`                            | Sets the age of the chromosome.                           |
| `get_age`            | `fn get_age(&self) -> i32`                                                | Gets the age of the chromosome.                           |
| `get_fitness_distance`| `fn get_fitness_distance(&self, fitness_target: &f64) -> f64`            | Computes distance to a target fitness.                    |

### ConfigurationT

| Method                          | Signature                                                        | Description                                              |
|----------------------------------|------------------------------------------------------------------|----------------------------------------------------------|
| `new`                           | `fn new() -> Self`                                               | Creates a new configuration instance.                    |
| `with_adaptive_ga`              | `fn with_adaptive_ga(&mut self, adaptive_ga: bool) -> &mut Self` | Enables adaptive GA features.                            |
| `with_threads`                  | `fn with_threads(&mut self, number_of_threads: i32) -> &mut Self`| Sets the number of threads for parallel execution.        |
| `with_logs`                     | `fn with_logs(&mut self, log_level: LogLevel) -> &mut Self`      | Sets the logging level.                                  |
| `with_survivor_method`          | `fn with_survivor_method(&mut self, method: Survivor) -> &mut Self`| Sets survivor selection method.                        |
| `with_problem_solving`          | `fn with_problem_solving(&mut self, problem_solving: ProblemSolving) -> &mut Self`| Sets problem-solving mode.          |
| `with_max_generations`          | `fn with_max_generations(&mut self, max_generations: i32) -> &mut Self`| Sets max number of generations.                     |
| `with_fitness_target`           | `fn with_fitness_target(&mut self, fitness_target: f64) -> &mut Self`| Sets target fitness value.                          |
| `with_population_size`          | `fn with_population_size(&mut self, population_size: i32) -> &mut Self`| Sets population size.                              |
| `with_genes_per_chromosome`     | `fn with_genes_per_chromosome(&mut self, genes_per_chromosome: i32) -> &mut Self`| Sets number of genes per chromosome.         |
| `with_needs_unique_ids`         | `fn with_needs_unique_ids(&mut self, needs_unique_ids: bool) -> &mut Self`| Requires unique gene IDs.                        |
| `with_alleles_can_be_repeated`  | `fn with_alleles_can_be_repeated(&mut self, alleles_can_be_repeated: bool) -> &mut Self`| Allows repeated alleles.          |
| `with_number_of_couples`        | `fn with_number_of_couples(&mut self, number_of_couples: i32) -> &mut Self`| Sets number of mating couples.                   |
| `with_selection_method`         | `fn with_selection_method(&mut self, selection_method: Selection) -> &mut Self`| Sets selection method.                      |
| `with_crossover_number_of_points`| `fn with_crossover_number_of_points(&mut self, number_of_points: i32) -> &mut Self`| Sets crossover points.                 |
| `with_crossover_probability_max`| `fn with_crossover_probability_max(&mut self, probability_max: f64) -> &mut Self`| Sets max crossover probability.         |
| `with_crossover_probability_min`| `fn with_crossover_probability_min(&mut self, probability_min: f64) -> &mut Self`| Sets min crossover probability.         |
| `with_crossover_method`         | `fn with_crossover_method(&mut self, method: Crossover) -> &mut Self`| Sets crossover method.                        |
| `with_sbx_eta`                  | `fn with_sbx_eta(&mut self, eta: f64) -> &mut Self`              | Sets SBX distribution index.                         |
| `with_blend_alpha`              | `fn with_blend_alpha(&mut self, alpha: f64) -> &mut Self`        | Sets BLX-α crossover alpha parameter.                 |
| `with_mutation_probability_max` | `fn with_mutation_probability_max(&mut self, probability_max: f64) -> &mut Self`| Sets max mutation probability.         |
| `with_mutation_probability_min` | `fn with_mutation_probability_min(&mut self, probability_min: f64) -> &mut Self`| Sets min mutation probability.         |
| `with_mutation_method`          | `fn with_mutation_method(&mut self, method: Mutation) -> &mut Self`| Sets mutation method.                          |
| `with_mutation_step`            | `fn with_mutation_step(&mut self, step: f64) -> &mut Self`       | Sets step size for Creep mutation.                   |
| `with_mutation_sigma`           | `fn with_mutation_sigma(&mut self, sigma: f64) -> &mut Self`     | Sets sigma for Gaussian mutation.                    |
| `with_save_progress`            | `fn with_save_progress(&mut self, save_progress: bool) -> &mut Self`| Enables saving progress.                       |
| `with_save_progress_interval`   | `fn with_save_progress_interval(&mut self, save_progress_interval: i32) -> &mut Self`| Sets save interval.           |
| `with_save_progress_path`       | `fn with_save_progress_path(&mut self, save_progress_path: String) -> &mut Self`| Sets save path.                       |
| `with_elitism`                  | `fn with_elitism(&mut self, elitism_count: usize) -> &mut Self`  | Sets number of elite chromosomes.                    |
| `with_stopping_criteria`        | `fn with_stopping_criteria(&mut self, criteria: StoppingCriteria) -> &mut Self`| Sets compound stopping criteria.           |

## Usage

### Basic Example

Implementing a custom gene and chromosome:

```rust
use std::borrow::Cow;
use my_ga_lib::traits::{GeneT, ChromosomeT, ConfigurationT};

// Custom gene type
#[derive(Default, Clone)]
pub struct MyGene {
    id: i32,
    value: f64,
}

impl GeneT for MyGene {
    fn new() -> Self {
        MyGene { id: 0, value: 0.0 }
    }
    fn default(mut self) -> Self {
        self.id = 0;
        self.value = 0.0;
        self
    }
    fn get_id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

// Custom chromosome type
#[derive(Default, Clone)]
pub struct MyChromosome {
    dna: Vec<MyGene>,
    fitness: f64,
    age: i32,
    fitness_fn: Option<Box<dyn Fn(&[MyGene]) -> f64 + Send + Sync>>,
}

impl ChromosomeT for MyChromosome {
    type Gene = MyGene;

    fn new() -> Self {
        MyChromosome::default()
    }

    fn get_dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        match dna {
            Cow::Borrowed(slice) => self.dna = slice.to_vec(),
            Cow::Owned(vec) => self.dna = vec,
        }
        self
    }

    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = Some(Box::new(fitness_fn));
        self
    }

    fn calculate_fitness(&mut self) {
        if let Some(ref f) = self.fitness_fn {
            self.fitness = f(&self.dna);
        }
    }

    fn get_fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn set_age(&mut self, age: i32) -> &mut Self {
        self.age = age;
        self
    }

    fn get_age(&self) -> i32 {
        self.age
    }
}
```

### Advanced Example

Configuring the genetic algorithm using the builder pattern and manipulating DNA efficiently:

```rust
use std::borrow::Cow;
use my_ga_lib::traits::{GeneT, ChromosomeT, ConfigurationT};
use my_ga_lib::operators::{Selection, Crossover, Mutation, Survivor};
use my_ga_lib::stopping::StoppingCriteria;
use my_ga_lib::logging::LogLevel;

// Assume MyGene and MyChromosome are implemented as above

// Example: Efficient DNA manipulation using Cow
let mut chromosome = MyChromosome::new();
let dna_vec = vec![MyGene::new(); 10];
chromosome.set_dna(Cow::Owned(dna_vec)); // Zero-copy if you own the Vec

// Example: Setting a fitness function and calculating fitness
chromosome.set_fitness_fn(|dna| {
    dna.iter().map(|gene| gene.value).sum()
});
chromosome.calculate_fitness();
println!("Fitness: {}", chromosome.get_fitness());

// Example: Using the configuration builder
let mut config = MyConfiguration::new()
    .with_population_size(100)
    .with_genes_per_chromosome(10)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Swap)
    .with_mutation_probability_max(0.1)
    .with_mutation_probability_min(0.01)
    .with_survivor_method(Survivor::AgeBased)
    .with_max_generations(1000)
    .with_fitness_target(1.0)
    .with_threads(4)
    .with_logs(LogLevel::Info)
    .with_stopping_criteria(StoppingCriteria::AnyReached)
    .with_elitism(2);

// Now pass `config` to your genetic algorithm runner.
```

## API Reference

### `GeneT`

Trait for gene representation.

**Methods:**

| Name     | Signature                                         | Description                                  |
|----------|---------------------------------------------------|----------------------------------------------|
| `new`    | `fn new() -> Self`                                | Creates a new gene instance.                 |
| `default`| `fn default(mut self) -> Self`                    | Resets gene to default state.                |
| `get_id` | `fn get_id(&self) -> i32`                         | Gets the gene's unique identifier.           |
| `set_id` | `fn set_id(&mut self, id: i32) -> &mut Self`      | Sets the gene's unique identifier.           |

### `ChromosomeT`

Trait for chromosome representation.

**Associated Types:**

| Name     | Description                                  |
|----------|----------------------------------------------|
| `Gene`   | The gene type used by the chromosome.        |

**Methods:**

| Name                  | Signature                                                                 | Description                                               |
|-----------------------|---------------------------------------------------------------------------|-----------------------------------------------------------|
| `new`                 | `fn new() -> Self`                                                        | Creates a new chromosome instance.                        |
| `default`             | `fn default(mut self) -> Self`                                            | Resets chromosome state.                                  |
| `new_gene`            | `fn new_gene() -> Self::Gene`                                             | Creates a new gene.                                       |
| `get_dna`             | `fn get_dna(&self) -> &[Self::Gene]`                                      | Returns DNA as slice.                                     |
| `set_dna`             | `fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self`      | Sets DNA efficiently using `Cow`.                         |
| `set_gene`            | `fn set_gene(&mut self, gene_index: usize, gene: Self::Gene) -> &mut Self`| Replaces gene at index.                                   |
| `set_fitness_fn`      | `fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self`             | Installs fitness function.                                |
| `calculate_fitness`   | `fn calculate_fitness(&mut self)`                                         | Computes and sets fitness.                                |
| `get_fitness`         | `fn get_fitness(&self) -> f64`                                            | Gets current fitness value.                               |
| `set_fitness`         | `fn set_fitness(&mut self, fitness: f64) -> &mut Self`                    | Sets fitness value.                                       |
| `set_age`             | `fn set_age(&mut self, age: i32) -> &mut Self`                            | Sets chromosome age.                                      |
| `get_age`             | `fn get_age(&self) -> i32`                                                | Gets chromosome age.                                      |
| `get_fitness_distance`| `fn get_fitness_distance(&self, fitness_target: &f64) -> f64`             | Computes distance to target fitness.                      |

### `ConfigurationT`

Trait for genetic algorithm configuration.

**Methods:**

| Name                          | Signature                                                        | Description                                              |
|-------------------------------|------------------------------------------------------------------|----------------------------------------------------------|
| `new`                         | `fn new() -> Self`                                               | Creates a new configuration.                             |
| `with_adaptive_ga`            | `fn with_adaptive_ga(&mut self, adaptive_ga: bool) -> &mut Self` | Enables adaptive GA features.                            |
| `with_threads`                | `fn with_threads(&mut self, number_of_threads: i32) -> &mut Self`| Sets number of threads.                                  |
| `with_logs`                   | `fn with_logs(&mut self, log_level: LogLevel) -> &mut Self`      | Sets logging level.                                      |
| `with_survivor_method`        | `fn with_survivor_method(&mut self, method: Survivor) -> &mut Self`| Sets survivor selection method.                        |
| `with_problem_solving`        | `fn with_problem_solving(&mut self, problem_solving: ProblemSolving) -> &mut Self`| Sets problem-solving mode.          |
| `with_max_generations`        | `fn with_max_generations(&mut self, max_generations: i32) -> &mut Self`| Sets max generations.                               |
| `with_fitness_target`         | `fn with_fitness_target(&mut self, fitness_target: f64) -> &mut Self`| Sets target fitness.                                |
| `with_population_size`        | `fn with_population_size(&mut self, population_size: i32) -> &mut Self`| Sets population size.                              |
| `with_genes_per_chromosome`   | `fn with_genes_per_chromosome(&mut self, genes_per_chromosome: i32) -> &mut Self`| Sets genes per chromosome.                 |
| `with_needs_unique_ids`       | `fn with_needs_unique_ids(&mut self, needs_unique_ids: bool) -> &mut Self`| Requires unique gene IDs.                        |
| `with_alleles_can_be_repeated`| `fn with_alleles_can_be_repeated(&mut self, alleles_can_be_repeated: bool) -> &mut Self`| Allows repeated alleles.          |
| `with_number_of_couples`      | `fn with_number_of_couples(&mut self, number_of_couples: i32) -> &mut Self`| Sets number of couples.                         |
| `with_selection_method`       | `fn with_selection_method(&mut self, selection_method: Selection) -> &mut Self`| Sets selection method.                      |
| `with_crossover_number_of_points`| `fn with_crossover_number_of_points(&mut self, number_of_points: i32) -> &mut Self`| Sets crossover points.                 |
| `with_crossover_probability_max`| `fn with_crossover_probability_max(&mut self, probability_max: f64) -> &mut Self`| Sets max crossover probability.         |
| `with_crossover_probability_min`| `fn with_crossover_probability_min(&mut self, probability_min: f64) -> &mut Self`| Sets min crossover probability.         |
| `with_crossover_method`       | `fn with_crossover_method(&mut self, method: Crossover) -> &mut Self`| Sets crossover method.                        |
| `with_sbx_eta`                | `fn with_sbx_eta(&mut self, eta: f64) -> &mut Self`              | Sets SBX distribution index.                         |
| `with_blend_alpha`            | `fn with_blend_alpha(&mut self, alpha: f64) -> &mut Self`        | Sets BLX-α crossover alpha parameter.                 |
| `with_mutation_probability_max`| `fn with_mutation_probability_max(&mut self, probability_max: f64) -> &mut Self`| Sets max mutation probability.         |
| `with_mutation_probability_min`| `fn with_mutation_probability_min(&mut self, probability_min: f64) -> &mut Self`| Sets min mutation probability.         |
| `with_mutation_method`        | `fn with_mutation_method(&mut self, method: Mutation) -> &mut Self`| Sets mutation method.                          |
| `with_mutation_step`          | `fn with_mutation_step(&mut self, step: f64) -> &mut Self`       | Sets step size for Creep mutation.                   |
| `with_mutation_sigma`         | `fn with_mutation_sigma(&mut self, sigma: f64) -> &mut Self`     | Sets sigma for Gaussian mutation.                    |
| `with_save_progress`          | `fn with_save_progress(&mut self, save_progress: bool) -> &mut Self`| Enables saving progress.                       |
| `with_save_progress_interval` | `fn with_save_progress_interval(&mut self, save_progress_interval: i32) -> &mut Self`| Sets save interval.           |
| `with_save_progress_path`     | `fn with_save_progress_path(&mut self, save_progress_path: String) -> &mut Self`| Sets save path.                       |
| `with_elitism`                | `fn with_elitism(&mut self, elitism_count: usize) -> &mut Self`  | Sets number of elite chromosomes.                    |
| `with_stopping_criteria`      | `fn with_stopping_criteria(&mut self, criteria: StoppingCriteria) -> &mut Self`| Sets compound stopping criteria.           |

## Observer Traits

The observer system provides structured, lifecycle-aware hooks into the GA run loop. Unlike
the legacy `Reporter` trait, observers are stored as `Arc<dyn GaObserver + Send + Sync>` and
receive `&self` references, enabling interior mutability and safe sharing across island threads.

### `GaObserver<U>`

Defined in `src/observe/observer/mod.rs`. All methods have default no-op implementations.

| Hook                             | When it fires                                           |
|----------------------------------|---------------------------------------------------------|
| `on_run_start`                   | Once before the first generation                        |
| `on_generation_start`            | Start of each generation, before any operators          |
| `on_selection_complete`          | After parent selection                                  |
| `on_crossover_complete`          | After crossover produces offspring                      |
| `on_mutation_complete`           | After mutation is applied                               |
| `on_fitness_evaluation_complete` | After fitness evaluation of the new population          |
| `on_survivor_selection_complete` | After survivor selection prunes the population          |
| `on_new_best`                    | When the population's best fitness improves             |
| `on_stagnation`                  | Each time the stagnation counter increments             |
| `on_extension_triggered`         | When an extension strategy fires due to low diversity   |
| `on_generation_end`              | End of each generation, after statistics collected      |
| `on_run_end`                     | Once after the GA loop exits                            |

Attach via `.with_observer(Arc::new(my_observer))` on the `Ga` builder.

### `IslandGaObserver<U>`

Island-model-specific hooks. Implement alongside `GaObserver<U>` to observe island
runs and migration events.

| Hook                        | When it fires                                  |
|-----------------------------|------------------------------------------------|
| `on_island_run_start`       | When an island's run starts                    |
| `on_island_run_end`         | When an island's run ends                      |
| `on_island_generation_end`  | At the end of each generation per island       |
| `on_migration_triggered`    | When migration between islands fires           |

### `Nsga2Observer<U>`

NSGA-II-specific hooks for observing Pareto front assignment and sort timing.

| Hook                           | When it fires                                       |
|--------------------------------|-----------------------------------------------------|
| `on_pareto_front_assigned`     | After Pareto fronts are assigned                    |
| `on_non_dominated_sort_complete` | After non-dominated sorting completes             |
| `on_crowding_distance_calculated` | After crowding distance calculation completes    |

### `AllObserver<U>`

Supertrait that combines `GaObserver<U>`, `IslandGaObserver<U>`, and `Nsga2Observer<U>`.
A blanket `impl` automatically applies to any type implementing all three. Used internally
by `CompositeObserver`.

### Built-in Observer Implementations

| Type                  | Feature flag        | Description                                                              |
|-----------------------|---------------------|--------------------------------------------------------------------------|
| `NoopObserver`        | always available    | Zero-sized no-op observer. Useful as a placeholder.                      |
| `LogObserver`         | always available    | Logs generation statistics using the `log` crate.                        |
| `CompositeObserver<U>` | always available   | Fan-out dispatch: forwards every hook to a list of inner observers.      |
| `TracingObserver`     | `observer-tracing`  | Emits structured `tracing` spans and events per hook.                    |
| `MetricsObserver`     | `observer-metrics`  | Collects numeric metrics into a thread-safe `MetricsStore`.              |

### Comparison: `Reporter` vs `GaObserver`

| Aspect          | `Reporter`                         | `GaObserver`                               |
|-----------------|------------------------------------|--------------------------------------------|
| Storage         | `Box<dyn Reporter + Send>`         | `Arc<dyn GaObserver + Send + Sync>`        |
| Mutability      | `&mut self`                        | `&self` (interior mutability)              |
| Hooks           | 4 lifecycle hooks                  | 12 hooks (lifecycle + operator + special)  |
| Thread safety   | `Send` only                        | `Send + Sync`                              |
| Island support  | No                                 | Yes (via `IslandGaObserver`)               |
| NSGA-II support | No                                 | Yes (via `Nsga2Observer`)                  |

## Related

- [chromosomes.md](chromosomes.md)
- [genotypes.md](genotypes.md)
- [configuration.md](configuration.md)
- [operators/selection.md](operators/selection.md)
- [operators/crossover.md](operators/crossover.md)
- [operators/mutation.md](operators/mutation.md)
- [operators/survivor.md](operators/survivor.md)
- [fitness.md](fitness.md)
- [population.md](population.md)
- [src/traits.rs](../src/traits.rs)
- [src/traits/gene.rs](../src/traits/gene.rs)
- [src/traits/chromosome.rs](../src/traits/chromosome.rs)
- [src/traits/configuration.rs](../src/traits/configuration.rs)
