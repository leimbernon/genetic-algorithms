//! Fitness-based survivor selection.
//!
//! Retains the best individuals (by fitness) after parents and offspring have
//! been merged. Sorting direction is determined by the [`ProblemSolving`] mode:
//! maximization keeps the highest, minimization the lowest, and fixed-fitness
//! keeps those closest to the target.

pub(crate) use crate::{
    configuration::{LimitConfiguration, ProblemSolving},
    traits::ChromosomeT,
};
use log::{debug, trace};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

/// Fitness-based survivor selection: sorts the combined population by fitness
/// and truncates to `population_size`.
///
/// - **Maximization** — highest fitness individuals survive.
/// - **Minimization** — lowest fitness individuals survive.
/// - **FixedFitness** — individuals closest to `fitness_target` survive.
///
/// # Arguments
///
/// * `chromosomes` - Combined parents + offspring (modified in place).
/// * `population_size` - Desired population size after selection.
/// * `limit_configuration` - Controls sorting direction and optional target.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::survivor::fitness_based;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::{LimitConfiguration, ProblemSolving};
/// let mut population: Vec<Binary> = vec![Binary::new(); 20];
/// let limits = LimitConfiguration::default().with_problem_solving(ProblemSolving::Maximization);
/// fitness_based(&mut population, 10, limits);
/// ```
pub fn fitness_based<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
    debug!(target="survivor_events", method="fitness_based"; "Starting fitness based survivor method");
    if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
        //We sort the chromosomes by their fitness if there is not a fixed fitness problem
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
        //We sort the chromosomes by their distance with the fitness target in a fixed fitness problem
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

    // Drop surplus individuals in a single bulk operation instead of
    // one-by-one Vec::remove calls (which are O(N) each).
    trace!(target="survivor_events", method="fitness_based"; "Chromosomes length {} - population size {}", chromosomes.len(), population_size);
    if chromosomes.len() > population_size {
        match limit_configuration.problem_solving {
            // Sorted descending: best (highest) at front, worst at tail.
            ProblemSolving::Maximization => {
                chromosomes.truncate(population_size);
            }
            // Sorted descending: worst (highest) at front.
            // Drain the excess from the front in one O(N) shift.
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                let excess = chromosomes.len() - population_size;
                chromosomes.drain(0..excess);
            }
        }
    }

    debug!(target="survivor_events", method="fitness_based"; "Fitness based survivor method finished");
}
