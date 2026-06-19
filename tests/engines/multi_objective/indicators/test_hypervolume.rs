use genetic_algorithms::multi_objective::indicators::hypervolume;
use genetic_algorithms::error::GaError;

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
fn test_hypervolume_basic_single_point() {
    let points = vec![vec![0.5, 0.5]];
    let result = hypervolume(&points, &[1.0, 1.0]).unwrap();
    let expected = 0.25;
    assert!((result - expected).abs() < 1e-10,
        "Expected {}, got {}", expected, result);
}

#[test]
fn test_hypervolume_two_point_front() {
    let points = vec![vec![0.2, 0.7], vec![0.6, 0.3]];
    let result = hypervolume(&points, &[1.0, 1.0]).unwrap();
    assert!((result - 0.40).abs() < 1e-10,
        "Expected 0.40, got {}", result);
}

#[test]
fn test_hypervolume_zdt1() {
    let points = zdt1_reference_front(1000);
    // Reference point must strictly dominate all points; (0, 1.0) on ZDT1
    // front equals ref (1.0, 1.0) in f2, so use (1.1, 1.1) instead.
    let result = hypervolume(&points, &[1.1, 1.1]).unwrap();
    assert!(result > 0.6, "Expected HV > 0.6 for ZDT1, got {}", result);
}

#[test]
fn test_hypervolume_zdt1_with_reference_1_1_1_1() {
    let points = zdt1_reference_front(1000);
    let result = hypervolume(&points, &[1.1, 1.1]).unwrap();
    assert!(result > 0.66, "Expected HV > 0.66 with wider ref, got {}", result);
}

#[test]
fn test_hypervolume_rejects_3d() {
    let points = vec![vec![1.0, 2.0, 3.0]];
    let result = hypervolume(&points, &[4.0, 4.0, 4.0]);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_hypervolume_rejects_empty() {
    let points: Vec<Vec<f64>> = vec![];
    let result = hypervolume(&points, &[1.0, 1.0]);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_hypervolume_rejects_non_dominating_reference() {
    let points = vec![vec![0.5, 0.5]];
    let result = hypervolume(&points, &[0.5, 0.5]);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}

#[test]
fn test_hypervolume_dimension_mismatch() {
    let points = vec![vec![1.0], vec![2.0]];
    let result = hypervolume(&points, &[3.0, 3.0]);
    assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
}
