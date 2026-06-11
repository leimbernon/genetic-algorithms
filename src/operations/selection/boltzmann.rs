//! Boltzmann selection operator.
//!
//! Uses a temperature parameter inspired by statistical mechanics to
//! control selective pressure. High temperatures yield nearly uniform
//! selection (exploration); low temperatures strongly favor the fittest
//! (exploitation). Lowering the temperature over generations produces a
//! simulated-annealing effect.

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
/// * `couples` - Number of parent groups to produce.
/// * `temperature` - Controls selective pressure. If `<= 0.0`, defaults to `1.0`.
/// * `num_parents` - Number of parents per group (must be >= 2).
///
/// # Returns
///
/// A vector of `Vec<usize>` parent index groups. Returns an empty vector if the
/// population has fewer than 2 individuals.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::selection::boltzmann_selection;
/// use genetic_algorithms::chromosomes::Binary;
/// let population: Vec<Binary> = vec![Binary::new(); 10];
/// let pairs = boltzmann_selection(&population, 5, 1.0, 2);
/// ```
pub fn boltzmann_selection<U: ChromosomeT>(
    chromosomes: &[U],
    couples: usize,
    temperature: f64,
    num_parents: usize,
) -> Vec<Vec<usize>> {
    let num_parents = num_parents.max(2);
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
    let total_parents = couples * num_parents;
    let mut selected = Vec::with_capacity(total_parents);

    for _ in 0..total_parents {
        let r: f64 = rng.random_range(0.0..1.0);
        let idx = cumulative.partition_point(|&cp| cp < r).min(n - 1);
        selected.push(idx);
    }

    // Group selected parents into N-ary groups
    let mut mating = Vec::new();
    for chunk in selected.chunks(num_parents) {
        if chunk.len() == num_parents {
            let group = chunk.to_vec();
            trace!(target="selection_events", method="boltzmann"; "Mating group: {:?}", group);
            mating.push(group);
        }
    }

    debug!(target="selection_events", method="boltzmann"; "Boltzmann selection finished with {} groups", mating.len());
    mating
}
