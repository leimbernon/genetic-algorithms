use crate::error::GaError;
use super::{
    squared_euclidean_distance, validate_dimension_consistency, validate_non_empty,
};

/// Computes the Spread indicator (Deb et al. 2002).
///
/// Spread measures the diversity of solutions along the Pareto front.
///
/// A value of 0.0 indicates perfectly uniform distribution with endpoints
/// matching the extreme points.
pub fn spread(
    approx_front: &[Vec<f64>],
    extreme_points: &[Vec<f64>],
) -> Result<f64, GaError> {
    let _ = (approx_front, extreme_points);
    Ok(0.0)
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
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }

    #[test]
    fn test_spread_rejects_empty() {
        let result = spread(&vec![], &vec![vec![0.0, 0.0]]);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));

        let result = spread(&vec![vec![0.0, 0.0], vec![1.0, 0.0]], &vec![]);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }

    #[test]
    fn test_spread_rejects_dimension_mismatch() {
        let points = vec![vec![0.0, 0.0], vec![1.0, 0.0]];
        let extremes = vec![vec![0.0, 0.0, 0.0]];
        let result = spread(&points, &extremes);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }
}
