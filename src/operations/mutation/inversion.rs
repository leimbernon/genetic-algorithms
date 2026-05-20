//! Inversion mutation operator.
//!
//! Reverses the order of genes between two randomly chosen positions in the
//! chromosome. Inversion mutation is especially useful for permutation
//! encodings (e.g., TSP) because it preserves the set of alleles while
//! changing their relative order.

pub(crate) use crate::traits::LinearChromosome;
use log::{debug, trace};
use rand::Rng;

/// Inversion mutation: reverses the sub-sequence of genes between two
/// randomly selected positions.
///
/// Two indices are chosen at random; the genes in the inclusive range
/// `[lower, upper]` are then reversed in place. If the DNA has fewer than
/// 2 genes the function is a no-op.
pub fn inversion<U: LinearChromosome>(individual: &mut U) {
    // Starting the inversion mutation and obtaining two random indices
    debug!(target="mutation_events", method="inversion"; "Starting the inversion mutation");
    if individual.dna().len() < 2 {
        return;
    }
    let mut rng = crate::rng::make_rng();
    let len = individual.dna().len();

    // Select two distinct random indices
    let (index_1, index_2) = (rng.random_range(0..len), rng.random_range(0..len));
    let (lower_index, higher_index) = if index_1 < index_2 {
        (index_1, index_2)
    } else {
        (index_2, index_1)
    };

    trace!(target="mutation_events", method="inversion"; "Mutation lower index: {}, mutation higher index: {}", lower_index, higher_index);

    // Reverse the sub-sequence in-place
    individual.dna_mut()[lower_index..=higher_index].reverse();

    debug!(target="mutation_events", method="inversion"; "Inversion mutation finished");
}
