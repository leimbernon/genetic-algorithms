//! Tests for `MultiGroupPmx` crossover — GEN-04: verifies that PMX is applied
//! within each permutation group independently, with no gene crossing group boundaries.

use genetic_algorithms::chromosomes::MultiUniqueChromosome;
use genetic_algorithms::genotypes::UniqueGenotype;
use genetic_algorithms::initializers::unique_random_initialization;
use genetic_algorithms::operations::crossover::multi_group_pmx::multi_group_pmx;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;
use std::collections::HashSet;

// -- Helper: build a MultiUniqueChromosome with known groups and DNA ----------

/// Build a MultiUniqueChromosome with groups of sizes [3, 3, 2] and fixed DNA.
///
/// Parent 1 DNA for groups [0,1,2], [10,20,30], [100,200]:
///   Group 0: values [0, 1, 2]    (identity permutation)
///   Group 1: values [10, 20, 30] (identity permutation)
///   Group 2: values [100, 200]   (identity permutation)
fn make_parent_1() -> MultiUniqueChromosome<i32> {
    let mut c = MultiUniqueChromosome::<i32>::new(vec![
        vec![0, 1, 2],
        vec![10, 20, 30],
        vec![100, 200],
    ]);
    // DNA: the identity permutation for each group (ids 0..7)
    let genes = vec![
        UniqueGenotype::new(0, 0),
        UniqueGenotype::new(1, 1),
        UniqueGenotype::new(2, 2),
        UniqueGenotype::new(3, 10),
        UniqueGenotype::new(4, 20),
        UniqueGenotype::new(5, 30),
        UniqueGenotype::new(6, 100),
        UniqueGenotype::new(7, 200),
    ];
    c.set_dna(Cow::Owned(genes));
    c
}

/// Build a MultiUniqueChromosome with the same groups but reversed permutations.
///
/// Parent 2 DNA:
///   Group 0: values [2, 1, 0]    (reversed)
///   Group 1: values [30, 20, 10] (reversed)
///   Group 2: values [200, 100]   (reversed)
fn make_parent_2() -> MultiUniqueChromosome<i32> {
    let mut c = MultiUniqueChromosome::<i32>::new(vec![
        vec![0, 1, 2],
        vec![10, 20, 30],
        vec![100, 200],
    ]);
    let genes = vec![
        UniqueGenotype::new(2, 2),
        UniqueGenotype::new(1, 1),
        UniqueGenotype::new(0, 0),
        UniqueGenotype::new(5, 30),
        UniqueGenotype::new(4, 20),
        UniqueGenotype::new(3, 10),
        UniqueGenotype::new(7, 200),
        UniqueGenotype::new(6, 100),
    ];
    c.set_dna(Cow::Owned(genes));
    c
}

// -- Helpers for multiset comparison ----------------------------------------

fn gene_value_set(slice: &[UniqueGenotype<i32>]) -> HashSet<i32> {
    slice.iter().map(|g| g.value).collect()
}

fn gene_id_set(slice: &[UniqueGenotype<i32>]) -> HashSet<i32> {
    slice.iter().map(|g| g.id).collect()
}

// -- Tests ------------------------------------------------------------------

/// PMX produces two children, each with the same length as the parents.
#[test]
fn multi_group_pmx_produces_two_children_correct_length() {
    let p1 = make_parent_1();
    let p2 = make_parent_2();
    let children = multi_group_pmx(&p1, &p2).expect("multi_group_pmx should succeed");
    assert_eq!(children.len(), 2, "PMX should produce exactly 2 children");
    assert_eq!(children[0].dna().len(), 8, "child 0 should have 8 genes");
    assert_eq!(children[1].dna().len(), 8, "child 1 should have 8 genes");
}

/// Each child's gene multiset within group 0 (indices 0..=2) matches the parent's multiset.
/// This proves no gene crossed the group-0 boundary.
#[test]
fn multi_group_pmx_group0_gene_multiset_preserved() {
    let p1 = make_parent_1();
    let p2 = make_parent_2();
    let children = multi_group_pmx(&p1, &p2).expect("multi_group_pmx should succeed");

    // Group 0 alphabet values: {0, 1, 2}
    let expected_values: HashSet<i32> = [0, 1, 2].into_iter().collect();

    for (i, child) in children.iter().enumerate() {
        let child_group0_values = gene_value_set(&child.dna()[0..=2]);
        assert_eq!(
            child_group0_values, expected_values,
            "child {} group-0 gene values should be {{0, 1, 2}}; got {:?}",
            i, child_group0_values
        );
    }
}

