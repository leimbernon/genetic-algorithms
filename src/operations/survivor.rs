pub use self::age::age_based;
pub use self::fitness::fitness_based;
pub(crate) use crate::configuration::LimitConfiguration;
use crate::error::GaError;
use crate::traits::ChromosomeT;

use super::Survivor;
pub mod age;
pub mod fitness;

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

    match survivor {
        Survivor::Fitness => fitness_based(chromosomes, population_size, limit_configuration),
        Survivor::Age => age_based(chromosomes, population_size),
    }
    Ok(())
}
