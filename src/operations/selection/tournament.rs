//! Tournament selection operator.
//!
//! Pairs of randomly chosen individuals compete, and the one with the higher
//! fitness wins. Winners are then paired into mating couples. The tournament
//! runs in parallel using Rayon for large populations.

use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;
use rayon::prelude::*;

/// Tournament selection: for each parent slot, two individuals are chosen at
/// random and the one with the higher fitness advances.
///
/// Winners are collected and paired sequentially to form mating couples.
/// The implementation uses Rayon for parallel tournament evaluation.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Desired number of parent pairs (clamped to `population / 2`).
/// * `_number_of_threads` - Unused; parallelism is managed by Rayon's global pool.
pub fn tournament<U>(
    chromosomes: &[U],
    couples: usize,
    _number_of_threads: usize,
) -> Vec<(usize, usize)>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    tournament_impl(chromosomes, couples)
}

/// Internal implementation of tournament selection using Rayon parallelism.
fn tournament_impl<U>(chromosomes: &[U], couples: usize) -> Vec<(usize, usize)>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    debug!(target="selection_events", method="tournament"; "Starting tournament selection");
    let couples = if couples * 2 > chromosomes.len() {
        chromosomes.len() / 2
    } else {
        couples
    };

    // Generate all indexes needed for the tournament
    let total_contestants = couples * 2;

    // Use rayon to run tournaments in parallel — each iteration picks 2 random contestants
    // and the winner goes to a results vector. We collect 2*couples winners and pair them.
    let winners: Vec<usize> = (0..total_contestants)
        .into_par_iter()
        .map(|_| {
            let mut rng = crate::rng::make_rng();
            let index_1 = rng.random_range(0..chromosomes.len());
            let index_2 = rng.random_range(0..chromosomes.len());

            trace!(target="selection_events", method="tournament"; "Tournament between {} and {}", index_1, index_2);

            if chromosomes[index_1].fitness() >= chromosomes[index_2].fitness() {
                index_1
            } else {
                index_2
            }
        })
        .collect();

    // Pair winners into mating pairs
    let mut mating = Vec::new();
    for chunk in winners.chunks(2) {
        if chunk.len() == 2 {
            mating.push((chunk[0], chunk[1]));
            trace!(target="selection_events", method="tournament"; "Mating index 1: {} - index 2: {}", chunk[0], chunk[1]);
        }
    }

    debug!(target="selection_events", method="tournament"; "Tournament selection finished");
    mating
}
