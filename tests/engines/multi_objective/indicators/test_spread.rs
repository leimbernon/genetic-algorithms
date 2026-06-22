use genetic_algorithms::error::GaError;
use genetic_algorithms::multi_objective::indicators::spread;

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
fn test_spread_perfect_uniform() {
    // Collinear points with equal spacing and matched extremes → spread = 0
    let points = vec![
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![2.0, 0.0],
        vec![3.0, 0.0],
    ];
    let extremes = vec![vec![0.0, 0.0], vec![3.0, 0.0]];
    let result = spread(&points, &extremes).unwrap();
    assert!(
        result.abs() < 1e-10,
        "Perfect uniform spread should be 0.0, got {}",
        result
    );
}

#[test]
fn test_spread_nonuniform() {
    // Gap at the end creates non-uniformity
    let points = vec![
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![2.0, 0.0],
        vec![5.0, 0.0],
    ];
    let extremes = vec![vec![0.0, 0.0], vec![5.0, 0.0]];
    let result = spread(&points, &extremes).unwrap();
    let expected = 8.0 / 15.0; // ≈ 0.533333...
    assert!(
        (result - expected).abs() < 1e-10,
        "Expected spread={}, got {}",
        expected,
        result
    );
}

#[test]
fn test_spread_zdt1_even() {
    // ZDT1 with many evenly-spaced points should have low spread
    let points = zdt1_reference_front(100);
    let extremes = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    let result = spread(&points, &extremes).unwrap();
    assert!(result >= 0.0, "Spread must be non-negative");
    assert!(
        result < 1.0,
        "Evenly spaced ZDT1 should have spread < 1, got {}",
        result
    );
}

#[test]
fn test_spread_rejects_single_point() {
    let points = vec![vec![0.0, 0.0]];
    let extremes = vec![vec![0.0, 0.0]];
    let result = spread(&points, &extremes);
    assert!(matches!(
        result,
        Err(GaError::InvalidIndicatorConfiguration(_))
    ));
}

#[test]
fn test_spread_rejects_empty() {
    let result = spread(&[], &[vec![0.0, 0.0]]);
    assert!(matches!(
        result,
        Err(GaError::InvalidIndicatorConfiguration(_))
    ));

    let result = spread(&[vec![0.0, 0.0], vec![1.0, 0.0]], &[]);
    assert!(matches!(
        result,
        Err(GaError::InvalidIndicatorConfiguration(_))
    ));
}

#[test]
fn test_spread_rejects_dimension_mismatch() {
    let points = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
    let extremes = vec![vec![0.0, 0.0, 0.0]];
    let result = spread(&points, &extremes);
    assert!(matches!(
        result,
        Err(GaError::InvalidIndicatorConfiguration(_))
    ));
}
