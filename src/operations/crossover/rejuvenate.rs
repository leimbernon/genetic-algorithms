//! Rejuvenate crossover implementation.

use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;

/// Rejuvenate crossover: clones parents as offspring and resets their ages to zero.
///
/// Given parents `P1` and `P2`, produces children that are exact clones with age reset:
/// - child_1 = clone of P1 with age set to 0
/// - child_2 = clone of P2 with age set to 0
///
/// # Use cases
///
/// - Combating population aging: refreshes the age of top-performing individuals so
///   they are not eliminated by age-based survivor selection.
/// - Restarting evolution with best-known solutions while treating them as "new" individuals.
///
/// # Errors
///
/// Returns `Err(GaError::CrossoverError)` if parents have different DNA lengths.
pub fn rejuvenate<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
    if parent_1.dna().len() != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            parent_1.dna().len(),
            parent_2.dna().len()
        )));
    }

    debug!(target="crossover_events", method="rejuvenate"; "Starting rejuvenate crossover");

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();

    child_1.set_age(0);
    child_2.set_age(0);

    debug!(target="crossover_events", method="rejuvenate"; "Rejuvenate crossover finished");
    Ok(vec![child_1, child_2])
}
