//! Opt-in trait for chromosomes that support real-valued mutation operators.
//!
//! This trait groups the 5 mutation operators that require numeric gene values:
//! polynomial, Cauchy, Lévy Flight, uniform reset, and self-adaptive Gaussian.
//!
//! Only [`Range<T>`](crate::chromosomes::Range) implements all 5 methods.
//! Other chromosome types inherit the default implementations that return
//! [`Err(GaError::MutationError(...))`](crate::error::GaError::MutationError).
//!
//! This trait replaces the previous runtime type-checking approach with
//! compile-time trait dispatch.
//!
//! # Examples
//!
//! ```rust,no_run
//! use genetic_algorithms::traits::RealValuedMutation;
//! use genetic_algorithms::chromosomes::Range;
//!
//! let mut chromosome = Range::<f64>::new();
//! // polynomial_mutation, cauchy_mutation, etc. are available on Range<T>
//! // via the RealValuedMutation trait:
//! let _ = chromosome.polynomial_mutation(20.0);
//! ```

use crate::error::GaError;
use crate::traits::LinearChromosome;

/// Opt-in trait for chromosomes that support real-valued mutation operators.
///
/// Default implementations return `Err(GaError::MutationError(...))`.
/// Override for chromosome types that support real-valued mutations.
///
/// # Trait bounds
///
/// Requires [`LinearChromosome`] as a supertrait, which provides access to
/// `dna()` and `dna_mut()` needed by the mutation operators.
pub trait RealValuedMutation: LinearChromosome {
    /// Applies polynomial mutation with distribution index `eta_m`.
    ///
    /// Default returns `Err(GaError::MutationError(...))` — override for
    /// `Range<T>` chromosomes.
    fn polynomial_mutation(&mut self, _eta_m: f64) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Polynomial mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }

    /// Applies Cauchy (Lorentzian) mutation with the given `scale`.
    ///
    /// Default returns `Err(GaError::MutationError(...))` — override for
    /// `Range<T>` chromosomes.
    fn cauchy_mutation(&mut self, _scale: f64) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Cauchy mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }

    /// Applies Lévy Flight mutation with stability index `alpha`.
    ///
    /// Default returns `Err(GaError::MutationError(...))` — override for
    /// `Range<T>` chromosomes.
    fn levy_flight_mutation(&mut self, _alpha: f64) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Lévy Flight mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }

    /// Applies uniform reset mutation (re-initializes a random gene).
    ///
    /// Default returns `Err(GaError::MutationError(...))` — override for
    /// `Range<T>` chromosomes.
    fn uniform_mutation(&mut self) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "Uniform mutation requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                .to_string(),
        ))
    }

    /// Applies self-adaptive Gaussian mutation with Evolution Strategy parameters.
    ///
    /// `tau` and `tau_prime` are per-dimension and global learning rates,
    /// `sigma_min` / `sigma_max` bound the strategy parameter range.
    ///
    /// Default returns `Err(GaError::MutationError(...))` — override for
    /// `Range<T>` chromosomes that implement [`SelfAdaptive`](crate::traits::SelfAdaptive).
    fn self_adaptive_gaussian_mutation(
        &mut self,
        _tau: f64,
        _tau_prime: f64,
        _sigma_min: f64,
        _sigma_max: Option<f64>,
    ) -> Result<(), GaError> {
        Err(GaError::MutationError(
            "SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive (RangeChromosome<T>)."
                .to_string(),
        ))
    }
}
