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
        }
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
}
