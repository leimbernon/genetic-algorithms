use super::pareto::{constrained_dominates, dominates, dominates_with_directions};
use crate::nsga2::configuration::ObjectiveDirection;

/// Performs non-dominated sorting on a population.
///
/// Returns a list of fronts, where each front is a vector of indices
/// into the original objectives list. Front 0 is the first (best) Pareto front.
///
/// # Arguments
///
/// * `objectives` - A slice of objective vectors, one per individual.
///
/// # Returns
///
/// `Vec<Vec<usize>>` — fronts ordered by dominance rank.
pub fn non_dominated_sort(objectives: &[&[f64]]) -> Vec<Vec<usize>> {
    non_dominated_sort_inner(objectives, dominates)
}

/// Performs non-dominated sorting with per-objective directions.
///
/// Behaves like [`non_dominated_sort`] but respects the specified optimization
/// direction for each objective.
pub fn non_dominated_sort_with_directions(
    objectives: &[&[f64]],
    directions: &[ObjectiveDirection],
) -> Vec<Vec<usize>> {
    non_dominated_sort_inner(objectives, |a, b| {
        dominates_with_directions(a, b, directions)
    })
}

/// Performs non-dominated sorting with constrained-domination.
///
/// Uses Deb's constrained-domination principle:
/// 1. Feasible dominates infeasible.
/// 2. Among infeasible, lower violation wins.
/// 3. Among feasible, standard Pareto dominance with directions.
pub fn non_dominated_sort_constrained(
    objectives: &[&[f64]],
    violations: &[f64],
    directions: &[ObjectiveDirection],
) -> Vec<Vec<usize>> {
    let n = objectives.len();
    if n == 0 {
        return vec![];
    }

    let mut domination_count: Vec<usize> = vec![0; n];
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];
    let mut fronts: Vec<Vec<usize>> = vec![];

    for i in 0..n {
        for j in (i + 1)..n {
            let vi = violations.get(i).copied().unwrap_or(0.0);
            let vj = violations.get(j).copied().unwrap_or(0.0);
            if constrained_dominates(objectives[i], objectives[j], vi, vj, directions) {
                dominated_set[i].push(j);
                domination_count[j] += 1;
            } else if constrained_dominates(objectives[j], objectives[i], vj, vi, directions) {
                dominated_set[j].push(i);
                domination_count[i] += 1;
            }
        }
    }

    let mut current_front: Vec<usize> = (0..n).filter(|&i| domination_count[i] == 0).collect();

    while !current_front.is_empty() {
        let mut next_front: Vec<usize> = vec![];
        for &i in &current_front {
            for &j in &dominated_set[i] {
                domination_count[j] -= 1;
                if domination_count[j] == 0 {
                    next_front.push(j);
                }
            }
        }
        fronts.push(current_front);
        current_front = next_front;
    }

    fronts
}

/// Generic non-dominated sorting driven by a domination predicate.
fn non_dominated_sort_inner<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool,
{
    let n = objectives.len();
    if n == 0 {
        return vec![];
    }

    let mut domination_count: Vec<usize> = vec![0; n];
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];
    let mut fronts: Vec<Vec<usize>> = vec![];

    // Build domination relationships
    for i in 0..n {
        for j in (i + 1)..n {
            if dom(objectives[i], objectives[j]) {
                dominated_set[i].push(j);
                domination_count[j] += 1;
            } else if dom(objectives[j], objectives[i]) {
                dominated_set[j].push(i);
                domination_count[i] += 1;
            }
        }
    }

    // First front: individuals with domination_count == 0
    let mut current_front: Vec<usize> = (0..n).filter(|&i| domination_count[i] == 0).collect();

    while !current_front.is_empty() {
        let mut next_front: Vec<usize> = vec![];
        for &i in &current_front {
            for &j in &dominated_set[i] {
                domination_count[j] -= 1;
                if domination_count[j] == 0 {
                    next_front.push(j);
                }
            }
        }
        fronts.push(current_front);
        current_front = next_front;
    }

    fronts
}

