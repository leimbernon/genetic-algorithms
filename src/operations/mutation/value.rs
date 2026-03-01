use rand::Rng;
use rand::distr::uniform::SampleUniform;
use crate::chromosomes::Range as RangeChromosome;
use crate::traits::ChromosomeT;
use std::borrow::Cow;
use std::any::Any;
use std::fmt::Debug;

use super::ValueMutable;
use super::swap::swap;

/// Value mutation for Range<T> chromosomes.
/// - Randomly selects a gene from the DNA.
/// - Picks one of its ranges and assigns a new value uniformly within that range.
/// - Writes back the mutated gene into the individual's DNA.
/// If the chromosome has no genes or the gene has no ranges, it does nothing.
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

/// Attempts value mutation on a chromosome. If the concrete type implements
/// `ValueMutable`, it performs value mutation. Otherwise, falls back to swap mutation.
///
/// Uses `Any` internally only as a dispatch mechanism to check known concrete types,
/// rather than coupling the factory to a single type.
pub fn try_value_mutation<U: ChromosomeT + 'static>(individual: &mut U) {
    let any = individual as &mut dyn Any;
    if let Some(r) = any.downcast_mut::<RangeChromosome<i32>>() {
        r.value_mutate();
    } else if let Some(r) = any.downcast_mut::<RangeChromosome<i64>>() {
        r.value_mutate();
    } else if let Some(r) = any.downcast_mut::<RangeChromosome<f32>>() {
        r.value_mutate();
    } else if let Some(r) = any.downcast_mut::<RangeChromosome<f64>>() {
        r.value_mutate();
    } else {
        // Fallback to swap mutation for types that don't support value mutation
        swap(individual);
    }
}
