//! Clearing selection operator.
//!
//! Implements a diversity-promoting selection strategy that prevents any single
//! niche from dominating the mating pool. The algorithm:
//!
//! 1. Sorts individuals by fitness (descending) to process the best first.
//! 2. Iterates through sorted individuals: the first unclaimed individual in a
//!    fitness-space niche becomes its *winner*. All others within `niche_radius`
//!    of that winner are marked as cleared (ineligible).
//! 3. Forms random parent pairs from the eligible pool (niche winners plus any
//!    individual not within any winner's niche radius).
//!
//! Distance between individuals A and B is `|f_a - f_b|` (fitness-space),
//! making the operator generic across all chromosome types.

use crate::traits::ChromosomeT;
use log::{debug, trace};

/// Clearing selection: builds an eligible pool by removing niche-dominated
/// individuals, then pairs eligible individuals randomly.
///
/// # Arguments
///
/// * `chromosomes` - Population slice to select from.
/// * `niche_radius` - Fitness-space radius; individuals within this distance of
///   a niche winner are ineligible for reproduction.
pub fn clearing_selection<U: ChromosomeT>(
    chromosomes: &[U],
    niche_radius: f64,
) -> Vec<(usize, usize)> {
    debug!(target="selection_events", method="clearing"; "Starting clearing selection with niche_radius={}", niche_radius);

    let n = chromosomes.len();

    // Build a list of (original_index, fitness) sorted by fitness descending.
    // Processing highest-fitness individuals first ensures the best individual
    // in each niche becomes its winner (D-02).
    let mut sorted: Vec<(usize, f64)> = chromosomes
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.fitness()))
        .collect();
    sorted.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // cleared[i] == true means individual i has been removed from the pool.
    let mut cleared = vec![false; n];

    // Identify niche winners and clear dominated individuals.
    for rank in 0..sorted.len() {
        let (winner_idx, winner_fitness) = sorted[rank];
        if cleared[winner_idx] {
            // Already dominated by a previous winner — skip.
            continue;
        }
        // This individual is a niche winner. Clear all uncleared individuals
        // (ranked below it, not yet winners) within niche_radius.
        trace!(target="selection_events", method="clearing"; "Niche winner: index={} fitness={}", winner_idx, winner_fitness);
        for &(candidate_idx, candidate_fitness) in &sorted[(rank + 1)..] {
            if !cleared[candidate_idx]
                && (winner_fitness - candidate_fitness).abs() <= niche_radius
            {
                cleared[candidate_idx] = true;
                trace!(target="selection_events", method="clearing"; "Cleared: index={} fitness={}", candidate_idx, candidate_fitness);
            }
        }
    }

    // Collect eligible pool (original indices of unclaimed individuals).
    let eligible: Vec<usize> = (0..n).filter(|&i| !cleared[i]).collect();

    trace!(target="selection_events", method="clearing"; "Eligible pool size: {}", eligible.len());

    // Pair eligible individuals randomly (Fisher-Yates partial shuffle).
    let mut rng = crate::rng::make_rng();
    let mut pool = eligible;
    let mut remaining = pool.len();
    let pair_count = remaining / 2;
    let mut mating = Vec::with_capacity(pair_count);

    use rand::Rng;
    while remaining >= 2 {
        let r1 = rng.random_range(0..remaining);
        let idx1 = pool[r1];
        remaining -= 1;
        pool.swap(r1, remaining);

        let r2 = rng.random_range(0..remaining);
        let idx2 = pool[r2];
        remaining -= 1;
        pool.swap(r2, remaining);

        mating.push((idx1, idx2));
        trace!(target="selection_events", method="clearing"; "Mating index {} with index {}", idx1, idx2);
    }

    debug!(target="selection_events", method="clearing"; "Clearing selection finished: {} pairs", mating.len());
    mating
}
