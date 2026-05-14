# Error Handling

> All fallible operations return `GaError`, a single enum covering configuration mistakes, operator failures, initialization problems, and I/O issues.

## Overview

The `GaError` enum is the single error type for the entire crate. All fallible operations return `Result<T, GaError>`, and the `?` operator works seamlessly for error propagation. The enum implements `std::error::Error` and `std::fmt::Display`.

## GaError Enum Variants

| Variant | Description | When It Occurs |
|---------|-------------|----------------|
| `ConfigurationError(String)` | Generic configuration error. | Missing or invalid parameters in GaConfiguration. |
| `ValidationError(String)` | Validation check failed. | DNA length mismatch, duplicate IDs. |
| `CrossoverError(String)` | Crossover operation failed. | Incompatible chromosome types, operator failure. |
| `MutationError(String)` | Mutation operation failed. | Unsupported mutation for chromosome type. |
| `InitializationError(String)` | Population initialization failed. | Missing initialization function, gene creation failure. |
| `SelectionError(String)` | Selection operation failed. | Empty population, insufficient individuals. |
| `InvalidIslandConfiguration(String)` | Island model configuration error. | Invalid number of islands, migration parameters. |
| `InvalidNichingConfiguration(String)` | Niching configuration error. | Invalid sigma_share, alpha values. |
| `InvalidNsga2Configuration(String)` | NSGA-II configuration error. | Missing objectives, invalid population parameters. |
| `InvalidNsga3Configuration(String)` | NSGA-III configuration error. | Missing reference points, invalid num_objectives. |
| `InvalidMoeaDConfiguration(String)` | MOEA/D configuration error. | Missing weight vectors, invalid neighborhood settings. |
| `InvalidSpea2Configuration(String)` | SPEA2 configuration error. | Invalid archive_size, missing objectives. |
| `InvalidSmsEmoaConfiguration(String)` | SMS-EMOA configuration error. | Invalid hypervolume reference point. |
| `InvalidIbeaConfiguration(String)` | IBEA configuration error. | Missing objective functions. |
| `InvalidConstraintConfiguration(String)` | Constraint configuration error. | Negative penalty coefficients, invalid dynamic parameters. |
| `InvalidIndicatorConfiguration(String)` | Quality indicator configuration error. | Empty point sets, dimension mismatches. |
| `MigrationError(String)` | Island migration failed. | Migrant selection or transfer failure. |
| `CheckpointError(String)` | Checkpoint save/load failed. | I/O errors, serialization failures (requires `serde` feature). |
| `LocalSearchError(String)` | Local search operation failed. | Hill climbing failure, bounds violation. |

## Usage Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::error::GaError;
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::initializers::binary_random_initialization;

fn run_ga() -> Result<(), GaError> {
    let mut ga = Ga::new()
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
        .build()?;  // Returns ConfigurationError on invalid params

    let population = ga.run()?;  // Returns operator/init errors
    println!("Best fitness: {:.2}", population.best_chromosome.fitness);
    Ok(())
}

fn main() {
    match run_ga() {
        Ok(()) => println!("GA completed successfully"),
        Err(e) => eprintln!("GA failed: {}", e),
    }
}
```

### Error Propagation in Multi-Objective Engines

```rust,ignore
use genetic_algorithms::error::GaError;

fn run_nsga3() -> Result<(), GaError> {
    // Configuration validation returns specific error variant
    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(91)
        .with_max_generations(300)
        .with_reference_points_auto(12);

    let ga_config = GaConfiguration::default()
        .with_crossover_method(Crossover::Sbx)
        .with_mutation_method(Mutation::Polynomial);

    let mut nsga3 = Nsga3Ga::<MyChromosome>::new(nsga3_config, ga_config)
        .with_initialization_fn(my_init_fn)
        .with_objective_fns(vec![/* ... */])
        .build()?;  // Returns InvalidNsga3Configuration on validation failure

    let front = nsga3.run()?;  // Returns MutationError, CrossoverError, etc.
    Ok(())
}
```

## Common Error Patterns

### Configuration Validation

Configuration errors occur at build time (`.build()?`). These always include a descriptive message explaining what is wrong:

```rust,ignore
use genetic_algorithms::error::GaError;

match ga.build() {
    Ok(ga) => ga.run(),
    Err(GaError::ConfigurationError(msg)) => {
        eprintln!("Please fix configuration: {}", msg);
        std::process::exit(1);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
        std::process::exit(1);
    }
}
```

### Operator Failures

Operator errors (`CrossoverError`, `MutationError`, etc.) occur during `run()`. These are typically caused by incompatible chromosome types or operator misconfiguration:

```rust,ignore
// Using Gaussian mutation on Binary chromosomes will fail
let result = ga.run();
if let Err(GaError::MutationError(msg)) = &result {
    eprintln!("Mutation failed: {}", msg);
    // Consider switching to BitFlip mutation
}
```

## Performance Considerations

- `GaError` is `Clone` + `PartialEq` — suitable for storing in collections and comparing.
- With the `serde` feature, `GaError` is serializable for checkpoint error logging.
- All error messages are `String` (heap-allocated). In hot paths (e.g., inside the generation loop), errors should be exceptional.

## See Also

- [Configuration](configuration.md) — Valid configuration patterns to avoid configuration errors
- [docs.rs/genetic_algorithms::error](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/error/index.html) — Module API reference
