use super::{nearest_distance, validate_dimension_consistency, validate_non_empty};
use crate::error::GaError;

/// Computes the Inverted Generational Distance (IGD) indicator.
///
/// IGD measures the average Euclidean distance from each point in the
/// **true** Pareto front to the nearest point in the **approximate** front.
/// Unlike GD (which measures convergence), IGD captures both convergence
/// AND coverage — a sparse approx front yields a large IGD because some
/// true-front regions have no nearby approx point.
///
/// # Formula
///
/// `IGD = (1/|T| * sum_{t in T} min_{a in A} ||t - a||_2^p)^{1/p}`
///
/// where T is the true front and A is the approximate front.
///
/// Lower values indicate better convergence and coverage.
///
/// # Arguments
///
/// * `approx_front` — The approximation set produced by an algorithm.
/// * `true_front` — The reference (true) Pareto front.
/// * `power` — The exponent for the distance norm. Default is 2.0 (Euclidean).
///
/// # Errors
///
/// Returns `GaError::InvalidIndicatorConfiguration` if either front is
/// empty, dimensions are inconsistent, fronts have mismatched dimensions,
/// or power is non-positive.
pub fn inverted_generational_distance(
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

    // Key difference from GD: iterate over TRUE front, find nearest in APPROX
    let sum: f64 = true_front
        .iter()
        .map(|point| nearest_distance(point, approx_front, power))
        .sum();

    let mean = sum / true_front.len() as f64;
    Ok(mean.powf(1.0 / power))
}
