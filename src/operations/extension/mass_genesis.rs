//! MassGenesis extension strategy.
//!
//! Trims the population to the 2 best chromosomes. The GA loop handles
//! regrowing the population back to its target size.

use crate::configuration::ProblemSolving;
use crate::traits::ChromosomeT;
use log::info;

/// Applies mass genesis: keeps only the 2 best chromosomes.
///
/// The population will be smaller after this operation; the GA loop handles regrowth.
pub fn mass_genesis<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    problem_solving: ProblemSolving,
) {
    if chromosomes.len() <= 2 {
        return;
    }

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

    chromosomes.truncate(2);

    info!(
        target = "extension_events",
        method = "mass_genesis";
        "MassGenesis applied: population trimmed to 2 best chromosomes"
    );
}
