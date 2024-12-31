use crate::traits::ChromosomeT;
pub use self::swap::swap;
pub use self::inversion::inversion;
pub use self::scramble::scramble;
use super::Mutation;

pub mod swap;
pub mod inversion;
pub mod scramble;

pub fn factory<U>(mutation: Mutation ,individual: &mut U)
where
U: ChromosomeT + 'static
{
    match mutation {
        Mutation::Swap => {swap(individual)},
        Mutation::Inversion => {inversion(individual)},
        Mutation::Scramble => {scramble(individual)},
    }
}

//Function to calculate the probability for adaptive genetic algorithms
pub fn aga_probability<U: ChromosomeT>(parent_1: &U, parent_2: &U, f_avg: f64, probability_max: f64, probability_min: f64)->f64{
    let larger_f = if parent_1.get_fitness() > parent_2.get_fitness() {parent_1.get_fitness()}else{parent_2.get_fitness()};

    if larger_f >= f_avg {
        probability_min
    }else{
        probability_max
    }

}