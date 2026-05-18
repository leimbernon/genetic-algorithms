use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::operations::crossover::edge_recombination::erx;
use genetic_algorithms::traits::{ChromosomeT, GeneT};
use std::borrow::Cow;
use std::collections::HashSet;

/// Build a permutation parent from a list of (id, value) pairs.
fn make_chromosome(genes: &[(i32, bool)]) -> BinaryChromosome {
    let mut c = BinaryChromosome::new();
    c.set_dna(Cow::Owned(
        genes
            .iter()
            .map(|(id, value)| BinaryGenotype { id: *id, value: *value })
            .collect(),
    ));
    c
}

/// Make the canonical permutation parents used in most tests.
/// p1 = [1,2,3,4,5], p2 = [3,5,1,2,4]
fn make_permutation_parents() -> (BinaryChromosome, BinaryChromosome) {
    let p1 = make_chromosome(&[(1, true), (2, false), (3, true), (4, false), (5, true)]);
    let p2 = make_chromosome(&[(3, true), (5, false), (1, true), (2, false), (4, true)]);
    (p1, p2)
}

#[test]
fn erx_produces_two_children() {
    let (p1, p2) = make_permutation_parents();
    let result = erx(&p1, &p2).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn erx_preserves_length() {
    let (p1, p2) = make_permutation_parents();
    let children = erx(&p1, &p2).unwrap();
    assert_eq!(children[0].dna().len(), p1.dna().len());
    assert_eq!(children[1].dna().len(), p1.dna().len());
}

#[test]
fn erx_produces_valid_permutations() {
    let (p1, p2) = make_permutation_parents();
    let parent_ids: HashSet<i32> = p1.dna().iter().map(|g| g.id()).collect();

    for _ in 0..50 {
        let children = erx(&p1, &p2).unwrap();
        for (idx, child) in children.iter().enumerate() {
            let child_ids: HashSet<i32> = child.dna().iter().map(|g| g.id()).collect();
            assert_eq!(
                child.dna().len(),
                parent_ids.len(),
                "Child {} has wrong length",
                idx
            );
            assert_eq!(
                child_ids, parent_ids,
                "Child {} has IDs {:?}, expected {:?}",
                idx, child_ids, parent_ids
            );
        }
    }
}

#[test]
fn erx_error_on_different_lengths() {
    let p1 = make_chromosome(&[(1, true), (2, false), (3, true), (4, false), (5, true)]);
    let p2 = make_chromosome(&[(1, true), (2, false), (3, true), (4, false)]);
    let result = erx(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "Expected CrossoverError for different lengths, got {:?}",
        result
    );
}

#[test]
fn erx_error_too_short() {
    // D-07: length < 2 must return CrossoverError
    let p1 = make_chromosome(&[(1, true)]);
    let p2 = make_chromosome(&[(1, true)]);
    let result = erx(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "Expected CrossoverError for length < 2, got {:?}",
        result
    );
}

#[test]
fn erx_error_duplicate_ids() {
    // D-08: parent_1 has two genes with id=1
    let p1 = make_chromosome(&[(1, true), (1, false), (3, true)]);
    let p2 = make_chromosome(&[(1, true), (2, false), (3, true)]);
    let result = erx(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "Expected CrossoverError for duplicate IDs, got {:?}",
        result
    );
}

#[test]
fn erx_fallback_exhausted_neighbors() {
    // D-06: construct two parents with the same gene order so that the union
    // adjacency list is quickly exhausted for any starting gene; the child
    // must still be a valid permutation via the random fallback.
    // p1 = [1,2,3], p2 = [1,2,3] — adjacency for every gene is a subset of
    // its two neighbours, and after placing gene 1 and 2 the adjacency list
    // for gene 3 is already empty (3's neighbours are 1 and 2, both visited).
    let p1 = make_chromosome(&[(1, true), (2, false), (3, true)]);
    let p2 = make_chromosome(&[(1, true), (2, false), (3, true)]);
    let parent_ids: HashSet<i32> = p1.dna().iter().map(|g| g.id()).collect();

    for _ in 0..20 {
        let children = erx(&p1, &p2).unwrap();
        for child in &children {
            let child_ids: HashSet<i32> = child.dna().iter().map(|g| g.id()).collect();
            assert_eq!(child.dna().len(), 3, "Child length must be 3");
            assert_eq!(child_ids, parent_ids, "Child must be a valid permutation");
        }
    }
}
