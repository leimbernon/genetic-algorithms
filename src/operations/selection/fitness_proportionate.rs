//! Fitness-proportionate selection operators.
//!
//! Provides two classic selection methods where an individual's probability of
//! being selected is proportional to its fitness:
//!
//! - [`roulette_wheel_selection`] — each parent is chosen independently by
//!   spinning a roulette wheel weighted by fitness.
//! - [`stochastic_universal_sampling`] — evenly spaced pointers on a single
//!   spin of the wheel, producing a lower-variance set of parents.

use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

/// Roulette-wheel selection (fitness-proportionate).
///
/// Each individual's selection probability equals its fitness divided by the
/// total population fitness. Parents are selected independently by generating
/// a random spin in `[0, total_fitness)` and walking the cumulative
/// distribution.
///
/// Returns an empty vector when the total fitness is zero or negative
/// (all individuals must have positive fitness for meaningful selection).
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `num_parents` - Number of parents per group (must be >= 2).
pub fn roulette_wheel_selection<U: ChromosomeT>(chromosomes: &[U], num_parents: usize) -> Vec<Vec<usize>> {
    let num_parents = num_parents.max(2);
    let mut mating = Vec::new();

    //1- Calculate the sum of all fitnesses
    debug!(target="selection_events", method="roulette_wheel_selection"; "Starting the roulette wheel selection");
    let total_fitness: f64 = chromosomes.iter().map(|ind| ind.fitness()).sum();

    let mut rng = crate::rng::make_rng();

    trace!(target="selection_events", method="roulette_wheel_selection"; "Total fitness: {}", total_fitness);

    // Guard against zero or negative total fitness
    if total_fitness <= 0.0 {
        debug!(target="selection_events", method="roulette_wheel_selection"; "Roulette wheel selection finished");
        return mating;
    }

    //2- Build cumulative fitness array
    let mut cumulative_fitness = Vec::with_capacity(chromosomes.len());
    let mut running_sum = 0.0;
    for chromosome in chromosomes.iter() {
        running_sum += chromosome.fitness();
        cumulative_fitness.push(running_sum);
    }

    //3- Select chromosomes.len() parents using the roulette wheel
    let num_selections = (chromosomes.len() / num_parents) * num_parents;
    let mut selected = Vec::with_capacity(num_selections);

    for _ in 0..num_selections {
        let spin = rng.random_range(0.0..total_fitness);
        // Find the first chromosome whose cumulative fitness exceeds the spin value
        let chosen = cumulative_fitness.partition_point(|&cumulative| cumulative <= spin);
        // Clamp to valid index range in case of floating point edge cases
        let chosen = chosen.min(chromosomes.len() - 1);
        selected.push(chosen);
        trace!(target="selection_events", method="roulette_wheel_selection"; "Selected chromosome {} with spin {}", chosen, spin);
    }

    //4- Group selected parents into N-ary groups
    for group in selected.chunks_exact(num_parents) {
        mating.push(group.to_vec());
    }

    debug!(target="selection_events", method="roulette_wheel_selection"; "Roulette wheel selection finished");
    mating
}

/// Stochastic Universal Sampling (SUS).
///
/// A low-variance alternative to roulette-wheel selection. A single random
/// starting point is chosen, then equally spaced pointers are placed along
/// the cumulative fitness distribution to select `couples * num_parents` parents.
///
/// SUS guarantees that individuals with very high fitness are selected
/// roughly the expected number of times, reducing the sampling noise
/// inherent in independent roulette spins.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `couples` - Number of parent groups to produce.
/// * `num_parents` - Number of parents per group (must be >= 2).
pub fn stochastic_universal_sampling<U: ChromosomeT>(
    chromosomes: &[U],
    couples: usize,
    num_parents: usize,
) -> Vec<Vec<usize>> {
    let num_parents = num_parents.max(2);
    debug!(target="selection_events", method="stochastic_universal_sampling"; "Starting the stochastic universal sampling selection");
    let mut mating = Vec::new();

    if chromosomes.is_empty() || couples == 0 {
        return mating;
    }

    let num_selections = couples * num_parents;
    trace!(target="selection_events", method="stochastic_universal_sampling"; "Chromosome selections: {}", num_selections);

    //1- Calculate total fitness and build cumulative fitness array
    let total: f64 = chromosomes.iter().map(|gen| gen.fitness()).sum();
    trace!(target="selection_events", method="stochastic_universal_sampling"; "Total fitness: {}", total);

    if total <= 0.0 {
        return mating;
    }

    let mut cumulative_fitness = Vec::with_capacity(chromosomes.len());
    let mut cumulative = 0.0;
    for genotype in chromosomes {
        cumulative += genotype.fitness();
        cumulative_fitness.push(cumulative);
        trace!(target="selection_events", method="stochastic_universal_sampling"; "Selection probability {}", cumulative / total);
    }

    //2- Calculate the pointer distance and the starting point between 0 and the pointer distance
    let pointer_distance = total / num_selections as f64;
    let mut rng = crate::rng::make_rng();
    let starting_point = rng.random_range(0.0..pointer_distance);
    trace!(target="selection_events", method="stochastic_universal_sampling"; "pointer distance {} - starting point {}", pointer_distance, starting_point);

    //3- Walk pointers and select individuals
    let mut selected = Vec::with_capacity(num_selections);
    let mut cumulative_idx = 0;

    for pointer_num in 0..num_selections {
        let pointer = starting_point + (pointer_num as f64) * pointer_distance;
        // Advance through the cumulative fitness array until we find the chromosome
        // that this pointer lands on
        while cumulative_idx < chromosomes.len() - 1 && cumulative_fitness[cumulative_idx] < pointer
        {
            cumulative_idx += 1;
        }
        selected.push(cumulative_idx);
    }

    //4- Group the selected indices into N-ary parent groups
    for group in selected.chunks(num_parents) {
        if group.len() == num_parents {
            mating.push(group.to_vec());
        }
    }

    debug!(target="mutation_events", method="stochastic_universal_sampling"; "Stochastic universal sampling finished");
    mating
}
