//! Simulated Binary Crossover (SBX) implementation.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Simulated Binary Crossover (SBX) for `Range<T>` chromosomes.
///
/// SBX simulates single-point crossover in continuous space. The distribution
/// index `eta` controls how close offspring are to their parents:
/// - Low eta (e.g., 2): offspring spread far from parents (exploration).
/// - High eta (e.g., 20): offspring stay close to parents (exploitation).
///
/// This is the standard crossover for continuous numerical optimization
/// (e.g., real-valued function optimization).
///
/// # Arguments
///
/// * `parent_1` - First parent chromosome.
/// * `parent_2` - Second parent chromosome.
/// * `eta` - Distribution index (typically 2–20).
///
/// # Returns
///
/// Two children chromosomes, or an error if parents have mismatched lengths.
pub fn sbx<T>(
    parent_1: &RangeChromosome<T>,
    parent_2: &RangeChromosome<T>,
    eta: f64,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + SbxConvertible,
{
    let len = parent_1.dna().len();
    if len != parent_2.dna().len() {
        return Err(GaError::CrossoverError(format!(
            "Parents must have the same DNA length. Parent 1: {}, Parent 2: {}",
            len,
            parent_2.dna().len()
        )));
    }

    debug!(target="crossover_events", method="sbx"; "Starting SBX crossover with eta={}", eta);

    let mut rng = crate::rng::make_rng();
    let dna1 = parent_1.dna();
    let dna2 = parent_2.dna();

    let mut child_dna_1 = Vec::with_capacity(len);
    let mut child_dna_2 = Vec::with_capacity(len);

    for i in 0..len {
        let p1_val: f64 = T::to_f64(dna1[i].value);
        let p2_val: f64 = T::to_f64(dna2[i].value);

        if (p1_val - p2_val).abs() < 1e-14 {
            // Parents are identical at this gene; children inherit the same value
            child_dna_1.push(dna1[i].clone());
            child_dna_2.push(dna2[i].clone());
            continue;
        }

        let u: f64 = rng.random_range(0.0..1.0);

        // Compute the spread factor beta
        let beta = if u <= 0.5 {
            (2.0 * u).powf(1.0 / (eta + 1.0))
        } else {
            (1.0 / (2.0 * (1.0 - u))).powf(1.0 / (eta + 1.0))
        };

        let c1_val = 0.5 * ((1.0 + beta) * p1_val + (1.0 - beta) * p2_val);
        let c2_val = 0.5 * ((1.0 - beta) * p1_val + (1.0 + beta) * p2_val);

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

    let mut child_1 = RangeChromosome::<T>::new();
    let mut child_2 = RangeChromosome::<T>::new();
    child_1.set_dna(Cow::Owned(child_dna_1));
    child_2.set_dna(Cow::Owned(child_dna_2));

    debug!(target="crossover_events", method="sbx"; "SBX crossover finished");
    Ok(vec![child_1, child_2])
}

/// Trait for types that can be converted to/from an f64 value (for SBX).
pub trait SbxConvertible {
    /// Converts an `f64` value to this type (e.g., rounding for integers).
    fn from_f64(val: f64) -> Self;
    /// Converts a value of this type to `f64`.
    fn to_f64(val: Self) -> f64;
}

impl SbxConvertible for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }
    fn to_f64(val: Self) -> f64 {
        val
    }
}

impl SbxConvertible for f32 {
    fn from_f64(val: f64) -> Self {
        val as f32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl SbxConvertible for i32 {
    fn from_f64(val: f64) -> Self {
        val.round() as i32
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}

impl SbxConvertible for i64 {
    fn from_f64(val: f64) -> Self {
        val.round() as i64
    }
    fn to_f64(val: Self) -> f64 {
        val as f64
    }
}
