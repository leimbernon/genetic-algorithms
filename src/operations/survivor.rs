//! Survivor-selection operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! survivor-selection strategies (fitness-based, age-based, mu+lambda,
//! mu,lambda). The correct strategy is selected at runtime based on the
//! [`Survivor`] variant in the configuration.

pub use self::age::age_based;
pub use self::deterministic_crowding::deterministic_crowding;
pub use self::fitness::fitness_based;
pub use self::mu_comma_lambda::mu_comma_lambda;
pub use self::mu_plus_lambda::mu_plus_lambda;
pub(crate) use crate::configuration::LimitConfiguration;
use crate::error::GaError;
use crate::traits::{ChromosomeT, SurvivorOperator};

use super::Survivor;
pub mod age;
pub mod deterministic_crowding;
pub mod fitness;
pub mod mu_comma_lambda;
pub mod mu_plus_lambda;

impl SurvivorOperator for Survivor {
    fn select_survivors<U: ChromosomeT>(
        &self,
        chromosomes: &mut Vec<U>,
        population_size: usize,
        limit_configuration: LimitConfiguration,
    ) -> Result<(), GaError> {
        match self {
            Survivor::Fitness => fitness_based(chromosomes, population_size, limit_configuration),
            Survivor::Age => age_based(chromosomes, population_size),
            Survivor::MuPlusLambda => {
                mu_plus_lambda(chromosomes, population_size, limit_configuration)
            }
            Survivor::MuCommaLambda => {
                mu_comma_lambda(chromosomes, population_size, limit_configuration)
            }
            Survivor::DeterministicCrowding => deterministic_crowding(chromosomes),
        }
        Ok(())
    }
}

/// Dispatches survivor selection according to the configured method.
///
/// # Returns
///
/// `Ok(())` after trimming the population, or `Err(GaError)` on failure.
pub fn factory<U: ChromosomeT>(
    survivor: Survivor,
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) -> Result<(), GaError> {
    // Guard: reject NaN fitness values which cause panics in sorting
    for (i, chromosome) in chromosomes.iter().enumerate() {
        if chromosome.fitness().is_nan() {
            return Err(GaError::SelectionError(format!(
                "Chromosome at index {} has NaN fitness. All chromosomes must have valid fitness before survivor selection.",
                i
            )));
        }
    }

    survivor.select_survivors(chromosomes, population_size, limit_configuration)
}
