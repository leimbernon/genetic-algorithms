//! Cycle crossover implementation.

use crate::error::GaError;
use crate::traits::{LinearChromosome, GeneT};
use log::{debug, trace};
use std::borrow::Cow;
use std::collections::HashMap;

/// Cycle crossover: identifies positional cycles between parents and alternates
/// which parent contributes each cycle to the children.
///
/// Preserves the absolute position of every gene (each gene ends up at the
/// same index it occupied in one of the parents), making it well-suited for
/// permutation-based problems.
///
/// # Errors
///
/// Returns `Err(GaError::CrossoverError)` if parents have different DNA lengths.
pub fn cycle<U: LinearChromosome>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
    let dna_len = parent_1.dna().len();

    // Check if parents have the same length DNA
    if dna_len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parent 1 and parent 2 must have the same dna length. Parent 1 has a length of {} and parent 2 has a length of {}",
            parent_1.dna().len(), parent_2.dna().len())));
    }

    // Pre-build gene-id → index map for parent_1 so each lookup is O(1)
    // instead of O(N) linear scan.
    let p1_id_to_idx: HashMap<i32, usize> = parent_1
        .dna()
        .iter()
        .enumerate()
        .map(|(i, gene)| (gene.id(), i))
        .collect();

    let mut child_1_dna = parent_1.dna().to_vec();
    let mut child_2_dna = parent_2.dna().to_vec();
    let mut visited = vec![false; dna_len];

    debug!(target="crossover_events", method="cycle_crossover"; "Starting the cycle crossover");
    let mut cycle_count: usize = 0;
    for start in 0..dna_len {
        if visited[start] {
            continue;
        }

        let mut idx = start;
        let use_parent_1 = cycle_count % 2 == 0;

        while !visited[idx] {
            trace!(target="crossover_events", method="cycle_crossover"; "Index {} not yet visited", idx);
            visited[idx] = true;

            if use_parent_1 {
                child_1_dna[idx] = parent_1.dna()[idx].clone();
                child_2_dna[idx] = parent_2.dna()[idx].clone();
            } else {
                child_1_dna[idx] = parent_2.dna()[idx].clone();
                child_2_dna[idx] = parent_1.dna()[idx].clone();
            }

            let next_gene_id = parent_2.dna()[idx].id();
            trace!(target="crossover_events", method="cycle_crossover"; "Next gene id {}", next_gene_id);
            match p1_id_to_idx.get(&next_gene_id) {
                Some(&pos) => idx = pos,
                None => {
                    return Err(GaError::CrossoverError(format!(
                        "Cycle crossover failed: gene id {} from parent 2 not found in parent 1",
                        next_gene_id
                    )))
                }
            }
        }
        cycle_count += 1;
    }

    let mut child_1 = U::new();
    let mut child_2 = U::new();
    child_1.set_dna(Cow::Owned(child_1_dna));
    child_2.set_dna(Cow::Owned(child_2_dna));
    debug!(target="crossover_events", method="cycle_crossover"; "Cycle crossover finished");

    Ok(vec![child_1, child_2])
}
