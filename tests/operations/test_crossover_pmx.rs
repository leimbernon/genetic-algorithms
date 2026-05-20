use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::operations::crossover::pmx::pmx;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};
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
