use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::operations::mutation::insertion::insertion_mutation;
use genetic_algorithms::traits::{ChromosomeT, GeneT};
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
