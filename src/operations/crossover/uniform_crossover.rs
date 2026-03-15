//! Uniform crossover implementation.

use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;
use std::borrow::Cow;

/// Uniform crossover: each gene is independently chosen from either parent.
///
/// For every gene position a coin is flipped — the child gets the gene from
/// parent 1 or parent 2 with equal probability.
///
/// # Errors
///
/// Returns `Err(GaError::CrossoverError)` if parents have different DNA lengths.
pub fn uniform<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
    //Before doing the operation, we check that the dna in the parent 1 has the same length of the dna in the parent 2
    if parent_1.dna().len() != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "parent 1 and parent 2 must have the same dna length. Currently parent 1 has a length of {} and parent 2 {}",
            parent_1.dna().len(), parent_2.dna().len())));
    }

    let mut rng = crate::rng::make_rng();

    //Creation of the children DNA using reserve + push to avoid redundant initialization
    let len = parent_1.dna().len();
    let mut dna_child_1 = Vec::with_capacity(len);
    let mut dna_child_2 = Vec::with_capacity(len);
    debug!(target="crossover_events", method="uniform"; "Starting the  uniform crossover");

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();

    for i in 0..len {
        let crossover = rng.random_range(0..2);
        trace!(target="crossover_events", method="uniform"; "Random crossover number {}", crossover);

        if crossover == 0 {
            dna_child_1.push(parent_1.dna().get(i).cloned().unwrap());
            dna_child_2.push(parent_2.dna().get(i).cloned().unwrap());
        } else {
            dna_child_1.push(parent_2.dna().get(i).cloned().unwrap());
            dna_child_2.push(parent_1.dna().get(i).cloned().unwrap());
        }
    }

    //Move the DNA into children to avoid extra clones
    child_1.set_dna(Cow::Owned(dna_child_1));
    child_2.set_dna(Cow::Owned(dna_child_2));
    debug!(target="crossover_events", method="uniform"; "Uniform crossover finished");

    Ok(vec![child_1, child_2])
}
