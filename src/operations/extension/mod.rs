//! Extension operators for population diversity control.
//!
//! This module provides the [`factory`] dispatch function and individual
//! extension strategies. The correct strategy is selected at runtime based on
//! the [`Extension`] variant in the configuration.

pub mod mass_deduplication;
pub mod mass_degeneration;
pub mod mass_extinction;
pub mod mass_genesis;

use crate::configuration::ProblemSolving;
use crate::error::GaError;
use crate::extension::configuration::ExtensionConfiguration;
use crate::traits::{LinearChromosome, ExtensionOperator};

use super::Extension;

impl ExtensionOperator for Extension {
    fn apply_extension<U: LinearChromosome>(
        &self,
        chromosomes: &mut Vec<U>,
        population_size: usize,
        problem_solving: ProblemSolving,
        config: &ExtensionConfiguration,
    ) -> Result<(), GaError> {
        match self {
            Extension::Noop => {}
            Extension::MassExtinction => {
                mass_extinction::mass_extinction(
                    chromosomes,
                    population_size,
                    problem_solving,
                    config.survival_rate,
                    config.elite_count,
                );
            }
            Extension::MassGenesis => {
                mass_genesis::mass_genesis(chromosomes, problem_solving);
            }
            Extension::MassDegeneration => {
                mass_degeneration::mass_degeneration(
                    chromosomes,
                    problem_solving,
                    config.mutation_rounds,
                    config.elite_count,
                );
            }
            Extension::MassDeduplication => {
                mass_deduplication::mass_deduplication(chromosomes, problem_solving);
            }
        }
        Ok(())
    }
}

/// Dispatches extension according to the configured method.
///
/// # Returns
///
/// `Ok(())` after applying the extension, or `Err(GaError)` on failure.
pub fn factory<U: LinearChromosome>(
    method: Extension,
    chromosomes: &mut Vec<U>,
    population_size: usize,
    problem_solving: ProblemSolving,
    config: &ExtensionConfiguration,
) -> Result<(), GaError> {
    method.apply_extension(chromosomes, population_size, problem_solving, config)
}
