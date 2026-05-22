//! Parent-selection operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! selection implementations (tournament, roulette wheel, stochastic
//! universal sampling, random, rank, Boltzmann, truncation). The correct
//! implementation is selected at runtime based on the [`Selection`] variant
//! in the configuration.

use crate::configuration::SelectionConfiguration;
use crate::error::GaError;
use crate::traits::{ChromosomeT, SelectionOperator};

pub use self::boltzmann::boltzmann_selection;
pub use self::clearing::clearing_selection;
pub use self::fitness_proportionate::roulette_wheel_selection;
pub use self::fitness_proportionate::stochastic_universal_sampling;
pub use self::random::random;
pub use self::rank::rank_selection;
pub use self::tournament::tournament;
pub use self::truncation::truncation_selection;

use super::Selection;

pub mod boltzmann;
pub mod clearing;
pub mod fitness_proportionate;
pub mod random;
pub mod rank;
pub mod tournament;
pub mod truncation;

impl SelectionOperator for Selection {
    fn select<U>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
    ) -> Vec<(usize, usize)>
    where
        U: ChromosomeT + Sync + Send + 'static + Clone,
    {
        match self {
            Selection::Random => random(chromosomes),
            Selection::RouletteWheel => roulette_wheel_selection(chromosomes),
            Selection::StochasticUniversalSampling => {
                stochastic_universal_sampling(chromosomes, number_of_couples)
            }
            Selection::Tournament => tournament(chromosomes, number_of_couples, number_of_threads),
            Selection::Rank => rank_selection(chromosomes, number_of_couples),
            Selection::Boltzmann => boltzmann_selection(chromosomes, number_of_couples, 1.0),
            Selection::Truncation => truncation_selection(chromosomes, number_of_couples),
            // WARNING: The `SelectionOperator` trait does not carry operator-specific
            // configuration, so `niche_radius` defaults to 0.1 on this path.
            // Island-model and NSGA-II callers that use `Selection::Clearing` with
            // a custom `niche_radius` must go through `selection::factory` instead.
            // The single-population GA always uses the factory path and is unaffected.
            Selection::Clearing => {
                log::warn!(target: "selection_events",
                    "Selection::Clearing called through SelectionOperator trait: \
                     niche_radius defaults to 0.1 (configured value ignored). \
                     Use selection::factory for the full configuration.");
                clearing_selection(chromosomes, 0.1, number_of_couples)
            }
            Selection::Lexicase | Selection::EpsilonLexicase => {
                panic!(
                    "Selection::Lexicase/EpsilonLexicase cannot be called through SelectionOperator \
                     trait: use selection::factory_lexicase for Lexicase/EpsilonLexicase operators. \
                     Island-model and NSGA-II paths do not support MultiCaseFitness."
                );
            }
        }
    }
}

/// Dispatches parent selection according to the configured method.
///
/// # Returns
///
/// `Ok(Vec<(usize, usize)>)` with the parent pairs, or `Err(GaError::SelectionError)` if
/// the population is too small to form pairs.
pub fn factory<U>(
    chromosomes: &[U],
    configuration: SelectionConfiguration,
    number_of_threads: usize,
) -> Result<Vec<(usize, usize)>, GaError>
where
    U: ChromosomeT + Sync + Send + 'static + Clone,
{
    if chromosomes.len() < 2 {
        return Err(GaError::SelectionError(format!(
            "Population size {} is too small for selection (minimum 2)",
            chromosomes.len()
        )));
    }

    // Guard: reject NaN fitness values which corrupt selection logic
    for (i, chromosome) in chromosomes.iter().enumerate() {
        if chromosome.fitness().is_nan() {
            return Err(GaError::SelectionError(format!(
                "Chromosome at index {} has NaN fitness. All chromosomes must have valid fitness before selection.",
                i
            )));
        }
    }

    let pairs = match configuration.method {
        Selection::Boltzmann => boltzmann_selection(
            chromosomes,
            configuration.number_of_couples,
            configuration.boltzmann_temperature,
        ),
        Selection::Clearing => clearing_selection(
            chromosomes,
            configuration.niche_radius,
            configuration.number_of_couples,
        ),
        Selection::Lexicase | Selection::EpsilonLexicase => {
            return Err(GaError::ConfigurationError(
                "Use selection::factory_lexicase for Lexicase/EpsilonLexicase; \
                 standard factory() does not support MultiCaseFitness bound."
                    .into(),
            ));
        }
        _ => configuration.method.select(
            chromosomes,
            configuration.number_of_couples,
            number_of_threads,
        ),
    };

    Ok(pairs)
}
