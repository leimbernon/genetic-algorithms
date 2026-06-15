//! Insertion mutation operator for permutation-based chromosomes.
//!
//! Removes a gene from one position and re-inserts it adjacent to another,
//! preserving all alleles while altering their relative order. This is a
//! standard operator for permutation encodings such as TSP and scheduling
//! problems.

use crate::error::GaError;
use crate::traits::LinearChromosome;
use rand::Rng;
use std::borrow::Cow;

/// Insertion mutation for permutation-based chromosomes.
///
/// This operator works with any chromosome type implementing `ChromosomeT`.
/// It removes a gene from one randomly chosen position and re-inserts it
/// adjacent to another randomly chosen position, preserving all genes
/// (no duplicates, no losses).
///
/// Insertion mutation is a standard operator for permutation encodings
/// (e.g., TSP, scheduling problems) because it maintains the set of alleles
/// while altering their relative order.
///
/// # Algorithm
///
/// 1. If DNA length < 2, return `Ok(())` (no-op).
/// 2. Pick two random distinct positions `i` and `j`.
/// 3. Remove the gene at position `j`.
/// 4. Insert it at position `i + 1` (or `i` when `i > j`, to account for
///    the index shift caused by the removal).
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
///
/// # Returns
///
/// `Ok(())` on success.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::insertion::insertion_mutation;
/// use genetic_algorithms::chromosomes::Binary;
/// let mut chromosome = Binary::new();
/// let _ = insertion_mutation(&mut chromosome);
/// ```
pub fn insertion_mutation<U: LinearChromosome>(individual: &mut U) -> Result<(), GaError> {
    let len = individual.dna().len();
    if len < 2 {
        crate::log_debug!(target="mutation_events", method="insertion"; "DNA length < 2, skipping insertion mutation");
        return Ok(());
    }

    let mut rng = crate::rng::make_rng();

    // Pick two distinct random positions
    let i = rng.random_range(0..len);
    let mut j = rng.random_range(0..len);
    while j == i {
        j = rng.random_range(0..len);
    }

    crate::log_debug!(target="mutation_events", method="insertion"; "Starting insertion mutation: remove gene at {} and insert near {}", j, i);

    let mut dna = individual.dna().to_vec();

    // Remove the gene at position j
    let gene = dna.remove(j);

    // Determine insertion position.
    // After removal, indices shift if j < i.
    let insert_pos = if j < i {
        // j was removed before i, so i has shifted left by 1.
        // We want to insert after the original position i, which is now at i-1.
        // So we insert at index i (which is after the element now at i-1).
        i
    } else {
        // j was after i, so i is unchanged. Insert after i.
        i + 1
    };

    // Clamp to length (insert_pos could equal dna.len(), which is valid for push)
    let insert_pos = insert_pos.min(dna.len());
    dna.insert(insert_pos, gene);

    individual.set_dna(Cow::Owned(dna));

    crate::log_debug!(target="mutation_events", method="insertion"; "Insertion mutation finished");
    Ok(())
}
