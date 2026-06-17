//! Self-adaptive mutation trait for Evolution Strategy sigma co-evolution.
//!
//! Implementors of this trait store per-dimension step-size parameters (σ)
//! alongside their DNA and update them via the log-normal rule on every
//! [`Mutation::SelfAdaptiveGaussian`](crate::operations::Mutation::SelfAdaptiveGaussian)
//! call.

use crate::traits::ChromosomeT;
use rand::Rng;

/// Opt-in trait enabling [`Mutation::SelfAdaptiveGaussian`](crate::operations::Mutation::SelfAdaptiveGaussian).
///
/// Each chromosome that implements `SelfAdaptive` stores a vector of strategy
/// parameters σ (one per gene dimension). These parameters are co-evolved with
/// the chromosome's genes using the standard Evolution Strategy log-normal update:
///
/// ```text
/// σ'_i = σ_i × exp(τ' × N(0,1)_global + τ × N_i(0,1)_local)
/// ```
///
/// where `τ` and `τ'` are the per-dimension and global learning rates, and the
/// result is clamped to `sigma_min` to prevent sigma collapse.
///
/// # Required methods
///
/// Implementors must provide `strategy_params()` and `set_strategy_params()`.
/// The `adapt_strategy_params()` update rule is provided as a default
/// implementation via the trait body, so implementors get it for free.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::chromosomes::Range;
/// use genetic_algorithms::genotypes::Range as RangeGenotype;
/// use genetic_algorithms::traits::{LinearChromosome, SelfAdaptive};
/// use std::borrow::Cow;
///
/// let mut c = Range::<f64>::new();
/// c.set_dna(Cow::Owned(vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.5)]));
/// assert_eq!(c.strategy_params(), &[1.0]);
///
/// c.set_strategy_params(vec![0.2]);
/// assert_eq!(c.strategy_params(), &[0.2]);
///
/// c.adapt_strategy_params(0.5, 0.5, 1e-5);
/// // After adaption, sigma may have changed but is still >= 1e-5
/// assert!(c.strategy_params()[0] >= 1e-5);
/// ```
pub trait SelfAdaptive: ChromosomeT {
    /// Returns the current strategy parameters (σ values) as a slice.
    ///
    /// Each element corresponds to one gene dimension. The slice is empty
    /// before [`LinearChromosome::set_dna`](crate::traits::LinearChromosome::set_dna)
    /// is called (lazy initialization deferred to the chromosome impl).
    fn strategy_params(&self) -> &[f64];

    /// Replaces the strategy parameters (σ values) with the provided vector.
    ///
    /// The vector length should match the number of genes. Typically called
    /// directly in tests or by the mutation operator after the log-normal update.
    fn set_strategy_params(&mut self, params: Vec<f64>);

    /// Updates all strategy parameters using the log-normal rule.
    ///
    /// Applies the standard ES update formula:
    ///
    /// ```text
    /// σ'_i = σ_i × exp(τ' × N_global(0,1) + τ × N_i_local(0,1))
    /// ```
    ///
    /// where `N_global` is one shared sample per call and `N_i_local` is an
    /// independent sample per dimension. The result is clamped to `sigma_min`
    /// to prevent sigma collapse.
    ///
    /// When `strategy_params().len() == 0`, this method returns immediately
    /// without sampling the RNG (no division-by-zero or panic risk).
    ///
    /// # Arguments
    ///
    /// * `tau` — Per-dimension learning rate. Literature default: `1.0 / sqrt(2.0 * n)`.
    /// * `tau_prime` — Global learning rate. Literature default: `1.0 / sqrt(2.0 * sqrt(n))`.
    /// * `sigma_min` — Lower bound for every σ after the update. Typical value: `1e-5`.
    fn adapt_strategy_params(&mut self, tau: f64, tau_prime: f64, sigma_min: f64) {
        let n = self.strategy_params().len();
        if n == 0 {
            return;
        }

        let mut rng = crate::rng::make_rng();

        // Draw one global Box-Muller sample shared across all dimensions
        let u1_global: f64 = rng.random_range(f64::EPSILON..1.0);
        let u2_global: f64 = rng.random_range(0.0..std::f64::consts::TAU);
        let global_noise: f64 = (-2.0 * u1_global.ln()).sqrt() * u2_global.cos();

        let mut new_params: Vec<f64> = self.strategy_params().to_vec();

        for sigma in new_params.iter_mut() {
            // Draw an independent Box-Muller local sample per dimension
            let u1_local: f64 = rng.random_range(f64::EPSILON..1.0);
            let u2_local: f64 = rng.random_range(0.0..std::f64::consts::TAU);
            let local_noise: f64 = (-2.0 * u1_local.ln()).sqrt() * u2_local.cos();

            *sigma = (*sigma * (tau_prime * global_noise + tau * local_noise).exp())
                .max(sigma_min);
        }

        self.set_strategy_params(new_params);
    }
}
