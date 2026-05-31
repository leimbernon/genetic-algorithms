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
/// 3. Use roulette-wheel sampling on ranks to select N-ary parent groups.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Number of parent groups to produce.
/// * `num_parents` - Number of parents per group (must be >= 2).
///
/// # Returns
///
/// A vector of `Vec<usize>` parent index groups.
pub fn rank_selection<U: ChromosomeT>(chromosomes: &[U], couples: usize, num_parents: usize) -> Vec<Vec<usize>> {
    let num_parents = num_parents.max(2);
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
    let total_parents = couples * num_parents;
    let mut selected = Vec::with_capacity(total_parents);

    for _ in 0..total_parents {
        let r: f64 = rng.random_range(0.0..1.0);
        let idx = cumulative.partition_point(|&(_, cp)| cp < r).min(n - 1);
        selected.push(cumulative[idx].0);
    }

    // Group selected parents into N-ary groups
    let mut mating = Vec::new();
    for chunk in selected.chunks(num_parents) {
        if chunk.len() == num_parents {
            let group = chunk.to_vec();
            trace!(target="selection_events", method="rank_selection"; "Mating group: {:?}", group);
            mating.push(group);
        }
    }

    debug!(target="selection_events", method="rank_selection"; "Rank-based selection finished with {} groups", mating.len());
    mating
}
