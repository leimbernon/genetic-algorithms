//! SPX (Simplex Crossover) for real-valued chromosomes.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::operations::crossover::sbx::SbxConvertible;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Simplex Crossover (SPX) for `RangeChromosome<T>`.
///
/// SPX defines a simplex from the parent chromosomes, expands it by a factor
/// `epsilon = sqrt(n_parents + 2)`, and samples an offspring uniformly from
/// the interior of the expanded simplex. This provides good global exploration
/// in multi-parent populations.
///
/// # Arguments
///
/// * `parents` - Slice of at least 3 parent chromosomes.
/// * `_num_parents` - Accepted but unused; `parents.len()` is authoritative.
///
/// # Returns
///
/// A `Vec` containing exactly 1 offspring, or a `GaError::CrossoverError` if
/// fewer than 3 parents are provided or parent DNA lengths are mismatched.
pub fn spx<T>(
    parents: &[&RangeChromosome<T>],
    _num_parents: usize,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + SbxConvertible,
{
    debug!(target: "crossover_events", method = "spx"; "Starting SPX crossover with {} parents", parents.len());

    if parents.len() < 3 {
        return Err(GaError::CrossoverError(
            "SPX requires at least 3 parents".to_string(),
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
        debug!(target: "crossover_events", method = "spx"; "SPX crossover finished");
        return Ok(vec![child]);
    }

    let n_par = parents.len();
    let epsilon = ((n_par + 2) as f64).sqrt();

    // Compute centroid
    let mut centroid = vec![0.0_f64; expected];
    for p in parents.iter() {
        for (i, gene) in p.dna().iter().enumerate() {
            centroid[i] += T::to_f64(gene.value);
        }
    }
    for c in centroid.iter_mut() {
        *c /= n_par as f64;
    }

    // Expanded simplex vertices: centroid + epsilon * (parent - centroid)
    let expanded: Vec<Vec<f64>> = parents
        .iter()
        .map(|p| {
            p.dna()
                .iter()
                .enumerate()
                .map(|(i, gene)| centroid[i] + epsilon * (T::to_f64(gene.value) - centroid[i]))
                .collect()
        })
        .collect();

    let mut rng = crate::rng::make_rng();

    // Sample r_k for k in 0..n_par-1, then push 1.0 so r.len() == n_par
    let mut r: Vec<f64> = (0..n_par - 1)
        .map(|k| {
            rng.random_range(0.0_f64..1.0)
                .powf(1.0 / (n_par - 1 - k) as f64)
        })
        .collect();
    r.push(1.0);

    // Iterative combination from the last expanded vertex inward
    let mut offspring_vals = expanded[n_par - 1].clone();
    for k in (0..n_par - 1).rev() {
        for i in 0..expected {
            offspring_vals[i] = r[k] * expanded[k][i] + (1.0 - r[k]) * offspring_vals[i];
        }
    }

    let dna0 = parents[0].dna();
    let mut child_dna = Vec::with_capacity(expected);

    for i in 0..expected {
        let clamped = if !dna0[i].ranges.is_empty() {
            let lo: f64 = T::to_f64(dna0[i].ranges[0].0);
            let hi: f64 = T::to_f64(dna0[i].ranges[0].1);
            offspring_vals[i].clamp(lo, hi)
        } else {
            offspring_vals[i]
        };

        let mut gene = dna0[i].clone();
        gene.value = T::from_f64(clamped);
        child_dna.push(gene);
    }

    let mut child = RangeChromosome::<T>::new();
    child.set_dna(Cow::Owned(child_dna));

    debug!(target: "crossover_events", method = "spx"; "SPX crossover finished");
    Ok(vec![child])
}
