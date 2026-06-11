//! Scramble mutation operator.
//!
//! Randomly shuffles the genes within a randomly chosen sub-range of the
//! chromosome. Like inversion mutation it preserves alleles, making it
//! suitable for permutation-based encodings.

use crate::traits::LinearChromosome;
use log::{debug, trace};
pub(crate) use rand::Rng;

/// Scramble mutation: randomly shuffles the genes between two randomly
/// selected positions in the chromosome.
///
/// Two indices `i` and `j` are picked such that `i < j`, and every gene
/// in `[i, j)` is swapped with another randomly chosen gene in the same
/// range. If the DNA has fewer than 2 genes the function is a no-op.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::scramble;
/// use genetic_algorithms::chromosomes::Binary;
/// let mut chromosome = Binary::new();
/// scramble(&mut chromosome);
/// ```
pub fn scramble<U: LinearChromosome>(chromosome: &mut U) {
    //Getting two random genes from the dna of the chromosome
    debug!(target="mutation_events", method="scramble"; "Starting the scramble mutation");
    if chromosome.dna().len() < 2 {
        return;
    }
    let mut rng = crate::rng::make_rng();
    let index_1 = rng.random_range(0..chromosome.dna().len() - 1);
    let index_2 = rng.random_range(index_1 + 1..chromosome.dna().len());
    trace!(target="mutation_events", method="scramble"; "Mutation index 1: {}, mutation index 2: {}", index_1, index_2);

    //We scramble genes in-place
    for i in index_1..index_2 {
        let random_index = rng.random_range(index_1..index_2);
        chromosome.dna_mut().swap(i, random_index);
    }

    debug!(target="mutation_events", method="scramble"; "Scramble mutation finished");
}
