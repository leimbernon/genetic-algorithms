//! Arithmetic crossover implementation.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use std::borrow::Cow;
use std::fmt::Debug;

/// Arithmetic (whole) crossover for `Range<T>` chromosomes.
///
/// Produces two offspring by taking weighted linear combinations of the parents'
/// gene values. For each gene position `i`:
///
/// ```text
/// child1_i = α * parent1_i + (1 - α) * parent2_i
/// child2_i = (1 - α) * parent1_i + α * parent2_i
/// ```
///
/// When `alpha = 0.5` this becomes **uniform arithmetic crossover**, where both
/// children are the midpoint of their parents at every gene.
///
/// Results are clamped to the gene's range bounds when available.
///
/// # Arguments
///
/// * `parent_1` - First parent chromosome.
/// * `parent_2` - Second parent chromosome.
/// * `alpha` - Weighting factor, typically in `[0, 1]`. `0.5` gives uniform
///   arithmetic crossover.
///
/// # Returns
///
/// Two children chromosomes, or an error if parents have mismatched DNA lengths.
pub fn arithmetic<T>(
    parent_1: &RangeChromosome<T>,
    parent_2: &RangeChromosome<T>,
    alpha: f64,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + ArithmeticConvertible,
{
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }

    debug!(target="crossover_events", method="arithmetic"; "Starting arithmetic crossover with alpha={}", alpha);

    let dna1 = parent_1.dna();
    let dna2 = parent_2.dna();

    let mut child_dna_1 = Vec::with_capacity(len);
    let mut child_dna_2 = Vec::with_capacity(len);

    for i in 0..len {
        let p1_val: f64 = T::to_f64(dna1[i].value);
        let p2_val: f64 = T::to_f64(dna2[i].value);

        let c1_val = alpha * p1_val + (1.0 - alpha) * p2_val;
        let c2_val = (1.0 - alpha) * p1_val + alpha * p2_val;

        // Clamp to range if available
        let (c1_clamped, c2_clamped) = if !dna1[i].ranges.is_empty() {
            let lo: f64 = T::to_f64(dna1[i].ranges[0].0);
            let hi: f64 = T::to_f64(dna1[i].ranges[0].1);
            (c1_val.clamp(lo, hi), c2_val.clamp(lo, hi))
        } else {
            (c1_val, c2_val)
        };

        let mut gene1 = dna1[i].clone();
        let mut gene2 = dna2[i].clone();
        gene1.value = T::from_f64(c1_clamped);
        gene2.value = T::from_f64(c2_clamped);

        child_dna_1.push(gene1);
        child_dna_2.push(gene2);
    }

    let mut child_1 = parent_1.clone();
    let mut child_2 = parent_2.clone();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    debug!(target="crossover_events", method="arithmetic"; "Arithmetic crossover finished");
    Ok(vec![child_1, child_2])
}

/// Trait for types that can be converted to/from an `f64` value (for arithmetic crossover).
pub trait ArithmeticConvertible {
    /// Convert an `f64` into `Self`.
    fn from_f64(val: f64) -> Self;
    /// Convert `Self` into an `f64`.
    fn to_f64(val: Self) -> f64;
}

impl ArithmeticConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl ArithmeticConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl ArithmeticConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl ArithmeticConvertible for i64 {
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

