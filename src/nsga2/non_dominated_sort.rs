use super::pareto::dominates;

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
pub fn non_dominated_sort(objectives: &[Vec<f64>]) -> Vec<Vec<usize>> {
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
            if dominates(&objectives[i], &objectives[j]) {
                dominated_set[i].push(j);
                domination_count[j] += 1;
            } else if dominates(&objectives[j], &objectives[i]) {
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
        let objectives = vec![vec![1.0, 3.0], vec![2.0, 2.0], vec![3.0, 1.0]];
        let fronts = non_dominated_sort(&objectives);
        assert_eq!(fronts.len(), 1);
        assert_eq!(fronts[0].len(), 3);
    }

    #[test]
    fn test_non_dominated_sort_two_fronts() {
        // [1,4], [2,2], [4,1] are mutually non-dominated (front 0).
        // [3,3] is dominated by [2,2] (front 1).
        let objectives = vec![
            vec![1.0, 4.0], // front 0
            vec![3.0, 3.0], // front 1 — dominated by [2,2]
            vec![2.0, 2.0], // front 0
            vec![4.0, 1.0], // front 0
        ];
        let fronts = non_dominated_sort(&objectives);
        assert_eq!(fronts.len(), 2);
        assert_eq!(fronts[0].len(), 3);
        assert_eq!(fronts[1].len(), 1);
        assert!(fronts[1].contains(&1));
    }

    #[test]
    fn test_non_dominated_sort_empty() {
        let objectives: Vec<Vec<f64>> = vec![];
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
}
