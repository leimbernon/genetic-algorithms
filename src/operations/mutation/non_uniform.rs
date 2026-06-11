//! Non-uniform mutation operator for range-encoded chromosomes.
//!
//! The mutation magnitude decreases over generations: broad exploration
//! early in the run, fine-tuning near the end. The decay rate is governed
//! by a time-dependent factor `tau = (1 - t/T)^b` where `t` is the
//! current generation, `T` the maximum, and `b` the decay exponent
//! (typically 2–5).
//!
//! Also defines the [`NonUniformConvertible`] trait for `f64` conversion.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::fmt::Debug;

/// Non-uniform mutation for `Range<T>` chromosomes.
///
/// The mutation magnitude decreases over generations, making it suitable for
/// algorithms that need broad exploration early on and fine-tuning later.
///
/// The time-dependent factor `tau = (1 - generation / max_generations)^b`
/// controls how rapidly the mutation amplitude decays:
/// - `b = 1`: linear decay
/// - `b = 2–5`: increasingly aggressive decay (typical values)
///
/// At early generations the operator can make large jumps; near the end of
/// the run it behaves almost like a local search.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `generation` - The current generation number (0-indexed).
/// * `max_generations` - The total number of generations planned.
/// * `b` - The decay exponent controlling how fast mutation shrinks (typically 2–5).
///
/// # Returns
///
/// `Ok(())` on success, or `Err(GaError::MutationError)` on invalid parameters.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::non_uniform_mutation;
/// use genetic_algorithms::chromosomes::Range;
/// let mut chromosome: Range<f64> = Range::new();
/// let _ = non_uniform_mutation(&mut chromosome, 10, 100, 2.0);
/// ```
pub fn non_uniform_mutation<T>(
    individual: &mut RangeChromosome<T>,
    generation: usize,
    max_generations: usize,
    b: f64,
) -> Result<(), GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + NonUniformConvertible,
{
    if max_generations == 0 {
        return Err(GaError::MutationError(
            "max_generations must be greater than 0".to_string(),
        ));
    }

    if b < 0.0 {
        return Err(GaError::MutationError(format!(
            "Decay exponent b must be non-negative, got {}",
            b
        )));
    }

    let len = individual.dna().len();
    if len == 0 {
        debug!(target="mutation_events", method="non_uniform"; "Empty DNA, skipping non-uniform mutation");
        return Ok(());
    }

    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);

    let mut gene = individual.dna()[idx].clone();

    if gene.ranges.is_empty() {
        debug!(target="mutation_events", method="non_uniform"; "Gene {} has no ranges, skipping", idx);
        return Ok(());
    }

    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let current_f64 = T::to_f64(gene.value);
    let lo_f64 = T::to_f64(lo);
    let hi_f64 = T::to_f64(hi);

    // Time-dependent factor: decays from 1.0 (generation 0) toward 0.0
    let t_ratio = generation as f64 / max_generations as f64;
    let tau = (1.0 - t_ratio.min(1.0)).powf(b);

    let r: f64 = rng.random_range(0.0_f64..1.0_f64);
    let coin: bool = rng.random_bool(0.5);

    let new_val_f64 = if coin {
        // Mutate toward upper bound
        current_f64 + (hi_f64 - current_f64) * (1.0 - r.powf(tau))
    } else {
        // Mutate toward lower bound
        current_f64 - (current_f64 - lo_f64) * (1.0 - r.powf(tau))
    };

    let clamped = new_val_f64.clamp(lo_f64, hi_f64);
    gene.value = T::from_f64(clamped);
    individual.set_gene(idx, gene);

    debug!(
        target="mutation_events", method="non_uniform";
        "Non-uniform mutation applied at gene {} (gen={}/{}, b={}, tau={:.4})",
        idx, generation, max_generations, b, tau
    );
    Ok(())
}

/// Trait for types that can be converted to/from an f64 value (for non-uniform mutation).
///
/// Implementations should do a reasonable conversion (e.g., rounding for integers).
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::mutation::NonUniformConvertible;
/// assert_eq!(f64::from_f64(0.5), 0.5_f64);
/// assert_eq!(f64::to_f64(1.0_f64), 1.0_f64);
/// ```
pub trait NonUniformConvertible {
    /// Converts an f64 value to this type.
    fn from_f64(val: f64) -> Self;
    /// Converts a value of this type to f64.
    fn to_f64(val: Self) -> f64;
}

impl NonUniformConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl NonUniformConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl NonUniformConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl NonUniformConvertible for i64 {
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}
