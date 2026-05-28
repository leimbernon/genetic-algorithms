//! Mutation operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! mutation implementations (swap, inversion, scramble, value, bit-flip,
//! creep, Gaussian, polynomial, non-uniform, permutation-insert, insertion,
//! deletion). The correct implementation is selected at runtime based on
//! the [`Mutation`] variant in the configuration.
//!
//! Chromosome types that need value-aware mutations should implement the
//! [`ValueMutable`] trait.
//!
//! ## Length-changing operators
//!
//! [`Mutation::Insertion`] and [`Mutation::Deletion`] require a
//! `chromosome_length: Some(ChromosomeLength::Variable { min, max })` in
//! `MutationConfiguration`. They return `GaError::MutationError` when called
//! without that configuration or when `ChromosomeLength::Fixed` is set.

pub use self::inversion::inversion;
pub use self::scramble::scramble;
pub use self::swap::swap;
use super::Mutation;
use crate::chromosomes::ChromosomeLength;
use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::{ChromosomeT, LinearChromosome, MutationOperator};
use log::warn;
use std::any::Any;

pub mod bit_flip;
pub mod cauchy;
pub mod creep;
pub mod differential;
pub mod gaussian;
pub mod insertion;
pub mod inversion;
pub mod length_mutation;
pub mod levy_flight;
pub mod list_value;
pub mod non_uniform;
pub mod polynomial;
pub mod scramble;
pub mod self_adaptive_gaussian;
pub mod swap;
pub mod uniform;
pub mod value;

/// Default distribution index for Polynomial mutation when none is configured.
const DEFAULT_POLYNOMIAL_ETA: f64 = 20.0;

/// Attempt polynomial mutation by downcasting a generic individual to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(()))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_polynomial<U: LinearChromosome + 'static>(
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

