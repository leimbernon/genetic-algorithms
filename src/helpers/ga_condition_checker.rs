use core::panic;

use crate::{configuration::GaConfiguration};


/**
 * Checks that for adaptive crossover all the requirements are set
 */
pub fn aga_crossover_probabilities(configuration: &GaConfiguration){
    if configuration.crossover_configuration.probability_max.is_none() || configuration.crossover_configuration.probability_min.is_none() {
        panic!("For Adaptive Genetic Algorithms, the probability_max and probability_min in the crossover_configuration are mandatory.");
    }else if configuration.crossover_configuration.probability_max <=  configuration.crossover_configuration.probability_min {
        panic!("For Adaptive Genetic Algorithms, the probability_max must be greater than probability_min in the crossover_configuration.");
    }
}

/**
 * Checks that for adaptive mutation all the requirements are set
 */
pub fn aga_mutation_probabilities(configuration: GaConfiguration){
    if configuration.mutation_configuration.probability_max.is_none() || configuration.mutation_configuration.probability_min.is_none(){
        panic!("For Adaptive Genetic Algorithms, the probability_max and probability_min in the mutation_configuration are mandatory.");
    }else if configuration.mutation_configuration.probability_max <= configuration.mutation_configuration.probability_min {
        panic!("For Adaptive Genetic Algorithms, the probability_max must be greater than probability_min in the mutation_configuration.");
    }
}

/**
 * Function to check that the number of couples is set
 */
pub fn check_number_of_couples_is_set(configuration: &GaConfiguration){
    if configuration.selection_configuration.number_of_couples <= 0 {
        panic!("The number of couples must be set.");
    }
}
