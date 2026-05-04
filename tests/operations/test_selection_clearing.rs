//! Tests for the Clearing selection operator.

#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{fitness::FitnessFnWrapper, operations::selection::clearing::clearing_selection};

fn make_chromosome(fitness: f64, dna: Vec<Gene>) -> Chromosome {
    Chromosome {
        dna,
        fitness,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    }
}

fn gene(id: i32) -> Gene {
    Gene { id }
}

/// Build a population of N individuals with distinct fitnesses.
fn pop_distinct(fitnesses: &[f64]) -> Vec<Chromosome> {
    fitnesses
        .iter()
        .map(|&f| make_chromosome(f, vec![gene(0)]))
        .collect()
}

// ---- Basic correctness ----

#[test]
fn test_clearing_returns_pairs_of_valid_indices() {
    // Six individuals with well-separated fitness values; niche_radius=0.5
    // means only individuals within 0.5 fitness units of a winner are cleared.
    let pop = pop_distinct(&[10.0, 9.0, 5.0, 4.5, 0.0, 0.3]);
    let pairs = clearing_selection(&pop, 0.5);

    for (a, b) in &pairs {
        assert!(*a < pop.len(), "Index {} out of bounds", a);
        assert!(*b < pop.len(), "Index {} out of bounds", b);
        assert_ne!(a, b, "Self-pairing not allowed");
    }
}

#[test]
fn test_clearing_returns_no_pairs_for_single_chromosome() {
    let pop = vec![make_chromosome(1.0, vec![gene(0)])];
    let pairs = clearing_selection(&pop, 0.1);
    assert!(pairs.is_empty(), "Cannot form pairs from a single individual");
}

#[test]
fn test_clearing_returns_no_pairs_for_empty_population() {
    let pop: Vec<Chromosome> = Vec::new();
    let pairs = clearing_selection(&pop, 0.1);
    assert!(pairs.is_empty());
}

// ---- Niche semantics ----

#[test]
fn test_clearing_clears_dominated_individuals_within_radius() {
    // Three individuals: fitness 10.0, 10.05, 5.0
    // With niche_radius=0.1, sorted descending: 10.05 (idx 1), 10.0 (idx 0), 5.0 (idx 2).
    //   - Winner of the first niche: index 1 (fitness 10.05, highest)
    //   - Index 0 (10.0) is within 0.1 of 10.05 -> cleared
    //   - Index 2 (5.0) is its own niche winner -> eligible
    // Eligible pool: [index 1 (10.05), index 2 (5.0)] -> 1 pair
    let pop = vec![
        make_chromosome(10.0, vec![gene(0)]),
        make_chromosome(10.05, vec![gene(1)]),
        make_chromosome(5.0, vec![gene(2)]),
    ];
    let pairs = clearing_selection(&pop, 0.1);
    assert_eq!(pairs.len(), 1);
    // Both indices must be from the eligible pool (1 and 2), not index 0
    for (a, b) in &pairs {
        assert_ne!(*a, 0, "Cleared individual (index 0) should not appear in pairs");
        assert_ne!(*b, 0, "Cleared individual (index 0) should not appear in pairs");
    }
}

#[test]
fn test_clearing_preserves_one_winner_per_niche() {
    // Population: two tight clusters with fitness 10.0 & 10.05 (one niche, winner=10.0)
    // and 5.0 & 4.97 (second niche, winner=5.0), niche_radius=0.1.
    // Eligible: index of 10.0 winner + index of 5.0 winner = 2 eligible -> 1 pair.
    let pop = vec![
        make_chromosome(10.0, vec![gene(0)]),
        make_chromosome(10.05, vec![gene(1)]),
        make_chromosome(5.0, vec![gene(2)]),
        make_chromosome(4.97, vec![gene(3)]),
    ];
    let pairs = clearing_selection(&pop, 0.1);
    assert_eq!(pairs.len(), 1);
}

#[test]
fn test_clearing_with_zero_radius_keeps_all_eligible() {
    // With niche_radius=0.0 no two distinct individuals can be in the same niche
    // (distance is always > 0 unless they have identical fitness). All 4 survive -> 2 pairs.
    let pop = pop_distinct(&[4.0, 3.0, 2.0, 1.0]);
    let pairs = clearing_selection(&pop, 0.0);
    assert_eq!(pairs.len(), 2);
}

#[test]
fn test_clearing_with_large_radius_keeps_only_one_winner() {
    // niche_radius = 100.0 clears everyone within 100 fitness units of the best.
    // Population fitness: 10, 5, 1 — only winner (10) is eligible -> no pairs.
    let pop = pop_distinct(&[10.0, 5.0, 1.0]);
    let pairs = clearing_selection(&pop, 100.0);
    // Only one eligible individual -> 0 pairs
    assert!(pairs.is_empty());
}

// ---- Factory / configuration dispatch ----

#[test]
fn test_clearing_via_factory_respects_niche_radius() {
    use genetic_algorithms::{
        configuration::SelectionConfiguration, operations::selection, operations::Selection,
    };

    let pop = vec![
        make_chromosome(10.0, vec![gene(0)]),
        make_chromosome(10.05, vec![gene(1)]),
        make_chromosome(5.0, vec![gene(2)]),
    ];
    let config = SelectionConfiguration {
        method: Selection::Clearing,
        number_of_couples: 3,
        niche_radius: 0.1,
        ..Default::default()
    };

    let result = selection::factory(&pop, config, 1);
    assert!(result.is_ok());
    let pairs = result.unwrap();
    // With niche_radius=0.1, sorted desc: 10.05 (idx 1) wins, clears 10.0 (idx 0).
    // 5.0 (idx 2) is its own winner. Eligible = {idx 1, idx 2} -> 1 pair.
    assert_eq!(pairs.len(), 1);
    for (a, b) in &pairs {
        assert_ne!(*a, 0, "Cleared individual (index 0) should not appear");
        assert_ne!(*b, 0, "Cleared individual (index 0) should not appear");
    }
}

#[test]
fn test_clearing_via_selection_enum() {
    use genetic_algorithms::{operations::Selection, traits::SelectionOperator};

    let pop = pop_distinct(&[10.0, 9.0, 5.0, 4.0]);
    // SelectionOperator uses the default niche_radius (0.1); all fitnesses are
    // well-separated so all 4 are eligible -> 2 pairs.
    let pairs = Selection::Clearing.select(&pop, 4, 1);
    assert_eq!(pairs.len(), 2);
}
