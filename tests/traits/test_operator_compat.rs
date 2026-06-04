// Wave 0 tests: OperatorCompat trait behavior
//
// These tests prove that:
// 1. Chromosome types with empty OperatorCompat impls return None (no restriction).
// 2. A chromosome type with a restricted crossover set causes operator_compat_check
//    to return Err(GaError::ConfigurationError) when a non-allowed crossover is configured.
// 3. A chromosome type with a restricted mutation set causes operator_compat_check
//    to return Err(GaError::ConfigurationError) when a non-allowed mutation is configured.
// 4. A chromosome type with a restricted crossover set allows a matching crossover.
//
// References:
//   - GEN-01: D-04 (OperatorCompat trait) and D-05 (build-time validation)
//   - Plan 48-01, Task 3

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::operations::{Crossover, Mutation};
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome, OperatorCompat};
use genetic_algorithms::validators::generic_validator;
use std::borrow::Cow;

// -- Minimal test chromosome and gene for restriction tests --------------------

/// Minimal test gene: only implements GeneT.
#[derive(Debug, Clone, Default, PartialEq)]
struct TestGene {
    id: i32,
}

impl GeneT for TestGene {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

/// A chromosome with restricted crossovers: only Pmx is allowed.
#[derive(Debug, Clone, Default)]
struct RestrictedCrossChromo {
    dna: Vec<TestGene>,
    fitness: f64,
    age: usize,
}

impl ChromosomeT for RestrictedCrossChromo {
    type Gene = TestGene;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn calculate_fitness(&mut self) {
        self.fitness = self.dna.len() as f64;
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }

    fn age(&self) -> usize {
        self.age
    }
}

impl LinearChromosome for RestrictedCrossChromo {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }

    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(s) => s.to_vec(),
            Cow::Owned(v) => v,
        };
        self
    }

    fn set_fitness_fn<F>(&mut self, _fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self
    }
}

/// Restricted to Pmx only.
impl OperatorCompat for RestrictedCrossChromo {
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        Some(&[Crossover::Pmx])
    }
}

/// A chromosome with restricted mutations: only Swap is allowed.
#[derive(Debug, Clone, Default)]
struct RestrictedMutChromo {
    dna: Vec<TestGene>,
    fitness: f64,
    age: usize,
}

impl ChromosomeT for RestrictedMutChromo {
    type Gene = TestGene;

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn calculate_fitness(&mut self) {
        self.fitness = self.dna.len() as f64;
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }

    fn age(&self) -> usize {
        self.age
    }
}

impl LinearChromosome for RestrictedMutChromo {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }

    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(s) => s.to_vec(),
            Cow::Owned(v) => v,
        };
        self
    }

    fn set_fitness_fn<F>(&mut self, _fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self
    }
}

/// Restricted to Swap mutation only.
impl OperatorCompat for RestrictedMutChromo {
    fn valid_mutations() -> Option<&'static [Mutation]> {
        Some(&[Mutation::Swap])
    }
}

// -- Helpers ------------------------------------------------------------------

fn default_config() -> GaConfiguration {
    use genetic_algorithms::traits::SelectionConfig;
    GaConfiguration::default().with_number_of_couples(1)
}

fn config_with_crossover(method: Crossover) -> GaConfiguration {
    use genetic_algorithms::traits::{CrossoverConfig, SelectionConfig};
    GaConfiguration::default()
        .with_number_of_couples(1)
        .with_crossover_method(method)
}

fn config_with_mutation(method: Mutation) -> GaConfiguration {
    use genetic_algorithms::traits::{MutationConfig, SelectionConfig};
    GaConfiguration::default()
        .with_number_of_couples(1)
        .with_mutation_method(method)
}

// -- Tests --------------------------------------------------------------------

/// BinaryChromosome has an empty OperatorCompat impl: both methods return None.
#[test]
fn default_none_returns() {
    assert!(
        <BinaryChromosome as OperatorCompat>::valid_crossovers().is_none(),
        "BinaryChromosome should have no crossover restriction (None)"
    );
    assert!(
        <BinaryChromosome as OperatorCompat>::valid_mutations().is_none(),
        "BinaryChromosome should have no mutation restriction (None)"
    );
}

/// A chromosome restricted to Pmx returns Err when SinglePoint is configured.
#[test]
fn restricted_crossover_rejected() {
    // Default crossover is Uniform; configure SinglePoint — neither is Pmx.
    let config = config_with_crossover(Crossover::SinglePoint);
    let result =
        generic_validator::operator_compat_check::<RestrictedCrossChromo>(&config);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError for incompatible crossover, got: {:?}",
        result
    );
}

/// A chromosome restricted to Swap mutation returns Err when Inversion is configured.
#[test]
fn restricted_mutation_rejected() {
    // Configure Inversion — not in valid set (only Swap).
    let config = config_with_mutation(Mutation::Inversion);
    let result =
        generic_validator::operator_compat_check::<RestrictedMutChromo>(&config);
    assert!(
        matches!(result, Err(GaError::ConfigurationError(_))),
        "Expected ConfigurationError for incompatible mutation, got: {:?}",
        result
    );
}

/// A chromosome restricted to Pmx returns Ok when Pmx is configured.
#[test]
fn allowed_operator_accepted() {
    let config = config_with_crossover(Crossover::Pmx);
    let result =
        generic_validator::operator_compat_check::<RestrictedCrossChromo>(&config);
    assert!(
        result.is_ok(),
        "Expected Ok for allowed crossover (Pmx), got: {:?}",
        result
    );
}

/// BinaryChromosome (unrestricted) always returns Ok from operator_compat_check.
#[test]
fn unrestricted_always_ok() {
    let config = default_config();
    let result =
        generic_validator::operator_compat_check::<BinaryChromosome>(&config);
    assert!(
        result.is_ok(),
        "Unrestricted chromosome should always pass operator_compat_check, got: {:?}",
        result
    );
}
