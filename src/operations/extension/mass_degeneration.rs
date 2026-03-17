//! MassDegeneration extension strategy.
//!
//! Applies multiple rounds of swap mutation to non-elite chromosomes,
//! then marks their fitness as NaN for re-evaluation.

use crate::configuration::ProblemSolving;
use crate::operations::mutation::swap::swap;
use crate::traits::ChromosomeT;
use log::info;

/// Applies mass degeneration: protects elite chromosomes and applies
/// `mutation_rounds` of swap mutation to all others, resetting their fitness.
pub fn mass_degeneration<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    problem_solving: ProblemSolving,
    mutation_rounds: usize,
    elite_count: usize,
) {
    if chromosomes.is_empty() {
        return;
    }

    let elite_count = elite_count.min(chromosomes.len());

    // Sort by fitness (best first)
    match problem_solving {
        ProblemSolving::Maximization => {
            chromosomes.sort_by(|a, b| {
                b.fitness()
                    .partial_cmp(&a.fitness())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
            chromosomes.sort_by(|a, b| {
                a.fitness()
                    .partial_cmp(&b.fitness())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    // Apply mutation rounds to non-elite chromosomes
    for chromosome in chromosomes.iter_mut().skip(elite_count) {
        for _ in 0..mutation_rounds {
            swap(chromosome);
        }
        // Mark fitness as NaN so it gets re-evaluated
        chromosome.set_fitness(f64::NAN);
    }

    info!(
        target = "extension_events",
        method = "mass_degeneration";
        "MassDegeneration applied: {} mutation rounds on {} non-elite chromosomes",
        mutation_rounds,
        chromosomes.len().saturating_sub(elite_count)
    );
}
