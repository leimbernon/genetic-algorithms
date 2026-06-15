//! Lévy Flight mutation for `Range<T>` chromosomes using Mantegna's algorithm.
//! Generates heavy-tailed jumps via the formula
//! `step = σ_u * u / |v|^(1/α)` where `u ~ N(0, σ_u²)`, `v ~ N(0, 1)`.
//! See [`Mutation::LevyFlight`](crate::operations::Mutation::LevyFlight) for user-facing docs.

use crate::chromosomes::Range as RangeChromosome;
use crate::operations::mutation::gaussian::GaussianConvertible;
use crate::traits::LinearChromosome;
use rand::Rng;
use std::fmt::Debug;

/// Computes Mantegna's σ_u for stability index `alpha`.
/// Formula: σ_u = (Γ(1+α) * sin(πα/2) / (Γ((1+α)/2) * α * 2^((α-1)/2)))^(1/α).
/// Numerically stable for `alpha ∈ (0.1, 1.99]`; caller is expected to clamp.
fn mantegna_sigma_u(alpha: f64) -> f64 {
    let g1pa = gamma_approx(1.0 + alpha);
    let g1pa_half = gamma_approx((1.0 + alpha) / 2.0);
    let num = g1pa * (std::f64::consts::PI * alpha / 2.0).sin();
    let den = g1pa_half * alpha * 2.0_f64.powf((alpha - 1.0) / 2.0);
    (num / den).powf(1.0 / alpha)
}

/// Recursive Gamma approximation. Anchors on a polynomial fit valid for `x ∈ [1, 2]`
/// and uses Γ(x+1) = x·Γ(x) to handle other positive arguments. Avoids an external
/// Gamma crate. Sufficient precision for GA Lévy step generation.
fn gamma_approx(x: f64) -> f64 {
    if x < 1.0 {
        return gamma_approx(x + 1.0) / x;
    }
    if x > 2.0 {
        return (x - 1.0) * gamma_approx(x - 1.0);
    }
    let t = x - 1.0;
    // Polynomial coefficients from Abramowitz & Stegun 6.1.36 (Γ on [1, 2]).
    1.0 + t * (-0.5748646 + t * (0.9512363 + t * (-0.6998588 + t * (0.4245549 - t * 0.1010678))))
}

/// Applies Lévy Flight perturbation to a single randomly selected gene of `individual`.
/// `alpha` is the Lévy stability index (typical range: (0.0, 2.0); default 1.5).
/// The step is scaled by the gene range width `(hi - lo)` so behavior is range-independent.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::levy_flight::levy_flight_mutation;
/// use genetic_algorithms::chromosomes::Range;
/// let mut chromosome: Range<f64> = Range::new();
/// levy_flight_mutation(&mut chromosome, 1.5);
/// ```
pub fn levy_flight_mutation<T>(individual: &mut RangeChromosome<T>, alpha: f64)
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

    // Guard against Mantegna overflow at the α ∈ {0, 2} extremes (Pitfall 4).
    let alpha_clamped = alpha.clamp(0.1, 1.99);
    let sigma_u = mantegna_sigma_u(alpha_clamped);

    // u ~ N(0, sigma_u²) via Box-Muller (gaussian.rs lines ~52-55 pattern).
    let bu1: f64 = rng.random_range(f64::EPSILON..1.0);
    let bu2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let u_normal: f64 = (-2.0 * bu1.ln()).sqrt() * bu2.cos() * sigma_u;

    // v ~ N(0, 1) via Box-Muller.
    let bv1: f64 = rng.random_range(f64::EPSILON..1.0);
    let bv2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let v_normal: f64 = (-2.0 * bv1.ln()).sqrt() * bv2.cos();

    // Mantegna step, scaled by gene range width (Pitfall 3).
    // Guard against v_abs == 0 (can occur when bv2.cos() == 0): skip perturbation
    // rather than producing ±inf that would snap the gene to its boundary.
    let v_abs = v_normal.abs().powf(1.0 / alpha_clamped);
    let levy_step: f64 = if v_abs < f64::MIN_POSITIVE {
        0.0 // degenerate sample; skip perturbation this call
    } else {
        u_normal / v_abs
    };
    let noise: f64 = levy_step * (hi_f64 - lo_f64);

    let new_val_f64 = (current + noise).clamp(lo_f64, hi_f64);
    gene.value = T::from_f64(new_val_f64);
    individual.set_gene(idx, gene);

    crate::log_debug!(
        target: "mutation_events",
        "LevyFlight mutation applied at idx={} range_idx={} alpha={} (clamped {})",
        idx, range_idx, alpha, alpha_clamped
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mantegna_sigma_u_finite_positive_at_default_alpha() {
        let s = mantegna_sigma_u(1.5);
        assert!(s.is_finite() && s > 0.0, "σ_u(1.5) = {}", s);
        // Expected ~0.6966 (Yang 2010); allow loose tolerance.
        assert!(
            (s - 0.6966).abs() < 0.05,
            "σ_u(1.5) = {}, expected ~0.6966",
            s
        );
    }

    #[test]
    fn gamma_approx_known_values() {
        // Γ(1) = 1, Γ(2) = 1, Γ(3) = 2, Γ(0.5) = √π ≈ 1.7724
        assert!((gamma_approx(1.0) - 1.0).abs() < 1e-3);
        assert!((gamma_approx(2.0) - 1.0).abs() < 1e-3);
        assert!((gamma_approx(3.0) - 2.0).abs() < 5e-3);
        assert!((gamma_approx(0.5) - std::f64::consts::PI.sqrt()).abs() < 5e-3);
    }
}
