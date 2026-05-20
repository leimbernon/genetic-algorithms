//! Multi-point crossover implementation.

use crate::error::GaError;
use crate::traits::LinearChromosome;
use log::{debug, trace};
use rand::Rng;

/// Multi-point crossover: alternates segments between parents at N random cut points.
///
/// # Errors
///
/// Returns `Err(GaError::CrossoverError)` if parents have different DNA lengths
/// or if `crossover_number_of_points` exceeds DNA length.
pub fn multipoint<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
    crossover_number_of_points: usize,
) -> Result<Vec<U>, GaError> {
    //Before doing the operation, we check that the dna in parent 1 has the same length of the dna in parent 2
    if parent_1.dna().len() != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parent 1 and parent 2 must have the same dna length. Parent 1 has a length of {} and parent 2 has a length of {}",
            parent_1.dna().len(), parent_2.dna().len())));
    }

    let mut child_1 = U::new();
    let mut child_2 = U::new();

    let mut dna_child_1 = Vec::new();
    let mut dna_child_2 = Vec::new();
    debug!(target="crossover_events", method="multipoint_crossover"; "Starting the  multipoint crossover");

    let dna_len = parent_1.dna().len();

    // Clamp the number of crossover points: at most dna_len - 1
    let n = {
        if crossover_number_of_points >= dna_len {
            dna_len - 1
        } else {
            crossover_number_of_points
        }
    };
    trace!(target="crossover_events", method="multipoint_crossover"; "Number of crossover points {}", n);

    // Generate N random, sorted, unique crossover point indices within 1..dna_len
    // Using Fisher-Yates partial shuffle on the range to pick N unique values
    let mut candidates: Vec<usize> = (1..dna_len).collect();
    let mut rng = crate::rng::make_rng();
    for i in 0..n {
        let j = rng.random_range(i..candidates.len());
        candidates.swap(i, j);
    }
    let mut crossover_points: Vec<usize> = candidates[..n].to_vec();
    crossover_points.sort();
    trace!(target="crossover_events", method="multipoint_crossover"; "Crossover points {:?}", crossover_points);

    // Walk through the DNA, alternating parent source at each crossover point
    let mut crossed = false;
    let mut cp_idx = 0;

    for gn in 0..dna_len {
        // Check if we've reached the next crossover point
        if cp_idx < crossover_points.len() && gn == crossover_points[cp_idx] {
            crossed = !crossed;
            cp_idx += 1;
        }

        if !crossed {
            dna_child_1.push(parent_1.dna().get(gn).cloned().unwrap());
            dna_child_2.push(parent_2.dna().get(gn).cloned().unwrap());
        } else {
            dna_child_1.push(parent_2.dna().get(gn).cloned().unwrap());
            dna_child_2.push(parent_1.dna().get(gn).cloned().unwrap());
        }
    }

    //Sets the dna into the children and return them
    child_1.set_dna(std::borrow::Cow::Owned(dna_child_1));
    child_2.set_dna(std::borrow::Cow::Owned(dna_child_2));
    debug!(target="crossover_events", method="multipoint_crossover"; "Multipoint crossover finished");

    Ok(vec![child_1, child_2])
}
