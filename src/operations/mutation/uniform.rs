//! Uniform reset mutation for `Range<T>` chromosomes.
//!
//! Picks a random gene and a random declared range, then resets the gene value
//! to a uniform sample within that range. Equivalent to gene re-initialization.
//!
//! See [`Mutation::Uniform`](crate::operations::Mutation::Uniform) for user-facing docs.

use crate::chromosomes::Range as RangeChromosome;
use crate::operations::mutation::gaussian::GaussianConvertible;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;
use std::fmt::Debug;

/// Resets a single randomly selected gene of `individual` to a uniform sample
/// within one of its declared ranges. Equivalent to gene re-initialization.
///
/// # Arguments
///
/// * `individual` - The chromosome to mutate (exactly one gene is altered).
pub fn uniform_mutation<T>(individual: &mut RangeChromosome<T>)
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

    // D-04: pick a random range when multiple are declared (mirrors gaussian.rs range_idx).
    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    let lo_f64: f64 = T::to_f64(lo);
    let hi_f64: f64 = T::to_f64(hi);

    // D-03: full reset to a uniform sample within the selected range.
    // No clamp needed — random_range stays in [lo_f64, hi_f64] by construction.
    // For integer types, T::from_f64 rounds (Pitfall 6) — intentional.
    let new_val_f64: f64 = rng.random_range(lo_f64..=hi_f64);
    gene.value = T::from_f64(new_val_f64);
    individual.set_gene(idx, gene);

    debug!(
        target: "mutation_events",
        "Uniform mutation applied at idx={} range_idx={}",
        idx, range_idx
    );
}
