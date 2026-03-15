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

    // Build (original_index, fitness) pairs and sort descending by fitness
    let mut indexed: Vec<(usize, f64)> = chromosomes
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.fitness()))
        .collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Truncation point: top 50%, but at least 2 individuals
    let truncation_size = (n / 2).max(2).min(n);
    let elite = &indexed[..truncation_size];

    trace!(
        target="selection_events", method="truncation";
        "Population size {}, truncation size {}", n, truncation_size
    );

    for (rank, &(original_idx, fit)) in elite.iter().enumerate() {
        trace!(
            target="selection_events", method="truncation";
            "Elite rank {} -> index {} fitness {}", rank, original_idx, fit
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::genotypes::Binary as BinaryGenotype;
    use std::borrow::Cow;

    fn make_chromosome(fitness: f64) -> BinaryChromosome {
        let mut c = BinaryChromosome::new();
        c.set_dna(Cow::Owned(vec![BinaryGenotype { id: 0, value: true }]));
        c.set_fitness(fitness);
        c
    }

    #[test]
    fn truncation_selection_produces_correct_number_of_pairs() {
        let pop: Vec<BinaryChromosome> =
            (0..10).map(|i| make_chromosome(i as f64 * 10.0)).collect();
        let pairs = truncation_selection(&pop, 4);
        assert_eq!(pairs.len(), 4);
    }

    #[test]
    fn truncation_selection_empty_on_small_population() {
        // Single chromosome — cannot form any pair
        let pop = vec![make_chromosome(42.0)];
        let pairs = truncation_selection(&pop, 1);
        assert!(pairs.is_empty());

        // Empty population
        let empty: Vec<BinaryChromosome> = Vec::new();
        let pairs = truncation_selection(&empty, 1);
        assert!(pairs.is_empty());
    }

    #[test]
    fn truncation_selection_selects_only_from_top_half() {
        // Create a population of 20 where fitness == index (0..19).
        // Top 50% = indices 10..19 (the 10 fittest).
        let pop: Vec<BinaryChromosome> = (0..20).map(|i| make_chromosome(i as f64)).collect();

        let top_half_indices: std::collections::HashSet<usize> = (10..20).collect();

        // Run many trials to ensure we never pick from the bottom half
        for _ in 0..200 {
            let pairs = truncation_selection(&pop, 5);
            for (a, b) in &pairs {
                assert!(
                    top_half_indices.contains(a),
                    "Index {} is not in the top half ({:?})",
                    a,
                    top_half_indices
                );
                assert!(
                    top_half_indices.contains(b),
                    "Index {} is not in the top half ({:?})",
                    b,
                    top_half_indices
                );
            }
        }
    }

    #[test]
    fn truncation_selection_handles_equal_fitness() {
        // All individuals have the same fitness — any of them could be in the
        // top half. Selection should still succeed without panicking.
        let pop: Vec<BinaryChromosome> = (0..10).map(|_| make_chromosome(5.0)).collect();
        let pairs = truncation_selection(&pop, 3);
        assert_eq!(pairs.len(), 3);
        for (a, b) in &pairs {
            assert!(*a < pop.len());
            assert!(*b < pop.len());
        }
    }

    #[test]
    fn truncation_selection_with_two_chromosomes() {
        // Minimum viable population — both are in the elite pool
        let pop = vec![make_chromosome(1.0), make_chromosome(2.0)];
        let pairs = truncation_selection(&pop, 1);
        assert_eq!(pairs.len(), 1);
        for (a, b) in &pairs {
            assert!(*a < 2);
            assert!(*b < 2);
        }
    }
}
