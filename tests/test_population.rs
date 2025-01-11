#[cfg(test)]
mod structures;

use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{fitness::FitnessFnWrapper, population::Population};

#[test]
fn test_add_individuals_aga(){

    //Setup of the project
    let individual_1 = Chromosome{dna: Vec::<Gene>::new(), fitness: 20.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let individual_2 = Chromosome{dna: Vec::<Gene>::new(), fitness: 40.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let individual_3 = Chromosome{dna: Vec::<Gene>::new(), fitness: 120.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let mut individuals = vec![individual_1, individual_2, individual_3];
    let mut population  = Population::new_empty();

    //We add the individuals in the population 1 by 1 
    population.add_individuals(&mut individuals);
    population.recalculate_aga();

    //We check the computations
    assert_eq!(population.f_max, 120.0);
    assert_eq!(population.f_avg, 60.0);
    assert_eq!(population.size(), 3);
}

#[test]
fn test_add_individuals(){

    //Setup of the project
    let individual_1 = Chromosome{dna: Vec::<Gene>::new(), fitness: 20.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let individual_2 = Chromosome{dna: Vec::<Gene>::new(), fitness: 40.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let individual_3 = Chromosome{dna: Vec::<Gene>::new(), fitness: 120.0, age: 0, fitness_fn: FitnessFnWrapper::default()};
    let mut individuals = vec![individual_1, individual_2, individual_3];
    let mut population  = Population::new_empty();

    //We add the individuals in the population 1 by 1 
    population.add_individuals(&mut individuals);

    //We check the computations
    assert_eq!(population.size(), 3);
}