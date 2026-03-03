# Validators

> Validation system for genetic algorithm configuration and population integrity.

## Overview

The validators module provides a set of functions to ensure the correctness and consistency of genetic algorithm configurations and populations. These validation routines help detect common issues such as mismatched chromosome lengths, duplicate gene IDs, missing configuration parameters, and other logical errors before running evolutionary operations. By using these validators, users can avoid subtle bugs and runtime panics that may arise from invalid input data or misconfigured algorithms.

Validators are typically invoked before starting the main genetic algorithm loop, or after key configuration changes, to guarantee that all requirements are satisfied. They are designed to work with generic population and configuration types, making them flexible and applicable to a wide range of genetic algorithm setups. Integrating these checks into your workflow will improve robustness and make debugging easier.

The validation system fits into the overall library architecture as a utility layer, complementing the core genetic operators and traits. It is especially useful when building custom algorithms or experimenting with new genotype representations, as it provides immediate feedback on structural and configuration errors.

## Key Concepts

Validators operate on two main abstractions: `Population<U>` and `GaConfiguration`. Each function targets a specific aspect of the genetic algorithm's state or setup.

| Function Name                       | Input Type(s)                  | Description                                                      |
|--------------------------------------|-------------------------------|------------------------------------------------------------------|
| `validate`                          | `Population<U>` or config      | Entry point for running all relevant validators                   |
| `unique_gene_ids`                   | `&Population<U>`               | Ensures all chromosomes have unique gene IDs                      |
| `fitness_target_is_some`             | `&GaConfiguration`             | Checks that the fitness target is set                             |
| `same_dna_length`                   | `&Population<U>`               | Ensures all chromosomes have the same DNA length                  |
| `chromosome_length_not_bigger_than_alleles` | `&Population<U>`        | Ensures chromosome length does not exceed number of alleles       |
| `aga_crossover_probabilities`        | `&GaConfiguration`             | Checks that adaptive crossover probabilities are set              |
| `number_of_couples_is_set`           | `&GaConfiguration`             | Ensures number of couples is specified in configuration           |

**Type Constraints:**

- `U` is a generic type representing the chromosome or genotype. It must satisfy the requirements of the population and configuration types used in the library.

## Usage

### Basic Example

```rust
use my_ga_lib::population::Population;
use my_ga_lib::validators::generic_validator::{unique_gene_ids, same_dna_length};
use my_ga_lib::chromosomes::BinaryChromosome;

fn main() -> Result<(), my_ga_lib::error::GaError> {
    // Create a population of binary chromosomes
    let population = Population::<BinaryChromosome>::new_random(100, 32);

    // Validate that all chromosomes have unique gene IDs
    unique_gene_ids(&population)?;

    // Validate that all chromosomes have the same DNA length
    same_dna_length(&population)?;

    Ok(())
}
```

### Advanced Example

```rust
use my_ga_lib::population::Population;
use my_ga_lib::validators::generic_validator::{
    unique_gene_ids, same_dna_length, chromosome_length_not_bigger_than_alleles,
    fitness_target_is_some, aga_crossover_probabilities, number_of_couples_is_set
};
use my_ga_lib::validators::validator_factory::validate;
use my_ga_lib::chromosomes::RangeChromosome;
use my_ga_lib::configuration::GaConfiguration;

fn main() -> Result<(), my_ga_lib::error::GaError> {
    // Create a configuration for the genetic algorithm
    let mut config = GaConfiguration::default();
    config.set_fitness_target(100.0);
    config.set_crossover_probabilities(vec![0.6, 0.8]);
    config.set_number_of_couples(50);

    // Validate configuration requirements
    fitness_target_is_some(&config)?;
    aga_crossover_probabilities(&config)?;
    number_of_couples_is_set(&config)?;

    // Create a population of range chromosomes
    let population = Population::<RangeChromosome>::new_random(50, 16);

    // Run all population validators
    unique_gene_ids(&population)?;
    same_dna_length(&population)?;
    chromosome_length_not_bigger_than_alleles(&population)?;

    // Use the validator factory to run all checks at once
    validate(&population)?;
    validate(&config)?;

    Ok(())
}
```

## API Reference

### `validate` (validator_factory)

Runs all relevant validators for a given population or configuration.

| Name     | Signature                                 | Description                                         |
|----------|-------------------------------------------|-----------------------------------------------------|
| `validate` | `fn validate<U>(target: &T) -> Result<(), GaError>` | Runs all applicable validators on the target. T may be `Population<U>` or `GaConfiguration`. |

### `unique_gene_ids`

Checks that every chromosome in the population has unique gene IDs within its DNA.

| Name             | Signature                                                  | Description                           |
|------------------|-----------------------------------------------------------|---------------------------------------|
| `unique_gene_ids`| `fn unique_gene_ids<U>(population: &Population<U>) -> Result<(), GaError>` | Ensures all chromosomes have unique gene IDs. |

### `fitness_target_is_some`

Checks that the fitness target is set in the genetic algorithm configuration.

| Name                   | Signature                                        | Description                           |
|------------------------|--------------------------------------------------|---------------------------------------|
| `fitness_target_is_some`| `fn fitness_target_is_some(config: &GaConfiguration) -> Result<(), GaError>` | Ensures fitness target is specified.  |

### `same_dna_length`

Checks that all chromosomes in the population have the same DNA length.

| Name             | Signature                                                  | Description                           |
|------------------|-----------------------------------------------------------|---------------------------------------|
| `same_dna_length`| `fn same_dna_length<U>(population: &Population<U>) -> Result<(), GaError>` | Ensures uniform DNA length.           |

### `chromosome_length_not_bigger_than_alleles`

Checks that the chromosome length does not exceed the number of alleles.

| Name                                 | Signature                                                  | Description                           |
|--------------------------------------|-----------------------------------------------------------|---------------------------------------|
| `chromosome_length_not_bigger_than_alleles` | `fn chromosome_length_not_bigger_than_alleles<U>(population: &Population<U>) -> Result<(), GaError>` | Ensures chromosome length is valid.   |

### `aga_crossover_probabilities`

Checks that all requirements for adaptive crossover are set in the configuration.

| Name                         | Signature                                        | Description                           |
|------------------------------|--------------------------------------------------|---------------------------------------|
| `aga_crossover_probabilities`| `fn aga_crossover_probabilities(config: &GaConfiguration) -> Result<(), GaError>` | Validates adaptive crossover settings.|

### `number_of_couples_is_set`

Checks that the number of couples is specified in the configuration.

| Name                     | Signature                                        | Description                           |
|--------------------------|--------------------------------------------------|---------------------------------------|
| `number_of_couples_is_set`| `fn number_of_couples_is_set(config: &GaConfiguration) -> Result<(), GaError>` | Ensures number of couples is set.     |

## Related

- [configuration.md](configuration.md) — Genetic algorithm configuration options
- [population.md](population.md) — Population management
- [traits.md](traits.md) — Core traits for chromosomes and genetic algorithms
- [src/validators/generic_validator.rs](../src/validators/generic_validator.rs) — Source code for generic validators
- [src/validators/validator_factory.rs](../src/validators/validator_factory.rs) — Source code for validator factory
- [examples.md](examples.md) — End-to-end genetic algorithm examples
