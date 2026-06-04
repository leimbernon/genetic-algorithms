
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::{
    binary_random_initialization, generic_random_initialization,
    generic_random_initialization_without_repetitions, range_random_initialization,
};
use genetic_algorithms::traits::LinearChromosome;

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

    let genes = generic_random_initialization::<Chromosome>(4, Some(alleles));
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
    let genes = binary_random_initialization(100, None);
    let mut chromosome = Binary::new();

    use std::borrow::Cow;
    chromosome.set_dna(Cow::Borrowed(genes.as_slice()));
    assert_eq!(100, chromosome.phenotype().len());
}

#[test]
fn test_range_random_initialization() {
    let alleles = vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0)];
    let genes = range_random_initialization(10, Some(&alleles));
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
    let genes = list_random_initialization(5, Some(&templates));
    assert_eq!(genes.len(), 5);
}

// Test: each gene's id is within valid range
#[test]
fn list_initializer_ids_in_valid_range() {
    let templates = make_list_templates();
    let genes = list_random_initialization(20, Some(&templates));
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
    let genes = list_random_initialization(20, Some(&templates));
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
    let genes = list_random_initialization(10, Some(&templates));
    for gene in &genes {
        assert_eq!(gene.alleles, vec!['a', 'b', 'c', 'd']);
    }
}

// Test: panics on None alleles
#[test]
#[should_panic(expected = "Alleles must be provided for list_random_initialization")]
fn list_initializer_panics_on_none_alleles() {
    list_random_initialization::<char>(5, None);
}

// Test: without_repetitions returns correct length
#[test]
fn list_initializer_without_repetitions_correct_length() {
    let templates = make_list_templates();
    let genes = list_random_initialization_without_repetitions(3, Some(&templates));
    assert_eq!(genes.len(), 3);
}

// Test: without_repetitions has no duplicate allele indices
#[test]
fn list_initializer_without_repetitions_no_duplicate_ids() {
    let templates = make_list_templates();
    for seed in 0..20u64 {
        genetic_algorithms::rng::set_seed(Some(seed));
        let genes = list_random_initialization_without_repetitions(4, Some(&templates));
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
    list_random_initialization_without_repetitions(10, Some(&templates));
}

// Test: without_repetitions value consistency
#[test]
fn list_initializer_without_repetitions_value_consistency() {
    let templates = make_list_templates();
    let genes = list_random_initialization_without_repetitions(4, Some(&templates));
    for gene in &genes {
        assert_eq!(
            gene.value, gene.alleles[gene.id as usize],
            "value must equal alleles[id]"
        );
    }
}

// ── unique_random_initialization tests ────────────────────────────────────────

use genetic_algorithms::initializers::unique_random_initialization;

/// Empty alphabet returns empty Vec.
#[test]
fn unique_initializer_empty_alphabet() {
    let alphabet: Vec<i32> = vec![];
    let dna = unique_random_initialization(&alphabet);
    assert!(dna.is_empty(), "empty alphabet should return empty vec");
}

/// Non-empty alphabet returns dna of the same length.
#[test]
fn unique_initializer_correct_length() {
    let alphabet = vec![10, 20, 30, 40, 50];
    let dna = unique_random_initialization(&alphabet);
    assert_eq!(dna.len(), alphabet.len(), "result length must equal alphabet length");
}

/// Permutation property: every alphabet element appears exactly once.
#[test]
fn unique_initializer_permutation_property() {
    let alphabet = vec![10, 20, 30, 40, 50];
    let dna = unique_random_initialization(&alphabet);

    // Collect values from dna and sort both for comparison
    let mut result_values: Vec<i32> = dna.iter().map(|g| g.value).collect();
    result_values.sort();

    let mut expected: Vec<i32> = alphabet.clone();
    expected.sort();

    assert_eq!(
        result_values, expected,
        "multiset of values in result must equal multiset of alphabet (permutation property)"
    );
}

// ── multi_range_random_initialization tests ───────────────────────────────────

use genetic_algorithms::initializers::multi_range_random_initialization;

/// Length of result equals bounds.len().
#[test]
fn multi_range_initialization_correct_length() {
    let bounds = vec![(0.0_f64, 1.0); 7];
    let rates = vec![0.1_f64; 7];
    let genes = multi_range_random_initialization(&bounds, &rates);
    assert_eq!(genes.len(), 7, "result length must equal bounds.len()");
}

/// Per-gene bounds enforcement: every gene value is within its own (lo, hi) range.
#[test]
fn multi_range_initialization_per_gene_bounds_enforcement() {
    let bounds = vec![(0.0_f64, 1.0), (10.0, 20.0), (-5.0, -1.0)];
    let rates = vec![0.1, 0.2, 0.3];
    // Run many times to get statistical coverage
    for _ in 0..100 {
        let genes = multi_range_random_initialization(&bounds, &rates);
        assert_eq!(genes.len(), 3);
        for (i, gene) in genes.iter().enumerate() {
            assert!(
                gene.value >= gene.lo && gene.value < gene.hi,
                "Gene {} value {} out of range [{}, {})",
                i, gene.value, gene.lo, gene.hi
            );
            // Also verify the lo/hi fields themselves were set correctly
            assert_eq!(gene.lo, bounds[i].0, "Gene {} lo mismatch", i);
            assert_eq!(gene.hi, bounds[i].1, "Gene {} hi mismatch", i);
        }
    }
}

/// Mutation rate assignment: every gene's mutation_rate matches mutation_rates[i].
#[test]
fn multi_range_initialization_mutation_rate_assignment() {
    let bounds = vec![(0.0_f64, 1.0), (0.0, 1.0), (0.0, 1.0)];
    let rates = vec![0.05_f64, 0.15, 0.30];
    let genes = multi_range_random_initialization(&bounds, &rates);
    for (i, gene) in genes.iter().enumerate() {
        assert_eq!(
            gene.mutation_rate, rates[i],
            "Gene {} mutation_rate should be {}, got {}",
            i, rates[i], gene.mutation_rate
        );
    }
}

/// Short mutation_rates slice: trailing genes get default rate of 0.1.
#[test]
fn multi_range_initialization_short_rates_defaults_to_0_1() {
    let bounds = vec![(0.0_f64, 1.0), (0.0, 1.0), (0.0, 1.0)];
    let rates = vec![0.5_f64]; // only one rate; other two should default to 0.1
    let genes = multi_range_random_initialization(&bounds, &rates);
    assert_eq!(genes[0].mutation_rate, 0.5);
    assert_eq!(genes[1].mutation_rate, 0.1);
    assert_eq!(genes[2].mutation_rate, 0.1);
}

/// Gene ids are sequential starting from 0.
#[test]
fn multi_range_initialization_gene_ids_sequential() {
    let bounds = vec![(0.0_f64, 10.0), (10.0, 20.0), (20.0, 30.0)];
    let rates = vec![0.1; 3];
    let genes = multi_range_random_initialization(&bounds, &rates);
    for (i, gene) in genes.iter().enumerate() {
        assert_eq!(gene.id, i as i32, "Gene {} should have id {}", i, i);
    }
}
