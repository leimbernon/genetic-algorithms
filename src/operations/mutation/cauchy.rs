//! Cauchy (Lorentzian) mutation for `Range<T>` chromosomes.
//!
//! Perturbs a randomly selected gene by adding noise drawn from a Cauchy
//! distribution with the given scale parameter (γ). Heavy-tailed perturbation
//! occasionally produces large jumps that help escape local optima, unlike
//! the thin-tailed Gaussian.
//!
//! See [`Mutation::Cauchy`](crate::operations::Mutation::Cauchy) for user-facing docs.

use crate::chromosomes::Range as RangeChromosome;
use crate::operations::mutation::gaussian::GaussianConvertible;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::fmt::Debug;

/// Applies Cauchy perturbation to a single randomly selected gene of `individual`.
///
/// Uses the inverse-CDF of the standard Cauchy distribution:
/// `noise = scale * tan(π * (u - 0.5))` where `u ~ Uniform(ε, 1-ε)`.
/// The `EPSILON` guards prevent `tan(±π/2) = ±∞` at the distribution poles.
/// The result is clamped to the gene's declared `[lo, hi]` range.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate (exactly one gene is altered).
/// * `scale` - The Cauchy γ parameter controlling the scale of the heavy tail.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::cauchy::cauchy_mutation;
/// use genetic_algorithms::chromosomes::Range;
/// let mut chromosome: Range<f64> = Range::new();
/// cauchy_mutation(&mut chromosome, 0.1);
/// ```
pub fn cauchy_mutation<T>(individual: &mut RangeChromosome<T>, scale: f64)
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + GaussianConvertible,
{
    let len = individual.dna().len();
    if len == 0 {
        return;
    }

    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    let mut gene = individual.dna()[idx].clone();
    if gene.ranges.is_empty() {
        return;
    }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current: f64 = T::to_f64(gene.value);
    let lo_f64: f64 = T::to_f64(lo);
    let hi_f64: f64 = T::to_f64(hi);

    // Inverse-CDF of standard Cauchy. EPSILON guards against tan(±π/2) = ±∞ at u ∈ {0, 1}.
    let u: f64 = rng.random_range(f64::EPSILON..1.0 - f64::EPSILON);
    let noise: f64 = scale * (std::f64::consts::PI * (u - 0.5)).tan();

    let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);
    gene.value = T::from_f64(new_val_f64);
    individual.set_gene(idx, gene);

    debug!(
        target: "mutation_events",
        "Cauchy mutation applied at idx={} range_idx={} scale={}",
        idx, range_idx, scale
    );
}