/// Attempt Cauchy mutation by downcasting a generic individual to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(()))` if the type
/// matched and mutation succeeded, `None` if no supported type matched.
fn try_cauchy<U: LinearChromosome + 'static>(
    individual: &mut U,
    scale: f64,
) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                cauchy::cauchy_mutation(ind, scale);
                return Some(Ok(()));
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Attempt Lévy Flight mutation by downcasting a generic individual to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(()))` if the type
/// matched and mutation succeeded, `None` if no supported type matched.
fn try_levy<U: LinearChromosome + 'static>(
    individual: &mut U,
    alpha: f64,
) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                levy_flight::levy_flight_mutation(ind, alpha);
                return Some(Ok(()));
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Attempt Uniform mutation by downcasting a generic individual to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(()))` if the type
/// matched and mutation succeeded, `None` if no supported type matched.
fn try_uniform<U: LinearChromosome + 'static>(
    individual: &mut U,
) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                uniform::uniform_mutation(ind);
                return Some(Ok(()));
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Attempt self-adaptive Gaussian mutation by downcasting a generic individual to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(()))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched
/// (indicating the chromosome does not implement [`SelfAdaptive`](crate::traits::SelfAdaptive)).
fn try_self_adaptive<U: LinearChromosome + 'static>(
    individual: &mut U,
    tau: f64,
    tau_prime: f64,
    sigma_min: f64,
    sigma_max: Option<f64>,
) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                return Some(self_adaptive_gaussian::self_adaptive_gaussian_mutation(
                    ind, tau, tau_prime, sigma_min, sigma_max,
                ));
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
pub trait ValueMutable: LinearChromosome {
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
        U: LinearChromosome + ValueMutable + 'static,
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
            Mutation::PermutationInsert => {
                return insertion::insertion_mutation(individual);
            }
            Mutation::Insertion => {
                // Length-growing insertion requires ChromosomeLength to be passed in context.
                // When called via factory_with_params (without config), return a descriptive error.
                return Err(GaError::MutationError(
                    "Mutation::Insertion requires ChromosomeLength::Variable configuration. \
                     Use with_chromosome_length(ChromosomeLength::Variable { min, max }) on your engine, \
                     or call length_mutation::length_insertion_mutation() directly with a ChromosomeLength."
                        .to_string(),
                ));
            }
            Mutation::Deletion => {
                // Length-shrinking deletion requires ChromosomeLength to be passed in context.
                return Err(GaError::MutationError(
                    "Mutation::Deletion requires ChromosomeLength::Variable configuration. \
                     Use with_chromosome_length(ChromosomeLength::Variable { min, max }) on your engine, \
                     or call length_mutation::length_deletion_mutation() directly with a ChromosomeLength."
                        .to_string(),
                ));
            }
            Mutation::ListValue => individual.value_mutate(),
            Mutation::Differential => {
                return Err(GaError::MutationError(
                    "Mutation::Differential requires population context. \
                     It is applied automatically by the GA engine when configured — \
                     do not call factory_with_params() directly."
                        .to_string(),
                ));
            }
            Mutation::Cauchy => {
                let scale = step.unwrap_or(1.0);
                return try_cauchy(individual, scale).unwrap_or_else(|| {
                    Err(GaError::MutationError(
                        "Cauchy mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                });
            }
            Mutation::LevyFlight => {
                let alpha = sigma.unwrap_or(1.5);
                return try_levy(individual, alpha).unwrap_or_else(|| {
                    Err(GaError::MutationError(
                        "Lévy Flight mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                });
            }
            Mutation::Uniform => {
                return try_uniform(individual).unwrap_or_else(|| {
                    Err(GaError::MutationError(
                        "Uniform mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                });
            }
            Mutation::SelfAdaptiveGaussian => {
                let n_hint = individual.dna().len().max(1);
                let tau = 1.0 / (2.0 * n_hint as f64).sqrt();
                let tau_prime = 1.0 / (2.0 * (n_hint as f64).sqrt()).sqrt();
                let sigma_min_val = 1e-5_f64;
                return try_self_adaptive(individual, tau, tau_prime, sigma_min_val, None)
                    .unwrap_or_else(|| {
                        Err(GaError::MutationError(
                            "SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive (RangeChromosome<T>)."
                                .to_string(),
                        ))
                    });
            }
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
    U: LinearChromosome + ValueMutable + 'static,
{
    factory_with_params(mutation, individual, None, None)
}

/// Applies the specified mutation operator with optional parameters for Creep/Gaussian.
///
/// # Arguments
///
/// * `mutation` - The mutation variant to apply.
/// * `individual` - Mutable reference to the chromosome to mutate.
/// * `step` - Step size for Creep mutation; **also used as the `scale` (γ) parameter
///   for `Mutation::Cauchy`** (default 1.0 when `None`).
/// * `sigma` - Sigma for Gaussian mutation; **also used as the stability index `α`
///   for `Mutation::LevyFlight`** (default 1.5 when `None`).
pub fn factory_with_params<U>(
    mutation: Mutation,
    individual: &mut U,
    step: Option<f64>,
    sigma: Option<f64>,
) -> Result<(), GaError>
where
    U: LinearChromosome + ValueMutable + 'static,
{
    mutation.mutate(individual, step, sigma)
}

/// Applies `Mutation::Insertion` or `Mutation::Deletion` with the given [`ChromosomeLength`].
///
/// This function is the correct entry point for length-changing operators.
/// For `Mutation::Insertion`:
/// - Clones a random existing gene and inserts it at a random position.
/// - No-op if `chromosome_length` is `Fixed` (returns `Err`).
/// - No-op if the chromosome is already at `max` length.
///
/// For `Mutation::Deletion`:
/// - Removes a gene at a random position.
/// - No-op if `chromosome_length` is `Fixed` (returns `Err`).
/// - No-op if the chromosome is already at `min` length.
///
/// All other [`Mutation`] variants fall through to [`factory_with_params`].
///
/// # Arguments
///
/// * `mutation` - The mutation variant to apply.
/// * `individual` - The chromosome to mutate.
/// * `chromosome_length` - Length policy; required for `Insertion`/`Deletion`.
/// * `step` - Optional step size (forwarded to `factory_with_params` for other variants).
/// * `sigma` - Optional sigma (forwarded to `factory_with_params` for other variants).
pub fn factory_with_chromosome_length<U>(
    mutation: Mutation,
    individual: &mut U,
    chromosome_length: Option<ChromosomeLength>,
    step: Option<f64>,
    sigma: Option<f64>,
) -> Result<(), GaError>
where
    U: LinearChromosome + ValueMutable + 'static,
{
    match mutation {
        Mutation::Insertion => {
            let cl = chromosome_length.unwrap_or(ChromosomeLength::Fixed(0));
            length_mutation::length_insertion_mutation(individual, cl)
        }
        Mutation::Deletion => {
            let cl = chromosome_length.unwrap_or(ChromosomeLength::Fixed(0));
            length_mutation::length_deletion_mutation(individual, cl)
        }
        other => factory_with_params(other, individual, step, sigma),
    }
}

/// Applies the `SelfAdaptiveGaussian` mutation operator with explicit ES parameters.
///
/// This is the `ga.rs` integration entry point for `Mutation::SelfAdaptiveGaussian`.
/// It forwards to the internal downcast dispatcher using the caller-supplied `tau`,
/// `tau_prime`, `sigma_min`, and `sigma_max` values, which may come from
/// [`crate::configuration::MutationConfiguration`] when the user has configured them
/// explicitly.
///
/// Returns `Err(GaError::MutationError)` if the chromosome does not downcast to a
/// supported `SelfAdaptive` type (i.e., `RangeChromosome<f64|f32|i32|i64>`).
pub fn factory_self_adaptive<U: LinearChromosome + 'static>(
    individual: &mut U,
    tau: Option<f64>,
    tau_prime: Option<f64>,
    sigma_min: Option<f64>,
    sigma_max: Option<f64>,
) -> Result<(), GaError> {
    let n_hint = individual.dna().len().max(1);
    let effective_tau = tau.unwrap_or_else(|| 1.0 / (2.0 * n_hint as f64).sqrt());
    let effective_tau_prime =
        tau_prime.unwrap_or_else(|| 1.0 / (2.0 * (n_hint as f64).sqrt()).sqrt());
    let effective_sigma_min = sigma_min.unwrap_or(1e-5_f64);
    try_self_adaptive(
        individual,
        effective_tau,
        effective_tau_prime,
        effective_sigma_min,
        sigma_max,
    )
    .unwrap_or_else(|| {
        Err(GaError::MutationError(
            "SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive \
             (RangeChromosome<f64|f32|i32|i64>)."
                .to_string(),
        ))
    })
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
    U: LinearChromosome + 'static,
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
        Mutation::PermutationInsert => {
            insertion::insertion_mutation(individual)
        }
        Mutation::Insertion => Err(GaError::MutationError(
            "Mutation::Insertion requires ChromosomeLength::Variable configuration. \
             Use with_chromosome_length(ChromosomeLength::Variable { min, max }) on your engine, \
             or call length_mutation::length_insertion_mutation() directly with a ChromosomeLength."
                .to_string(),
        )),
        Mutation::Deletion => Err(GaError::MutationError(
            "Mutation::Deletion requires ChromosomeLength::Variable configuration. \
             Use with_chromosome_length(ChromosomeLength::Variable { min, max }) on your engine, \
             or call length_mutation::length_deletion_mutation() directly with a ChromosomeLength."
                .to_string(),
        )),
        Mutation::ListValue => Err(GaError::MutationError(
            "Mutation::ListValue requires a ListChromosome type. \
                 Use Swap, Inversion, or Scramble instead."
                .to_string(),
        )),
        Mutation::Differential => Err(GaError::MutationError(
            "Mutation::Differential requires Range<T> chromosomes and population context. \
             Use Swap, Inversion, or Scramble instead.".to_string(),
        )),
        Mutation::Cauchy => Err(GaError::MutationError(
            "Mutation::Cauchy requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
             Use Swap, Inversion, or Scramble for non-Range chromosomes."
                .to_string(),
        )),
        Mutation::LevyFlight => Err(GaError::MutationError(
            "Mutation::LevyFlight requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
             Use Swap, Inversion, or Scramble for non-Range chromosomes.".to_string(),
        )),
        Mutation::Uniform => Err(GaError::MutationError(
            "Mutation::Uniform requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
             Use Swap, Inversion, or Scramble for non-Range chromosomes.".to_string(),
        )),
        Mutation::SelfAdaptiveGaussian => Err(GaError::MutationError(
            "Mutation::SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive. \
             Use Swap, Inversion, or Scramble for non-SelfAdaptive chromosomes.".to_string(),
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
