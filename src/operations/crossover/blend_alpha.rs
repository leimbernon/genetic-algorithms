//! Blend Crossover (BLX-alpha) implementation.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::LinearChromosome;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Blend Crossover (BLX-α) for `Range<T>` chromosomes.
///
/// Generates offspring values uniformly within an extended interval around the
/// parents' values. For each gene, if parents have values `p1` and `p2`:
///
/// ```text
/// d = |p1 - p2|
/// child ∈ [min(p1,p2) - α*d, max(p1,p2) + α*d]
/// ```
///
/// The parameter `alpha` controls the amount of exploration:
/// - α = 0: children are strictly between parents.
/// - α = 0.5: standard BLX-α, extends 50% beyond parents.
/// - α > 0.5: wider exploration.
///
/// # Arguments
///
/// * `parent_1` - First parent.
/// * `parent_2` - Second parent.
/// * `alpha` - Exploration parameter (typically 0.5).
///
/// # Returns
///
/// Two children chromosomes, or an error if parents have mismatched lengths.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::blend_alpha::blend_alpha;
/// use genetic_algorithms::chromosomes::Range;
/// let parent1: Range<f64> = Range::new();
/// let parent2: Range<f64> = Range::new();
/// let _ = blend_alpha(&parent1, &parent2, 0.5);
/// ```
pub fn blend_alpha<T>(
    parent_1: &RangeChromosome<T>,
    parent_2: &RangeChromosome<T>,
    alpha: f64,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + BlendConvertible,
{
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }

    crate::log_debug!(target="crossover_events", method="blend_alpha"; "Starting BLX-α crossover with alpha={}", alpha);

    let mut rng = crate::rng::make_rng();
    let dna1 = parent_1.dna();
    let dna2 = parent_2.dna();

    let mut child_dna_1 = Vec::with_capacity(len);
    let mut child_dna_2 = Vec::with_capacity(len);

    for i in 0..len {
        let p1_val: f64 = T::to_f64(dna1[i].value);
        let p2_val: f64 = T::to_f64(dna2[i].value);

        let min_val = p1_val.min(p2_val);
        let max_val = p1_val.max(p2_val);
        let d = max_val - min_val;

        let lo = min_val - alpha * d;
        let hi = max_val + alpha * d;

        // Clamp to gene range if available
        let (clamped_lo, clamped_hi) = if !dna1[i].ranges.is_empty() {
            let range_lo: f64 = T::to_f64(dna1[i].ranges[0].0);
            let range_hi: f64 = T::to_f64(dna1[i].ranges[0].1);
            (lo.max(range_lo), hi.min(range_hi))
        } else {
            (lo, hi)
        };

        let c1_val = rng.random_range(clamped_lo..=clamped_hi);
        let c2_val = rng.random_range(clamped_lo..=clamped_hi);

        let mut gene1 = dna1[i].clone();
        let mut gene2 = dna2[i].clone();
        gene1.value = T::from_f64(c1_val);
        gene2.value = T::from_f64(c2_val);

        child_dna_1.push(gene1);
        child_dna_2.push(gene2);
    }

    let mut child_1 = RangeChromosome::<T>::new();
    let mut child_2 = RangeChromosome::<T>::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    crate::log_debug!(target="crossover_events", method="blend_alpha"; "BLX-α crossover finished");
    Ok(vec![child_1, child_2])
}

/// Trait for types that can be converted to/from an f64 value (for BLX-α).
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::crossover::blend_alpha::BlendConvertible;
/// assert_eq!(<f64 as BlendConvertible>::from_f64(0.5), 0.5_f64);
/// assert_eq!(<f64 as BlendConvertible>::to_f64(1.0_f64), 1.0_f64);
/// ```
pub trait BlendConvertible {
    /// Converts an `f64` value to this type (e.g., rounding for integers).
    fn from_f64(val: f64) -> Self;
    /// Converts a value of this type to `f64`.
    fn to_f64(val: Self) -> f64;
}

impl BlendConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl BlendConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl BlendConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl BlendConvertible for i64 {
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}
