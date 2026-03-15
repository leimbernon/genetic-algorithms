//! Order crossover (OX) implementation.

use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::collections::HashSet;

/// Order Crossover (OX): preserves the relative order of genes.
///
/// Essential for TSP and permutation problems where gene order matters.
///
/// Algorithm:
/// 1. Select two random crossover points.
/// 2. Copy the segment between the points from parent 1 to child 1.
/// 3. Fill the remaining positions with genes from parent 2, in order, skipping duplicates.
/// 4. Repeat symmetrically for child 2.
pub fn order<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }
    if len < 3 {
        return Err(GaError::CrossoverError(
            "DNA length must be at least 3 for order crossover".to_string(),
        ));
    }

    debug!(target="crossover_events", method="order"; "Starting order crossover (OX)");
    let mut rng = crate::rng::make_rng();

    let mut p1 = rng.random_range(0..len);
    let mut p2 = rng.random_range(0..len);
    while p1 == p2 {
        p2 = rng.random_range(0..len);
    }
    if p1 > p2 {
        std::mem::swap(&mut p1, &mut p2);
    }

    let child_dna_1 = ox_build_child(parent_1.dna(), parent_2.dna(), p1, p2);
    let child_dna_2 = ox_build_child(parent_2.dna(), parent_1.dna(), p1, p2);

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    debug!(target="crossover_events", method="order"; "Order crossover finished with points ({}, {})", p1, p2);
    Ok(vec![child_1, child_2])
}

fn ox_build_child<G: crate::traits::GeneT>(
    donor: &[G],
    filler: &[G],
    p1: usize,
    p2: usize,
) -> Vec<G> {
    let len = donor.len();
    let mut child: Vec<Option<G>> = vec![None; len];

    // Copy the segment from donor
    for i in p1..=p2 {
        child[i] = Some(donor[i].clone());
    }

    // Collect gene IDs in the segment to detect duplicates (O(1) lookup)
    let segment_ids: HashSet<i32> = (p1..=p2).map(|i| donor[i].id()).collect();

    // Collect filler genes not in the segment, starting from position after p2
    let mut filler_genes: Vec<G> = Vec::new();
    let mut pos = (p2 + 1) % len;
    for _ in 0..len {
        if !segment_ids.contains(&filler[pos].id()) {
            filler_genes.push(filler[pos].clone());
        }
        pos = (pos + 1) % len;
    }

    // Fill remaining positions in order, starting after p2
    let mut filler_iter = filler_genes.into_iter();
    let mut child_pos = (p2 + 1) % len;
    for _ in 0..len {
        if child[child_pos].is_none() {
            if let Some(gene) = filler_iter.next() {
                child[child_pos] = Some(gene);
            }
        }
        child_pos = (child_pos + 1) % len;
    }

    child
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            g.unwrap_or_else(|| {
                panic!(
                    "Order crossover: child position {} was not filled. \
                     This indicates non-unique gene IDs in the parents.",
                    i
                )
            })
        })
        .collect()
}
