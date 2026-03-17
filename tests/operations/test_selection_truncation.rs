#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper,
    operations::selection::truncation::truncation_selection,
};

fn make_chromosome(fitness: f64) -> Chromosome {
    Chromosome {
        dna: vec![Gene { id: 0 }],
        fitness,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    }
}

#[test]
fn test_truncation_selection_produces_correct_number_of_pairs() {
    let pop: Vec<Chromosome> = (0..10).map(|i| make_chromosome(i as f64 * 10.0)).collect();
    let pairs = truncation_selection(&pop, 4);
    assert_eq!(pairs.len(), 4);
}

#[test]
fn test_truncation_selection_empty_on_small_population() {
    // Single chromosome — cannot form any pair
    let pop = vec![make_chromosome(42.0)];
    let pairs = truncation_selection(&pop, 1);
    assert!(pairs.is_empty());

    // Empty population
    let empty: Vec<Chromosome> = Vec::new();
    let pairs = truncation_selection(&empty, 1);
    assert!(pairs.is_empty());
}

#[test]
fn test_truncation_selection_selects_only_from_top_half() {
    // Create a population of 20 where fitness == index (0..19).
    // Top 50% = indices 10..19 (the 10 fittest).
    let pop: Vec<Chromosome> = (0..20).map(|i| make_chromosome(i as f64)).collect();

    let top_half_indices: std::collections::HashSet<usize> = (10..20).collect();

    // Run many trials to ensure we never pick from the bottom half
    for _ in 0..200 {
        let pairs = truncation_selection(&pop, 5);
        for (a, b) in &pairs {
            assert!(
                top_half_indices.contains(a),
                "Index {} is not in the top half ({:?})",
                a,
                top_half_indices
            );
            assert!(
                top_half_indices.contains(b),
                "Index {} is not in the top half ({:?})",
                b,
                top_half_indices
            );
        }
    }
}

#[test]
fn test_truncation_selection_handles_equal_fitness() {
    // All individuals have the same fitness — any of them could be in the
    // top half. Selection should still succeed without panicking.
    let pop: Vec<Chromosome> = (0..10).map(|_| make_chromosome(5.0)).collect();
    let pairs = truncation_selection(&pop, 3);
    assert_eq!(pairs.len(), 3);
    for (a, b) in &pairs {
        assert!(*a < pop.len());
        assert!(*b < pop.len());
    }
}

#[test]
fn test_truncation_selection_with_two_chromosomes() {
    // Minimum viable population — both are in the elite pool
    let pop = vec![make_chromosome(1.0), make_chromosome(2.0)];
    let pairs = truncation_selection(&pop, 1);
    assert_eq!(pairs.len(), 1);
    for (a, b) in &pairs {
        assert!(*a < 2);
        assert!(*b < 2);
    }
}
