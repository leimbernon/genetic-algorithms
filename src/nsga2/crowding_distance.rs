/// Assigns crowding distance to individuals within a single Pareto front.
///
/// For each objective, the front is sorted by that objective value. Boundary
/// individuals (smallest and largest for each objective) receive `f64::INFINITY`.
/// Interior individuals receive the sum of normalized neighbor differences
/// across all objectives.
///
/// # Arguments
///
/// * `objectives` - A slice of objective vectors, one per individual **in this front**.
/// * `crowding` - Mutable slice (same length as `objectives`) to fill with crowding distance values.
pub fn assign_crowding_distance(objectives: &[&[f64]], crowding: &mut [f64]) {
    let n = objectives.len();
    if n == 0 {
        return;
    }

    // Reset
    for c in crowding.iter_mut() {
        *c = 0.0;
    }

    if n <= 2 {
        for c in crowding.iter_mut() {
            *c = f64::INFINITY;
        }
        return;
    }

    let num_objectives = objectives[0].len();

    for m in 0..num_objectives {
        // Extract values for objective m
        let obj_col: Vec<f64> = objectives.iter().map(|o| o[m]).collect();

        // Sort indices by objective m
        let mut sorted_indices: Vec<usize> = (0..n).collect();
        sorted_indices.sort_by(|&a, &b| {
            obj_col[a]
                .partial_cmp(&obj_col[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Boundary individuals get infinity
        crowding[sorted_indices[0]] = f64::INFINITY;
        crowding[sorted_indices[n - 1]] = f64::INFINITY;

        // Objective range for normalization
        let obj_min = obj_col[sorted_indices[0]];
        let obj_max = obj_col[sorted_indices[n - 1]];
        let range = obj_max - obj_min;

        if range < f64::EPSILON {
            continue;
        }

        // Interior individuals
        for k in 1..(n - 1) {
            let prev = sorted_indices[k - 1];
            let next = sorted_indices[k + 1];
            let idx = sorted_indices[k];
            if crowding[idx].is_finite() {
                crowding[idx] += (obj_col[next] - obj_col[prev]) / range;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crowding_distance_two_individuals() {
        let objectives: Vec<Vec<f64>> = vec![vec![1.0, 2.0], vec![3.0, 1.0]];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let mut crowding = vec![0.0; 2];
        assign_crowding_distance(&refs, &mut crowding);
        assert!(crowding[0].is_infinite());
        assert!(crowding[1].is_infinite());
    }

    #[test]
    fn test_crowding_distance_three_individuals() {
        let objectives: Vec<Vec<f64>> = vec![vec![1.0, 4.0], vec![2.0, 2.0], vec![4.0, 1.0]];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let mut crowding = vec![0.0; 3];
        assign_crowding_distance(&refs, &mut crowding);
        // Boundary individuals get infinity
        assert!(crowding[0].is_infinite());
        assert!(crowding[2].is_infinite());
        // Middle individual gets finite value
        assert!(crowding[1].is_finite());
        assert!(crowding[1] > 0.0);
    }

    #[test]
    fn test_crowding_distance_empty() {
        let objectives: Vec<&[f64]> = vec![];
        let mut crowding: Vec<f64> = vec![];
        assign_crowding_distance(&objectives, &mut crowding);
        assert!(crowding.is_empty());
    }

    #[test]
    fn test_crowding_distance_same_values() {
        // All individuals have the same objectives
        let objectives: Vec<Vec<f64>> = vec![vec![1.0, 1.0], vec![1.0, 1.0], vec![1.0, 1.0]];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let mut crowding = vec![0.0; 3];
        assign_crowding_distance(&refs, &mut crowding);
        // Boundary individuals still get infinity
        assert!(crowding[0].is_infinite());
        assert!(crowding[2].is_infinite());
        // Middle one stays at 0 since range is 0
    }
}
