use crate::genotypes::Binary as BinaryGenotype;
use crate::traits::ChromosomeT;
use log::debug;
use rand::Rng;

/// Bit flip mutation: randomly flips the boolean value of one gene in a Binary chromosome.
///
/// Selects a random position and inverts its value (true ↔ false).
/// This is the natural mutation operator for binary-encoded chromosomes.
pub fn bit_flip<U>(chromosome: &mut U)
where
    U: ChromosomeT<Gene = BinaryGenotype>,
{
    let len = chromosome.dna().len();
    if len == 0 {
        return;
    }

    debug!(target="mutation_events", method="bit_flip"; "Starting the bit flip mutation");
    let mut rng = rand::rng();
    let index = rng.random_range(0..len);

    let mut gene = chromosome.dna()[index];
    gene.value = !gene.value;
    chromosome.set_gene(index, gene);
    debug!(target="mutation_events", method="bit_flip"; "Bit flip mutation finished");
}
