//! Clone crossover implementation.

use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;

/// Clone crossover: copies parents directly as offspring without any genetic exchange.
///
/// Given parents `P1` and `P2`, produces children that are exact clones:
/// child_1 = clone of P1, child_2 = clone of P2.
///
/// # Use cases
///
/// - Mutation-only evolutionary strategies where crossover should be a no-op.
/// - Baseline comparisons in experiments to isolate the effect of crossover.
///
/// # Errors
///
/// Returns `Err(GaError::CrossoverError)` if parents have different DNA lengths.
pub fn clone_crossover<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
    if parent_1.dna().len() != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            parent_1.dna().len(),
            parent_2.dna().len()
        )));
    }

    debug!(target="crossover_events", method="clone"; "Starting clone crossover");

    let child_1 = parent_1.clone();
    let child_2 = parent_2.clone();

    debug!(target="crossover_events", method="clone"; "Clone crossover finished");
    Ok(vec![child_1, child_2])
}
