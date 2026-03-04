use crate::configuration::SelectionConfiguration;
use crate::error::GaError;
use crate::traits::{ChromosomeT, SelectionOperator};

pub use self::fitness_proportionate::roulette_wheel_selection;
pub use self::fitness_proportionate::stochastic_universal_sampling;
pub use self::random::random;
pub use self::rank::rank_selection;
pub use self::tournament::tournament;

use super::Selection;

pub mod fitness_proportionate;
pub mod random;
pub mod rank;
pub mod tournament;

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

    let pairs = configuration.method.select(
        chromosomes,
        configuration.number_of_couples,
        number_of_threads,
    );

    Ok(pairs)
}
