//! Binomial and exponential crossover for Differential Evolution.

use super::configuration::DeCrossoverMode;
use crate::traits::RealGene;
use rand::Rng;

/// Apply crossover between the `target` (current individual) and `mutant`,
/// returning the trial vector.
///
/// At least one gene is always taken from the `mutant` (via the mandatory
/// `j_rand` in binomial, or the starting index in exponential).
pub fn crossover<G: RealGene>(
    mode: &DeCrossoverMode,
    target: &[G],
    mutant: &[G],
    cr: f64,
    rng: &mut impl Rng,
) -> Vec<G> {
    let dim = target.len();
    match mode {
        DeCrossoverMode::Binomial => binomial(target, mutant, cr, dim, rng),
        DeCrossoverMode::Exponential => exponential(target, mutant, cr, dim, rng),
    }
}

/// Binomial crossover: each gene independently comes from the mutant with
/// probability `cr`; the mandatory `j_rand` gene always comes from mutant.
fn binomial<G: RealGene>(
    target: &[G],
    mutant: &[G],
    cr: f64,
    dim: usize,
    rng: &mut impl Rng,
) -> Vec<G> {
    let j_rand = rng.random_range(0..dim);
    (0..dim)
        .map(|j| {
            if j == j_rand || rng.random::<f64>() < cr {
                mutant[j].clone()
            } else {
                target[j].clone()
            }
        })
        .collect()
}

/// Exponential crossover: copy a contiguous block from mutant starting at a
/// random index `l`, continuing while `rand() < cr`, wrapping around if needed.
fn exponential<G: RealGene>(
    target: &[G],
    mutant: &[G],
    cr: f64,
    dim: usize,
    rng: &mut impl Rng,
) -> Vec<G> {
    let l = rng.random_range(0..dim);
    let mut take = vec![false; dim];
    take[l] = true;
    let mut j = (l + 1) % dim;
    while j != l && rng.random::<f64>() < cr {
        take[j] = true;
        j = (j + 1) % dim;
    }
    (0..dim)
        .map(|k| {
            if take[k] {
                mutant[k].clone()
            } else {
                target[k].clone()
            }
        })
        .collect()
}
