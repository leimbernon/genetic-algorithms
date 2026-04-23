use genetic_algorithms::niching::sharing::{
    apply_fitness_sharing, apply_fitness_sharing_with_dna, compute_distance_matrix,
    sharing_function,
};

#[test]
fn test_sharing_function_within_radius() {
    // d=0.5, sigma=1.0, alpha=1.0 => 1 - 0.5 = 0.5
    let sh = sharing_function(0.5, 1.0, 1.0);
    assert!((sh - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_sharing_function_at_zero() {
    // d=0.0 => sh = 1.0
    let sh = sharing_function(0.0, 1.0, 1.0);
    assert!((sh - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_sharing_function_outside_radius() {
    let sh = sharing_function(1.5, 1.0, 1.0);
    assert!((sh - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_sharing_function_at_boundary() {
    // d == sigma_share => not < sigma_share => 0.0
    let sh = sharing_function(1.0, 1.0, 1.0);
    assert!((sh - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_sharing_function_with_alpha() {
    // d=0.5, sigma=1.0, alpha=2.0 => 1 - (0.5)^2 = 1 - 0.25 = 0.75
    let sh = sharing_function(0.5, 1.0, 2.0);
    assert!((sh - 0.75).abs() < f64::EPSILON);
}

#[test]
fn test_apply_fitness_sharing_identical() {
    let mut fitnesses = vec![10.0, 10.0, 10.0];
    let distances = vec![
        vec![0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.0],
    ];
    apply_fitness_sharing(&mut fitnesses, &distances, 1.0, 1.0);
    for f in &fitnesses {
        assert!((*f - 10.0 / 3.0).abs() < 1e-10);
    }
}

#[test]
fn test_apply_fitness_sharing_distant() {
    // All individuals are far apart (distance > sigma_share)
    // niche_count for each = 1.0 (only self at distance 0)
    let mut fitnesses = vec![10.0, 20.0, 30.0];
    let distances = vec![
        vec![0.0, 5.0, 5.0],
        vec![5.0, 0.0, 5.0],
        vec![5.0, 5.0, 0.0],
    ];
    apply_fitness_sharing(&mut fitnesses, &distances, 1.0, 1.0);
    assert!((fitnesses[0] - 10.0).abs() < 1e-10);
    assert!((fitnesses[1] - 20.0).abs() < 1e-10);
    assert!((fitnesses[2] - 30.0).abs() < 1e-10);
}

#[test]
fn test_apply_fitness_sharing_empty() {
    let mut fitnesses: Vec<f64> = vec![];
    let distances: Vec<Vec<f64>> = vec![];
    apply_fitness_sharing(&mut fitnesses, &distances, 1.0, 1.0);
    assert!(fitnesses.is_empty());
}

#[test]
fn test_compute_distance_matrix_symmetric() {
    let dna1: Vec<f64> = vec![0.0, 0.0];
    let dna2: Vec<f64> = vec![3.0, 4.0];
    let dna3: Vec<f64> = vec![1.0, 0.0];

    let slices: Vec<&[f64]> = vec![&dna1, &dna2, &dna3];

    let matrix = compute_distance_matrix(&slices, |a, b| {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    });

    assert_eq!(matrix.len(), 3);
    // matrix[0][1] should be 5.0 (3-4-5 triangle)
    assert!((matrix[0][1] - 5.0).abs() < 1e-10);
    // Symmetric
    assert!((matrix[0][1] - matrix[1][0]).abs() < 1e-10);
    // Diagonal is 0
    assert!((matrix[0][0] - 0.0).abs() < 1e-10);
}

#[test]
fn test_apply_fitness_sharing_with_dna_matches_matrix_version() {
    let dna1 = vec![1i32, 2, 3];
    let dna2 = vec![1i32, 3, 3];
    let dna3 = vec![4i32, 5, 6];
    let slices: Vec<&[i32]> = vec![&dna1, &dna2, &dna3];

    let mut fitnesses_matrix = vec![10.0f64, 20.0, 30.0];
    let mut fitnesses_dna = vec![10.0f64, 20.0, 30.0];

    let hamming_fn = |a: &[i32], b: &[i32]| -> f64 {
        a.iter().zip(b.iter()).filter(|(x, y)| x != y).count() as f64
    };

    let distances = compute_distance_matrix(&slices, hamming_fn);
    apply_fitness_sharing(&mut fitnesses_matrix, &distances, 2.0, 1.0);
    apply_fitness_sharing_with_dna(&mut fitnesses_dna, &slices, hamming_fn, 2.0, 1.0);

    for i in 0..3 {
        assert!(
            (fitnesses_matrix[i] - fitnesses_dna[i]).abs() < 1e-10,
            "Mismatch at index {}: matrix={}, dna={}",
            i,
            fitnesses_matrix[i],
            fitnesses_dna[i]
        );
    }
}

#[test]
fn test_apply_fitness_sharing_with_dna_empty() {
    let mut fitnesses: Vec<f64> = vec![];
    let slices: Vec<&[i32]> = vec![];
    let hamming_fn = |a: &[i32], b: &[i32]| -> f64 {
        a.iter().zip(b.iter()).filter(|(x, y)| x != y).count() as f64
    };
    apply_fitness_sharing_with_dna(&mut fitnesses, &slices, hamming_fn, 1.0, 1.0);
    assert!(fitnesses.is_empty());
}

#[test]
fn test_apply_fitness_sharing_with_dna_distant() {
    let dna1 = vec![0i32, 0, 0];
    let dna2 = vec![10i32, 10, 10];
    let dna3 = vec![20i32, 20, 20];
    let slices: Vec<&[i32]> = vec![&dna1, &dna2, &dna3];

    let mut fitnesses = vec![10.0f64, 20.0, 30.0];

    let hamming_fn = |a: &[i32], b: &[i32]| -> f64 {
        a.iter().zip(b.iter()).filter(|(x, y)| x != y).count() as f64
    };

    // sigma_share=0.5 means all non-self distances (>=1.0) are > sigma_share,
    // so each individual only counts itself (distance 0 => sh(0) = 1.0)
    apply_fitness_sharing_with_dna(&mut fitnesses, &slices, hamming_fn, 0.5, 1.0);

    assert!(
        (fitnesses[0] - 10.0).abs() < 1e-10,
        "Expected 10.0, got {}",
        fitnesses[0]
    );
    assert!(
        (fitnesses[1] - 20.0).abs() < 1e-10,
        "Expected 20.0, got {}",
        fitnesses[1]
    );
    assert!(
        (fitnesses[2] - 30.0).abs() < 1e-10,
        "Expected 30.0, got {}",
        fitnesses[2]
    );
}
