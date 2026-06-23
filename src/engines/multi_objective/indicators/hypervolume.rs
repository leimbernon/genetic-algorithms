use super::{validate_dimension, validate_dimension_consistency, validate_non_empty};
use crate::error::GaError;

/// Computes the 2D Hypervolume indicator (Lebesgue measure).
///
/// The hypervolume is the area of the union of rectangles formed by each point
/// and the reference point, bounded by \[f1_i, ref\[0\]\] x \[f2_i, ref\[1\]\].
/// A larger hypervolume indicates better convergence and spread (for minimization).
///
/// # Algorithm
///
/// 1. Filter points that are strictly dominated by the reference point
///    (f1 < ref\[0\] AND f2 < ref\[1\]).
/// 2. Sort by first objective ascending.
/// 3. Sweep left-to-right, tracking the running minimum of f2. At each step,
///    the rectangle from the previous f1 to the current f1 has height
///    (ref\[1\] - best_f2), where best_f2 is the smallest f2 seen so far.
/// 4. Final segment from the last f1 to ref\[0\].
///
/// Complexity: O(n log n) due to sorting.
///
/// # Errors
///
/// Returns `GaError::InvalidIndicatorConfiguration` if:
/// - `points` is empty
/// - Points do not all have exactly 2 objectives
/// - `reference_point` has the wrong dimension
/// - Any point is not strictly dominated by the reference point
///   (i.e., for each dimension i: point\[i\] < reference_point\[i\])
///
/// # Panics
///
/// Does not panic on valid input.
pub fn hypervolume(points: &[Vec<f64>], reference_point: &[f64]) -> Result<f64, GaError> {
    validate_non_empty("points", points)?;
    let dim = validate_dimension_consistency(points)?;

    if dim != 2 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Hypervolume requires exactly 2 objectives".to_string(),
        ));
    }

    validate_dimension("reference_point", reference_point, 2)?;

    let mut dominated_points: Vec<(f64, f64)> = Vec::with_capacity(points.len());
    for (i, point) in points.iter().enumerate() {
        let f1 = point[0];
        let f2 = point[1];
        if f1 >= reference_point[0] || f2 >= reference_point[1] {
            return Err(GaError::InvalidIndicatorConfiguration(format!(
                "Point {} ({}, {}) must be strictly dominated by reference point ({}, {})",
                i, f1, f2, reference_point[0], reference_point[1]
            )));
        }
        dominated_points.push((f1, f2));
    }

    dominated_points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut hv = 0.0;
    let mut best_f2 = reference_point[1];
    let mut x_cursor = 0.0;

    for &(f1, f2) in &dominated_points {
        let width = f1 - x_cursor;
        let height = reference_point[1] - best_f2;
        hv += width * height;
        best_f2 = best_f2.min(f2);
        x_cursor = f1;
    }

    let width = reference_point[0] - x_cursor;
    let height = reference_point[1] - best_f2;
    hv += width * height;

    Ok(hv)
}
