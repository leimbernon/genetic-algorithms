use super::{squared_euclidean_distance, validate_dimension_consistency, validate_non_empty};
use crate::error::GaError;

/// Computes the Spread indicator (Deb et al. 2002).
///
/// Spread measures the diversity of solutions along the Pareto front. It
/// combines two components:
///
/// 1. **Extreme-point distance** (df + dl) — how far the obtained front's
///    endpoints are from the true extreme points.
/// 2. **Uniformity** — how evenly the solutions are distributed along
///    the front (sum of |d_i - d_bar|).
///
/// # Formula
///
/// `Spread = (df + dl + sum_i |d_i - d_bar|) / (df + dl + (n-1) * d_bar)`
///
/// where:
/// - d_i are Euclidean distances between consecutive solutions (sorted by f1)
/// - d_bar is the mean of d_i
/// - df is the minimum distance from `extreme_points` to the first front point
/// - dl is the minimum distance from `extreme_points` to the last front point
///
/// A value of 0.0 indicates perfectly uniform distribution with endpoints
/// matching the extreme points.
///
/// # Arguments
///
/// * `approx_front` — The approximation set (requires at least 2 points).
/// * `extreme_points` — The extreme points of the true Pareto front.
///
/// # Errors
///
/// Returns `GaError::InvalidIndicatorConfiguration` if:
/// - `approx_front` has fewer than 2 points
/// - `approx_front` is empty
/// - `extreme_points` is empty
/// - Dimensions are inconsistent
pub fn spread(approx_front: &[Vec<f64>], extreme_points: &[Vec<f64>]) -> Result<f64, GaError> {
    validate_non_empty("approx_front", approx_front)?;
    validate_non_empty("extreme_points", extreme_points)?;
    let dim = validate_dimension_consistency(approx_front)?;
    let _ = validate_dimension_consistency(extreme_points)?;

    // Verify extreme_points have same dimension as approx_front
    if extreme_points[0].len() != dim {
        return Err(GaError::InvalidIndicatorConfiguration(format!(
            "Dimension mismatch: approx_front has {} dimensions, extreme_points have {}",
            dim,
            extreme_points[0].len(),
        )));
    }

    let n = approx_front.len();
    if n < 2 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Spread requires at least 2 points in approx_front".to_string(),
        ));
    }

    // Sort by first objective ascending (per Deb 2002)
    let mut sorted: Vec<&[f64]> = approx_front.iter().map(|v| v.as_slice()).collect();
    sorted.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));

    // Consecutive Euclidean distances
    let mut d = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let dist = squared_euclidean_distance(sorted[i], sorted[i + 1]).sqrt();
        d.push(dist);
    }

    let d_bar: f64 = d.iter().sum::<f64>() / d.len() as f64;

    // df: minimum distance from extreme_points to the first front member
    let df = extreme_points
        .iter()
        .map(|ep| squared_euclidean_distance(ep, sorted[0]).sqrt())
        .fold(f64::INFINITY, f64::min);

    // dl: minimum distance from extreme_points to the last front member
    let dl = extreme_points
        .iter()
        .map(|ep| squared_euclidean_distance(ep, sorted[n - 1]).sqrt())
        .fold(f64::INFINITY, f64::min);

    let sum_deviation: f64 = d.iter().map(|di| (di - d_bar).abs()).sum();

    let numerator = df + dl + sum_deviation;
    let denominator = df + dl + (n as f64 - 1.0) * d_bar;

    if denominator == 0.0 {
        Ok(0.0) // Perfect spread (all distances equal and endpoints match extremes)
    } else {
        Ok(numerator / denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spread_perfect_uniform() {
        let points = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![2.0, 0.0],
            vec![3.0, 0.0],
        ];
        let extremes = vec![vec![0.0, 0.0], vec![3.0, 0.0]];
        let result = spread(&points, &extremes).unwrap();
        assert!((result - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_spread_nonuniform() {
        let points = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![2.0, 0.0],
            vec![5.0, 0.0],
        ];
        let extremes = vec![vec![0.0, 0.0], vec![5.0, 0.0]];
        let result = spread(&points, &extremes).unwrap();
        let expected = 8.0 / 15.0;
        assert!((result - expected).abs() < 1e-10);
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
}
