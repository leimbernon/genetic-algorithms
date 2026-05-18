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
/// * `number_of_couples` - Target number of parent pairs to produce. Pairs are
///   drawn with replacement when the eligible pool is smaller than required.
pub fn clearing_selection<U: ChromosomeT>(
    chromosomes: &[U],
    niche_radius: f64,
    number_of_couples: usize,
) -> Vec<(usize, usize)> {
    debug!(target="selection_events", method="clearing"; "Starting clearing selection with niche_radius={} number_of_couples={}", niche_radius, number_of_couples);

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

    // Need at least 2 eligible individuals to form any pair.
    if eligible.len() < 2 {
        debug!(target="selection_events", method="clearing"; "Clearing selection finished: 0 pairs (eligible pool too small)");
        return Vec::new();
    }

    // Draw exactly `number_of_couples` pairs from the eligible pool.
    // Pairs are sampled with replacement so that the full requested count is
    // always produced even when the eligible pool is smaller than
    // 2 * number_of_couples. Each pair consists of two distinct indices.
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(number_of_couples);

    use rand::Rng;
    while mating.len() < number_of_couples {
        let i1 = rng.random_range(0..eligible.len());
        // Pick a second index different from i1 by offsetting into the remaining range.
        let i2_raw = rng.random_range(0..(eligible.len() - 1));
        let i2 = if i2_raw >= i1 { i2_raw + 1 } else { i2_raw };
        let idx1 = eligible[i1];
        let idx2 = eligible[i2];
        mating.push((idx1, idx2));
        trace!(target="selection_events", method="clearing"; "Mating index {} with index {}", idx1, idx2);
    }

    debug!(target="selection_events", method="clearing"; "Clearing selection finished: {} pairs", mating.len());
    mating
}
