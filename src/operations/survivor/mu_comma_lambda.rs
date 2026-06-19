//! (mu,lambda) survivor selection strategy.
//!
//! Only offspring (individuals born in the most recent generation) are eligible
//! to survive; all parents are unconditionally discarded. This enforces that
//! every generation is composed entirely of newly created individuals, preventing
//! stagnation from long-lived parents.

pub(crate) use crate::{
    configuration::{LimitConfiguration, ProblemSolving},
    traits::ChromosomeT,
};
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

/// Select survivors using the (mu,lambda) strategy.
///
/// In a (mu,lambda) evolutionary strategy, only the offspring (individuals born
/// in the most recent generation, i.e. those with the maximum age value in the
/// merged population) are eligible to survive. All parents are unconditionally
/// discarded. This enforces that every generation is composed entirely of newly
/// created individuals, preventing stagnation from long-lived parents.
///
/// If fewer offspring exist than `population_size`, all offspring are kept
/// (resulting in a temporarily smaller population). The remaining offspring are
/// then ranked by fitness and truncated to `population_size`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::survivor::mu_comma_lambda;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::{LimitConfiguration, ProblemSolving};
/// let mut population: Vec<Binary> = vec![Binary::new(); 20];
/// let limits = LimitConfiguration { problem_solving: ProblemSolving::Maximization, ..LimitConfiguration::default() };
/// mu_comma_lambda(&mut population, 10, limits);
/// ```
pub fn mu_comma_lambda<U: ChromosomeT>(
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
) {
    crate::log_debug!(target="survivor_events", method="mu_comma_lambda"; "Starting (mu,lambda) survivor selection");

    // Offspring are stamped with `set_age(age)` where `age` is the generation
    // counter (starts at 1). The freshest individuals — the offspring — carry
    // the highest age value in the merged parent+offspring population. Retain
    // only those, discarding all parents born in earlier generations.
    let offspring_age = chromosomes.iter().map(|c| c.age()).max().unwrap_or(0);
    chromosomes.retain(|c| c.age() == offspring_age);
    crate::log_trace!(target="survivor_events", method="mu_comma_lambda"; "Offspring count after parent removal (offspring_age={}): {}", offspring_age, chromosomes.len());

    if chromosomes.len() <= population_size {
        crate::log_debug!(target="survivor_events", method="mu_comma_lambda"; "(mu,lambda) survivor selection finished (all offspring kept)");
        return;
    }

    // Rank offspring by fitness and truncate.
    if limit_configuration.problem_solving != ProblemSolving::FixedFitness {
        #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
        chromosomes.par_sort_unstable_by(|a, b| {
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
        chromosomes.sort_unstable_by(|a, b| {
            b.fitness()
                .partial_cmp(&a.fitness())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        let target = limit_configuration.fitness_target.unwrap_or(0.0);
        #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
        chromosomes.par_sort_unstable_by(|a, b| {
            b.fitness_distance(&target)
                .partial_cmp(&a.fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
        chromosomes.sort_unstable_by(|a, b| {
            b.fitness_distance(&target)
                .partial_cmp(&a.fitness_distance(&target))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

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

    crate::log_debug!(target="survivor_events", method="mu_comma_lambda"; "(mu,lambda) survivor selection finished");
}
