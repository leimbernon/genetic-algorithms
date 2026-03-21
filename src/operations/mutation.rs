//! Mutation operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! mutation implementations (swap, inversion, scramble, value, bit-flip,
//! creep, Gaussian, polynomial, non-uniform, insertion). The correct
//! implementation is selected at runtime based on the [`Mutation`] variant
//! in the configuration.
//!
//! Chromosome types that need value-aware mutations should implement the
//! [`ValueMutable`] trait.

pub use self::inversion::inversion;
pub use self::scramble::scramble;
pub use self::swap::swap;
use super::Mutation;
use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::{ChromosomeT, MutationOperator};
use log::warn;
use std::any::Any;

pub mod bit_flip;
pub mod creep;
pub mod gaussian;
pub mod insertion;
pub mod inversion;
pub mod non_uniform;
pub mod polynomial;
pub mod scramble;
pub mod list_value;
pub mod swap;
pub mod value;

/// Default distribution index for Polynomial mutation when none is configured.
const DEFAULT_POLYNOMIAL_ETA: f64 = 20.0;

/// Attempt polynomial mutation by downcasting a generic individual to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(()))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_polynomial<U: ChromosomeT + 'static>(
    individual: &mut U,
    eta_m: f64,
) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                return Some(polynomial::polynomial_mutation(ind, eta_m));
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Trait for chromosomes that support specialized mutation operators.
///
/// Implementing this trait allows a chromosome to be used with `Mutation::Value`,
/// `Mutation::BitFlip`, `Mutation::Creep`, and `Mutation::Gaussian`.
///
/// The default implementations log a warning and fall back to swap mutation.
/// Override the methods relevant to your chromosome type:
/// - **Binary chromosomes**: override `bit_flip_mutate`
/// - **Range chromosomes**: override `value_mutate`, `creep_mutate`, `gaussian_mutate`
pub trait ValueMutable: ChromosomeT {
    /// Performs value mutation on this chromosome in-place.
    ///
    /// The default implementation logs a warning and falls back to swap mutation.
    /// Override this for chromosome types that have a meaningful value range per gene.
    fn value_mutate(&mut self) {
        warn!(
            "value_mutate() not overridden for this chromosome type; \
             falling back to swap mutation. Implement ValueMutable::value_mutate() \
             for proper value mutation behavior."
        );
        swap(self);
    }

    /// Performs bit flip mutation on this chromosome in-place.
    ///
    /// The default implementation logs a warning and falls back to swap mutation.
    /// Override this for Binary chromosomes to flip a random gene's boolean value.
    fn bit_flip_mutate(&mut self) {
        warn!(
            "bit_flip_mutate() not overridden for this chromosome type; \
             falling back to swap mutation. Implement ValueMutable::bit_flip_mutate() \
             for proper bit-flip behavior."
        );
        swap(self);
    }

    /// Performs creep mutation on this chromosome in-place.
    ///
    /// The default implementation logs a warning and falls back to swap mutation.
    /// Override this for `Range<T>` chromosomes to apply small uniform perturbation.
    fn creep_mutate(&mut self, _step: f64) {
        warn!(
            "creep_mutate() not overridden for this chromosome type; \
             falling back to swap mutation. Implement ValueMutable::creep_mutate() \
             for proper creep mutation behavior."
        );
        swap(self);
    }

    /// Performs gaussian mutation on this chromosome in-place.
    ///
    /// The default implementation logs a warning and falls back to swap mutation.
    /// Override this for `Range<T>` chromosomes to apply gaussian perturbation.
    fn gaussian_mutate(&mut self, _sigma: f64) {
        warn!(
            "gaussian_mutate() not overridden for this chromosome type; \
             falling back to swap mutation. Implement ValueMutable::gaussian_mutate() \
             for proper gaussian mutation behavior."
        );
        swap(self);
    }
}

