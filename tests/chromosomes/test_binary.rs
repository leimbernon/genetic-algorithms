use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::traits::{ChromosomeT, GeneT};

#[cfg(test)]

#[test]
fn test_dna_from_string(){
    let dna = "11010010";
    let mut binary_chromosome =  BinaryChromosome::new();
    binary_chromosome.dna_from_string(dna).unwrap();

    assert_eq!(binary_chromosome.get_dna().len(), 8);

    assert_eq!(binary_chromosome.get_dna()[0].value, true);
    assert_eq!(binary_chromosome.get_dna()[1].value, true);
    assert_eq!(binary_chromosome.get_dna()[2].value, false);
    assert_eq!(binary_chromosome.get_dna()[3].value, true);
    assert_eq!(binary_chromosome.get_dna()[4].value, false);
    assert_eq!(binary_chromosome.get_dna()[5].value, false);
    assert_eq!(binary_chromosome.get_dna()[6].value, true);
    assert_eq!(binary_chromosome.get_dna()[7].value, false);

    assert_eq!(binary_chromosome.get_dna()[0].get_id(), 0);
    assert_eq!(binary_chromosome.get_dna()[1].get_id(), 1);
    assert_eq!(binary_chromosome.get_dna()[2].get_id(), 2);
    assert_eq!(binary_chromosome.get_dna()[3].get_id(), 3);
    assert_eq!(binary_chromosome.get_dna()[4].get_id(), 4);
    assert_eq!(binary_chromosome.get_dna()[5].get_id(), 5);
    assert_eq!(binary_chromosome.get_dna()[6].get_id(), 6);
    assert_eq!(binary_chromosome.get_dna()[7].get_id(), 7);
}

#[test]
fn test_phenotype(){
    let dna = vec![BinaryGenotype {id:1, value:false}, BinaryGenotype {id:2, value:true},
                   BinaryGenotype{id:3, value:false}, BinaryGenotype{id:4, value:true},
                   BinaryGenotype{id:5, value:false}];

    let mut binary_chromosome =  BinaryChromosome::new();
    use std::borrow::Cow;
    binary_chromosome.set_dna(Cow::Borrowed(&dna));
    assert_eq!(binary_chromosome.phenotype(), "01010");
}