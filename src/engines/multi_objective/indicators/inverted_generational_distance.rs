use crate::error::GaError;
use super::generational_distance::DEFAULT_POWER;
use super::{nearest_distance, validate_dimension_consistency, validate_non_empty};

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

    // Key difference from GD: iterate over TRUE front, find nearest in APPROX
    let sum: f64 = true_front
        .iter()
        .map(|point| nearest_distance(point, approx_front, power))
        .sum();

    let mean = sum / true_front.len() as f64;
    Ok(mean.powf(1.0 / power))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_igd_identical_fronts() {
        let front = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let result = inverted_generational_distance(&front, &front, 2.0).unwrap();
        assert!((result - 0.0).abs() < 1e-15);
    }

    #[test]
    fn test_igd_sparse_approx() {
        // true = [(1,2), (2,1)], approx = [(1,2)] — approx is missing (2,1)
        // For true[0]=(1,2): nearest in approx = (1,2) → dist=0 → nearest=0
        // For true[1]=(2,1): nearest in approx = (1,2) → dist=√2 → squared=2 → nearest=2
        // IGD = ((0 + 2) / 2)^(1/2) = 1.0
        let true_front = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let approx = vec![vec![1.0, 2.0]];
        let result = inverted_generational_distance(&approx, &true_front, 2.0).unwrap();
        assert!((result - 1.0).abs() < 1e-10,
            "Expected IGD=1.0 for sparse approx, got {}", result);
    }

    #[test]
    fn test_igd_gt_gd_for_sparse() {
        // true front has 2 points, approx only has 1
        // GD(approx, true) = 0.0 (the one approx point is on the front)
        // IGD(true, approx) = 1.0 (the missing true point has distance √2 → 1.0)
        let true_front = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let approx = vec![vec![1.0, 2.0]];
        let gd = super::super::generational_distance::generational_distance(
            &approx, &true_front, 2.0
        ).unwrap();
        let igd = inverted_generational_distance(&approx, &true_front, 2.0).unwrap();
        assert!(igd > gd, "IGD={} must exceed GD={} for sparse front", igd, gd);
    }

    #[test]
    fn test_igd_rejects_dimension_mismatch() {
        let approx = vec![vec![1.0, 2.0]];
        let true_front = vec![vec![1.0, 2.0, 3.0]];
        let result = inverted_generational_distance(&approx, &true_front, 2.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }

    #[test]
    fn test_igd_rejects_empty() {
        let result = inverted_generational_distance(&vec![], &vec![vec![1.0, 2.0]], 2.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));

        let result = inverted_generational_distance(&vec![vec![1.0, 2.0]], &vec![], 2.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }

    #[test]
    fn test_igd_rejects_non_positive_power() {
        let front = vec![vec![1.0, 2.0]];
        let result = inverted_generational_distance(&front, &front, 0.0);
        assert!(matches!(result, Err(GaError::InvalidIndicatorConfiguration(_))));
    }
}
