//! Partially Mapped Crossover (PMX) implementation.

use crate::error::GaError;
use crate::traits::{GeneT, LinearChromosome};
use rand::Rng;
use std::borrow::Cow;
use std::collections::HashMap;

/// Partially Mapped Crossover (PMX) for permutation-based chromosomes.
///
/// PMX preserves absolute positions within a randomly selected segment and
/// relative order outside it, always producing valid permutations.
///
/// Algorithm:
/// 1. Select two random crossover points defining a segment `[start..end]`.
/// 2. Copy the segment from parent 1 into child 1 (and from parent 2 into child 2).
/// 3. For each gene in the segment of the other parent that is not already present,
///    follow the mapping chain to find the correct position in the child.
/// 4. Fill remaining positions directly from the other parent.
///
/// Both parents must have the same DNA length (≥ 2) and contain unique gene IDs.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::pmx;
/// use genetic_algorithms::chromosomes::Binary;
/// let parent1 = Binary::new();
/// let parent2 = Binary::new();
/// let _ = pmx(&parent1, &parent2);
/// ```
pub fn pmx<U: LinearChromosome>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
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
            "PMX crossover requires DNA of length >= 2".to_string(),
        ));
    }

    crate::log_debug!(target="crossover_events", method="pmx"; "Starting PMX crossover");

    let mut rng = crate::rng::make_rng();

    let mut start = rng.random_range(0..len);
    let mut end = rng.random_range(0..len);
    while start == end {
        end = rng.random_range(0..len);
    }
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }

    let child_dna_1 = pmx_build_child(parent_1.dna(), parent_2.dna(), start, end);
    let child_dna_2 = pmx_build_child(parent_2.dna(), parent_1.dna(), start, end);

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    crate::log_debug!(target="crossover_events", method="pmx"; "PMX crossover finished with points ({}, {})", start, end);

    Ok(vec![child_1, child_2])
}

/// Builds one PMX child.
///
/// `donor` provides the copied segment; `other` provides the remaining genes
/// and the mapping source.
pub(crate) fn pmx_build_child<G: GeneT>(
    donor: &[G],
    other: &[G],
    start: usize,
    end: usize,
) -> Vec<G> {
    // Pre-fill child from `other`; the segment will be overwritten from `donor`.
    let mut child = other.to_vec();

    // Step 1: Copy the segment from the donor parent (overwrites pre-filled values).
    child[start..=end].clone_from_slice(&donor[start..=end]);

    // Build a set of gene IDs already placed in the child (the donor segment).
    let segment_ids: HashMap<i32, usize> = (start..=end).map(|i| (donor[i].id(), i)).collect();

    // Build a position map for `other`: gene ID → index. O(n) lookup replaces
    // the previous O(n) linear scan inside the chain-following loop.
    let pos_in_other: HashMap<i32, usize> =
        other.iter().enumerate().map(|(i, g)| (g.id(), i)).collect();

    // Step 2: For each position in the segment of `other`, if that gene is not
    // already in the child's segment, follow the mapping chain to find the
    // correct position.
    for i in start..=end {
        let gene = &other[i];
        if segment_ids.contains_key(&gene.id()) {
            // This gene ID is already placed in the child's segment; skip.
            continue;
        }

        // Follow the mapping chain: the gene at donor[i] occupies position i
        // in the child; find where `gene` should go by following the chain.
        let mut mapped_id = donor[i].id();
        loop {
            // O(1) HashMap lookup replaces the previous O(n) linear scan.
            let pos = *pos_in_other
                .get(&mapped_id)
                .expect("PMX: gene ID not found in other parent — parents must be permutations");

            if pos < start || pos > end {
                // This position is outside the segment; the pre-fill already
                // placed other[pos] here, so overwrite it with `gene`.
                child[pos] = gene.clone();
                break;
            }

            // The position is inside the segment, so follow the chain one more step.
            mapped_id = donor[pos].id();
        }
    }

    child
}