/// Each child's gene multiset within group 1 (indices 3..=5) matches the parent's multiset.
#[test]
fn multi_group_pmx_group1_gene_multiset_preserved() {
    let p1 = make_parent_1();
    let p2 = make_parent_2();
    let children = multi_group_pmx(&p1, &p2).expect("multi_group_pmx should succeed");

    let expected_values: HashSet<i32> = [10, 20, 30].into_iter().collect();

    for (i, child) in children.iter().enumerate() {
        let child_group1_values = gene_value_set(&child.dna()[3..=5]);
        assert_eq!(
            child_group1_values, expected_values,
            "child {} group-1 gene values should be {{10, 20, 30}}; got {:?}",
            i, child_group1_values
        );
    }
}

/// Each child's gene multiset within group 2 (indices 6..=7) matches the parent's multiset.
#[test]
fn multi_group_pmx_group2_gene_multiset_preserved() {
    let p1 = make_parent_1();
    let p2 = make_parent_2();
    let children = multi_group_pmx(&p1, &p2).expect("multi_group_pmx should succeed");

    let expected_values: HashSet<i32> = [100, 200].into_iter().collect();

    for (i, child) in children.iter().enumerate() {
        let child_group2_values = gene_value_set(&child.dna()[6..=7]);
        assert_eq!(
            child_group2_values, expected_values,
            "child {} group-2 gene values should be {{100, 200}}; got {:?}",
            i, child_group2_values
        );
    }
}

/// No gene from group 0 appears in group 1 or group 2 of any child (boundary test).
#[test]
fn multi_group_pmx_no_gene_crosses_group_boundary() {
    let p1 = make_parent_1();
    let p2 = make_parent_2();
    let children = multi_group_pmx(&p1, &p2).expect("multi_group_pmx should succeed");

    let group0_values: HashSet<i32> = [0, 1, 2].into_iter().collect();
    let group1_values: HashSet<i32> = [10, 20, 30].into_iter().collect();
    let group2_values: HashSet<i32> = [100, 200].into_iter().collect();

    for (i, child) in children.iter().enumerate() {
        let dna = child.dna();
        // Group 0 genes must only have values from group0_values
        for g in &dna[0..=2] {
            assert!(
                group0_values.contains(&g.value),
                "child {} group-0 gene value {} is out of group-0 alphabet",
                i, g.value
            );
        }
        // Group 1 genes must only have values from group1_values
        for g in &dna[3..=5] {
            assert!(
                group1_values.contains(&g.value),
                "child {} group-1 gene value {} is out of group-1 alphabet",
                i, g.value
            );
        }
        // Group 2 genes must only have values from group2_values
        for g in &dna[6..=7] {
            assert!(
                group2_values.contains(&g.value),
                "child {} group-2 gene value {} is out of group-2 alphabet",
                i, g.value
            );
        }
    }
}

/// gene IDs within each group slice are unique in all children.
#[test]
fn multi_group_pmx_children_have_unique_gene_ids_per_group() {
    let p1 = make_parent_1();
    let p2 = make_parent_2();
    let children = multi_group_pmx(&p1, &p2).expect("multi_group_pmx should succeed");

    for (ci, child) in children.iter().enumerate() {
        let dna = child.dna();
        // Group 0 ids
        let ids_g0 = gene_id_set(&dna[0..=2]);
        assert_eq!(ids_g0.len(), 3, "child {} group-0 should have 3 unique gene ids", ci);
        // Group 1 ids
        let ids_g1 = gene_id_set(&dna[3..=5]);
        assert_eq!(ids_g1.len(), 3, "child {} group-1 should have 3 unique gene ids", ci);
        // Group 2 ids
        let ids_g2 = gene_id_set(&dna[6..=7]);
        assert_eq!(ids_g2.len(), 2, "child {} group-2 should have 2 unique gene ids", ci);
    }
}

