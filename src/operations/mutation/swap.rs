use crate::traits::ChromosomeT;
use log::{debug, trace};
pub(crate) use rand::Rng;

pub fn swap<U: ChromosomeT>(chromosome: &mut U) {
    //Getting two random genes from the dna of the chromosome
    debug!(target="mutation_events", method="swap"; "Starting the swap mutation");
    if chromosome.get_dna().len() < 2 {
        return;
    }
    let mut rng = rand::rng();
    let index_1 = rng.random_range(0..chromosome.get_dna().len());
    let index_2 = rng.random_range(0..chromosome.get_dna().len());
    trace!(target="mutation_events", method="swap"; "Mutation index 1: {}, mutation index 2: {}", index_1, index_2);

    let gene_1 = chromosome.get_dna().get(index_1).cloned().unwrap();
    let gene_2 = chromosome.get_dna().get(index_2).cloned().unwrap();

    //Swapping both genes
    chromosome.set_gene(index_1, gene_2);
    chromosome.set_gene(index_2, gene_1);
    debug!(target="mutation_events", method="swap"; "Swap mutation finished");
}
