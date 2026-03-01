use crate::chromosomes::Range as RangeChromosome;
use crate::traits::ChromosomeT;
use rand::distr::uniform::SampleUniform;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;

use super::ValueMutable;

/// Value mutation for Range<T> chromosomes.
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
    let len = individual.get_dna().len();
    if len == 0 {
        return;
    }

    let mut rng = rand::rng();
    let idx = rng.random_range(0..len);

    let mut dna = individual.get_dna().to_vec();
    let mut gene = dna[idx].clone();

    if gene.ranges.is_empty() {
        return;
    }

    // Pick a random range in the gene
    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];

    // Generate a new value uniformly in [lo, hi]
    let new_val = rng.random_range(lo..=hi);

    gene.value = new_val;
    dna[idx] = gene;

    // Set the new DNA into the individual (move to avoid cloning)
    individual.set_dna(Cow::Owned(dna));
}

/// Implement ValueMutable for Range<i32>
impl ValueMutable for RangeChromosome<i32> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
}

/// Implement ValueMutable for Range<i64>
impl ValueMutable for RangeChromosome<i64> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
}

/// Implement ValueMutable for Range<f32>
impl ValueMutable for RangeChromosome<f32> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
}

/// Implement ValueMutable for Range<f64>
impl ValueMutable for RangeChromosome<f64> {
    fn value_mutate(&mut self) {
        value_mutation(self);
    }
}
