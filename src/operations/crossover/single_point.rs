use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::borrow::Cow;

/// Single-point crossover: splits parents at one random point and swaps tails.
///
/// Given parents `[A B C D E]` and `[V W X Y Z]` with crossover point 2,
/// produces children `[A B X Y Z]` and `[V W C D E]`.
///
/// # Errors
///
/// Returns `Err(GaError::CrossoverError)` if parents have different DNA lengths.
pub fn single_point<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }
    if len < 2 {
        return Err(GaError::CrossoverError(
            "DNA length must be at least 2 for single-point crossover".to_string(),
        ));
    }

    debug!(target="crossover_events", method="single_point"; "Starting single-point crossover");
    let mut rng = rand::rng();
    // Crossover point: between 1 and len-1 (exclusive bounds ensure both parts are non-empty)
    let point = rng.random_range(1..len);

    let dna1 = parent_1.dna();
    let dna2 = parent_2.dna();

    let mut child_dna_1 = Vec::with_capacity(len);
    let mut child_dna_2 = Vec::with_capacity(len);

    child_dna_1.extend_from_slice(&dna1[..point]);
    child_dna_1.extend_from_slice(&dna2[point..]);

    child_dna_2.extend_from_slice(&dna2[..point]);
    child_dna_2.extend_from_slice(&dna1[point..]);

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    debug!(target="crossover_events", method="single_point"; "Single-point crossover finished at point {}", point);
    Ok(vec![child_1, child_2])
}
