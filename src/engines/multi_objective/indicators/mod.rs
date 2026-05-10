//! Quality indicators for multi-objective optimization.
//!
//! This module provides pure functions for evaluating multi-objective solution
//! sets: [`hypervolume`] (2D Lebesgue measure), [`generational_distance`],
//! [`inverted_generational_distance`], and [`spread`] (Deb et al. 2002).
//!
//! All functions return `Result<f64, GaError>`. Input validation fails fast:
//! empty sets, dimension mismatches, and invalid parameters all return
//! [`GaError::InvalidIndicatorConfiguration`].
//!
//! Consumed by indicator-based MOEA engines (SMS-EMOA, IBEA — Phase 38)
//! and callable from user code for post-run Pareto-front analysis.

use crate::error::GaError;

mod hypervolume;
mod generational_distance;
mod inverted_generational_distance;
mod spread;

pub use hypervolume::hypervolume;
pub use generational_distance::generational_distance;
pub use inverted_generational_distance::inverted_generational_distance;
pub use spread::spread;

// ---------------------------------------------------------------------------
// Shared validation helpers (pub(crate) — internal use only)
// ---------------------------------------------------------------------------

/// Validate that a point set is non-empty.
pub(crate) fn validate_non_empty(name: &str, points: &[Vec<f64>]) -> Result<(), GaError> {
    if points.is_empty() {
        return Err(GaError::InvalidIndicatorConfiguration(
            format!("{} must not be empty", name),
        ));
    }
    Ok(())
}

/// Validate that all points in a slice have the same dimension.
/// Returns the dimension if valid.
pub(crate) fn validate_dimension_consistency(points: &[Vec<f64>]) -> Result<usize, GaError> {
    if points.is_empty() {
        return Ok(0);
    }
    let dim = points[0].len();
    if dim == 0 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Points must have at least 1 dimension".to_string(),
        ));
    }
    for (i, point) in points.iter().enumerate().skip(1) {
        if point.len() != dim {
            return Err(GaError::InvalidIndicatorConfiguration(
                format!(
                    "Dimension mismatch at index {}: expected {} dimensions, got {}",
                    i,
                    dim,
                    point.len()
                ),
            ));
        }
    }
    Ok(dim)
}

/// Validate that a single point has the expected dimension.
pub(crate) fn validate_dimension(
    name: &str,
    point: &[f64],
    expected_dim: usize,
) -> Result<(), GaError> {
    if point.len() != expected_dim {
        return Err(GaError::InvalidIndicatorConfiguration(
            format!(
                "{} has {} dimensions, expected {}",
                name,
                point.len(),
                expected_dim
            ),
        ));
    }
    Ok(())
}

/// Squared Euclidean distance between two points.
#[inline]
pub(crate) fn squared_euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum()
}

/// Euclidean distance from a single point to the nearest point in a front,
/// raised to the given power.
///
/// Computes `min(||point - ref_i||_2)^power` for all `ref_i` in `front`.
pub(crate) fn nearest_distance(point: &[f64], front: &[Vec<f64>], power: f64) -> f64 {
    let min_sq_dist = front
        .iter()
        .map(|ref_point| squared_euclidean_distance(point, ref_point))
        .fold(f64::INFINITY, f64::min);
    // min_sq_dist^(power/2) = (sqrt(sq))^power = ||dist||^power
    min_sq_dist.powf(power / 2.0)
}
