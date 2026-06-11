//! UNDX (Unimodal Normal Distribution Crossover) for real-valued chromosomes.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::operations::crossover::sbx::SbxConvertible;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

/// Unimodal Normal Distribution Crossover (UNDX) for `RangeChromosome<T>`.
///
/// UNDX generates offspring centered at the centroid of all parents, with
/// perturbation normally distributed along the primary inter-parent direction
/// (sigma_eta) and orthogonal directions (sigma_xi).
///
/// Standard parameters: `sigma_xi = 0.35 / sqrt(n_parents - 1)` for orthogonal
/// components and `sigma_eta = 0.35 / sqrt(n_parents)` for the principal direction.
///
/// # Arguments
///
/// * `parents` - Slice of at least 3 parent chromosomes.
/// * `_num_parents` - Accepted but unused; `parents.len()` is authoritative.
/// * `sigma_xi_override` - Optional override for σ_xi. When `None`, the standard
///   value `0.35 / sqrt(n_parents - 1)` is used.
/// * `sigma_eta_override` - Optional override for σ_eta. When `None`, the standard
///   value `0.35 / sqrt(n_parents)` is used.
///
/// # Returns
///
/// A `Vec` containing exactly 1 offspring, or a `GaError::CrossoverError` if
/// fewer than 3 parents are provided or parent DNA lengths are mismatched.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::crossover::undx::undx;
/// use genetic_algorithms::chromosomes::Range;
/// let p1: Range<f64> = Range::new();
/// let p2: Range<f64> = Range::new();
/// let p3: Range<f64> = Range::new();
/// let _ = undx(&[&p1, &p2, &p3], 3, None, None);
/// ```
pub fn undx<T>(
    parents: &[&RangeChromosome<T>],
    _num_parents: usize,
    sigma_xi_override: Option<f64>,
    sigma_eta_override: Option<f64>,
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + SbxConvertible,
{
    debug!(target: "crossover_events", method = "undx"; "Starting UNDX crossover with {} parents", parents.len());

    if parents.len() < 3 {
        return Err(GaError::CrossoverError(
            "UNDX requires at least 3 parents".to_string(),
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
        debug!(target: "crossover_events", method = "undx"; "UNDX crossover finished");
        return Ok(vec![child]);
    }

    let n_par = parents.len() as f64;

    // Per-gene centroid
    let mut centroid = vec![0.0_f64; expected];
    for p in parents.iter() {
        for (i, gene) in p.dna().iter().enumerate() {
            centroid[i] += T::to_f64(gene.value);
        }
    }
    for c in centroid.iter_mut() {
        *c /= n_par;
    }

    let sigma_xi = sigma_xi_override.unwrap_or_else(|| 0.35 / (n_par - 1.0).max(1.0).sqrt());
    let sigma_eta = sigma_eta_override.unwrap_or_else(|| 0.35 / n_par.sqrt());

    // Primary direction: parents[0] - centroid
    let mut dir: Vec<f64> = parents[0]
        .dna()
        .iter()
        .enumerate()
        .map(|(i, gene)| T::to_f64(gene.value) - centroid[i])
        .collect();
    let dir_norm = dir.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-14);
    for d in dir.iter_mut() {
        *d /= dir_norm;
    }

    let mut rng = crate::rng::make_rng();

    // Draw one global eta for the primary direction (Box-Muller)
    let u1_eta: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2_eta: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let eta_noise: f64 = (-2.0 * u1_eta.ln()).sqrt() * u2_eta.cos() * sigma_eta;

    // Draw raw isotropic xi noise vector (Box-Muller per gene)
    let raw_xi: Vec<f64> = (0..expected)
        .map(|_| {
            let u1: f64 = rng.random_range(f64::EPSILON..1.0);
            let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
            (-2.0 * u1.ln()).sqrt() * u2.cos()
        })
        .collect();

    // Project out the primary-direction component so xi is orthogonal to dir.
    // dir is already a unit vector; dot product gives the projection coefficient.
    let dot: f64 = raw_xi.iter().zip(dir.iter()).map(|(n, d)| n * d).sum::<f64>();
    let xi_perp: Vec<f64> = raw_xi
        .iter()
        .zip(dir.iter())
        .map(|(n, d)| (n - dot * d) * sigma_xi)
        .collect();

    let mut child_dna = Vec::with_capacity(expected);
    let dna0 = parents[0].dna();

    for i in 0..expected {
        let raw = centroid[i] + eta_noise * dir[i] + xi_perp[i];

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

    debug!(target: "crossover_events", method = "undx"; "UNDX crossover finished");
    Ok(vec![child])
}
