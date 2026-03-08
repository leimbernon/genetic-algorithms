use crate::nsga2::configuration::ObjectiveDirection;
use crate::traits::ChromosomeT;

/// Wrapper around a chromosome that holds multi-objective evaluation results.
///
/// Stores the objective values, the non-domination rank and crowding distance
/// assigned during NSGA-II sorting.
#[derive(Debug, Clone)]
pub struct ParetoIndividual<U>
where
    U: ChromosomeT,
{
    /// The underlying chromosome.
    pub chromosome: U,
    /// Objective function values (one per objective).
    pub objectives: Vec<f64>,
    /// Non-domination rank (0 = first Pareto front).
    pub rank: usize,
    /// Crowding distance for diversity preservation.
    pub crowding_distance: f64,
    /// Sum of constraint violations (0.0 = feasible, > 0.0 = infeasible).
    pub constraint_violation: f64,
}

impl<U> ParetoIndividual<U>
where
    U: ChromosomeT,
{
    /// Creates a new `ParetoIndividual` wrapping the given chromosome.
    pub fn new(chromosome: U, objectives: Vec<f64>) -> Self {
        ParetoIndividual {
            chromosome,
            objectives,
            rank: 0,
            crowding_distance: 0.0,
            constraint_violation: 0.0,
        }
    }

    /// Returns `true` if this individual is feasible (no constraint violations).
    pub fn is_feasible(&self) -> bool {
        self.constraint_violation <= 0.0
    }
}

/// A collection of individuals on a Pareto front.
#[derive(Debug, Clone)]
pub struct ParetoFront<U>
where
    U: ChromosomeT,
{
    /// The individuals in this front.
    pub individuals: Vec<ParetoIndividual<U>>,
}

impl<U> ParetoFront<U>
where
    U: ChromosomeT,
{
    /// Creates a new `ParetoFront` from the given individuals.
    pub fn new(individuals: Vec<ParetoIndividual<U>>) -> Self {
        ParetoFront { individuals }
    }

    /// Returns the number of individuals in the front.
    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    /// Returns `true` if the front contains no individuals.
    pub fn is_empty(&self) -> bool {
        self.individuals.is_empty()
    }
}

/// Returns `true` if individual `a` dominates individual `b` (all-objectives minimization).
///
/// `a` dominates `b` if `a` is no worse on all objectives and strictly better on at least one.
pub fn dominates(a: &[f64], b: &[f64]) -> bool {
    let mut at_least_one_better = false;
    for (ai, bi) in a.iter().zip(b.iter()) {
        if ai > bi {
            return false;
        }
        if ai < bi {
            at_least_one_better = true;
        }
    }
    at_least_one_better
}

/// Returns `true` if individual `a` dominates individual `b` respecting per-objective directions.
///
/// For `Minimize` objectives, lower is better. For `Maximize` objectives, higher is better.
/// `a` dominates `b` if `a` is no worse on all objectives and strictly better on at least one.
///
/// If `directions` is empty or shorter than the objectives, the missing entries default
/// to `Minimize`.
pub fn dominates_with_directions(a: &[f64], b: &[f64], directions: &[ObjectiveDirection]) -> bool {
    let mut at_least_one_better = false;
    for (idx, (ai, bi)) in a.iter().zip(b.iter()).enumerate() {
        let dir = directions
            .get(idx)
            .copied()
            .unwrap_or(ObjectiveDirection::Minimize);
        let (better, worse) = match dir {
            ObjectiveDirection::Minimize => (ai < bi, ai > bi),
            ObjectiveDirection::Maximize => (ai > bi, ai < bi),
        };
        if worse {
            return false;
        }
        if better {
            at_least_one_better = true;
        }
    }
    at_least_one_better
}

