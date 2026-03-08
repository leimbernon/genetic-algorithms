use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

/// Boltzmann selection: uses a temperature parameter to control selective pressure,
/// inspired by the Boltzmann probability distribution from statistical mechanics.
///
/// Each individual's selection probability is proportional to `exp(fitness / temperature)`.
/// - **High temperature** flattens the distribution, making selection nearly uniform
///   (exploration).
/// - **Low temperature** sharpens the distribution, strongly favoring high-fitness
///   individuals (exploitation).
///
/// This allows the algorithm to start with broad exploration and gradually increase
/// selective pressure by lowering the temperature (simulated-annealing style).
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Number of parent pairs to produce.
/// * `temperature` - Controls selective pressure. If `<= 0.0`, defaults to `1.0`.
///
/// # Returns
///
/// A vector of `(usize, usize)` parent index pairs. Returns an empty vector if the
/// population has fewer than 2 individuals.
pub fn boltzmann_selection<U: ChromosomeT>(
    chromosomes: &[U],
    couples: usize,
    temperature: f64,
) -> Vec<(usize, usize)> {
    debug!(target="selection_events", method="boltzmann"; "Starting Boltzmann selection");

    let n = chromosomes.len();
    if n < 2 {
        debug!(target="selection_events", method="boltzmann"; "Population too small ({}), returning empty", n);
        return Vec::new();
    }

    let temp = if temperature <= 0.0 {
        debug!(target="selection_events", method="boltzmann"; "Temperature {} <= 0.0, using fallback 1.0", temperature);
        1.0
    } else {
        temperature
    };

    // Compute raw Boltzmann weights: exp(fitness_i / temperature).
    // To avoid overflow we subtract the maximum exponent before exponentiating.
    let fitnesses: Vec<f64> = chromosomes.iter().map(|c| c.fitness()).collect();
    let max_fitness = fitnesses.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let weights: Vec<f64> = fitnesses
        .iter()
        .map(|&f| ((f - max_fitness) / temp).exp())
        .collect();

    let total_weight: f64 = weights.iter().sum();

    // Build cumulative probability distribution
    let mut cumulative = Vec::with_capacity(n);
    let mut cum = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        let prob = if total_weight > 0.0 {
            w / total_weight
        } else {
            // All weights are zero (e.g. all -inf fitness); fall back to uniform
            1.0 / n as f64
        };
        cum += prob;
        cumulative.push(cum);
        trace!(target="selection_events", method="boltzmann"; "Index {} fitness {} weight {} cum_prob {}", i, fitnesses[i], w, cum);
    }

    // Correct any floating-point drift so the last entry is exactly 1.0
    if let Some(last) = cumulative.last_mut() {
        *last = 1.0;
    }

    // Select parents via roulette-wheel sampling on Boltzmann probabilities
    let mut rng = crate::rng::make_rng();
    let total_parents = couples * 2;
    let mut selected = Vec::with_capacity(total_parents);

    for _ in 0..total_parents {
        let r: f64 = rng.random_range(0.0..1.0);
        let idx = cumulative.iter().position(|&cp| cp >= r).unwrap_or(n - 1);
        selected.push(idx);
    }

    // Pair selected parents
    let mut mating = Vec::new();
    for chunk in selected.chunks(2) {
        if chunk.len() == 2 {
            mating.push((chunk[0], chunk[1]));
            trace!(target="selection_events", method="boltzmann"; "Mating pair: {} - {}", chunk[0], chunk[1]);
        }
    }

    debug!(target="selection_events", method="boltzmann"; "Boltzmann selection finished with {} pairs", mating.len());
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
    fn boltzmann_selection_produces_correct_number_of_pairs() {
        let pop: Vec<BinaryChromosome> =
            (0..10).map(|i| make_chromosome(i as f64 * 10.0)).collect();
        let pairs = boltzmann_selection(&pop, 5, 1.0);
        assert_eq!(pairs.len(), 5);
    }

    #[test]
    fn boltzmann_selection_empty_on_small_population() {
        // Single chromosome
        let pop = vec![make_chromosome(10.0)];
        let pairs = boltzmann_selection(&pop, 3, 1.0);
        assert!(pairs.is_empty());

        // Empty population
        let empty: Vec<BinaryChromosome> = Vec::new();
        let pairs = boltzmann_selection(&empty, 3, 1.0);
        assert!(pairs.is_empty());
    }

    #[test]
    fn boltzmann_selection_high_temperature_approaches_uniform() {
        // With a very high temperature the Boltzmann distribution should be
        // nearly uniform, so every individual should be selected roughly the
        // same number of times.
        let n = 5;
        let pop: Vec<BinaryChromosome> =
            (0..n).map(|i| make_chromosome(i as f64 * 100.0)).collect();

        let trials = 2000;
        let couples_per_trial = 50;
        let mut counts = vec![0usize; n];

        for _ in 0..trials {
            let pairs = boltzmann_selection(&pop, couples_per_trial, 1e12);
            for (a, b) in &pairs {
                counts[*a] += 1;
                counts[*b] += 1;
            }
        }

        let total_selections: usize = counts.iter().sum();
        let expected_per_individual = total_selections as f64 / n as f64;

        for (i, &count) in counts.iter().enumerate() {
            let ratio = count as f64 / expected_per_individual;
            assert!(
                (0.8..=1.2).contains(&ratio),
                "Individual {} selected {} times (ratio {:.3}), expected roughly {:.0} — \
                 distribution is not uniform enough at high temperature",
                i,
                count,
                ratio,
                expected_per_individual,
            );
        }
    }

    #[test]
    fn boltzmann_selection_low_temperature_favors_fittest() {
        let pop: Vec<BinaryChromosome> =
            (0..10).map(|i| make_chromosome(i as f64 * 10.0)).collect();

        let mut fittest_count = 0;
        let trials = 500;
        for _ in 0..trials {
            let pairs = boltzmann_selection(&pop, 5, 0.01);
            for (a, b) in &pairs {
                if *a == 9 {
                    fittest_count += 1;
                }
                if *b == 9 {
                    fittest_count += 1;
                }
            }
        }

        // With very low temperature the fittest individual (index 9) should
        // dominate selections. Total selections = 500 * 10 = 5000.
        assert!(
            fittest_count > 3000,
            "Fittest individual appeared {} times out of 5000, expected > 3000 at low temperature",
            fittest_count,
        );
    }

    #[test]
    fn boltzmann_selection_handles_equal_fitness() {
        // When all fitnesses are equal, selection should be effectively uniform
        let pop: Vec<BinaryChromosome> = (0..4).map(|_| make_chromosome(42.0)).collect();
        let pairs = boltzmann_selection(&pop, 3, 1.0);
        assert_eq!(pairs.len(), 3);
        for (a, b) in &pairs {
            assert!(*a < pop.len());
            assert!(*b < pop.len());
        }
    }

    #[test]
    fn boltzmann_selection_invalid_temperature_uses_fallback() {
        let pop: Vec<BinaryChromosome> = (0..5).map(|i| make_chromosome(i as f64)).collect();

        // Zero temperature
        let pairs = boltzmann_selection(&pop, 2, 0.0);
        assert_eq!(pairs.len(), 2);

        // Negative temperature
        let pairs = boltzmann_selection(&pop, 2, -5.0);
        assert_eq!(pairs.len(), 2);
    }
}
