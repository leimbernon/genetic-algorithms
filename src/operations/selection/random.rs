//! Random selection operator.
//!
//! Pairs parents by shuffling the population indices with a Fisher-Yates
//! partial shuffle, then pairing them consecutively. Every individual is
//! selected at most once (no replacement), and the cost is *O(N)*.

use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

/// Random selection: groups individuals randomly without regard to fitness.
///
/// Uses a Fisher-Yates partial shuffle so each pick is *O(1)* (swap-to-end)
/// instead of *O(N)* (`Vec::remove` shifting). Total cost: *O(N)*.
///
/// Each individual participates in at most one group; if the population size
/// is not evenly divisible by `num_parents`, remaining individuals are left ungrouped.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `num_parents` - Number of parents per group (must be >= 2).
pub fn random<U: ChromosomeT>(chromosomes: &[U], num_parents: usize) -> Vec<Vec<usize>> {
    let num_parents = num_parents.max(2);
    let n = chromosomes.len();
    let group_count = n / num_parents;
    let mut mating = Vec::with_capacity(group_count);
    let mut indexes: Vec<usize> = (0..n).collect();
    let mut rng = crate::rng::make_rng();
    let mut remaining = n;
    debug!(target="selection_events", method="random"; "Starting random selection");

    // Pick groups via Fisher-Yates: swap chosen element with the last
    // unprocessed element and shrink the working range.
    while remaining >= num_parents {
        let mut group = Vec::with_capacity(num_parents);
        for _ in 0..num_parents {
            let r = rng.random_range(0..remaining);
            let index_value = indexes[r];
            remaining -= 1;
            indexes.swap(r, remaining);
            group.push(index_value);
        }
        trace!(target="selection_events", method="random"; "Mating group: {:?}", group);
        mating.push(group);
    }

    mating
}
