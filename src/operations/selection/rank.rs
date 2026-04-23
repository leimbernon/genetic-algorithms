//! Rank-based selection operator.
//!
//! Individuals are sorted by fitness and assigned selection probabilities
//! proportional to their rank rather than absolute fitness. This avoids
//! premature convergence caused by a few very fit individuals dominating
//! selection.

use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

/// Rank-based selection: individuals are ranked by fitness and selection
/// probability is proportional to rank, not absolute fitness.
///
/// This avoids dominance by very fit individuals (unlike roulette wheel)
/// and provides more uniform selective pressure, which helps maintain
/// population diversity.
///
/// Algorithm:
/// 1. Sort chromosomes by fitness (ascending — worst = rank 1, best = rank N).
/// 2. Assign each individual a selection probability proportional to its rank.
/// 3. Use roulette-wheel sampling on ranks to select parent pairs.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Number of parent pairs to produce.
///
/// # Returns
///
/// A vector of `(usize, usize)` parent index pairs.
pub fn rank_selection<U: ChromosomeT>(chromosomes: &[U], couples: usize) -> Vec<(usize, usize)> {
    debug!(target="selection_events", method="rank_selection"; "Starting rank-based selection");

    let n = chromosomes.len();
    if n < 2 {
        return Vec::new();
    }

    // Create (original_index, fitness) pairs and sort by fitness ascending
    let mut indexed: Vec<(usize, f64)> = chromosomes
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.fitness()))
        .collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Assign rank: worst = 1, best = n
    // rank_sum = n*(n+1)/2
    let rank_sum = (n * (n + 1)) / 2;

    // Build cumulative probabilities based on rank
    let mut cumulative = Vec::with_capacity(n);
    let mut cum = 0.0;
    for (rank_minus_1, &(original_idx, _)) in indexed.iter().enumerate() {
        let rank = (rank_minus_1 + 1) as f64;
        cum += rank / rank_sum as f64;
        cumulative.push((original_idx, cum));
        trace!(target="selection_events", method="rank_selection"; "Index {} rank {} cum_prob {}", original_idx, rank_minus_1 + 1, cum);
    }

    // Select parents via roulette on ranks
    let mut rng = crate::rng::make_rng();
    let total_parents = couples * 2;
    let mut selected = Vec::with_capacity(total_parents);

    for _ in 0..total_parents {
        let r: f64 = rng.random_range(0.0..1.0);
        let idx = cumulative.partition_point(|&(_, cp)| cp < r).min(n - 1);
        selected.push(cumulative[idx].0);
    }

    // Pair selected parents
    let mut mating = Vec::new();
    for chunk in selected.chunks(2) {
        if chunk.len() == 2 {
            mating.push((chunk[0], chunk[1]));
            trace!(target="selection_events", method="rank_selection"; "Mating pair: {} - {}", chunk[0], chunk[1]);
        }
    }

    debug!(target="selection_events", method="rank_selection"; "Rank-based selection finished with {} pairs", mating.len());
    mating
}
