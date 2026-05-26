//! Creep mutation operator for range-encoded chromosomes.
//!
//! Applies a small uniform perturbation to a single gene. The magnitude
//! of the perturbation is bounded by a user-specified `step` size and the
//! result is always clamped to the gene's declared range. Creep mutation
//! is ideal for fine-tuning solutions in continuous or integer search
//! spaces.

use crate::chromosomes::Range as RangeChromosome;
use crate::traits::LinearChromosome;
use rand::distr::uniform::SampleUniform;
use rand::Rng;
use std::fmt::Debug;

/// Creep mutation for `Range<T>` chromosomes.
///
/// Applies a small uniform perturbation to a randomly selected gene.
/// The perturbation is drawn from [-step, +step] and the result is clamped
/// to the gene's declared range.
///
/// This operator is useful for fine-tuning solutions in continuous or
/// integer optimization problems.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate.
/// * `step` - The maximum perturbation magnitude in each direction.
pub fn creep_mutation<T>(individual: &mut RangeChromosome<T>, step: T)
where
    T: Sync
        + Send
        + Clone
        + Default
        + Debug
        + PartialOrd
        + SampleUniform
        + Copy
        + 'static
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>,
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

    // Compute perturbation bounds clamped to the gene's range
    let current = gene.value;
    let perturb_lo = if current - step > lo {
        current - step
    } else {
        lo
    };
    let perturb_hi = if current + step < hi {
        current + step
    } else {
        hi
    };

    let new_val = rng.random_range(perturb_lo..=perturb_hi);
    gene.value = new_val;
    individual.set_gene(idx, gene);
}
