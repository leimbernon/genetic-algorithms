use genetic_algorithms::niching::sharing::{
    apply_fitness_sharing, compute_distance_matrix, sharing_function,
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
