use crate::configuration::SelectionConfiguration;
use crate::traits::ChromosomeT;

pub use self::random::random;
pub use self::fitness_proportionate::roulette_wheel_selection;
pub use self::fitness_proportionate::stochastic_universal_sampling;
pub use self::tournament::tournament;

use super::Selection;

pub mod random;
pub mod fitness_proportionate;
pub mod tournament;

pub fn factory<U>(chromosomes: &Vec<U>, configuration: SelectionConfiguration, number_of_threads: i32) -> Vec<(usize, usize)>
where
U: ChromosomeT + Sync + Send + 'static + Clone
{
    match configuration.method {
        Selection::Random => {random(chromosomes)},
        Selection::RouletteWheel => {roulette_wheel_selection(chromosomes)},
        Selection::StochasticUniversalSampling => {stochastic_universal_sampling(chromosomes, configuration.number_of_couples)},
        Selection::Tournament => {tournament(chromosomes, configuration.number_of_couples, number_of_threads)},
    }
}