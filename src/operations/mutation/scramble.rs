pub(crate) use rand::Rng;
use crate::traits::ChromosomeT;
use log::{trace, debug};

pub fn scramble<U: ChromosomeT>(chromosome: &mut U){

    //Getting two random genes from the dna of the chromosome
    debug!(target="mutation_events", method="scramble"; "Starting the scramble mutation");
    let mut rng = rand::rng();
    let index_1 = rng.random_range(0..chromosome.get_dna().len()-1);
    let index_2 = rng.random_range(index_1+1..chromosome.get_dna().len());
    trace!(target="mutation_events", method="scramble"; "Mutation index 1: {}, mutation index 2: {}", index_1, index_2);

    //We scramble genes
    for i in index_1..index_2{
        let random_index = rng.random_range(index_1..index_2);

        let current_gene = chromosome.get_dna().get(i).cloned().unwrap();
        let random_gene = chromosome.get_dna().get(random_index).cloned().unwrap();

        chromosome.set_gene(i, random_gene);
        chromosome.set_gene(random_index, current_gene);
    }

    debug!(target="mutation_events", method="scramble"; "Scramble mutation finished");
}