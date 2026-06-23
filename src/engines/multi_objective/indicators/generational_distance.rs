use super::{nearest_distance, validate_dimension_consistency, validate_non_empty};
use crate::error::GaError;

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
        return Err(GaError::InvalidIndicatorConfiguration(format!(
            "Dimension mismatch: approx_front has {} dimensions, true_front has {}",
            approx_dim, true_dim,
        )));
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
