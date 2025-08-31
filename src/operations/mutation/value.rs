use rand::Rng;
use crate::chromosomes::Range as RangeChromosome;
use std::borrow::Cow;
use crate::traits::ChromosomeT;

/// Value mutation for Range<i32> chromosomes.
/// - Randomly selects a gene from the DNA.
/// - Picks one of its ranges and assigns a new integer value uniformly within that range.
/// - Writes back the mutated gene into the individual's DNA.
/// If the chromosome has no genes or the gene has no ranges, it does nothing.
pub fn value_mutation(individual: &mut RangeChromosome<i32>) {
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

    // Generate a new integer value uniformly in [lo, hi]
    let new_val = rng.random_range(lo..=hi);

    gene.value = new_val;
    dna[idx] = gene;

    // Set the new DNA into the individual (move to avoid cloning)
    individual.set_dna(Cow::Owned(dna));
}
