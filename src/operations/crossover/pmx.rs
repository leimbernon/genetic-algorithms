use crate::error::GaError;
use crate::traits::{ChromosomeT, GeneT};
use log::debug;
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
pub fn pmx<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
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

    debug!(target="crossover_events", method="pmx"; "Starting PMX crossover");

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

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    debug!(target="crossover_events", method="pmx"; "PMX crossover finished with points ({}, {})", start, end);

    Ok(vec![child_1, child_2])
}

/// Builds one PMX child.
///
/// `donor` provides the copied segment; `other` provides the remaining genes
/// and the mapping source.
fn pmx_build_child<G: GeneT>(donor: &[G], other: &[G], start: usize, end: usize) -> Vec<G> {
    let len = donor.len();
    let mut child: Vec<Option<G>> = vec![None; len];

    // Step 1: Copy the segment from the donor parent.
    for i in start..=end {
        child[i] = Some(donor[i].clone());
    }

    // Build a set of gene IDs already placed in the child (the segment).
    let segment_ids: HashMap<i32, usize> = (start..=end).map(|i| (donor[i].id(), i)).collect();

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
            // Find where `mapped_id` sits in `other`.
            let pos_in_other = other
                .iter()
                .position(|g| g.id() == mapped_id)
                .expect("PMX: gene ID not found in other parent — parents must be permutations");

            if pos_in_other < start || pos_in_other > end {
                // This position is outside the segment and free in the child.
                child[pos_in_other] = Some(gene.clone());
                break;
            }

            // The position is inside the segment, so follow the chain one more step.
            mapped_id = donor[pos_in_other].id();
        }
    }

    // Step 3: Fill remaining empty positions from `other`.
    for i in 0..len {
        if child[i].is_none() {
            child[i] = Some(other[i].clone());
        }
    }

    child
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            g.unwrap_or_else(|| {
                panic!(
                    "PMX crossover: child position {} was not filled. \
                     This indicates non-unique gene IDs in the parents.",
                    i
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::genotypes::Binary as BinaryGenotype;
    use std::borrow::Cow;
    use std::collections::HashSet;

    fn make_parents() -> (BinaryChromosome, BinaryChromosome) {
        let mut p1 = BinaryChromosome::new();
        let mut p2 = BinaryChromosome::new();
        p1.set_dna(Cow::Owned(vec![
            BinaryGenotype { id: 1, value: true },
            BinaryGenotype {
                id: 2,
                value: false,
            },
            BinaryGenotype { id: 3, value: true },
            BinaryGenotype {
                id: 4,
                value: false,
            },
            BinaryGenotype { id: 5, value: true },
        ]));
        p2.set_dna(Cow::Owned(vec![
            BinaryGenotype { id: 3, value: true },
            BinaryGenotype {
                id: 5,
                value: false,
            },
            BinaryGenotype { id: 1, value: true },
            BinaryGenotype {
                id: 2,
                value: false,
            },
            BinaryGenotype { id: 4, value: true },
        ]));
        (p1, p2)
    }

    #[test]
    fn pmx_preserves_length() {
        let (p1, p2) = make_parents();
        let children = pmx(&p1, &p2).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].dna().len(), p1.dna().len());
        assert_eq!(children[1].dna().len(), p1.dna().len());
    }

    #[test]
    fn pmx_preserves_all_gene_ids() {
        let (p1, p2) = make_parents();
        let parent_ids: HashSet<i32> = p1.dna().iter().map(|g| g.id()).collect();

        // Run multiple times to exercise different random crossover points.
        for _ in 0..50 {
            let children = pmx(&p1, &p2).unwrap();
            for (idx, child) in children.iter().enumerate() {
                let child_ids: HashSet<i32> = child.dna().iter().map(|g| g.id()).collect();
                assert_eq!(
                    child_ids, parent_ids,
                    "Child {} has IDs {:?}, expected {:?}",
                    idx, child_ids, parent_ids
                );
            }
        }
    }

    #[test]
    fn pmx_error_on_different_lengths() {
        let mut p1 = BinaryChromosome::new();
        let mut p2 = BinaryChromosome::new();
        p1.set_dna(Cow::Owned(vec![
            BinaryGenotype { id: 1, value: true },
            BinaryGenotype {
                id: 2,
                value: false,
            },
        ]));
        p2.set_dna(Cow::Owned(vec![
            BinaryGenotype { id: 1, value: true },
            BinaryGenotype {
                id: 2,
                value: false,
            },
            BinaryGenotype { id: 3, value: true },
        ]));
        let result = pmx(&p1, &p2);
        assert!(result.is_err());
    }

    #[test]
    fn pmx_error_on_length_less_than_two() {
        let mut p1 = BinaryChromosome::new();
        let mut p2 = BinaryChromosome::new();
        p1.set_dna(Cow::Owned(vec![BinaryGenotype { id: 1, value: true }]));
        p2.set_dna(Cow::Owned(vec![BinaryGenotype { id: 1, value: true }]));
        let result = pmx(&p1, &p2);
        assert!(result.is_err());
    }

    #[test]
    fn pmx_identical_parents_produce_identical_children() {
        let (p1, _) = make_parents();
        let p2 = p1.clone();
        let children = pmx(&p1, &p2).unwrap();
        let parent_ids: Vec<i32> = p1.dna().iter().map(|g| g.id()).collect();
        for child in &children {
            let child_ids: Vec<i32> = child.dna().iter().map(|g| g.id()).collect();
            assert_eq!(child_ids, parent_ids);
        }
    }

    #[test]
    fn pmx_no_duplicate_ids_in_children() {
        let (p1, p2) = make_parents();
        for _ in 0..50 {
            let children = pmx(&p1, &p2).unwrap();
            for child in &children {
                let ids: Vec<i32> = child.dna().iter().map(|g| g.id()).collect();
                let unique: HashSet<i32> = ids.iter().copied().collect();
                assert_eq!(
                    ids.len(),
                    unique.len(),
                    "Child has duplicate gene IDs: {:?}",
                    ids
                );
            }
        }
    }
}
