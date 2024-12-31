pub(crate) use super::Crossover;
use crate::configuration::CrossoverConfiguration;
use crate::traits::ChromosomeT;
pub use self::cycle::cycle;
pub use self::multipoint::multipoint;
pub use self::uniform_crossover::uniform;

pub mod cycle;
pub mod multipoint;
pub mod uniform_crossover;


pub fn factory<U: ChromosomeT>(parent_1: &U, parent_2: &U, configuration: CrossoverConfiguration) -> Option<Vec<U>>{
    match configuration.method {
        Crossover::Cycle => {cycle(parent_1, parent_2)},
        Crossover::MultiPoint => { multipoint(parent_1, parent_2, &configuration.number_of_points.unwrap())},
        Crossover::Uniform => {uniform(parent_1, parent_2)},
    }
}

//Function to calculate the probability for adaptive genetic algorithms
pub fn aga_probability<U: ChromosomeT>(parent_1: &U, parent_2: &U, f_max: f64, f_avg: f64, probability_max: f64, probability_min: f64)->f64{
    let larger_f = if parent_1.get_fitness() > parent_2.get_fitness() {parent_1.get_fitness()}else{parent_2.get_fitness()};

    if larger_f >= f_avg {
        probability_max*((f_max - larger_f) / (f_max - f_avg))
    }else{
        probability_min
    }
}