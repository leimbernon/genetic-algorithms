pub(crate) use crate::traits::ChromosomeT;
use log::{debug, trace};
use rand::Rng;

pub fn inversion<U: ChromosomeT>(individual: &mut U) {
    // Starting the inversion mutation and obtaining two random indices
    debug!(target="mutation_events", method="inversion"; "Starting the inversion mutation");
    let mut rng = rand::rng();
    let len = individual.get_dna().len();

    // Select two distinct random indices
    let (index_1, index_2) = (rng.random_range(0..len), rng.random_range(0..len));
    let (lower_index, higher_index) = if index_1 < index_2 {
        (index_1, index_2)
    } else {
        (index_2, index_1)
    };

    trace!(target="mutation_events", method="inversion"; "Mutation lower index: {}, mutation higher index: {}", lower_index, higher_index);

    // Swap genes between the selected indices
    for i in 0..(higher_index - lower_index) / 2 {
        // Retrieve genes at both ends
        let gene_start = individual.get_dna()[lower_index + i].clone();
        let gene_end = individual.get_dna()[higher_index - i].clone();

        // Swap the genes using `set_gene` directly
        individual.set_gene(lower_index + i, gene_end);
        individual.set_gene(higher_index - i, gene_start);
    }

    debug!(target="mutation_events", method="inversion"; "Inversion mutation finished");
}
