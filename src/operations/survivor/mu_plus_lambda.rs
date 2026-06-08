//! (mu+lambda) survivor selection strategy.
//!
//! All individuals — both the *mu* parents and the *lambda* offspring —
//! compete together for survival. The best `population_size` individuals
//! are kept based on fitness. This is the standard approach in
//! evolution-strategy literature and is functionally identical to
//! fitness-based selection.

pub(crate) use crate::{
    configuration::{LimitConfiguration, ProblemSolving},
    traits::ChromosomeT,
};
use log::{debug, trace};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

/// Select survivors using the (mu+lambda) strategy.
///
/// In a (mu+lambda) evolutionary strategy, all individuals -- both the mu parents
/// and the lambda offspring -- compete together for survival. The best
/// `population_size` individuals are kept based on fitness.
///
/// This is functionally identical to fitness-based survivor selection. The
/// semantic distinction is that (mu+lambda) explicitly frames the competition as
/// "parents + offspring together", which is the convention in evolution-strategy
/// literature.
pub fn mu_plus_lambda<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
    debug!(target="survivor_events", method="mu_plus_lambda"; "Starting (mu+lambda) survivor selection");
    if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
        #[cfg(not(target_arch = "wasm32"))]
        chromosomes.par_sort_unstable_by(|a, b| {
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        #[cfg(target_arch = "wasm32")]
        chromosomes.sort_unstable_by(|a, b| {
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        let target = limit_configuration.fitness_target.unwrap_or(0.0);
        #[cfg(not(target_arch = "wasm32"))]
        chromosomes.par_sort_unstable_by(|a, b| {
            b.fitness_distance(&target)
                .partial_cmp(&a.fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        #[cfg(target_arch = "wasm32")]
        chromosomes.sort_unstable_by(|a, b| {
            b.fitness_distance(&target)
                .partial_cmp(&a.fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    trace!(target="survivor_events", method="mu_plus_lambda"; "Chromosomes length {} - population size {}", chromosomes.len(), population_size);
    if chromosomes.len() > population_size {
        match limit_configuration.problem_solving {
            ProblemSolving::Maximization => {
                chromosomes.truncate(population_size);
            }
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                let excess = chromosomes.len() - population_size;
                chromosomes.drain(0..excess);
            }
        }
    }

    debug!(target="survivor_events", method="mu_plus_lambda"; "(mu+lambda) survivor selection finished");
}
