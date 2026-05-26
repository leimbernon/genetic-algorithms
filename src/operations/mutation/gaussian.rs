//! Gaussian mutation operator for range-encoded chromosomes.
//!
//! Perturbs a randomly selected gene by adding noise drawn from a normal
//! distribution *N(0, sigma)*. The result is clamped to the gene's declared
//! range. This is the standard mutation operator for continuous numerical
//! optimization.
//!
//! Also defines the [`GaussianConvertible`] trait required for converting
//! gene values to and from `f64`.

use crate::chromosomes::Range as RangeChromosome;
use crate::traits::LinearChromosome;
use rand::Rng;
use std::fmt::Debug;

/// Gaussian mutation for `Range<T>` chromosomes where `T` can be converted to/from `f64`.
///
/// Applies a perturbation drawn from a normal distribution N(0, sigma) to a
/// randomly selected gene. The result is clamped to the gene's declared range.
///
/// This is the standard mutation operator for continuous numerical optimization.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `sigma` - The standard deviation of the Gaussian perturbation.
pub fn gaussian_mutation<T>(individual: &mut RangeChromosome<T>, sigma: f64)
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

    // Box-Muller transform for N(0,1) sample
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma;
    let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);

    gene.value = T::from_f64(new_val_f64);
    individual.set_gene(idx, gene);
}

/// Trait for types that can be converted to/from an f64 value.
///
/// This is needed because Rust's standard library doesn't provide a universal
/// `From<f64>` for numeric types. Implementations should do a reasonable
/// conversion (e.g., rounding for integers).
pub trait GaussianConvertible {
    /// Converts an `f64` value to this type (e.g., rounding for integers).
    fn from_f64(val: f64) -> Self;
    /// Converts a value of this type to `f64`.
    fn to_f64(val: Self) -> f64;
}

impl GaussianConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl GaussianConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl GaussianConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl GaussianConvertible for i64 {
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

/// Per-gene Gaussian mutation for `MultiRangeChromosome<T>`.
///
/// Unlike [`gaussian_mutation`] (which reads `gene.ranges` on `RangeChromosome<T>`),
/// this function reads `gene.lo`, `gene.hi`, and `gene.mutation_rate` directly from
/// each `MultiRangeGenotype<T>` gene — enabling independent noise scales per gene
/// (decision D-10).
///
/// # Behaviour
///
/// 1. Selects one gene at random.
/// 2. Computes Box-Muller noise scaled by `gene.mutation_rate` (NOT the `_sigma` arg).
/// 3. Clamps the result to `[gene.lo, gene.hi]`.
/// 4. Writes the new value back via `set_gene`.
///
/// The `_sigma` parameter is accepted for API consistency but is intentionally
/// ignored — per-gene `mutation_rate` is the authoritative scale.
pub fn multi_range_gaussian_mutation<T>(
    individual: &mut crate::chromosomes::MultiRangeChromosome<T>,
    _sigma: f64,
)
where
    T: Sync + Send + Copy + Default + std::fmt::Debug + PartialOrd + 'static + GaussianConvertible,
{
    use crate::traits::LinearChromosome;
    use rand::Rng;

    let len = individual.dna().len();
    if len == 0 {
        return;
    }

    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    let mut gene = individual.dna()[idx].clone();

    let current: f64 = T::to_f64(gene.value);
    let lo_f64: f64 = T::to_f64(gene.lo);
    let hi_f64: f64 = T::to_f64(gene.hi);

    // Box-Muller transform — scale by gene.mutation_rate (D-10), not global sigma
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * gene.mutation_rate;
    let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);

    gene.value = T::from_f64(new_val_f64);
    individual.set_gene(idx, gene);
}
