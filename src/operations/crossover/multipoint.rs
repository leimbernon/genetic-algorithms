use crate::traits::ChromosomeT;
use log::{trace, debug};

pub fn multipoint<U: ChromosomeT>(parent_1: &U, parent_2: &U, crossover_number_of_points: &i32) -> Option<Vec<U>>{

    //Before doing the operation, we check that the dna in parent 1 has the same length of the dna in parent 2
    if parent_1.get_dna().len() != parent_2.get_dna().len() {
        panic!("Parent 1 and parent 2 must have the same dna length. Parent 1 has a length of {} and parent 2 has a length of {}", parent_1.get_dna().len(), parent_2.get_dna().len());
    }

    let mut child_1 = U::new();
    let mut child_2 = U::new();

    let mut dna_child_1 = Vec::new();
    let mut dna_child_2 = Vec::new();
    debug!(target="crossover_events", method="multipoint_crossover"; "Starting the  multipoint crossover");

    //We check if the number of points are higher than the dna, we take the dna lenght
    let number_of_blocks = if (*crossover_number_of_points as usize) + 1 > parent_1.get_dna().len(){parent_1.get_dna().len()}else{(*crossover_number_of_points as usize) + 1};
    trace!(target="crossover_events", method="multipoint_crossover"; "Number of blocks {}", number_of_blocks);

    //We get the number of genes per block
    let number_of_genes_per_block = (parent_1.get_dna().len() / number_of_blocks) as i64;
    let mut gene_number = 0;
    let mut crossed = false;
    trace!(target="crossover_events", method="multipoint_crossover"; "Number of genes per block {}", number_of_genes_per_block);

    //Here we set the genes to the children
    for gn in 0..parent_1.get_dna().len(){
        
        //Sets the genes of the children
        if !crossed {
            dna_child_1.push(parent_1.get_dna().get(gn).cloned().unwrap());
            dna_child_2.push(parent_2.get_dna().get(gn).cloned().unwrap());
        }else{
            dna_child_1.push(parent_2.get_dna().get(gn).cloned().unwrap());
            dna_child_2.push(parent_1.get_dna().get(gn).cloned().unwrap());
        }

        //Sets the point change
        if gene_number >= number_of_genes_per_block - 1 {
            crossed = !crossed;
            gene_number = 0;
        }else{
            gene_number += 1;
        }
    }

    //Sets the dna into the children and return them
    child_1.set_dna(dna_child_1.as_slice());
    child_2.set_dna(dna_child_2.as_slice());
    debug!(target="crossover_events", method="multipoint_crossover"; "Multipoint crossover finished");

    Some(vec![child_1, child_2])
}