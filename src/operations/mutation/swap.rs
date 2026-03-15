//! Swap mutation operator.
//!
//! Selects two random positions in a chromosome's DNA and exchanges their
//! genes. This is one of the simplest mutation operators and works with any
//! chromosome type.

use crate::traits::ChromosomeT;
use log::{debug, trace};
pub(crate) use rand::Rng;

/// Swap mutation: randomly selects two positions in the chromosome and
/// exchanges their genes.
///
/// This operator preserves all alleles (no values are created or destroyed),
/// making it suitable for both value-encoded and permutation-encoded
/// chromosomes. If the DNA has fewer than 2 genes the function is a no-op.
pub fn swap<U: ChromosomeT>(chromosome: &mut U) {
    //Getting two random genes from the dna of the chromosome
    debug!(target="mutation_events", method="swap"; "Starting the swap mutation");
    if chromosome.dna().len() < 2 {
        return;
    }
    let mut rng = crate::rng::make_rng();
    let index_1 = rng.random_range(0..chromosome.dna().len());
    let index_2 = rng.random_range(0..chromosome.dna().len());
    trace!(target="mutation_events", method="swap"; "Mutation index 1: {}, mutation index 2: {}", index_1, index_2);

    let gene_1 = chromosome.dna().get(index_1).cloned().unwrap();
    let gene_2 = chromosome.dna().get(index_2).cloned().unwrap();

    //Swapping both genes
    chromosome.set_gene(index_1, gene_2);
    chromosome.set_gene(index_2, gene_1);
    debug!(target="mutation_events", method="swap"; "Swap mutation finished");
}
