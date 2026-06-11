//! Bit-flip mutation operator for binary-encoded chromosomes.
//!
//! Inverts a single randomly chosen boolean gene (`true` ↔ `false`).
//! This is the canonical mutation operator for binary / bit-string
//! representations.

use crate::genotypes::Binary as BinaryGenotype;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;

/// Bit flip mutation: randomly flips the boolean value of one gene in a Binary chromosome.
///
/// Selects a random position and inverts its value (true ↔ false).
/// This is the natural mutation operator for binary-encoded chromosomes.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::mutation::bit_flip;
/// use genetic_algorithms::chromosomes::Binary;
/// let mut chromosome = Binary::new();
/// bit_flip(&mut chromosome);
/// ```
pub fn bit_flip<U>(chromosome: &mut U)
where
    U: LinearChromosome<Gene = BinaryGenotype>,
{
    let len = chromosome.dna().len();
    if len == 0 {
        return;
    }

    debug!(target="mutation_events", method="bit_flip"; "Starting the bit flip mutation");
    let mut rng = crate::rng::make_rng();
    let index = rng.random_range(0..len);

    let mut gene = chromosome.dna()[index];
    gene.value = !gene.value;
    chromosome.set_gene(index, gene);
    debug!(target="mutation_events", method="bit_flip"; "Bit flip mutation finished");
}
