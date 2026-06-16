//! Extracted from src/engines/ga.rs in phase 69-04 — stopping criteria check (limit_reached).

use super::*;

/// Checks termination limits according to `LimitConfiguration`.
///
/// - For Minimization: stops when any chromosome has fitness within 1e-9 of `0.0`.
/// - For Maximization: stops when any chromosome has fitness >= `fitness_target` (if set).
/// - For FixedFitness: stops when any chromosome has fitness within a relative epsilon of `fitness_target`.
///
/// Epsilon comparison is used instead of exact `==` to handle floating-point fitness
/// functions that approach but never exactly reach the target value.
pub(crate) fn limit_reached<U>(limit: LimitConfiguration, chromosomes: &[U]) -> bool
where
    U: LinearChromosome,
{
    let mut result = false;

    if limit.problem_solving == ProblemSolving::Minimization {
        //If the problem-solving is minimization, fitness must be near 0
        for chromosome in chromosomes {
            if chromosome.fitness().abs() < 1e-9 {
                result = true;
                break;
            }
        }
    } else if limit.problem_solving == ProblemSolving::Maximization {
        //If the problem-solving is maximization, check against fitness_target if set
        if let Some(target) = limit.fitness_target {
            for chromosome in chromosomes {
                if chromosome.fitness() >= target {
                    result = true;
                    break;
                }
            }
        }
    } else if limit.problem_solving == ProblemSolving::FixedFitness {
        //If the problem-solving is a fixed fitness, use epsilon comparison
        if let Some(target) = limit.fitness_target {
            for chromosome in chromosomes {
                // Relative epsilon: tolerates floating-point imprecision near target
                if (chromosome.fitness() - target).abs() <= target.abs() * 1e-9 + 1e-12 {
                    result = true;
                    break;
                }
            }
        }
    }

    result
}
