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
/// for reproduction, and parents are randomly grouped from that elite subset.
///
/// Algorithm:
/// 1. Sort individuals by fitness in descending order (best first).
/// 2. Truncate to the top 50% of the population (at least 2 individuals).
/// 3. Randomly group individuals from the truncated pool into N-ary parent groups.
///
/// This is a high-pressure selection strategy — weak individuals are
/// completely excluded from reproduction, which can accelerate convergence
/// but may reduce diversity.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Number of parent groups to produce.
/// * `num_parents` - Number of parents per group (must be >= 2).
///
/// # Returns
///
/// A vector of `Vec<usize>` parent index groups drawn exclusively from
/// the top half of the population. Returns an empty vector if fewer than 2
/// chromosomes are provided.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::selection::truncation_selection;
/// use genetic_algorithms::chromosomes::Binary;
/// let population: Vec<Binary> = vec![Binary::new(); 10];
/// let pairs = truncation_selection(&population, 5, 2);
/// ```
pub fn truncation_selection<U: ChromosomeT>(
    chromosomes: &[U],
    couples: usize,
    num_parents: usize,
) -> Vec<Vec<usize>> {
    let num_parents = num_parents.max(2);
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

    // Randomly group individuals from the elite pool into N-ary parent groups
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(couples);

    for _ in 0..couples {
        let mut group = Vec::with_capacity(num_parents);
        for _ in 0..num_parents {
            group.push(elite[rng.random_range(0..truncation_size)].0);
        }
        trace!(
            target="selection_events", method="truncation";
            "Mating group: {:?}", group
        );
        mating.push(group);
    }

    debug!(
        target="selection_events", method="truncation";
        "Truncation selection finished with {} groups", mating.len()
    );
    mating
}
