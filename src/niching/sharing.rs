use log::debug;

/// Computes the sharing function value `sh(d)` for a given distance.
///
/// The sharing function is:
/// - `sh(d) = 1 - (d / sigma_share)^alpha` if `d < sigma_share`
/// - `sh(d) = 0` otherwise
///
/// # Arguments
///
/// * `distance` - Distance between two individuals.
/// * `sigma_share` - Sharing radius.
/// * `alpha` - Shape parameter.
///
/// # Returns
///
/// The sharing value in [0, 1].
///
/// # Examples
///
/// ```
/// use genetic_algorithms::niching::sharing::sharing_function;
///
/// let sh = sharing_function(0.5, 1.0, 1.0);
/// assert!((sh - 0.5).abs() < f64::EPSILON);
///
/// let sh = sharing_function(1.5, 1.0, 1.0);
/// assert!((sh - 0.0).abs() < f64::EPSILON);
/// ```
pub fn sharing_function(distance: f64, sigma_share: f64, alpha: f64) -> f64 {
    if distance < sigma_share {
        1.0 - (distance / sigma_share).powf(alpha)
    } else {
        0.0
    }
}

/// Applies fitness sharing to a population's fitness values.
///
/// For each individual `i`, the shared fitness is:
/// `f'(i) = f(i) / niche_count(i)`
///
/// where `niche_count(i) = sum_j(sh(d(i, j)))` over all individuals `j`.
///
/// # Arguments
///
/// * `fitness_values` - Mutable slice of fitness values to be adjusted in-place.
/// * `distances` - A symmetric distance matrix where `distances[i][j]` is the
///   distance between individual `i` and individual `j`.
/// * `sigma_share` - Sharing radius.
/// * `alpha` - Shape parameter for the sharing function.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::niching::sharing::apply_fitness_sharing;
///
/// let mut fitnesses = vec![10.0, 10.0, 10.0];
/// // All individuals are identical (distance 0)
/// let distances = vec![
///     vec![0.0, 0.0, 0.0],
///     vec![0.0, 0.0, 0.0],
///     vec![0.0, 0.0, 0.0],
/// ];
/// apply_fitness_sharing(&mut fitnesses, &distances, 1.0, 1.0);
/// // niche_count for each = 3.0 (sh(0) = 1.0 for each pair)
/// // shared fitness = 10.0 / 3.0
/// for f in &fitnesses {
///     assert!((*f - 10.0 / 3.0).abs() < 1e-10);
/// }
/// ```
pub fn apply_fitness_sharing(
    fitness_values: &mut [f64],
    distances: &[Vec<f64>],
    sigma_share: f64,
    alpha: f64,
) {
    let n = fitness_values.len();
    if n == 0 {
        return;
    }

    let raw_fitnesses: Vec<f64> = fitness_values.to_vec();

    for i in 0..n {
        let mut niche_count = 0.0;
        for j in 0..n {
            let d = if i < distances.len() && j < distances[i].len() {
                distances[i][j]
            } else {
                f64::INFINITY
            };
            niche_count += sharing_function(d, sigma_share, alpha);
        }

        if niche_count > 0.0 {
            fitness_values[i] = raw_fitnesses[i] / niche_count;
        }
    }

    debug!(
        target: "niching_events",
        "Applied fitness sharing to {} individuals with sigma_share={}, alpha={}",
        n,
        sigma_share,
        alpha
    );
}

/// Computes a distance matrix from a slice of chromosomes using a distance function.
///
/// # Arguments
///
/// * `dna_slices` - Slice of DNA slice references.
/// * `distance_fn` - A function that computes distance between two DNA slices.
///
/// # Returns
///
/// A symmetric matrix (Vec of Vec) of distances.
pub fn compute_distance_matrix<G, F>(dna_slices: &[&[G]], distance_fn: F) -> Vec<Vec<f64>>
where
    F: Fn(&[G], &[G]) -> f64,
{
    let n = dna_slices.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let d = distance_fn(dna_slices[i], dna_slices[j]);
            matrix[i][j] = d;
            matrix[j][i] = d;
        }
    }

    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharing_function_within_radius() {
        // d=0.5, sigma=1.0, alpha=1.0 => 1 - 0.5 = 0.5
        let sh = sharing_function(0.5, 1.0, 1.0);
        assert!((sh - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sharing_function_at_zero() {
        // d=0.0 => sh = 1.0
        let sh = sharing_function(0.0, 1.0, 1.0);
        assert!((sh - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sharing_function_outside_radius() {
        let sh = sharing_function(1.5, 1.0, 1.0);
        assert!((sh - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sharing_function_at_boundary() {
        // d == sigma_share => not < sigma_share => 0.0
        let sh = sharing_function(1.0, 1.0, 1.0);
        assert!((sh - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sharing_function_with_alpha() {
        // d=0.5, sigma=1.0, alpha=2.0 => 1 - (0.5)^2 = 1 - 0.25 = 0.75
        let sh = sharing_function(0.5, 1.0, 2.0);
        assert!((sh - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_fitness_sharing_identical() {
        let mut fitnesses = vec![10.0, 10.0, 10.0];
        let distances = vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        apply_fitness_sharing(&mut fitnesses, &distances, 1.0, 1.0);
        for f in &fitnesses {
            assert!((*f - 10.0 / 3.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_apply_fitness_sharing_distant() {
        // All individuals are far apart (distance > sigma_share)
        // niche_count for each = 1.0 (only self at distance 0)
        let mut fitnesses = vec![10.0, 20.0, 30.0];
        let distances = vec![
            vec![0.0, 5.0, 5.0],
            vec![5.0, 0.0, 5.0],
            vec![5.0, 5.0, 0.0],
        ];
        apply_fitness_sharing(&mut fitnesses, &distances, 1.0, 1.0);
        assert!((fitnesses[0] - 10.0).abs() < 1e-10);
        assert!((fitnesses[1] - 20.0).abs() < 1e-10);
        assert!((fitnesses[2] - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_fitness_sharing_empty() {
        let mut fitnesses: Vec<f64> = vec![];
        let distances: Vec<Vec<f64>> = vec![];
        apply_fitness_sharing(&mut fitnesses, &distances, 1.0, 1.0);
        assert!(fitnesses.is_empty());
    }

    #[test]
    fn test_compute_distance_matrix_symmetric() {
        let dna1: Vec<f64> = vec![0.0, 0.0];
        let dna2: Vec<f64> = vec![3.0, 4.0];
        let dna3: Vec<f64> = vec![1.0, 0.0];

        let slices: Vec<&[f64]> = vec![&dna1, &dna2, &dna3];

        let matrix = compute_distance_matrix(&slices, |a, b| {
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f64>()
                .sqrt()
        });

        assert_eq!(matrix.len(), 3);
        // matrix[0][1] should be 5.0 (3-4-5 triangle)
        assert!((matrix[0][1] - 5.0).abs() < 1e-10);
        // Symmetric
        assert!((matrix[0][1] - matrix[1][0]).abs() < 1e-10);
        // Diagonal is 0
        assert!((matrix[0][0] - 0.0).abs() < 1e-10);
    }
}