/// Assigns non-domination ranks to a mutable slice of ranks, given the fronts.
///
/// # Arguments
///
/// * `ranks` - Mutable slice to fill with rank values.
/// * `fronts` - Fronts as returned by [`non_dominated_sort`].
pub fn assign_ranks(ranks: &mut [usize], fronts: &[Vec<usize>]) {
    for (rank, front) in fronts.iter().enumerate() {
        for &idx in front {
            if idx < ranks.len() {
                ranks[idx] = rank;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_dominated_sort_single_front() {
        // Three non-dominated points
        let objectives: Vec<Vec<f64>> = vec![vec![1.0, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let fronts = non_dominated_sort(&refs);
        assert_eq!(fronts.len(), 1);
        assert_eq!(fronts[0].len(), 3);
    }

    #[test]
    fn test_non_dominated_sort_two_fronts() {
        // [1,4], [2,2], [4,1] are mutually non-dominated (front 0).
        // [3,3] is dominated by [2,2] (front 1).
        let objectives: Vec<Vec<f64>> = vec![
            vec![1.0, 4.0], // front 0
            vec![3.0, 3.0], // front 1 — dominated by [2,2]
            vec![2.0, 2.0], // front 0
            vec![4.0, 1.0], // front 0
        ];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let fronts = non_dominated_sort(&refs);
        assert_eq!(fronts.len(), 2);
        assert_eq!(fronts[0].len(), 3);
        assert_eq!(fronts[1].len(), 1);
        assert!(fronts[1].contains(&1));
    }

    #[test]
    fn test_non_dominated_sort_empty() {
        let objectives: Vec<&[f64]> = vec![];
        let fronts = non_dominated_sort(&objectives);
        assert!(fronts.is_empty());
    }

    #[test]
    fn test_assign_ranks() {
        let fronts = vec![vec![0, 2, 3], vec![1]];
        let mut ranks = vec![0; 4];
        assign_ranks(&mut ranks, &fronts);
        assert_eq!(ranks, vec![0, 1, 0, 0]);
    }

    // --- Tests for direction-aware sorting ---

    #[test]
    fn test_non_dominated_sort_with_maximize_directions() {
        let dirs = vec![ObjectiveDirection::Maximize, ObjectiveDirection::Maximize];
        // With maximize: [3,3] dominates [1,1] and [2,2]
        let objectives: Vec<Vec<f64>> = vec![
            vec![1.0, 1.0], // front 2
            vec![3.0, 3.0], // front 0
            vec![2.0, 2.0], // front 1
        ];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let fronts = non_dominated_sort_with_directions(&refs, &dirs);
        assert_eq!(fronts.len(), 3);
        assert!(fronts[0].contains(&1)); // [3,3] is front 0
        assert!(fronts[1].contains(&2)); // [2,2] is front 1
        assert!(fronts[2].contains(&0)); // [1,1] is front 2
    }

    #[test]
    fn test_non_dominated_sort_mixed_directions() {
        // obj0: minimize, obj1: maximize
        let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Maximize];
        let objectives: Vec<Vec<f64>> = vec![
            vec![1.0, 3.0], // front 0: low obj0, high obj1 → ideal
            vec![2.0, 1.0], // front 1: both worse than [1,3]
            vec![3.0, 2.0], // front 1: obj0 worse, obj1 worse than [1,3]
        ];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let fronts = non_dominated_sort_with_directions(&refs, &dirs);
        assert_eq!(fronts.len(), 2);
        assert!(fronts[0].contains(&0));
    }

    // --- Tests for constrained sorting ---

    #[test]
    fn test_non_dominated_sort_constrained_feasible_dominates_infeasible() {
        let dirs = vec![ObjectiveDirection::Minimize];
        // Individual 0: obj=[100.0], violation=0 (feasible)
        // Individual 1: obj=[1.0], violation=5.0 (infeasible)
        let objectives: Vec<Vec<f64>> = vec![vec![100.0], vec![1.0]];
        let violations = vec![0.0, 5.0];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let fronts = non_dominated_sort_constrained(&refs, &violations, &dirs);
        // Feasible individual should be in front 0
        assert_eq!(fronts.len(), 2);
        assert!(fronts[0].contains(&0));
        assert!(fronts[1].contains(&1));
    }

    #[test]
    fn test_non_dominated_sort_constrained_less_violation_preferred() {
        let dirs = vec![ObjectiveDirection::Minimize];
        // Both infeasible
        let objectives: Vec<Vec<f64>> = vec![vec![100.0], vec![1.0]];
        let violations = vec![2.0, 5.0];
        let refs: Vec<&[f64]> = objectives.iter().map(|v| v.as_slice()).collect();
        let fronts = non_dominated_sort_constrained(&refs, &violations, &dirs);
        // Individual 0 has less violation → should be in front 0
        assert_eq!(fronts.len(), 2);
        assert!(fronts[0].contains(&0));
        assert!(fronts[1].contains(&1));
    }
}
