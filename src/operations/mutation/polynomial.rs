//! Polynomial mutation operator for range-encoded chromosomes.
//!
//! Uses a polynomial probability distribution to perturb a single gene.
//! The spread is controlled by the distribution index `eta_m`:
//!
//! - Low `eta_m` (1–5): larger perturbations (exploration).
//! - High `eta_m` (20–100): smaller perturbations (exploitation).
//!
//! This operator is widely used in NSGA-II and other multi-objective
//! evolutionary algorithms. Also defines the [`PolynomialConvertible`]
//! trait for `f64` conversion.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::fmt::Debug;

/// Polynomial mutation for `Range<T>` chromosomes.
///
/// This operator is commonly used in NSGA-II and other multi-objective
/// evolutionary algorithms. It perturbs a randomly selected gene using a
/// polynomial probability distribution controlled by the distribution index
/// `eta_m`.
///
/// - Low `eta_m` (e.g., 1–5): larger perturbations (exploration).
/// - High `eta_m` (e.g., 20–100): smaller perturbations (exploitation).
///
/// The result is always clamped to the gene's declared range.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `eta_m` - The distribution index controlling mutation spread.
///
/// # Returns
///
/// `Ok(())` on success, or `Err(GaError::MutationError)` if `eta_m` is negative.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::polynomial::polynomial_mutation;
/// use genetic_algorithms::chromosomes::Range;
/// let mut chromosome: Range<f64> = Range::new();
/// let _ = polynomial_mutation(&mut chromosome, 20.0);
/// ```
pub fn polynomial_mutation<T>(
    individual: &mut RangeChromosome<T>,
    eta_m: f64,
) -> Result<(), GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + PolynomialConvertible,
{
    if eta_m < 0.0 {
        return Err(GaError::MutationError(format!(
            "Distribution index eta_m must be non-negative, got {}",
            eta_m
        )));
    }

    let len = individual.dna().len();
    if len == 0 {
        debug!(target="mutation_events", method="polynomial"; "Empty DNA, skipping polynomial mutation");
        return Ok(());
    }

    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    let mut gene = individual.dna()[idx].clone();

    if gene.ranges.is_empty() {
        debug!(target="mutation_events", method="polynomial"; "Gene {} has no ranges, skipping", idx);
        return Ok(());
    }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current_f64 = T::to_f64(gene.value);
    let lo_f64 = T::to_f64(lo);
    let hi_f64 = T::to_f64(hi);
    let range_span = hi_f64 - lo_f64;

    if range_span <= 0.0 {
        return Ok(());
    }

    // Sample u from (0, 1) — exclude endpoints to avoid domain errors in powf
    let u: f64 = rng.random_range(f64::EPSILON..(1.0 - f64::EPSILON));

    // Compute delta using polynomial distribution
    let delta = if u < 0.5 {
        (2.0 * u).powf(1.0 / (eta_m + 1.0)) - 1.0
    } else {
        1.0 - (2.0 * (1.0 - u)).powf(1.0 / (eta_m + 1.0))
    };

    let new_val_f64 = (current_f64 + delta * range_span).clamp(lo_f64, hi_f64);

    gene.value = T::from_f64(new_val_f64);
    individual.set_gene(idx, gene);

    debug!(target="mutation_events", method="polynomial"; "Polynomial mutation applied at gene {} with eta_m={}", idx, eta_m);
    Ok(())
}

/// Trait for types that can be converted to/from an f64 value (for polynomial mutation).
///
/// Implementations should do a reasonable conversion (e.g., rounding for integers).
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::mutation::polynomial::PolynomialConvertible;
/// assert_eq!(<f64 as PolynomialConvertible>::from_f64(0.5), 0.5_f64);
/// assert_eq!(<f64 as PolynomialConvertible>::to_f64(1.0_f64), 1.0_f64);
/// ```
pub trait PolynomialConvertible {
    /// Converts an f64 value to this type.
    fn from_f64(val: f64) -> Self;
    /// Converts a value of this type to f64.
    fn to_f64(val: Self) -> f64;
}

impl PolynomialConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl PolynomialConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl PolynomialConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl PolynomialConvertible for i64 {
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}
