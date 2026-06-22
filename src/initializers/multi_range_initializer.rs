//! Multi-range chromosome initializer.
//!
//! Creates genes for [`crate::chromosomes::MultiRangeChromosome`] by sampling
//! each gene's value uniformly from its own `(lo_i, hi_i)` bounds and assigning
//! its own `mutation_rate_i`.
//!
//! # Parallel-slice semantics
//!
//! `bounds` and `mutation_rates` are parallel slices — element `i` in `bounds`
//! corresponds to element `i` in `mutation_rates`. The resulting chromosome has
//! exactly `bounds.len()` genes. If `mutation_rates` is shorter than `bounds`,
//! any remaining genes receive the default rate of `0.1`.
//!
//! # Panics
//!
//! Panics if `lo >= hi` for any bound — `rng.random_range(lo..hi)` requires
//! a non-empty range. Ensure all bounds satisfy `lo < hi`.

use crate::genotypes::MultiRangeGenotype;
use rand::distr::uniform::SampleUniform;
use rand::Rng;
use std::fmt::Debug;

/// Initializes a vector of `MultiRangeGenotype` where each gene is sampled
/// from its own `(lo_i, hi_i)` range with its own mutation rate.
///
/// # Arguments
///
/// * `bounds` — One `(lo, hi)` pair per gene position. The resulting vector
///   has exactly `bounds.len()` genes.
/// * `mutation_rates` — One mutation rate per gene position. If shorter than
///   `bounds`, trailing genes receive a default rate of `0.1`.
///
/// # Returns
///
/// A vector of `MultiRangeGenotype<T>` genes, each with value sampled from
/// `[lo_i, hi_i)` and mutation rate set to `mutation_rates[i]` (or `0.1`).
///
/// # Panics
///
/// Panics if any `lo >= hi` (required by `rng.random_range`).
///
/// # Examples
///
/// ```
/// use genetic_algorithms::initializers::multi_range_random_initialization;
///
/// let bounds = vec![(0.0_f64, 1.0), (10.0, 20.0), (-5.0, -1.0)];
/// let rates = vec![0.1, 0.5, 0.2];
/// let genes = multi_range_random_initialization(&bounds, &rates);
/// assert_eq!(genes.len(), 3);
/// for gene in &genes {
///     assert!(gene.value >= gene.lo && gene.value < gene.hi);
/// }
/// ```
pub fn multi_range_random_initialization<T>(
    bounds: &[(T, T)],
    mutation_rates: &[f64],
) -> Vec<MultiRangeGenotype<T>>
where
    T: Sync + Send + Copy + Default + Debug + 'static + PartialOrd + SampleUniform,
{
    let mut rng = crate::rng::make_rng();
    let mut genes = Vec::with_capacity(bounds.len());
    for (i, &(lo, hi)) in bounds.iter().enumerate() {
        let value = rng.random_range(lo..hi);
        let rate = mutation_rates.get(i).copied().unwrap_or(0.1);
        genes.push(MultiRangeGenotype::new(i as i32, lo, hi, value, rate));
    }
    genes
}
