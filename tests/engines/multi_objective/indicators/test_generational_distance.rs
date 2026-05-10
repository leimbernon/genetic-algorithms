use genetic_algorithms::multi_objective::indicators::generational_distance;
use genetic_algorithms::GaError;

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
fn test_gd_identical_fronts() {
    let front = vec![vec![1.0, 2.0], vec![2.0, 1.0], vec![1.5, 1.5]];
    let result = generational_distance(&front, &front, 2.0).unwrap();
    assert!((result - 0.0).abs() < 1e-15,
        "GD of identical fronts must be 0, got {}", result);
}

#[test]
fn test_gd_shifted_fronts() {
    let approx = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
    let true_front = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    let result = generational_distance(&approx, &true_front, 2.0).unwrap();
    let expected = (2.0f64).sqrt();
    assert!((result - expected).abs() < 1e-10,
        "Expected {}, got {}", expected, result);
}

#[test]
fn test_gd_zdt1_subset() {
    let approx = zdt1_reference_front(10);
    let true_front = zdt1_reference_front(1000);
    let result = generational_distance(&approx, &true_front, 2.0).unwrap();
    assert!(result > 0.0, "GD should be positive for non-identical fronts");
    assert!(result < 0.1, "GD should be small for ZDT1 subset, got {}", result);
}

#[test]
fn test_gd_power_1() {
    let approx = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
    let true_front = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    let result = generational_distance(&approx, &true_front, 1.0).unwrap();
    let expected = (2.0f64).sqrt();
    assert!((result - expected).abs() < 1e-10,
        "GD with power=1: expected {}, got {}", expected, result);
}

#[test]
fn test_gd_dimension_mismatch() {
    let approx = vec![vec![1.0, 2.0]];
    let true_front = vec![vec![1.0, 2.0, 3.0]];
    let result = generational_distance(&approx, &true_front, 2.0);
    assert!(
        matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))),
        "Expected InvalidIndicatorConfiguration, got {:?}", result
    );
}

#[test]
fn test_gd_empty_approx() {
    let result = generational_distance(&vec![], &vec![vec![1.0, 2.0]], 2.0);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_gd_empty_true() {
    let result = generational_distance(&vec![vec![1.0, 2.0]], &vec![], 2.0);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_gd_zero_power() {
    let approx = vec![vec![1.0, 2.0]];
    let true_front = vec![vec![1.0, 2.0]];
    let result = generational_distance(&approx, &true_front, 0.0);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}