/// multi_group_pmx on a chromosome with empty groups returns Err(ConfigurationError).
#[test]
fn multi_group_pmx_empty_groups_returns_error() {
    let p1 = MultiUniqueChromosome::<i32>::default(); // empty groups
    let p2 = MultiUniqueChromosome::<i32>::default();
    let result = multi_group_pmx(&p1, &p2);
    assert!(
        matches!(result, Err(genetic_algorithms::error::GaError::ConfigurationError(_))),
        "Expected ConfigurationError for empty-groups PMX; got {:?}",
        result
    );
}

/// multi_group_pmx with mismatched parent group structures returns Err(ConfigurationError).
#[test]
fn multi_group_pmx_mismatched_group_structures_returns_error() {
    // Parent 1: two groups
    let mut p1 = MultiUniqueChromosome::<i32>::new(vec![vec![0, 1, 2], vec![10, 20]]);
    p1.set_dna(Cow::Owned(vec![
        UniqueGenotype::new(0, 0),
        UniqueGenotype::new(1, 1),
        UniqueGenotype::new(2, 2),
        UniqueGenotype::new(3, 10),
        UniqueGenotype::new(4, 20),
    ]));
    // Parent 2: three groups (structure mismatch)
    let mut p2 = MultiUniqueChromosome::<i32>::new(vec![vec![0, 1], vec![10, 20], vec![100]]);
    p2.set_dna(Cow::Owned(vec![
        UniqueGenotype::new(0, 0),
        UniqueGenotype::new(1, 1),
        UniqueGenotype::new(2, 10),
        UniqueGenotype::new(3, 20),
        UniqueGenotype::new(4, 100),
    ]));
    let result = multi_group_pmx(&p1, &p2);
    assert!(
        matches!(result, Err(genetic_algorithms::error::GaError::ConfigurationError(_))),
        "Expected ConfigurationError for mismatched group structures; got {:?}",
        result
    );
}

/// Stress test: 100 runs with random DNA still preserve group membership.
#[test]
fn multi_group_pmx_stress_100_runs_group_membership_preserved() {
    let alphabet_g0: Vec<i32> = (0..5).collect();
    let alphabet_g1: Vec<i32> = (100..105).collect();
    let alphabet_g2: Vec<i32> = (200..202).collect();

    for _ in 0..100 {
        let mut p1 = MultiUniqueChromosome::<i32>::new(vec![
            alphabet_g0.clone(),
            alphabet_g1.clone(),
            alphabet_g2.clone(),
        ]);
        let mut p2 = MultiUniqueChromosome::<i32>::new(vec![
            alphabet_g0.clone(),
            alphabet_g1.clone(),
            alphabet_g2.clone(),
        ]);
        // Initialize DNA as shuffled permutations using unique_random_initialization
        // IDs from unique_random_initialization are the alphabet indices (0..n).
        // Groups may share the same ID space — that's fine, as multi_group_pmx processes
        // each group slice independently, so IDs only need to be unique within a group.
        let mut all_dna_p1 = unique_random_initialization(&alphabet_g0);
        all_dna_p1.extend(unique_random_initialization(&alphabet_g1));
        all_dna_p1.extend(unique_random_initialization(&alphabet_g2));

        let mut all_dna_p2 = unique_random_initialization(&alphabet_g0);
        all_dna_p2.extend(unique_random_initialization(&alphabet_g1));
        all_dna_p2.extend(unique_random_initialization(&alphabet_g2));

        p1.set_dna(Cow::Owned(all_dna_p1));
        p2.set_dna(Cow::Owned(all_dna_p2));

        let children = multi_group_pmx(&p1, &p2).expect("multi_group_pmx should succeed");

        let set_g0: HashSet<i32> = alphabet_g0.iter().cloned().collect();
        let set_g1: HashSet<i32> = alphabet_g1.iter().cloned().collect();
        let set_g2: HashSet<i32> = alphabet_g2.iter().cloned().collect();

        for child in &children {
            let dna = child.dna();
            // Group 0: indices 0..4
            let vals_g0 = gene_value_set(&dna[0..5]);
            assert_eq!(vals_g0, set_g0, "group-0 multiset mismatch in child");
            // Group 1: indices 5..9
            let vals_g1 = gene_value_set(&dna[5..10]);
            assert_eq!(vals_g1, set_g1, "group-1 multiset mismatch in child");
            // Group 2: indices 10..11
            let vals_g2 = gene_value_set(&dna[10..12]);
            assert_eq!(vals_g2, set_g2, "group-2 multiset mismatch in child");
        }
    }
}
