#[cfg(test)]
mod structures;

use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::initializers::{binary_random_initialization, generic_random_initialization, generic_random_initialization_without_repetitions, range_random_initialization};
use genetic_algorithms::traits::ChromosomeT;
use genetic_algorithms::genotypes::Range as RangeGenotype;
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

#[test]
fn test_range_random_initialization(){
    let alleles = vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0)];
    let genes = range_random_initialization(10, Some(&alleles), Some(false));
    assert_eq!(genes.len(), 10);
    for gene in genes {
        assert!(gene.value >= 0.0 && gene.value <= 1.0);
    }
}