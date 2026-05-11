use crate::error::GaError;
use super::{nearest_distance, validate_dimension_consistency, validate_non_empty};

/// Computes the Generational Distance (GD) indicator.
///
/// GD measures the average Euclidean distance from each point in the
/// approximate front to the nearest point in the true Pareto front.
/// Lower values indicate better convergence.
///
/// # Formula
///
/// `GD = (1/n * sum_{i=1}^n d_i^p)^{1/p}`
///
/// where `d_i` is the Euclidean distance from the i-th approximate point
/// to the nearest true-front point.
///
/// # Arguments
///
/// * `approx_front` — The approximation set produced by an algorithm.
/// * `true_front` — The reference (true) Pareto front.
/// * `power` — The exponent for the distance norm. Default is 2.0 (Euclidean).
///   Use 1.0 for Manhattan. Very large values approximate the Hausdorff metric.
///
/// # Errors
///
/// Returns `GaError::InvalidIndicatorConfiguration` if either front is
/// empty, dimensions are inconsistent, fronts have mismatched dimensions,
/// or power is non-positive.
pub fn generational_distance(
    approx_front: &[Vec<f64>],
    true_front: &[Vec<f64>],
    power: f64,
) -> Result<f64, GaError> {
    validate_non_empty("approx_front", approx_front)?;
    validate_non_empty("true_front", true_front)?;
    let approx_dim = validate_dimension_consistency(approx_front)?;
    let true_dim = validate_dimension_consistency(true_front)?;

    if approx_dim != true_dim {
        return Err(GaError::InvalidIndicatorConfiguration(
            format!(
                "Dimension mismatch: approx_front has {} dimensions, true_front has {}",
                approx_dim, true_dim,
            ),
        ));
    }

    if power <= 0.0 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Power must be positive".to_string(),
        ));
    }

    let sum: f64 = approx_front
        .iter()
        .map(|point| nearest_distance(point, true_front, power))
        .sum();

    let mean = sum / approx_front.len() as f64;
    Ok(mean.powf(1.0 / power))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gd_identical_fronts() {
        let approx = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let true_front = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let result = generational_distance(&approx, &true_front, 2.0).unwrap();
        assert!((result - 0.0).abs() < 1e-15);
    }

    #[test]
    fn test_gd_shifted_fronts() {
        let approx = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let true_front = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let result = generational_distance(&approx, &true_front, 2.0).unwrap();
        let expected = (2.0f64).sqrt();
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_gd_power_1_manhattan() {
        let approx = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let true_front = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let result = generational_distance(&approx, &true_front, 1.0).unwrap();
        let expected = (2.0f64).sqrt();
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_gd_rejects_dimension_mismatch() {
        let approx = vec![vec![1.0, 2.0]];
        let true_front = vec![vec![1.0, 2.0, 3.0]];
        let result = generational_distance(&approx, &true_front, 2.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }

    #[test]
    fn test_gd_rejects_empty() {
        let result = generational_distance(&vec![], &vec![vec![1.0, 2.0]], 2.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));

        let result = generational_distance(&vec![vec![1.0, 2.0]], &vec![], 2.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }

    #[test]
    fn test_gd_rejects_non_positive_power() {
        let approx = vec![vec![1.0, 2.0]];
        let true_front = vec![vec![1.0, 2.0]];
        let result = generational_distance(&approx, &true_front, 0.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));

        let result = generational_distance(&approx, &true_front, -1.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }
}
