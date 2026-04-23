//! Truncation selection operator.
//!
//! Only the top portion (50%) of the population is eligible for
//! reproduction. Parents are randomly paired from that elite subset.
//! This is a high-pressure strategy that accelerates convergence at
//! the cost of reduced diversity.

use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

/// Truncation selection: only the top portion of the population is eligible
/// for reproduction, and parents are randomly paired from that elite subset.
///
/// Algorithm:
/// 1. Sort individuals by fitness in descending order (best first).
/// 2. Truncate to the top 50% of the population (at least 2 individuals).
/// 3. Randomly pair individuals from the truncated pool to form parent pairs.
///
/// This is a high-pressure selection strategy — weak individuals are
/// completely excluded from reproduction, which can accelerate convergence
/// but may reduce diversity.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Number of parent pairs to produce.
///
/// # Returns
///
/// A vector of `(usize, usize)` parent index pairs drawn exclusively from
/// the top half of the population. Returns an empty vector if fewer than 2
/// chromosomes are provided.
pub fn truncation_selection<U: ChromosomeT>(
    chromosomes: &[U],
    couples: usize,
) -> Vec<(usize, usize)> {
    debug!(target="selection_events", method="truncation"; "Starting truncation selection");

    let n = chromosomes.len();
    if n < 2 {
        return Vec::new();
    }

    // Truncation point: top 50%, but at least 2 individuals
    let truncation_size = (n / 2).max(2).min(n);

    // Build (original_index, fitness) pairs and partition top-k in O(n)
    let mut indexed: Vec<(usize, f64)> = chromosomes
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.fitness()))
        .collect();
    indexed.select_nth_unstable_by(truncation_size - 1, |a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    let elite = &indexed[..truncation_size];

    trace!(
        target="selection_events", method="truncation";
        "Population size {}, truncation size {}", n, truncation_size
    );

    for &(original_idx, fit) in elite.iter() {
        trace!(
            target="selection_events", method="truncation";
            "Elite member -> index {} fitness {}", original_idx, fit
        );
    }

    // Randomly pair individuals from the elite pool
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(couples);

    for _ in 0..couples {
        let a = elite[rng.random_range(0..truncation_size)].0;
        let b = elite[rng.random_range(0..truncation_size)].0;
        mating.push((a, b));
        trace!(
            target="selection_events", method="truncation";
            "Mating pair: {} - {}", a, b
        );
    }

    debug!(
        target="selection_events", method="truncation";
        "Truncation selection finished with {} pairs", mating.len()
    );
    mating
}
