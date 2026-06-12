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
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::niching::sharing::compute_distance_matrix;
///
/// let a = vec![1.0_f64, 2.0];
/// let b = vec![4.0_f64, 6.0];
/// let matrix = compute_distance_matrix(&[&a, &b], |x, y| {
///     x.iter().zip(y.iter()).map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt()
/// });
/// assert_eq!(matrix.len(), 2);
/// assert!((matrix[0][1] - 5.0).abs() < 1e-9);
/// assert!((matrix[1][0] - 5.0).abs() < 1e-9);
/// ```
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

/// Applies fitness sharing by computing distances on-the-fly from DNA slices,
/// avoiding allocation of an O(n^2) distance matrix.
///
/// This is functionally equivalent to calling [`compute_distance_matrix`]
/// followed by [`apply_fitness_sharing`], but uses O(n) memory instead of O(n^2).
///
/// # Arguments
///
/// * `fitness_values` - Mutable slice of fitness values to be adjusted in-place.
/// * `dna_slices` - Slice of DNA slice references.
/// * `distance_fn` - A function that computes distance between two DNA slices.
/// * `sigma_share` - Sharing radius.
/// * `alpha` - Shape parameter for the sharing function.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::niching::sharing::apply_fitness_sharing_with_dna;
///
/// let a = vec![0.0_f64];
/// let b = vec![10.0_f64];
/// let mut fitness = vec![1.0, 1.0];
/// apply_fitness_sharing_with_dna(&mut fitness, &[&a, &b], |x, y| (x[0] - y[0]).abs(), 5.0, 1.0);
/// assert!(fitness[0] > 0.0);
/// ```
pub fn apply_fitness_sharing_with_dna<G, F>(
    fitness_values: &mut [f64],
    dna_slices: &[&[G]],
    distance_fn: F,
    sigma_share: f64,
    alpha: f64,
) where
    F: Fn(&[G], &[G]) -> f64,
{
    let n = fitness_values.len();
    if n == 0 {
        return;
    }

    let raw_fitnesses: Vec<f64> = fitness_values.to_vec();
    let mut niche_counts = vec![0.0f64; n];

    for i in 0..n {
        for j in 0..n {
            let d = if i < dna_slices.len() && j < dna_slices.len() {
                distance_fn(dna_slices[i], dna_slices[j])
            } else {
                f64::INFINITY
            };
            niche_counts[i] += sharing_function(d, sigma_share, alpha);
        }
    }

    for i in 0..n {
        if niche_counts[i] > 0.0 {
            fitness_values[i] = raw_fitnesses[i] / niche_counts[i];
        }
    }

    debug!(
        target: "niching_events",
        "Applied fitness sharing (with_dna) to {} individuals with sigma_share={}, alpha={}",
        n,
        sigma_share,
        alpha
    );
}
