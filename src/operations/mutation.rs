use crate::traits::ChromosomeT;
pub use self::swap::swap;
pub use self::inversion::inversion;
pub use self::scramble::scramble;
use super::Mutation;

pub mod swap;
pub mod inversion;
pub mod scramble;
pub mod value;

/// Trait for chromosomes that support value mutation.
///
/// Implementing this trait allows a chromosome to be used with `Mutation::Value`.
/// It is automatically implemented for `Range<T>` chromosomes where T supports
/// the necessary numeric operations.
pub trait ValueMutable {
    fn value_mutate(&mut self);
}

pub fn factory<U>(mutation: Mutation, individual: &mut U)
where
U: ChromosomeT + 'static
{
    match mutation {
        Mutation::Swap => { swap(individual) },
        Mutation::Inversion => { inversion(individual) },
        Mutation::Scramble => { scramble(individual) },
        Mutation::Value => {
            // Value mutation requires ValueMutable trait — handled at compile time
            // by the caller. If a type doesn't support value mutation, Swap is used
            // as a fallback. This is dispatched via value::try_value_mutation.
            value::try_value_mutation(individual);
        },
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