pub(crate) use crate::configuration::LimitConfiguration;
use crate::traits::ChromosomeT;
pub use self::fitness::fitness_based;
pub use self::age::age_based;

use super::Survivor;
pub mod fitness;
pub mod age;

pub fn factory<U: ChromosomeT>(survivor: Survivor, chromosomes: &mut Vec<U>, population_size: usize, limit_configuration: LimitConfiguration){
    match survivor {
        Survivor::Fitness => {fitness_based(chromosomes, population_size, limit_configuration)},
        Survivor::Age => {age_based(chromosomes, population_size)},
    }
}