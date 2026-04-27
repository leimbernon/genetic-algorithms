
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::{
    binary_random_initialization, generic_random_initialization,
    generic_random_initialization_without_repetitions, range_random_initialization,
};
use genetic_algorithms::traits::ChromosomeT;

#[test]
fn test_initializers_generic_random_initialization() {
    let binding = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
    ];
    let alleles = binding.as_slice();

    let genes = generic_random_initialization::<Chromosome>(4, Some(alleles), Some(false));
    assert_eq!(genes.len(), 4);
}

#[test]
fn test_initializers_generic_random_initialization_without_repetitions() {
    let binding = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
    ];
    let alleles = binding.as_slice();
    let genes = generic_random_initialization_without_repetitions::<Chromosome>(
        6,
        Some(alleles),
        Some(false),
    );

    //Checks that any allele is repeated
    let mut alleles_ids = Vec::new();

    for gene in genes {
        if !alleles_ids.is_empty() {
            assert!(!alleles_ids.contains(&gene.id));
        }
        alleles_ids.push(gene.id);
    }
}

#[test]
fn test_binary_random_initialization() {
    let genes = binary_random_initialization(100, None, None);
    let mut chromosome = Binary::new();

    use std::borrow::Cow;
    chromosome.set_dna(Cow::Borrowed(genes.as_slice()));
    assert_eq!(100, chromosome.phenotype().len());
}

#[test]
fn test_range_random_initialization() {
    let alleles = vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0)];
    let genes = range_random_initialization(10, Some(&alleles), Some(false));
    assert_eq!(genes.len(), 10);
    for gene in genes {
        assert!(gene.value >= 0.0 && gene.value <= 1.0);
    }
}

// ── Unit tests migrated from src/initializers/list_initializer.rs ────────────

use genetic_algorithms::genotypes::List;
use genetic_algorithms::initializers::{
    list_random_initialization, list_random_initialization_without_repetitions,
};
use std::collections::HashSet;

fn make_list_templates() -> Vec<List<char>> {
    vec![List::new(0, vec!['a', 'b', 'c', 'd'], 'a').unwrap()]
}

// Test: list_random_initialization returns correct length
#[test]
fn list_initializer_returns_correct_length() {
    let templates = make_list_templates();
    let genes = list_random_initialization(5, Some(&templates), None);
    assert_eq!(genes.len(), 5);
}

// Test: each gene's id is within valid range
#[test]
fn list_initializer_ids_in_valid_range() {
    let templates = make_list_templates();
    let genes = list_random_initialization(20, Some(&templates), None);
    for gene in &genes {
        assert!(
            (gene.id as usize) < gene.alleles.len(),
            "gene.id {} out of range (alleles.len() = {})",
            gene.id,
            gene.alleles.len()
        );
        assert!(gene.id >= 0, "gene.id must be non-negative");
    }
}

// Test: value == alleles[id] consistency
#[test]
fn list_initializer_value_consistency() {
    let templates = make_list_templates();
    let genes = list_random_initialization(20, Some(&templates), None);
    for gene in &genes {
        assert_eq!(
            gene.value, gene.alleles[gene.id as usize],
            "value must equal alleles[id]"
        );
    }
}

// Test: allele sets are preserved from templates
#[test]
fn list_initializer_alleles_preserved() {
    let templates = make_list_templates();
    let genes = list_random_initialization(10, Some(&templates), None);
    for gene in &genes {
        assert_eq!(gene.alleles, vec!['a', 'b', 'c', 'd']);
    }
}

// Test: panics on None alleles
#[test]
#[should_panic(expected = "Alleles must be provided for list_random_initialization")]
fn list_initializer_panics_on_none_alleles() {
    list_random_initialization::<char>(5, None, None);
}

// Test: without_repetitions returns correct length
#[test]
fn list_initializer_without_repetitions_correct_length() {
    let templates = make_list_templates();
    let genes = list_random_initialization_without_repetitions(3, Some(&templates), None);
    assert_eq!(genes.len(), 3);
}

// Test: without_repetitions has no duplicate allele indices
#[test]
fn list_initializer_without_repetitions_no_duplicate_ids() {
    let templates = make_list_templates();
    for seed in 0..20u64 {
        genetic_algorithms::rng::set_seed(Some(seed));
        let genes = list_random_initialization_without_repetitions(4, Some(&templates), None);
        let ids: HashSet<i32> = genes.iter().map(|g| g.id).collect();
        assert_eq!(
            ids.len(),
            genes.len(),
            "duplicate allele indices found (seed {})",
            seed
        );
    }
    genetic_algorithms::rng::set_seed(None);
}

// Test: without_repetitions panics when genes_per_chromosome > alleles.len()
#[test]
#[should_panic(
    expected = "genes_per_chromosome exceeds allele set size for without-repetitions initialization"
)]
fn list_initializer_without_repetitions_panics_on_overflow() {
    let templates = make_list_templates(); // alleles.len() == 4
    list_random_initialization_without_repetitions(10, Some(&templates), None);
}

// Test: without_repetitions value consistency
#[test]
fn list_initializer_without_repetitions_value_consistency() {
    let templates = make_list_templates();
    let genes = list_random_initialization_without_repetitions(4, Some(&templates), None);
    for gene in &genes {
        assert_eq!(
            gene.value, gene.alleles[gene.id as usize],
            "value must equal alleles[id]"
        );
    }
}
