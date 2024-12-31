use rand::Rng;
use crate::traits::{ChromosomeT, GeneT};

/**
 * Function to initialize the dna of an individual
 */
pub fn generic_random_initialization<U>(genes_per_chromosome: i32, alleles: Option<&[U::Gene]>, needs_unique_ids: Option<bool>) ->Vec<U::Gene>
where
    U: ChromosomeT + Send + Sync + 'static + Clone{

    if alleles.is_none() || needs_unique_ids.is_none(){
        panic!("Alleles and needs_unique_ids must be provided");
    }

    let alleles = alleles.unwrap();
    let needs_unique_ids = needs_unique_ids.unwrap();

    let mut rng = rand::thread_rng();
    let mut dna = Vec::new();

    //Selects the genes randomly from the vector without repeating them
    for j in 0..genes_per_chromosome {
        let index = rng.gen_range(0..alleles.len());
        let mut gene = alleles.get(index).cloned().unwrap();

        //If we need unique ids
        if needs_unique_ids {
            gene.set_id(j);
        }

        dna.push(gene);
    }

    dna
}


/**
 * Function to initialize the dna of an individual without repeating an array of alleles
 */
pub fn generic_random_initialization_without_repetitions<U>(genes_per_chromosome: i32, alleles: Option<&[U::Gene]>, needs_unique_ids: Option<bool>) ->Vec<U::Gene>
where
    U: ChromosomeT + Send + Sync + 'static + Clone{

    if alleles.is_none() || needs_unique_ids.is_none(){
        panic!("Alleles and needs_unique_ids must be provided");
    }

    let alleles = alleles.unwrap();
    let needs_unique_ids = needs_unique_ids.unwrap();

    let mut rng = rand::thread_rng();
    let mut dna = Vec::new();

    let mut tmp_alleles = alleles.to_vec().clone();

    //Selects the genes randomly from the vector without repeating them
    for j in 0..genes_per_chromosome {
        let index = rng.gen_range(0..tmp_alleles.len());
        let mut gene = tmp_alleles.get(index).cloned().unwrap();

        //If we need unique ids
        if needs_unique_ids {
            gene.set_id(j);
        }

        tmp_alleles.remove(index);

        dna.push(gene);
    }

    dna
}