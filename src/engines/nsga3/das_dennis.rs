//! Das-Dennis simplex lattice generator for NSGA-III reference points.
//!
//! Generates `C(p + M - 1, M - 1)` uniformly-spaced points on the unit
//! (M-1)-simplex by enumerating all non-negative integer vectors
//! `(n_1, ..., n_M)` with `n_1 + ... + n_M = p`, then normalizing each to
//! `(n_1/p, ..., n_M/p)`.
//!
//! Reference: Das & Dennis 1998; used in Deb & Jain 2014 for NSGA-III.

/// Generates Das-Dennis reference points for `num_objectives` objectives with
/// subdivision count `p`.
///
/// Produces exactly `C(p + num_objectives - 1, num_objectives - 1)` points,
/// each a vector of length `num_objectives` whose components sum to 1.0.
///
/// # Edge cases
///
/// - `num_objectives == 0` returns an empty vector.
/// - `num_objectives == 1` returns `vec![vec![1.0]]` regardless of `p`.
/// - `p == 0` returns `vec![vec![0.0; num_objectives]]` (a single all-zero point).
pub fn generate_das_dennis(num_objectives: usize, p: usize) -> Vec<Vec<f64>> {
    if num_objectives == 0 {
        return Vec::new();
    }
    if num_objectives == 1 {
        // Single objective: only one point, weight = 1.0 (or 0.0 if p == 0).
        let v = if p == 0 { 0.0 } else { 1.0 };
        return vec![vec![v]];
    }
    let mut result: Vec<Vec<f64>> = Vec::new();
    let mut current: Vec<usize> = vec![0; num_objectives];
    enumerate_partitions(p, num_objectives, 0, p, &mut current, &mut result);
    result
}

fn enumerate_partitions(
    total: usize,
    m: usize,
    dim: usize,
    remaining: usize,
    current: &mut Vec<usize>,
    result: &mut Vec<Vec<f64>>,
) {
    if dim == m - 1 {
        current[dim] = remaining;
        let denom = if total == 0 { 1.0 } else { total as f64 };
        result.push(current.iter().map(|&n| n as f64 / denom).collect());
        return;
    }
    for v in 0..=remaining {
        current[dim] = v;
        enumerate_partitions(total, m, dim + 1, remaining - v, current, result);
    }
}
