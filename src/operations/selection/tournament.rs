//! Tournament selection operator.
//!
//! Pairs of randomly chosen individuals compete, and the one with the higher
//! fitness wins. Winners are then paired into mating couples. The tournament
//! runs in parallel using Rayon for large populations.

use crate::traits::ChromosomeT;
use rand::Rng;
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

/// Tournament selection: for each parent slot, two individuals are chosen at
/// random and the one with the higher fitness advances.
///
/// Winners are collected and grouped into N-ary parent groups.
/// The implementation uses Rayon for parallel tournament evaluation.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Desired number of parent groups (clamped to `population / num_parents`).
/// * `_number_of_threads` - Unused; parallelism is managed by Rayon's global pool.
/// * `num_parents` - Number of parents per group (must be >= 2).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::selection::tournament;
/// use genetic_algorithms::chromosomes::Binary;
/// let population: Vec<Binary> = vec![Binary::new(); 10];
/// let pairs = tournament(&population, 5, 1, 2);
/// ```
pub fn tournament<U>(
    chromosomes: &[U],
    couples: usize,
    _number_of_threads: usize,
    num_parents: usize,
) -> Vec<Vec<usize>>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    tournament_impl(chromosomes, couples, num_parents)
}

/// Internal implementation of tournament selection using Rayon parallelism.
fn tournament_impl<U>(chromosomes: &[U], couples: usize, num_parents: usize) -> Vec<Vec<usize>>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    crate::log_debug!(target="selection_events", method="tournament"; "Starting tournament selection");
    let num_parents = num_parents.max(2);
    let couples = if couples * num_parents > chromosomes.len() {
        chromosomes.len() / num_parents
    } else {
        couples
    };

    // Generate all indexes needed for the tournament
    let total_contestants = couples * num_parents;

    // Use rayon to run tournaments in parallel — each iteration picks 2 random contestants
    // and the winner goes to a results vector. We collect num_parents*couples winners and group them.
    #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
    let winners: Vec<usize> = (0..total_contestants)
        .into_par_iter()
        .map(|_| {
            let mut rng = crate::rng::make_rng();
            let index_1 = rng.random_range(0..chromosomes.len());
            let index_2 = rng.random_range(0..chromosomes.len());

            crate::log_trace!(target="selection_events", method="tournament"; "Tournament between {} and {}", index_1, index_2);

            if chromosomes[index_1].fitness() >= chromosomes[index_2].fitness() {
                index_1
            } else {
                index_2
            }
        })
        .collect();
    #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
    let winners: Vec<usize> = (0..total_contestants)
        .map(|_| {
            let mut rng = crate::rng::make_rng();
            let index_1 = rng.random_range(0..chromosomes.len());
            let index_2 = rng.random_range(0..chromosomes.len());

            crate::log_trace!(target="selection_events", method="tournament"; "Tournament between {} and {}", index_1, index_2);

            if chromosomes[index_1].fitness() >= chromosomes[index_2].fitness() {
                index_1
            } else {
                index_2
            }
        })
        .collect();

    // Group winners into N-ary parent groups
    let mut mating = Vec::new();
    for chunk in winners.chunks(num_parents) {
        if chunk.len() == num_parents {
            let group = chunk.to_vec();
            crate::log_trace!(target="selection_events", method="tournament"; "Mating group: {:?}", group);
            mating.push(group);
        }
    }

    crate::log_debug!(target="selection_events", method="tournament"; "Tournament selection finished");
    mating
}