impl MutationOperator for Mutation {
    fn mutate<U>(
        &self,
        individual: &mut U,
        step: Option<f64>,
        sigma: Option<f64>,
    ) -> Result<(), GaError>
    where
        U: ChromosomeT + ValueMutable + 'static,
    {
        match self {
            Mutation::Swap => swap(individual),
            Mutation::Inversion => inversion(individual),
            Mutation::Scramble => scramble(individual),
            Mutation::Value => individual.value_mutate(),
            Mutation::BitFlip => individual.bit_flip_mutate(),
            Mutation::Creep => {
                let s = step.unwrap_or(1.0);
                individual.creep_mutate(s);
            }
            Mutation::Gaussian => {
                let s = sigma.unwrap_or(1.0);
                individual.gaussian_mutate(s);
            }
            Mutation::Polynomial => {
                let eta = step.unwrap_or(DEFAULT_POLYNOMIAL_ETA);
                return try_polynomial(individual, eta).unwrap_or_else(|| {
                    Err(GaError::MutationError(
                        "Polynomial mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                });
            }
            Mutation::NonUniform => {
                return Err(GaError::MutationError(
                    "Mutation::NonUniform requires generation context (generation, max_generations). \
                     Call non_uniform::non_uniform_mutation() directly."
                        .to_string(),
                ));
            }
            Mutation::Insertion => {
                return insertion::insertion_mutation(individual);
            }
            Mutation::ListValue => individual.value_mutate(),
        }
        Ok(())
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
    factory_with_params(mutation, individual, None, None)
}

/// Applies the specified mutation operator with optional parameters for Creep/Gaussian.
///
/// # Arguments
///
/// * `mutation` - The mutation variant to apply.
/// * `individual` - Mutable reference to the chromosome to mutate.
/// * `step` - Optional step size for Creep mutation.
/// * `sigma` - Optional sigma for Gaussian mutation.
pub fn factory_with_params<U>(
    mutation: Mutation,
    individual: &mut U,
    step: Option<f64>,
    sigma: Option<f64>,
) -> Result<(), GaError>
where
    U: ChromosomeT + ValueMutable + 'static,
{
    mutation.mutate(individual, step, sigma)
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
        Mutation::Swap => {
            swap(individual);
            Ok(())
        }
        Mutation::Inversion => {
            inversion(individual);
            Ok(())
        }
        Mutation::Scramble => {
            scramble(individual);
            Ok(())
        }
        Mutation::Value => Err(GaError::MutationError(
            "Mutation::Value requires the chromosome type to implement ValueMutable. \
                 Use Swap, Inversion, or Scramble instead, or implement ValueMutable for your type."
                .to_string(),
        )),
        Mutation::BitFlip => Err(GaError::MutationError(
            "Mutation::BitFlip requires a Binary chromosome type. \
                 Use Swap, Inversion, or Scramble instead."
                .to_string(),
        )),
        Mutation::Creep => Err(GaError::MutationError(
            "Mutation::Creep requires the chromosome type to implement ValueMutable. \
                 Use Swap, Inversion, or Scramble instead, or implement ValueMutable for your type."
                .to_string(),
        )),
        Mutation::Gaussian => Err(GaError::MutationError(
            "Mutation::Gaussian requires the chromosome type to implement ValueMutable. \
                 Use Swap, Inversion, or Scramble instead, or implement ValueMutable for your type."
                .to_string(),
        )),
        Mutation::Polynomial => Err(GaError::MutationError(
            "Mutation::Polynomial requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
                 Use Swap, Inversion, or Scramble instead."
                .to_string(),
        )),
        Mutation::NonUniform => Err(GaError::MutationError(
            "Mutation::NonUniform requires Range<T> chromosomes and generation context. \
                 Call non_uniform::non_uniform_mutation() directly."
                .to_string(),
        )),
        Mutation::Insertion => {
            insertion::insertion_mutation(individual)
        }
        Mutation::ListValue => Err(GaError::MutationError(
            "Mutation::ListValue requires a ListChromosome type. \
                 Use Swap, Inversion, or Scramble instead."
                .to_string(),
        )),
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
    let larger_f = if parent_1.fitness() > parent_2.fitness() {
        parent_1.fitness()
    } else {
        parent_2.fitness()
    };

    if larger_f >= f_avg {
        probability_min
    } else {
        probability_max
    }
}

/// Computes population cardinality as the ratio of unique fitness values to population size.
///
/// Returns a value in `[0.0, 1.0]` where 1.0 means all individuals have distinct fitness.
pub fn compute_cardinality<U: ChromosomeT>(chromosomes: &[U]) -> f64 {
    if chromosomes.is_empty() {
        return 0.0;
    }
    let mut seen = std::collections::HashSet::new();
    for c in chromosomes {
        // Use bits representation for exact f64 comparison via HashSet
        seen.insert(c.fitness().to_bits());
    }
    seen.len() as f64 / chromosomes.len() as f64
}

/// Adjusts mutation probability based on population cardinality vs target.
///
/// Increases probability when cardinality is below target (low diversity),
/// decreases it when cardinality is above target (high diversity).
pub fn dynamic_probability(
    current_probability: f64,
    cardinality: f64,
    target_cardinality: f64,
    probability_step: f64,
    probability_max: f64,
    probability_min: f64,
) -> f64 {
    if cardinality < target_cardinality {
        (current_probability + probability_step).min(probability_max)
    } else if cardinality > target_cardinality {
        (current_probability - probability_step).max(probability_min)
    } else {
        current_probability
    }
}
