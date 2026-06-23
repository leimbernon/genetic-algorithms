use super::pareto::{constrained_dominates, dominates, dominates_with_directions};
use super::ObjectiveDirection;
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

/// Performs non-dominated sorting on a population.
///
/// Returns a list of fronts, where each front is a vector of indices
/// into the original objectives list. Front 0 is the first (best) Pareto front.
///
/// For populations with 100 or more individuals, the pairwise comparison
/// phase is parallelized via rayon (when compiled with the `parallel`
/// feature on a non-WASM target).
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
///
/// For populations with 100 or more individuals, the pairwise comparison
/// phase is parallelized via rayon (when compiled with the `parallel`
/// feature on a non-WASM target).
pub fn non_dominated_sort_constrained(
    objectives: &[&[f64]],
    violations: &[f64],
    directions: &[ObjectiveDirection],
) -> Vec<Vec<usize>> {
    let n = objectives.len();
    if n == 0 {
        return vec![];
    }

    // Parallel path for large populations
    #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
    if n >= 100 {
        return non_dominated_sort_constrained_parallel(objectives, violations, directions);
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
    F: Fn(&[f64], &[f64]) -> bool + Sync,
{
    let n = objectives.len();
    if n == 0 {
        return vec![];
    }

    // Parallel path for large populations
    #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
    if n >= 100 {
        return non_dominated_sort_inner_parallel(objectives, dom);
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

/// Parallel non-dominated sorting driven by a domination predicate.
///
/// Splits the O(N²) pairwise comparison across rayon threads, then merges
/// results sequentially. The front extraction (O(N²) integer ops) is
/// intentionally sequential — the expensive `dom()` floating-point
/// comparisons are fully parallelized in phase 1.
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
fn non_dominated_sort_inner_parallel<F>(objectives: &[&[f64]], dom: F) -> Vec<Vec<usize>>
where
    F: Fn(&[f64], &[f64]) -> bool + Sync,
{
    let n = objectives.len();

    // Phase 1: parallel pairwise comparison — each thread handles one i
    let results: Vec<(Vec<usize>, Vec<usize>)> = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut dominates = Vec::new();
            let mut dominated_by = Vec::new();
            for j in (i + 1)..n {
                if dom(objectives[i], objectives[j]) {
                    dominates.push(j);
                } else if dom(objectives[j], objectives[i]) {
                    dominated_by.push(j);
                }
            }
            (dominates, dominated_by)
        })
        .collect();

    // Phase 2: sequential merge (O(N²) integer ops, no floating-point)
    // dominated_set[i] = individuals that i dominates (outgoing edges)
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];
    for (i, result) in results.iter().enumerate().take(n) {
        dominated_set[i] = result.0.clone();
    }
    // Cross-thread merge: for each i, add i to dominated_set[j] for each j that dominates i
    for (i, result) in results.iter().enumerate().take(n) {
        for &j in &result.1 {
            dominated_set[j].push(i);
        }
    }
    // Deduplicate dominated_set to avoid double-counting in front extraction
    for set in &mut dominated_set {
        set.sort_unstable();
        set.dedup();
    }
    // Derive domination_count[i] = number of individuals that dominate i (incoming edges)
    // by inverting the dominated_set: if i is in dominated_set[j], then j dominates i.
    let mut domination_count: Vec<usize> = vec![0; n];
    for dominated in dominated_set.iter().take(n) {
        for &i in dominated {
            domination_count[i] += 1;
        }
    }

    // Front extraction (unchanged from sequential)
    let mut fronts: Vec<Vec<usize>> = vec![];
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

/// Parallel constrained non-dominated sorting.
///
/// Same algorithm as [`non_dominated_sort_inner_parallel`] but uses
/// [`constrained_dominates`] for the domination predicate.
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
fn non_dominated_sort_constrained_parallel(
    objectives: &[&[f64]],
    violations: &[f64],
    directions: &[ObjectiveDirection],
) -> Vec<Vec<usize>> {
    let n = objectives.len();

    let results: Vec<(Vec<usize>, Vec<usize>)> = (0..n)
        .into_par_iter()
        .map(|i| {
            let vi = violations.get(i).copied().unwrap_or(0.0);
            let mut dominates = Vec::new();
            let mut dominated_by = Vec::new();
            for j in (i + 1)..n {
                let vj = violations.get(j).copied().unwrap_or(0.0);
                if constrained_dominates(objectives[i], objectives[j], vi, vj, directions) {
                    dominates.push(j);
                } else if constrained_dominates(objectives[j], objectives[i], vj, vi, directions) {
                    dominated_by.push(j);
                }
            }
            (dominates, dominated_by)
        })
        .collect();

    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];
    for (i, result) in results.iter().enumerate().take(n) {
        dominated_set[i] = result.0.clone();
    }
    for (i, result) in results.iter().enumerate().take(n) {
        for &j in &result.1 {
            dominated_set[j].push(i);
        }
    }
    for set in &mut dominated_set {
        set.sort_unstable();
        set.dedup();
    }
    let mut domination_count: Vec<usize> = vec![0; n];
    for dominated in dominated_set.iter().take(n) {
        for &i in dominated {
            domination_count[i] += 1;
        }
    }

    let mut fronts: Vec<Vec<usize>> = vec![];
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
