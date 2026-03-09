use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

/**
 * Function to make the random parent selection between the list of chromosomes.
 *
 * Uses a Fisher-Yates partial shuffle so each pick is O(1) (swap-to-end)
 * instead of O(N) (`Vec::remove` shifting). Total cost: O(N).
 */
pub fn random<U: ChromosomeT>(chromosomes: &[U]) -> Vec<(usize, usize)> {
    let n = chromosomes.len();
    let pair_count = n / 2;
    let mut mating = Vec::with_capacity(pair_count);
    let mut indexes: Vec<usize> = (0..n).collect();
    let mut rng = crate::rng::make_rng();
    let mut remaining = n;
    debug!(target="selection_events", method="random"; "Starting random selection");

    // Pick pairs via Fisher-Yates: swap chosen element with the last
    // unprocessed element and shrink the working range.
    while remaining >= 2 {
        // Pick first parent
        let r1 = rng.random_range(0..remaining);
        let index_value_1 = indexes[r1];
        remaining -= 1;
        indexes.swap(r1, remaining);

        // Pick second parent
        let r2 = rng.random_range(0..remaining);
        let index_value_2 = indexes[r2];
        remaining -= 1;
        indexes.swap(r2, remaining);

        mating.push((index_value_1, index_value_2));
        trace!(target="selection_events", method="random"; "Mating index 1 {} with index 2 {}", index_value_1, index_value_2);
    }

    mating
}
