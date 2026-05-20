//! Value mutation operator for range-encoded chromosomes.
//!
//! Replaces a single gene's value with a new one drawn uniformly from that
//! gene's declared range. This module also provides the [`ValueMutable`]
//! trait implementations for the built-in numeric `Range<T>` chromosome
//! types (`i32`, `i64`, `f32`, `f64`).

use crate::chromosomes::Range as RangeChromosome;
use crate::traits::LinearChromosome;
use rand::distr::uniform::SampleUniform;
use rand::Rng;
use std::fmt::Debug;

use super::ValueMutable;

/// Value mutation for `Range<T>` chromosomes.
///
/// - Randomly selects a gene from the DNA.
/// - Picks one of its ranges and assigns a new value uniformly within that range.
/// - Writes back the mutated gene into the individual's DNA.
///
/// If the chromosome has no genes or the selected gene has no ranges, it does nothing.
pub fn value_mutation<T>(individual: &mut RangeChromosome<T>)
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + SampleUniform + Copy + 'static,
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

    // Pick a random range in the gene
    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    // Generate a new value uniformly in [lo, hi]
    let new_val = rng.random_range(lo..=hi);

    gene.value = new_val;
    individual.set_gene(idx, gene);
}

/// `ValueMutable` implementation for `Range<i32>`.
impl ValueMutable for RangeChromosome<i32> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
    fn creep_mutate(&mut self, step: f64) {
        super::creep::creep_mutation(self, step as i32);
    }
    fn gaussian_mutate(&mut self, sigma: f64) {
        super::gaussian::gaussian_mutation(self, sigma);
    }
}

/// `ValueMutable` implementation for `Range<i64>`.
impl ValueMutable for RangeChromosome<i64> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
    fn creep_mutate(&mut self, step: f64) {
        super::creep::creep_mutation(self, step as i64);
    }
    fn gaussian_mutate(&mut self, sigma: f64) {
        super::gaussian::gaussian_mutation(self, sigma);
    }
}

/// `ValueMutable` implementation for `Range<f32>`.
impl ValueMutable for RangeChromosome<f32> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
    fn creep_mutate(&mut self, step: f64) {
        super::creep::creep_mutation(self, step as f32);
    }
    fn gaussian_mutate(&mut self, sigma: f64) {
        super::gaussian::gaussian_mutation(self, sigma);
    }
}

/// `ValueMutable` implementation for `Range<f64>`.
impl ValueMutable for RangeChromosome<f64> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
    fn creep_mutate(&mut self, step: f64) {
        super::creep::creep_mutation(self, step);
    }
    fn gaussian_mutate(&mut self, sigma: f64) {
        super::gaussian::gaussian_mutation(self, sigma);
    }
}
