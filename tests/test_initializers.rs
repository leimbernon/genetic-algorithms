#[cfg(test)]
mod structures;

use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::initializers::{binary_random_initialization, generic_random_initialization, generic_random_initialization_without_repetitions};
use genetic_algorithms::traits::ChromosomeT;
use crate::structures::{Gene, Chromosome};

#[test]
fn test_initializers_generic_random_initialization(){

    let binding =  vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4},
                                   Gene{id:5}, Gene{id:6}, Gene{id:7}, Gene{id:8}];
    let alleles = binding.as_slice();

    let genes = generic_random_initialization::<Chromosome>(4, Some(alleles), Some(false));
    assert_eq!(genes.len(), 4);
}

#[test]
fn test_initializers_generic_random_initialization_without_repetitions(){

    let binding =  vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4},
                                   Gene{id:5}, Gene{id:6}, Gene{id:7}, Gene{id:8}];
    let alleles = binding.as_slice();
    let genes = generic_random_initialization_without_repetitions::<Chromosome>(6, Some(alleles), Some(false));

    //Checks that any allele is repeated
    let mut alleles_ids = Vec::new();
    
    for gene in genes {
        if !alleles_ids.is_empty(){
            assert!(!alleles_ids.contains(&gene.id));
        }
        alleles_ids.push(gene.id);
    }

}

#[test]
fn test_binary_random_initialization(){
    let genes = binary_random_initialization(100, None, None);
    let mut chromosome = Binary::new();

    chromosome.set_dna(genes.as_slice());
    assert_eq!(100, chromosome.phenotype().len());
}