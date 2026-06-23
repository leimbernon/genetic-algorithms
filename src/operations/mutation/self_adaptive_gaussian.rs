//! Self-adaptive Gaussian mutation for chromosomes implementing `SelfAdaptive`.
//!
//! Applies the standard Evolution Strategy two-step mutation:
//! 1. Update all strategy parameters (σ) via the log-normal rule.
//! 2. Perturb a randomly selected gene using the updated σ for that dimension.
//!
//! This operator is restricted to [`crate::chromosomes::Range`] which implements
//! [`SelfAdaptive`] and carries per-dimension σ values alongside its genes.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::operations::mutation::gaussian::GaussianConvertible;
use crate::traits::SelfAdaptive;
use rand::Rng;
use std::fmt::Debug;

/// Applies self-adaptive Gaussian mutation to a `RangeChromosome<T>` individual.
///
/// This is the two-step Evolution Strategy mutation:
///
/// **Step 1 — Sigma update:** All strategy parameters σ are updated using the
/// log-normal rule `σ'_i = σ_i × exp(τ' × N_global(0,1) + τ × N_i_local(0,1))`.
/// Every σ is clamped to `sigma_min` after the update to prevent sigma collapse.
/// If `sigma_max` is `Some(v)`, every σ is also clamped to `v` from above to
/// prevent unbounded explosion.
/// This is delegated to [`SelfAdaptive::adapt_strategy_params`].
///
/// **Step 2 — Gene mutation:** One gene is chosen at random. Its value is
/// perturbed by Gaussian noise `N(0, σ'_i)` using the Box-Muller transform,
/// then clamped to the gene's declared range.
///
/// # Arguments
///
/// * `individual` — The chromosome to mutate (in-place).
/// * `tau` — Per-dimension learning rate for the sigma update.
/// * `tau_prime` — Global learning rate for the sigma update.
/// * `sigma_min` — Lower bound for every σ after the update (prevents collapse).
/// * `sigma_max` — Optional upper bound for every σ after the update (prevents explosion).
///
/// # Returns
///
/// `Ok(())` always (all error conditions handled gracefully: empty DNA, empty
/// gene ranges). No panics.
///
/// # WASM safety
///
/// Does not use `rayon`, `std::time::Instant`, or `SystemTime`.
/// Uses [`crate::rng::make_rng`] for all randomness.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::self_adaptive_gaussian::self_adaptive_gaussian_mutation;
/// use genetic_algorithms::chromosomes::Range;
/// let mut chromosome: Range<f64> = Range::new();
/// let _ = self_adaptive_gaussian_mutation(&mut chromosome, 0.1, 0.05, 1e-5, None);
/// ```
pub fn self_adaptive_gaussian_mutation<T>(
    individual: &mut RangeChromosome<T>,
    tau: f64,
    tau_prime: f64,
    sigma_min: f64,
    sigma_max: Option<f64>,
) -> Result<(), GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + GaussianConvertible,
    RangeChromosome<T>: SelfAdaptive,
{
    // Access the concrete `dna` field directly to avoid associated-type ambiguity
    // that arises when `RangeChromosome<T>: SelfAdaptive` is in the where clause.
    let len = individual.dna.len();
    if len == 0 {
        return Ok(());
    }

    // Step 1: update all sigmas via log-normal rule (delegated to trait default impl)
    individual.adapt_strategy_params(tau, tau_prime, sigma_min);

    // Apply optional upper bound to prevent unbounded sigma explosion
    if let Some(max) = sigma_max {
        let mut capped = individual.strategy_params().to_vec();
        for s in capped.iter_mut() {
            *s = s.min(max);
        }
        individual.set_strategy_params(capped);
    }

    // Step 2: mutate one randomly selected gene using its updated sigma
    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    // Defensive fallback: if strategy_params is shorter than dna (user set a short vector)
    let sigma = individual
        .strategy_params()
        .get(idx)
        .copied()
        .unwrap_or(1.0);

    // Clone the gene — `Range<T>` gene has concrete `ranges` and `value` fields
    let mut gene = individual.dna[idx].clone();

    if gene.ranges.is_empty() {
        return Ok(());
    }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current: f64 = T::to_f64(gene.value);
    let lo_f64: f64 = T::to_f64(lo);
    let hi_f64: f64 = T::to_f64(hi);

    // Box-Muller transform for N(0, sigma)
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma;

    let new_val = (current + noise).clamp(lo_f64, hi_f64);
    gene.value = T::from_f64(new_val);
    individual.dna[idx] = gene;

    crate::log_debug!(
        target: "mutation_events",
        "SelfAdaptiveGaussian mutation applied at idx={} sigma={}",
        idx,
        sigma
    );

    Ok(())
}
