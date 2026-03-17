#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{fitness::FitnessFnWrapper, operations::selection::rank::rank_selection};

#[test]
fn test_rank_selection_produces_correct_pairs() {
    let population: Vec<Chromosome> = (0..10)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: i as f64 * 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    let pairs = rank_selection(&population, 3);
    assert_eq!(pairs.len(), 3);

    for (a, b) in &pairs {
        assert!(*a < population.len(), "Index out of bounds: {}", a);
        assert!(*b < population.len(), "Index out of bounds: {}", b);
    }
}

#[test]
fn test_rank_selection_with_two_chromosomes() {
    let population = vec![
        Chromosome {
            dna: vec![Gene { id: 1 }],
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: vec![Gene { id: 2 }],
            fitness: 90.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];

    let pairs = rank_selection(&population, 1);
    assert_eq!(pairs.len(), 1);
}

#[test]
fn test_rank_selection_returns_valid_indices() {
    let pop: Vec<Chromosome> = (0..6)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = rank_selection(&pop, 3);
    for (a, b) in &pairs {
        assert!(*a < pop.len(), "Index {} out of bounds", a);
        assert!(*b < pop.len(), "Index {} out of bounds", b);
    }
}

#[test]
fn test_rank_selection_favors_higher_fitness() {
    let population: Vec<Chromosome> = (0..20)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    // Run many selections and count how often high-fitness individuals appear
    let mut high_fitness_count = 0;
    for _ in 0..200 {
        let pairs = rank_selection(&population, 5);
        for (a, b) in &pairs {
            // Indices 15-19 are the fittest
            if *a >= 15 {
                high_fitness_count += 1;
            }
            if *b >= 15 {
                high_fitness_count += 1;
            }
        }
    }
    // Should appear more than uniform random (5/20 = 25%)
    // With rank selection, top 5 should appear more than ~500 times out of 2000
    assert!(
        high_fitness_count > 600,
        "High-fitness individuals appeared {} times, expected more with rank selection",
        high_fitness_count
    );
}
