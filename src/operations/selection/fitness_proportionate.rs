use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

pub fn roulette_wheel_selection<U: ChromosomeT>(chromosomes: &[U]) -> Vec<(usize, usize)> {
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
    let num_selections = chromosomes.len();
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

    //4- Pair selected parents into couples
    for pair in selected.chunks_exact(2) {
        mating.push((pair[0], pair[1]));
    }

    debug!(target="selection_events", method="roulette_wheel_selection"; "Roulette wheel selection finished");
    mating
}

pub fn stochastic_universal_sampling<U: ChromosomeT>(
    chromosomes: &[U],
    couples: usize,
) -> Vec<(usize, usize)> {
    debug!(target="selection_events", method="stochastic_universal_sampling"; "Starting the stochastic universal sampling selection");
    let mut mating = Vec::new();

    if chromosomes.is_empty() || couples == 0 {
        return mating;
    }

    let num_selections = couples * 2;
    trace!(target="selection_events", method="stochastic_universal_sampling"; "Chromosome couples: {}", num_selections);

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

    //4- Pair the selected indices into couples
    for pair in selected.chunks(2) {
        if pair.len() == 2 {
            mating.push((pair[0], pair[1]));
        }
    }

    debug!(target="mutation_events", method="stochastic_universal_sampling"; "Stochastic universal sampling finished");
    mating
}
