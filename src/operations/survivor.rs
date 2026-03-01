pub(crate) use crate::configuration::LimitConfiguration;
use crate::error::GaError;
use crate::traits::ChromosomeT;
pub use self::fitness::fitness_based;
pub use self::age::age_based;

use super::Survivor;
pub mod fitness;
pub mod age;

/// Dispatches survivor selection according to the configured method.
///
/// # Returns
///
/// `Ok(())` after trimming the population, or `Err(GaError)` on failure.
pub fn factory<U: ChromosomeT>(survivor: Survivor, chromosomes: &mut Vec<U>, population_size: usize, limit_configuration: LimitConfiguration) -> Result<(), GaError> {
    match survivor {
        Survivor::Fitness => fitness_based(chromosomes, population_size, limit_configuration),
        Survivor::Age => age_based(chromosomes, population_size),
    }
    Ok(())
}