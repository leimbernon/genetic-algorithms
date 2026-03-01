use crate::configuration::SelectionConfiguration;
use crate::error::GaError;
use crate::traits::ChromosomeT;

pub use self::random::random;
pub use self::fitness_proportionate::roulette_wheel_selection;
pub use self::fitness_proportionate::stochastic_universal_sampling;
pub use self::tournament::tournament;

use super::Selection;

pub mod random;
pub mod fitness_proportionate;
pub mod tournament;

/// Dispatches parent selection according to the configured method.
///
/// # Returns
///
/// `Ok(Vec<(usize, usize)>)` with the parent pairs, or `Err(GaError::SelectionError)` if
/// the population is too small to form pairs.
pub fn factory<U>(chromosomes: &Vec<U>, configuration: SelectionConfiguration, number_of_threads: i32) -> Result<Vec<(usize, usize)>, GaError>
where
    U: ChromosomeT + Sync + Send + 'static + Clone,
{
    if chromosomes.len() < 2 {
        return Err(GaError::SelectionError(
            format!("Population size {} is too small for selection (minimum 2)", chromosomes.len()),
        ));
    }

    let pairs = match configuration.method {
        Selection::Random => random(chromosomes),
        Selection::RouletteWheel => roulette_wheel_selection(chromosomes),
        Selection::StochasticUniversalSampling => {
            stochastic_universal_sampling(chromosomes, configuration.number_of_couples)
        }
        Selection::Tournament => {
            tournament(chromosomes, configuration.number_of_couples, number_of_threads)
        }
    };

    Ok(pairs)
}