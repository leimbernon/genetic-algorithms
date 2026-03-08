use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
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
pub fn insertion_mutation<U: ChromosomeT>(individual: &mut U) -> Result<(), GaError> {
    let len = individual.dna().len();
    if len < 2 {
        debug!(target="mutation_events", method="insertion"; "DNA length < 2, skipping insertion mutation");
        return Ok(());
    }

    let mut rng = rand::rng();

    // Pick two distinct random positions
    let i = rng.random_range(0..len);
    let mut j = rng.random_range(0..len);
    while j == i {
        j = rng.random_range(0..len);
    }

    debug!(target="mutation_events", method="insertion"; "Starting insertion mutation: remove gene at {} and insert near {}", j, i);

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

    debug!(target="mutation_events", method="insertion"; "Insertion mutation finished");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::genotypes::Binary as BinaryGenotype;
    use crate::traits::GeneT;
    use std::borrow::Cow;
    use std::collections::HashSet;

    fn build_binary_chromosome(n: usize) -> BinaryChromosome {
        let mut c = BinaryChromosome::new();
        let dna: Vec<_> = (0..n)
            .map(|i| BinaryGenotype {
                id: i as i32,
                value: i % 2 == 0,
            })
            .collect();
        c.set_dna(Cow::Owned(dna));
        c
    }

    #[test]
    fn insertion_mutation_preserves_all_gene_ids() {
        let mut c = build_binary_chromosome(10);
        let before_ids: HashSet<i32> = c.dna().iter().map(|g| g.id()).collect();

        for _ in 0..100 {
            insertion_mutation(&mut c).unwrap();
            let after_ids: HashSet<i32> = c.dna().iter().map(|g| g.id()).collect();
            assert_eq!(
                before_ids, after_ids,
                "Gene IDs changed after insertion mutation"
            );
            assert_eq!(
                c.dna().len(),
                10,
                "DNA length changed after insertion mutation"
            );
        }
    }

    #[test]
    fn insertion_mutation_empty_dna_does_nothing() {
        let mut c = BinaryChromosome::new();
        let result = insertion_mutation(&mut c);
        assert!(result.is_ok());
        assert_eq!(c.dna().len(), 0);
    }

    #[test]
    fn insertion_mutation_single_gene_does_nothing() {
        let mut c = BinaryChromosome::new();
        let dna = vec![BinaryGenotype { id: 0, value: true }];
        c.set_dna(Cow::Owned(dna));

        let result = insertion_mutation(&mut c);
        assert!(result.is_ok());
        assert_eq!(c.dna().len(), 1);
        assert_eq!(c.dna()[0].id(), 0);
    }

    #[test]
    fn insertion_mutation_preserves_dna_length() {
        let mut c = build_binary_chromosome(20);
        for _ in 0..200 {
            insertion_mutation(&mut c).unwrap();
            assert_eq!(c.dna().len(), 20, "DNA length changed");
        }
    }

    #[test]
    fn insertion_mutation_can_change_order() {
        let mut c = build_binary_chromosome(10);
        let mut changed = false;
        for _ in 0..200 {
            let before: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
            insertion_mutation(&mut c).unwrap();
            let after: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
            if before != after {
                changed = true;
                break;
            }
        }
        assert!(
            changed,
            "Insertion mutation did not change gene order after 200 attempts"
        );
    }

    #[test]
    fn insertion_mutation_two_genes_swaps_them() {
        let mut c = build_binary_chromosome(2);
        let original: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
        let mut swapped = false;
        for _ in 0..50 {
            insertion_mutation(&mut c).unwrap();
            let current: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
            if current != original {
                swapped = true;
            }
            // Always preserves gene set
            let ids: HashSet<i32> = current.into_iter().collect();
            let orig_ids: HashSet<i32> = original.iter().cloned().collect();
            assert_eq!(ids, orig_ids);
        }
        assert!(swapped, "Two-gene chromosome was never reordered");
    }
}
