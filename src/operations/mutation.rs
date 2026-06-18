//! Mutation operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! mutation implementations (swap, inversion, scramble, value, bit-flip,
//! creep, Gaussian, polynomial, non-uniform, permutation-insert, insertion,
//! deletion). The correct implementation is selected at runtime based on
//! the [`Mutation`] variant in the configuration.
//!
//! Chromosome types that need value-aware mutations should implement the
//! [`ValueMutable`] trait. Real-valued mutations (polynomial, Cauchy,
//! Lévy Flight, uniform, self-adaptive Gaussian) dispatch via the
//! [`RealValuedMutation`] trait at
//! compile time, replacing previous runtime downcasting.
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
use super::{
    CauchyParams, CreepParams, GaussianParams, LevyFlightParams, PolynomialParams,
    SelfAdaptiveGaussianParams,
};
use super::Mutation;
use crate::chromosomes::ChromosomeLength;
use crate::error::GaError;
use crate::traits::{ChromosomeT, LinearChromosome, MutationOperator, RealValuedMutation};

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

/// Trait for chromosomes that support specialized mutation operators.
///
/// Implementing this trait allows a chromosome to be used with `Mutation::Value`,
/// `Mutation::BitFlip`, `Mutation::Creep`, and `Mutation::Gaussian`.
///
/// The default implementations log a warning and fall back to swap mutation.
/// Override the methods relevant to your chromosome type:
/// - **Binary chromosomes**: override `bit_flip_mutate`
/// - **Range chromosomes**: override `value_mutate`, `creep_mutate`, `gaussian_mutate`
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::ValueMutable;
/// use genetic_algorithms::chromosomes::Binary;
/// let mut chromosome = Binary::new();
/// chromosome.bit_flip_mutate();
/// ```
pub trait ValueMutable: LinearChromosome {
    /// Performs value mutation on this chromosome in-place.
    ///
    /// The default implementation logs a warning and falls back to swap mutation.
    /// Override this for chromosome types that have a meaningful value range per gene.
    fn value_mutate(&mut self) {
        crate::log_warn!(
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
        crate::log_warn!(
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
        crate::log_warn!(
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
        crate::log_warn!(
            "gaussian_mutate() not overridden for this chromosome type; \
             falling back to swap mutation. Implement ValueMutable::gaussian_mutate() \
             for proper gaussian mutation behavior."
        );
        swap(self);
    }
}

impl MutationOperator for Mutation {
    fn mutate<U>(&self, individual: &mut U, mutation: &Mutation) -> Result<(), GaError>
    where
        U: LinearChromosome + ValueMutable + RealValuedMutation + 'static,
    {
        match mutation {
            Mutation::Swap => swap(individual),
            Mutation::Inversion => inversion(individual),
            Mutation::Scramble => scramble(individual),
            Mutation::Value => individual.value_mutate(),
            Mutation::BitFlip => individual.bit_flip_mutate(),
            Mutation::Creep(CreepParams { step }) => {
                let s = step.unwrap_or(0.01);
                individual.creep_mutate(s);
            }
            Mutation::Gaussian(GaussianParams { sigma }) => {
                let s = sigma.unwrap_or(0.1);
                individual.gaussian_mutate(s);
            }
            Mutation::Polynomial(PolynomialParams { eta }) => {
                let eta_val = eta.unwrap_or(DEFAULT_POLYNOMIAL_ETA);
                return individual.polynomial_mutation(eta_val);
            }
            Mutation::NonUniform(..) => {
                return Err(GaError::MutationError(
                    "Mutation::NonUniform requires generation context (generation, max_generations). \
                     It is applied automatically by the GA engine."
                        .to_string(),
                ));
            }
            Mutation::PermutationInsert => {
                return insertion::insertion_mutation(individual);
            }
            Mutation::Insertion => {
                // Length-growing insertion requires ChromosomeLength to be passed in context.
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
            Mutation::Differential(..) => {
                return Err(GaError::MutationError(
                    "Mutation::Differential requires population context. \
                     It is applied automatically by the GA engine when configured."
                        .to_string(),
                ));
            }
            Mutation::Cauchy(CauchyParams { scale }) => {
                let s = scale.unwrap_or(1.0);
                return individual.cauchy_mutation(s);
            }
            Mutation::LevyFlight(LevyFlightParams { alpha }) => {
                let a = alpha.unwrap_or(1.5);
                return individual.levy_flight_mutation(a);
            }
            Mutation::Uniform => {
                return individual.uniform_mutation();
            }
            Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
                tau,
                tau_prime,
                sigma_min,
                sigma_max,
            }) => {
                let n_hint = individual.dna().len().max(1);
                let effective_tau = tau.unwrap_or_else(|| 1.0 / (2.0 * n_hint as f64).sqrt());
                let effective_tau_prime =
                    tau_prime.unwrap_or_else(|| 1.0 / (2.0 * (n_hint as f64).sqrt()).sqrt());
                let effective_sigma_min = sigma_min.unwrap_or(1e-5_f64);
                return individual.self_adaptive_gaussian_mutation(
                    effective_tau,
                    effective_tau_prime,
                    effective_sigma_min,
                    *sigma_max,
                );
            }
        }
        Ok(())
    }
}

/// Applies the specified mutation operator to the given individual.
///
/// # Arguments
///
/// * `mutation` - The mutation variant to apply. Inline variant parameters are used directly.
/// * `individual` - Mutable reference to the chromosome to mutate.
///
/// # Returns
///
/// `Ok(())` if the mutation succeeded, or `Err(GaError::MutationError)` if the
/// mutation cannot be applied to this chromosome type.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::{mutation::factory, Mutation};
///
/// let mut individual = Binary::new();
/// factory(Mutation::BitFlip, &mut individual).unwrap();
/// ```
pub fn factory<U>(mutation: Mutation, individual: &mut U) -> Result<(), GaError>
where
    U: LinearChromosome + ValueMutable + RealValuedMutation + 'static,
{
    mutation.mutate(individual, &mutation.clone())
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
/// All other [`Mutation`] variants fall through to [`factory`].
///
/// # Arguments
///
/// * `mutation` - The mutation variant to apply.
/// * `individual` - The chromosome to mutate.
/// * `chromosome_length` - Length policy; required for `Insertion`/`Deletion`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::{Binary, ChromosomeLength};
/// use genetic_algorithms::operations::{mutation::factory_with_chromosome_length, Mutation};
///
/// let mut individual = Binary::new();
/// let cl = ChromosomeLength::Variable { min: 2, max: 10 };
/// factory_with_chromosome_length(Mutation::Insertion, &mut individual, Some(cl)).unwrap();
/// ```
pub fn factory_with_chromosome_length<U>(
    mutation: Mutation,
    individual: &mut U,
    chromosome_length: Option<ChromosomeLength>,
) -> Result<(), GaError>
where
    U: LinearChromosome + ValueMutable + RealValuedMutation + 'static,
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
        other => factory(other, individual),
    }
}

/// Applies the `SelfAdaptiveGaussian` mutation operator with explicit ES parameters.
///
/// This is a convenience entry point for `Mutation::SelfAdaptiveGaussian`. It
/// constructs the parameterized variant and delegates to the trait implementation.
///
/// Returns `Err(GaError::MutationError)` if the chromosome does not implement
/// the `RealValuedMutation` trait.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Range;
/// use genetic_algorithms::operations::mutation::factory_self_adaptive;
///
/// let mut individual = Range::<f64>::new();
/// factory_self_adaptive(&mut individual, Some(0.1), Some(0.01), Some(0.001), Some(1.0)).unwrap();
/// ```
pub fn factory_self_adaptive<U: LinearChromosome + ValueMutable + RealValuedMutation + 'static>(
    individual: &mut U,
    tau: Option<f64>,
    tau_prime: Option<f64>,
    sigma_min: Option<f64>,
    sigma_max: Option<f64>,
) -> Result<(), GaError> {
    let variant = Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
        tau,
        tau_prime,
        sigma_min,
        sigma_max,
    });
    variant.mutate(individual, &variant.clone())
}

