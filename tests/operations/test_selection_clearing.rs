//! Tests for the Clearing selection operator.

#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper, operations::selection::clearing::clearing_selection,
};

fn make_chromosome(fitness: f64, dna: Vec<Gene>) -> Chromosome {
    Chromosome {
        dna,
        fitness,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
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
    let pairs = clearing_selection(&pop, 0.5, 3, 2);

    for group in &pairs {
        assert!(group[0] < pop.len(), "Index {} out of bounds", group[0]);
        assert!(group[1] < pop.len(), "Index {} out of bounds", group[1]);
        assert_ne!(group[0], group[1], "Self-pairing not allowed");
    }
}

#[test]
fn test_clearing_returns_no_pairs_for_single_chromosome() {
    let pop = vec![make_chromosome(1.0, vec![gene(0)])];
    let pairs = clearing_selection(&pop, 0.1, 1, 2);
    assert!(
        pairs.is_empty(),
        "Cannot form pairs from a single individual"
    );
}

#[test]
fn test_clearing_returns_no_pairs_for_empty_population() {
    let pop: Vec<Chromosome> = Vec::new();
    let pairs = clearing_selection(&pop, 0.1, 1, 2);
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
    // Eligible pool: [index 1 (10.05), index 2 (5.0)]. Request 1 pair.
    let pop = vec![
        make_chromosome(10.0, vec![gene(0)]),
        make_chromosome(10.05, vec![gene(1)]),
        make_chromosome(5.0, vec![gene(2)]),
    ];
    let pairs = clearing_selection(&pop, 0.1, 1, 2);
    assert_eq!(pairs.len(), 1);
    // Both indices must be from the eligible pool (1 and 2), not index 0
    for group in &pairs {
        assert_ne!(
            group[0], 0,
            "Cleared individual (index 0) should not appear in pairs"
        );
        assert_ne!(
            group[1], 0,
            "Cleared individual (index 0) should not appear in pairs"
        );
    }
}

#[test]
fn test_clearing_preserves_one_winner_per_niche() {
    // Population: two tight clusters with fitness 10.0 & 10.05 (one niche, winner=10.0)
    // and 5.0 & 4.97 (second niche, winner=5.0), niche_radius=0.1.
    // Eligible: index of 10.0 winner + index of 5.0 winner = 2 eligible -> 1 pair requested.
    let pop = vec![
        make_chromosome(10.0, vec![gene(0)]),
        make_chromosome(10.05, vec![gene(1)]),
        make_chromosome(5.0, vec![gene(2)]),
        make_chromosome(4.97, vec![gene(3)]),
    ];
    let pairs = clearing_selection(&pop, 0.1, 1, 2);
    assert_eq!(pairs.len(), 1);
}

#[test]
fn test_clearing_with_zero_radius_keeps_all_eligible() {
    // With niche_radius=0.0, only individuals with *exactly identical* fitness
    // are cleared (|f_a - f_b| == 0.0 satisfies <= 0.0). All 4 have distinct
    // fitness values, so all survive. Request 2 pairs -> 2 pairs returned.
    let pop = pop_distinct(&[4.0, 3.0, 2.0, 1.0]);
    let pairs = clearing_selection(&pop, 0.0, 2, 2);
    assert_eq!(pairs.len(), 2);
}

#[test]
fn test_clearing_with_large_radius_keeps_only_one_winner() {
    // niche_radius = 100.0 clears everyone within 100 fitness units of the best.
    // Population fitness: 10, 5, 1 — only winner (10) is eligible -> no pairs.
    let pop = pop_distinct(&[10.0, 5.0, 1.0]);
    let pairs = clearing_selection(&pop, 100.0, 1, 2);
    // Only one eligible individual -> 0 pairs regardless of request
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

    let result = selection::factory(&pop, config, 1, 2);
    assert!(result.is_ok());
    let pairs = result.unwrap();
    // With niche_radius=0.1, sorted desc: 10.05 (idx 1) wins, clears 10.0 (idx 0).
    // 5.0 (idx 2) is its own winner. Eligible = {idx 1, idx 2}.
    // number_of_couples=3 is honored: 3 pairs are drawn with replacement from the
    // 2-element eligible pool.
    assert_eq!(pairs.len(), 3);
    for group in &pairs {
        assert_ne!(
            group[0], 0,
            "Cleared individual (index 0) should not appear"
        );
        assert_ne!(
            group[1], 0,
            "Cleared individual (index 0) should not appear"
        );
        assert_ne!(group[0], group[1], "Self-pairing not allowed");
    }
}

#[test]
fn test_clearing_via_selection_enum() {
    use genetic_algorithms::{operations::Selection, traits::SelectionOperator};

    let pop = pop_distinct(&[10.0, 9.0, 5.0, 4.0]);
    // SelectionOperator uses the default niche_radius (0.1); all fitnesses are
    // well-separated so all 4 are eligible. number_of_couples=4 is honored -> 4 pairs.
    let pairs = Selection::Clearing.select(&pop, 4, 1, 2).expect("clearing selection should succeed");
    assert_eq!(pairs.len(), 4);
}
