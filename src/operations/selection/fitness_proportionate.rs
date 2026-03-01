use crate::traits::ChromosomeT;
use rand::Rng;
use log::{trace, debug};

pub fn roulette_wheel_selection<U:ChromosomeT>(chromosomes: &Vec<U>) -> Vec<(usize, usize)>{

    let mut mating = Vec::new();

    //1- Calculate the sum of all fitnesses
    debug!(target="selection_events", method="roulette_wheel_selection"; "Starting the roulette wheel selection");
    let total_fitness: f64 = chromosomes.iter().map(|ind| ind.get_fitness()).sum();

    let mut rng = rand::rng();

    trace!(target="selection_events", method="roulette_wheel_selection"; "Total fitness: {}", total_fitness);

    //2- Identifies what chromosomes will be parents
    let mut parent_1 = None;
    for index in  0..chromosomes.len(){

        //We get the probability
        if rng.random_range(0.0..total_fitness) >= chromosomes.get(index).unwrap().get_fitness(){

            if parent_1.is_none() {
                //If parent 1 is not set, we set it
                parent_1 = Some(index);
            }else{
                //If parent 1 is set, we set the mating
                mating.push((parent_1.unwrap(), index));
                parent_1 = None;
            }

        }
    }

    debug!(target="selection_events", method="roulette_wheel_selection"; "Roulette wheel selection finished");
    mating
}


pub fn stochastic_universal_sampling<U:ChromosomeT>(chromosomes: &Vec<U>, couples: i32) -> Vec<(usize, usize)>{
    
    debug!(target="selection_events", method="stochastic_universal_sampling"; "Starting the stochastic universal sampling selection");
    let mut mating = Vec::new();
    let chromosome_couples = (couples*2) as usize;
    trace!(target="selection_events", method="stochastic_universal_sampling"; "Chromosome couples: {}", chromosome_couples);

    //1- Calculate the selection probabilities
    let total: f64 = chromosomes.iter().map(|gen| gen.get_fitness()).sum();
    let mut last_selection_value = 0.0;
    let mut selection_probabilities = Vec::new();
    let mut rng = rand::rng();

    trace!(target="selection_events", method="stochastic_universal_sampling"; "Total fitness: {}", total);
    for genotype in chromosomes {
        let selection_probability = (genotype.get_fitness() / total) + last_selection_value;
        last_selection_value = selection_probability;
        selection_probabilities.push(selection_probability);
        trace!(target="selection_events", method="stochastic_universal_sampling"; "Selection probability {}", selection_probability);
    }

    //2- Calculate the pointer distance and the starting point between 0 and the pointer distance
    let pointer_distance = 1.0 / chromosome_couples as f64;
    let starting_point = rng.random_range(0.0..pointer_distance);
    trace!(target="selection_events", method="stochastic_universal_sampling"; "pointer distance {} - starting point {}", pointer_distance, starting_point);

    //3- Parent identification
    let mut current_point = starting_point;
    let mut next_chromosome = 1;

    let mut end_of_chromosomes = false;
    let mut couple_completed = false;
    let mut first_mate = 0;

    for i in 0..chromosome_couples {
       
        //We check that there are enough chromosomes
        if i >= chromosomes.len(){
            break;
        }else if next_chromosome >= chromosomes.len(){
            end_of_chromosomes = true;
        }

        //We check if the pointer is between the current and the next chromosome
        if !end_of_chromosomes && current_point >= selection_probabilities[i] &&
            current_point < selection_probabilities[next_chromosome] {

            if couple_completed {
                mating.push((first_mate, i));
            }else{
                first_mate = i;
            }

            couple_completed = !couple_completed;
            current_point += pointer_distance;

        } else if end_of_chromosomes && current_point >= selection_probabilities[i] {
            if couple_completed {
                mating.push((first_mate, i));
                couple_completed = !couple_completed;
            }else{
                first_mate = i;
            }
        }

        next_chromosome += 1;
    }

    debug!(target="mutation_events", method="stochastic_universal_sampling"; "Stochastic universal sampling finished");
    mating
}