/// Returns `true` if `a` dominates `b` using constrained-domination.
///
/// Constrained-domination rules (Deb, 2002):
/// 1. A feasible solution dominates any infeasible solution.
/// 2. Among two infeasible solutions, the one with smaller total constraint violation is preferred.
/// 3. Among two feasible solutions, standard Pareto dominance applies.
pub fn constrained_dominates(
    a_obj: &[f64],
    b_obj: &[f64],
    a_violation: f64,
    b_violation: f64,
    directions: &[ObjectiveDirection],
) -> bool {
    let a_feasible = a_violation <= 0.0;
    let b_feasible = b_violation <= 0.0;

    match (a_feasible, b_feasible) {
        // Both feasible: standard Pareto dominance with directions
        (true, true) => dominates_with_directions(a_obj, b_obj, directions),
        // a feasible, b infeasible: a dominates
        (true, false) => true,
        // a infeasible, b feasible: a does NOT dominate
        (false, true) => false,
        // Both infeasible: a dominates if it has strictly less violation
        (false, false) => a_violation < b_violation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dominates_clear() {
        assert!(dominates(&[1.0, 1.0], &[2.0, 2.0]));
    }

    #[test]
    fn test_dominates_equal() {
        assert!(!dominates(&[1.0, 1.0], &[1.0, 1.0]));
    }

    #[test]
    fn test_dominates_partial() {
        // a is better on first, equal on second
        assert!(dominates(&[1.0, 2.0], &[2.0, 2.0]));
    }

    #[test]
    fn test_dominates_incomparable() {
        // Neither dominates: a better on first, worse on second
        assert!(!dominates(&[1.0, 3.0], &[2.0, 2.0]));
    }

    #[test]
    fn test_dominates_reversed() {
        assert!(!dominates(&[2.0, 2.0], &[1.0, 1.0]));
    }

    #[test]
    fn test_pareto_front_len() {
        let front: ParetoFront<crate::chromosomes::Binary> = ParetoFront::new(vec![]);
        assert_eq!(front.len(), 0);
        assert!(front.is_empty());
    }

    // --- Tests for dominates_with_directions ---

    #[test]
    fn test_dominates_with_directions_all_minimize() {
        let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Minimize];
        assert!(dominates_with_directions(&[1.0, 1.0], &[2.0, 2.0], &dirs));
        assert!(!dominates_with_directions(&[2.0, 2.0], &[1.0, 1.0], &dirs));
    }

    #[test]
    fn test_dominates_with_directions_all_maximize() {
        let dirs = vec![ObjectiveDirection::Maximize, ObjectiveDirection::Maximize];
        // Higher is better for maximize
        assert!(dominates_with_directions(&[3.0, 3.0], &[1.0, 1.0], &dirs));
        assert!(!dominates_with_directions(&[1.0, 1.0], &[3.0, 3.0], &dirs));
    }

    #[test]
    fn test_dominates_with_directions_mixed() {
        let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Maximize];
        // a = [1.0, 3.0]: obj0 minimize → 1 < 2 (better), obj1 maximize → 3 > 1 (better)
        assert!(dominates_with_directions(&[1.0, 3.0], &[2.0, 1.0], &dirs));
        // a = [2.0, 1.0]: obj0 minimize → 2 > 1 (worse)
        assert!(!dominates_with_directions(&[2.0, 1.0], &[1.0, 3.0], &dirs));
    }

    #[test]
    fn test_dominates_with_directions_incomparable_mixed() {
        let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Maximize];
        // a = [1.0, 1.0], b = [2.0, 3.0]
        // obj0: 1 < 2 (a better), obj1: 1 < 3 (b better since maximize)
        assert!(!dominates_with_directions(&[1.0, 1.0], &[2.0, 3.0], &dirs));
        assert!(!dominates_with_directions(&[2.0, 3.0], &[1.0, 1.0], &dirs));
    }

    #[test]
    fn test_dominates_with_empty_directions_defaults_to_minimize() {
        // Empty directions should behave like all-minimize
        assert!(dominates_with_directions(&[1.0, 1.0], &[2.0, 2.0], &[]));
        assert!(!dominates_with_directions(&[2.0, 2.0], &[1.0, 1.0], &[]));
    }

    // --- Tests for constrained_dominates ---

    #[test]
    fn test_constrained_dominates_both_feasible() {
        let dirs = vec![ObjectiveDirection::Minimize, ObjectiveDirection::Minimize];
        // Both feasible → standard dominance
        assert!(constrained_dominates(
            &[1.0, 1.0],
            &[2.0, 2.0],
            0.0,
            0.0,
            &dirs
        ));
        assert!(!constrained_dominates(
            &[2.0, 2.0],
            &[1.0, 1.0],
            0.0,
            0.0,
            &dirs
        ));
    }

    #[test]
    fn test_constrained_dominates_feasible_beats_infeasible() {
        let dirs = vec![ObjectiveDirection::Minimize];
        // a feasible, b infeasible → a dominates regardless of objectives
        assert!(constrained_dominates(&[100.0], &[1.0], 0.0, 5.0, &dirs));
    }

    #[test]
    fn test_constrained_dominates_infeasible_does_not_beat_feasible() {
        let dirs = vec![ObjectiveDirection::Minimize];
        // a infeasible, b feasible → a does NOT dominate
        assert!(!constrained_dominates(&[1.0], &[100.0], 5.0, 0.0, &dirs));
    }

    #[test]
    fn test_constrained_dominates_both_infeasible_less_violation_wins() {
        let dirs = vec![ObjectiveDirection::Minimize];
        // Both infeasible: a has violation 2.0 < b's 5.0 → a dominates
        assert!(constrained_dominates(&[100.0], &[1.0], 2.0, 5.0, &dirs));
        assert!(!constrained_dominates(&[1.0], &[100.0], 5.0, 2.0, &dirs));
    }

    #[test]
    fn test_constrained_dominates_both_infeasible_equal_violation() {
        let dirs = vec![ObjectiveDirection::Minimize];
        // Both infeasible with same violation → neither dominates
        assert!(!constrained_dominates(&[1.0], &[2.0], 3.0, 3.0, &dirs));
    }

    #[test]
    fn test_pareto_individual_is_feasible() {
        let mut ind = ParetoIndividual::new(crate::chromosomes::Binary::new(), vec![1.0]);
        assert!(ind.is_feasible());
        ind.constraint_violation = 0.5;
        assert!(!ind.is_feasible());
    }
}
