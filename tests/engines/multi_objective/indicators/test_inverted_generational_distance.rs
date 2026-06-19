use genetic_algorithms::multi_objective::indicators::{
    generational_distance, inverted_generational_distance,
};
use genetic_algorithms::error::GaError;

/// ZDT1 reference Pareto front: f1 in [0, 1], f2 = 1 - sqrt(f1).
fn zdt1_reference_front(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            let f1 = i as f64 / (n - 1) as f64;
            let f2 = 1.0 - f1.sqrt();
            vec![f1, f2]
        })
        .collect()
}

#[test]
fn test_igd_identical_fronts() {
    let front = vec![vec![1.0, 2.0], vec![2.0, 1.0], vec![1.5, 1.5]];
    let result = inverted_generational_distance(&front, &front, 2.0).unwrap();
    assert!((result - 0.0).abs() < 1e-15);
}

#[test]
fn test_igd_sparse_approx() {
    // true has 2 points, approx only has 1 → the missing point contributes distance
    let true_front = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
    let approx = vec![vec![1.0, 2.0]];
    let result = inverted_generational_distance(&approx, &true_front, 2.0).unwrap();
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn test_igd_gt_gd_for_sparse_front() {
    // When approx is a sparse subset of true, IGD captures missing coverage
    let true_front = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
    let approx = vec![vec![1.0, 2.0]];
    let gd = generational_distance(&approx, &true_front, 2.0).unwrap();
    let igd = inverted_generational_distance(&approx, &true_front, 2.0).unwrap();
    assert!(
        igd > gd,
        "IGD={} must exceed GD={} for sparse approx — IGD detects missing true-front coverage that GD misses",
        igd, gd
    );
}

#[test]
fn test_igd_zdt1_subset() {
    // Sparse approx (10 points) vs dense true front (1000 points)
    let approx = zdt1_reference_front(10);
    let true_front = zdt1_reference_front(1000);
    let result = inverted_generational_distance(&approx, &true_front, 2.0).unwrap();
    assert!(result > 0.0, "IGD should be positive for non-identical fronts");
    assert!(result < 0.1, "IGD should be small for ZDT1 subset");
}

#[test]
fn test_igd_dimension_mismatch() {
    let approx = vec![vec![1.0, 2.0]];
    let true_front = vec![vec![1.0, 2.0, 3.0]];
    let result = inverted_generational_distance(&approx, &true_front, 2.0);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_igd_empty_approx() {
    let result = inverted_generational_distance(&vec![], &vec![vec![1.0, 2.0]], 2.0);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_igd_empty_true() {
    let result = inverted_generational_distance(&vec![vec![1.0, 2.0]], &vec![], 2.0);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_igd_zero_power() {
    let front = vec![vec![1.0, 2.0]];
    let result = inverted_generational_distance(&front, &front, 0.0);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}