/// Applies a non-value mutation operator to the given individual.
///
/// This is a convenience function for chromosome types that don't implement `ValueMutable`.
/// It only supports `Swap`, `Inversion`, `Scramble`, and `PermutationInsert`.
///
/// # Returns
///
/// `Ok(())` on success, or `Err(GaError::MutationError)` if the variant requires
/// a chromosome type that implements `ValueMutable` or `SelfAdaptive`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::{mutation::factory_non_value, Mutation};
///
/// let mut individual = Binary::new();
/// factory_non_value(Mutation::Swap, &mut individual).unwrap();
/// ```
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
        Mutation::Creep(..) => Err(GaError::MutationError(
            "Mutation::Creep requires the chromosome type to implement ValueMutable. \
                 Use Swap, Inversion, or Scramble instead, or implement ValueMutable for your type."
                .to_string(),
        )),
        Mutation::Gaussian(..) => Err(GaError::MutationError(
            "Mutation::Gaussian requires the chromosome type to implement ValueMutable. \
                 Use Swap, Inversion, or Scramble instead, or implement ValueMutable for your type."
                .to_string(),
        )),
        Mutation::Polynomial(..) => Err(GaError::MutationError(
            "Mutation::Polynomial requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
                 Use Swap, Inversion, or Scramble instead."
                .to_string(),
        )),
        Mutation::NonUniform(..) => Err(GaError::MutationError(
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
        Mutation::Differential(..) => Err(GaError::MutationError(
            "Mutation::Differential requires Range<T> chromosomes and population context. \
             Use Swap, Inversion, or Scramble instead.".to_string(),
        )),
        Mutation::Cauchy(..) => Err(GaError::MutationError(
            "Mutation::Cauchy requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
             Use Swap, Inversion, or Scramble for non-Range chromosomes."
                .to_string(),
        )),
        Mutation::LevyFlight(..) => Err(GaError::MutationError(
            "Mutation::LevyFlight requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
             Use Swap, Inversion, or Scramble for non-Range chromosomes.".to_string(),
        )),
        Mutation::Uniform => Err(GaError::MutationError(
            "Mutation::Uniform requires Range<T> chromosomes where T is f64, f32, i32, or i64. \
             Use Swap, Inversion, or Scramble for non-Range chromosomes.".to_string(),
        )),
        Mutation::SelfAdaptiveGaussian(..) => Err(GaError::MutationError(
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
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::ChromosomeT;
/// use genetic_algorithms::operations::mutation::aga_probability;
///
/// let mut p1 = Binary::new();
/// p1.set_fitness(0.8);
/// let mut p2 = Binary::new();
/// p2.set_fitness(0.4);
/// let prob = aga_probability(&p1, &p2, 0.6, 0.9, 0.1);
/// assert_eq!(prob, 0.1);
/// ```
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
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::ChromosomeT;
/// use genetic_algorithms::operations::mutation::compute_cardinality;
///
/// let mut c1 = Binary::new();
/// c1.set_fitness(1.0);
/// let mut c2 = Binary::new();
/// c2.set_fitness(2.0);
/// let mut c3 = Binary::new();
/// c3.set_fitness(1.0);
/// let cardinality = compute_cardinality(&[c1, c2, c3]);
/// assert!((cardinality - 2.0 / 3.0).abs() < 1e-9);
/// ```
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
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::mutation::dynamic_probability;
///
/// let prob = dynamic_probability(0.05, 0.3, 0.5, 0.01, 0.9, 0.01);
/// assert!((prob - 0.06).abs() < 1e-9);
/// let prob = dynamic_probability(0.05, 0.7, 0.5, 0.01, 0.9, 0.01);
/// assert!((prob - 0.04).abs() < 1e-9);
/// ```
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
