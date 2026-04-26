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
