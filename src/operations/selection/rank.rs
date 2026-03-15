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
        // Find the first individual whose cumulative probability >= r
        let idx = cumulative
            .iter()
            .position(|&(_, cp)| cp >= r)
            .unwrap_or(n - 1);
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
    fn rank_selection_produces_correct_number_of_pairs() {
        let pop: Vec<BinaryChromosome> =
            (0..10).map(|i| make_chromosome(i as f64 * 10.0)).collect();
        let pairs = rank_selection(&pop, 3);
        assert_eq!(pairs.len(), 3);
    }

    #[test]
    fn rank_selection_returns_valid_indices() {
        let pop: Vec<BinaryChromosome> = (0..6).map(|i| make_chromosome(i as f64)).collect();
        let pairs = rank_selection(&pop, 3);
        for (a, b) in &pairs {
            assert!(*a < pop.len(), "Index {} out of bounds", a);
            assert!(*b < pop.len(), "Index {} out of bounds", b);
        }
    }

    #[test]
    fn rank_selection_empty_on_small_population() {
        let pop = vec![make_chromosome(10.0)];
        let pairs = rank_selection(&pop, 1);
        assert!(pairs.is_empty());
    }

    #[test]
    fn rank_selection_favors_fitter_individuals() {
        // Create population where the last individual has highest fitness
        let pop: Vec<BinaryChromosome> = (0..20).map(|i| make_chromosome(i as f64)).collect();

        // Run many selections and count how often the fittest (index 19) appears
        let mut fittest_count = 0;
        let trials = 500;
        for _ in 0..trials {
            let pairs = rank_selection(&pop, 5);
            for (a, b) in &pairs {
                if *a == 19 {
                    fittest_count += 1;
                }
                if *b == 19 {
                    fittest_count += 1;
                }
            }
        }
        // The fittest should appear more than uniform random would suggest
        // Uniform: 10 selections per trial * 500 trials / 20 individuals = 250
        // With rank selection the fittest should appear much more often
        assert!(
            fittest_count > 300,
            "Fittest individual appeared {} times, expected > 300 with rank selection",
            fittest_count
        );
    }
}
