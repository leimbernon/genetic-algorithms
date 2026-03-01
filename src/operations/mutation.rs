pub use self::inversion::inversion;
pub use self::scramble::scramble;
pub use self::swap::swap;
use super::Mutation;
use crate::error::GaError;
use crate::traits::ChromosomeT;

pub mod inversion;
pub mod scramble;
pub mod swap;
pub mod value;

/// Trait for chromosomes that support value mutation.
///
/// Implementing this trait allows a chromosome to be used with `Mutation::Value`.
/// It is automatically implemented for `Range<T>` chromosomes where T supports
/// the necessary numeric operations.
///
/// The default implementation falls back to swap mutation, so chromosome types
/// that don't support value mutation can still be used with the GA orchestrator.
pub trait ValueMutable: ChromosomeT {
    /// Performs value mutation on this chromosome in-place.
    ///
    /// The default implementation falls back to swap mutation.
    fn value_mutate(&mut self) {
        swap(self);
    }
}

/// Applies the specified mutation operator to the given individual.
///
/// # Arguments
///
/// * `mutation` - The mutation variant to apply.
/// * `individual` - Mutable reference to the chromosome to mutate.
///
/// # Returns
///
/// `Ok(())` if the mutation succeeded, or `Err(GaError::MutationError)` if
/// `Mutation::Value` is requested on a type that does not implement `ValueMutable`.
pub fn factory<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>
where
    U: ChromosomeT + ValueMutable + 'static,
{
    match mutation {
        Mutation::Swap => swap(individual),
        Mutation::Inversion => inversion(individual),
        Mutation::Scramble => scramble(individual),
        Mutation::Value => individual.value_mutate(),
    }
    Ok(())
}

/// Applies a non-value mutation operator to the given individual.
///
/// This is a convenience function for chromosome types that don't implement `ValueMutable`.
/// It only supports `Swap`, `Inversion`, and `Scramble`.
///
/// # Returns
///
/// `Ok(())` on success, or `Err(GaError::MutationError)` if `Mutation::Value` is requested.
pub fn factory_non_value<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>
where
    U: ChromosomeT + 'static,
{
    match mutation {
        Mutation::Swap => { swap(individual); Ok(()) },
        Mutation::Inversion => { inversion(individual); Ok(()) },
        Mutation::Scramble => { scramble(individual); Ok(()) },
        Mutation::Value => {
            Err(GaError::MutationError(
                "Mutation::Value requires the chromosome type to implement ValueMutable. \
                 Use Swap, Inversion, or Scramble instead, or implement ValueMutable for your type."
                    .to_string(),
            ))
        },
    }
}

/// Calculates the mutation probability for adaptive genetic algorithms (AGA).
///
/// # Arguments
///
/// * `parent_1` - First parent chromosome.
/// * `parent_2` - Second parent chromosome.
/// * `f_avg` - Average fitness of the population.
/// * `probability_max` - Maximum mutation probability.
/// * `probability_min` - Minimum mutation probability.
///
/// # Returns
///
/// The adapted mutation probability.
pub fn aga_probability<U: ChromosomeT>(
    parent_1: &U,
    parent_2: &U,
    f_avg: f64,
    probability_max: f64,
    probability_min: f64,
) -> f64 {
    let larger_f = if parent_1.get_fitness() > parent_2.get_fitness() {
        parent_1.get_fitness()
    } else {
        parent_2.get_fitness()
    };

    if larger_f >= f_avg {
        probability_min
    } else {
        probability_max
    }
}
