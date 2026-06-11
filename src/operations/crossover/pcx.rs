//! PCX (Parent-Centric Crossover) for real-valued chromosomes.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::operations::crossover::sbx::SbxConvertible;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Parent-Centric Crossover (PCX) for `RangeChromosome<T>`.
///
/// PCX generates offspring centered around the primary parent (`parents[0]`)
/// with perturbations along the directions to other parents (directional noise
/// scaled by `sigma_eta`) and an orthogonal noise term proportional to the
/// per-gene spread across parents (`sigma_zeta`). This operator is more
/// exploitative than UNDX or SPX.
///
/// Default parameters: `sigma_eta = 0.1`, `sigma_zeta = 0.1`.
///
/// # Arguments
///
/// * `parents` - Slice of at least 3 parent chromosomes; `parents[0]` is the primary parent.
/// * `_num_parents` - Accepted but unused; `parents.len()` is authoritative.
/// * `sigma_eta_override` - Optional override for the directional noise scale.
///   When `None`, the default value `0.1` is used.
/// * `sigma_zeta_override` - Optional override for the orthogonal noise scale.
///   When `None`, the default value `0.1` is used.
///
/// # Returns
///
/// A `Vec` containing exactly 1 offspring, or a `GaError::CrossoverError` if
/// fewer than 3 parents are provided or parent DNA lengths are mismatched.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::pcx::pcx;
/// use genetic_algorithms::chromosomes::Range;
/// let p1: Range<f64> = Range::new();
/// let p2: Range<f64> = Range::new();
/// let p3: Range<f64> = Range::new();
/// let _ = pcx(&[&p1, &p2, &p3], 3, None, None);
/// ```
pub fn pcx<T>(
    parents: &[&RangeChromosome<T>],
    _num_parents: usize,
    sigma_eta_override: Option<f64>,
    sigma_zeta_override: Option<f64>,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + SbxConvertible,
{
    debug!(target: "crossover_events", method = "pcx"; "Starting PCX crossover with {} parents", parents.len());

    if parents.len() < 3 {
        return Err(GaError::CrossoverError(
            "PCX requires at least 3 parents".to_string(),
        ));
    }

    let expected = parents[0].dna().len();
    for (idx, p) in parents.iter().enumerate().skip(1) {
        let actual = p.dna().len();
        if actual != expected {
            return Err(GaError::CrossoverError(format!(
                "All parents must have the same DNA length. Expected {}, got {} (parent {})",
                expected, actual, idx
            )));
        }
    }

    if expected == 0 {
        let child = RangeChromosome::<T>::new();
        debug!(target: "crossover_events", method = "pcx"; "PCX crossover finished");
        return Ok(vec![child]);
    }

    let sigma_eta = sigma_eta_override.unwrap_or(0.1_f64);
    let sigma_zeta = sigma_zeta_override.unwrap_or(0.1_f64);

    // Compute per-gene spread across all parents
    let spread: Vec<f64> = (0..expected)
        .map(|i| {
            let max_val = parents
                .iter()
                .map(|p| T::to_f64(p.dna()[i].value))
                .fold(f64::NEG_INFINITY, f64::max);
            let min_val = parents
                .iter()
                .map(|p| T::to_f64(p.dna()[i].value))
                .fold(f64::INFINITY, f64::min);
            max_val - min_val
        })
        .collect();

    let mut rng = crate::rng::make_rng();
    let dna0 = parents[0].dna();
    let p0_vals: Vec<f64> = (0..expected)
        .map(|i| T::to_f64(dna0[i].value))
        .collect();

    // ONE Box-Muller draw per parent direction vector; accumulate across all gene dimensions
    let mut directional = vec![0.0_f64; expected];
    for p in parents.iter().skip(1) {
        let u1: f64 = rng.random_range(f64::EPSILON..1.0);
        let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
        let eta_j: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_eta;
        for i in 0..expected {
            directional[i] += eta_j * (T::to_f64(p.dna()[i].value) - p0_vals[i]);
        }
    }

    // Per-gene orthogonal perturbation: independent noise scaled by spread
    let mut child_dna = Vec::with_capacity(expected);
    for i in 0..expected {
        let u1_z: f64 = rng.random_range(f64::EPSILON..1.0);
        let u2_z: f64 = rng.random_range(0.0..std::f64::consts::TAU);
        let zeta: f64 = (-2.0 * u1_z.ln()).sqrt() * u2_z.cos() * sigma_zeta * spread[i];

        let raw = p0_vals[i] + directional[i] + zeta;

        let clamped = if !dna0[i].ranges.is_empty() {
            let lo: f64 = T::to_f64(dna0[i].ranges[0].0);
            let hi: f64 = T::to_f64(dna0[i].ranges[0].1);
            raw.clamp(lo, hi)
        } else {
            raw
        };

        let mut gene = dna0[i].clone();
        gene.value = T::from_f64(clamped);
        child_dna.push(gene);
    }

    let mut child = RangeChromosome::<T>::new();
    child.set_dna(Cow::Owned(child_dna));

    debug!(target: "crossover_events", method = "pcx"; "PCX crossover finished");
    Ok(vec![child])
}
