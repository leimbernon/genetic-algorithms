#[cfg(test)]
use crate::structures::Chromosome;
use genetic_algorithms::{
    fitness::FitnessFnWrapper,
    operations::mutation::{compute_cardinality, dynamic_probability},
};

// ==================== compute_cardinality tests ====================

#[test]
fn test_compute_cardinality_empty() {
    let chromosomes: Vec<Chromosome> = vec![];
    assert_eq!(compute_cardinality(&chromosomes), 0.0);
}

#[test]
fn test_compute_cardinality_all_unique() {
    let chromosomes = vec![
        Chromosome {
            dna: vec![],
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
    ];
    assert!((compute_cardinality(&chromosomes) - 1.0).abs() < 1e-10);
}

#[test]
fn test_compute_cardinality_all_same() {
    let chromosomes = vec![
        Chromosome {
            dna: vec![],
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
    ];
    assert!((compute_cardinality(&chromosomes) - 0.25).abs() < 1e-10);
}

#[test]
fn test_compute_cardinality_partial() {
    let chromosomes = vec![
        Chromosome {
            dna: vec![],
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![],
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
    ];
    // 3 unique out of 4
    assert!((compute_cardinality(&chromosomes) - 0.75).abs() < 1e-10);
}

// ==================== dynamic_probability tests ====================

#[test]
fn test_dynamic_probability_low_diversity_increases() {
    // cardinality (0.3) < target (0.5) → increase
    let result = dynamic_probability(0.5, 0.3, 0.5, 0.05, 1.0, 0.0);
    assert!((result - 0.55).abs() < 1e-10);
}

#[test]
fn test_dynamic_probability_high_diversity_decreases() {
    // cardinality (0.8) > target (0.5) → decrease
    let result = dynamic_probability(0.5, 0.8, 0.5, 0.05, 1.0, 0.0);
    assert!((result - 0.45).abs() < 1e-10);
}

#[test]
fn test_dynamic_probability_at_target_unchanged() {
    let result = dynamic_probability(0.5, 0.5, 0.5, 0.05, 1.0, 0.0);
    assert!((result - 0.5).abs() < 1e-10);
}

#[test]
fn test_dynamic_probability_clamped_at_max() {
    // Already at max, low diversity → should not exceed max
    let result = dynamic_probability(0.95, 0.2, 0.5, 0.1, 1.0, 0.0);
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn test_dynamic_probability_clamped_at_min() {
    // Already at min, high diversity → should not go below min
    let result = dynamic_probability(0.05, 0.9, 0.5, 0.1, 1.0, 0.0);
    assert!((result - 0.0).abs() < 1e-10);
}
