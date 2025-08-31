use crate::traits::{ChromosomeT, GeneT};
use log::{trace, debug};
use std::borrow::Cow;

pub fn cycle<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Option<Vec<U>> {
    let dna_len = parent_1.get_dna().len();

    // Check if parents have the same length DNA
    if dna_len != parent_2.get_dna().len() {
        panic!("Parent 1 and parent 2 must have the same dna length. Parent 1 has a length of {} and parent 2 has a length of {}", parent_1.get_dna().len(), parent_2.get_dna().len());
    }

    let mut child_1_dna = parent_1.get_dna().to_vec();
    let mut child_2_dna = parent_2.get_dna().to_vec();
    let mut visited = vec![false; dna_len];

    debug!(target="crossover_events", method="cycle_crossover"; "Starting the cycle crossover");
    for start in 0..dna_len {
        if visited[start] {
            continue;
        }

        let mut idx = start;
        let use_parent_1 = idx % 2 == 0;

        while !visited[idx] {
            trace!(target="crossover_events", method="cycle_crossover"; "Index {} not yet visited", idx);
            visited[idx] = true;

            if use_parent_1 {
                child_1_dna[idx] = parent_1.get_dna()[idx].clone();
                child_2_dna[idx] = parent_2.get_dna()[idx].clone();
            } else {
                child_1_dna[idx] = parent_2.get_dna()[idx].clone();
                child_2_dna[idx] = parent_1.get_dna()[idx].clone();
            }

            let next_gene_id = parent_2.get_dna()[idx].get_id();
            trace!(target="crossover_events", method="cycle_crossover"; "Next gene id {}", next_gene_id);
            idx = parent_1.get_dna().iter().position(|gene| gene.get_id() == next_gene_id)?;
        }
    }

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();
    child_1.set_dna(Cow::Owned(child_1_dna));
    child_2.set_dna(Cow::Owned(child_2_dna));
    debug!(target="crossover_events", method="cycle_crossover"; "Cycle crossover finished");

    Some(vec![child_1, child_2])
}