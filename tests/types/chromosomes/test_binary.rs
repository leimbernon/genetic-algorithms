use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};

#[cfg(test)]
#[test]
fn test_dna_from_string() {
    let dna = "11010010";
    let mut binary_chromosome = BinaryChromosome::new();
    binary_chromosome.dna_from_string(dna).unwrap();

    assert_eq!(binary_chromosome.dna().len(), 8);

    assert!(binary_chromosome.dna()[0].value);
    assert!(binary_chromosome.dna()[1].value);
    assert!(!binary_chromosome.dna()[2].value);
    assert!(binary_chromosome.dna()[3].value);
    assert!(!binary_chromosome.dna()[4].value);
    assert!(!binary_chromosome.dna()[5].value);
    assert!(binary_chromosome.dna()[6].value);
    assert!(!binary_chromosome.dna()[7].value);

    assert_eq!(binary_chromosome.dna()[0].id(), 0);
    assert_eq!(binary_chromosome.dna()[1].id(), 1);
    assert_eq!(binary_chromosome.dna()[2].id(), 2);
    assert_eq!(binary_chromosome.dna()[3].id(), 3);
    assert_eq!(binary_chromosome.dna()[4].id(), 4);
    assert_eq!(binary_chromosome.dna()[5].id(), 5);
    assert_eq!(binary_chromosome.dna()[6].id(), 6);
    assert_eq!(binary_chromosome.dna()[7].id(), 7);
}

#[test]
fn test_phenotype() {
    let dna = vec![
        BinaryGenotype {
            id: 1,
            value: false,
        },
        BinaryGenotype { id: 2, value: true },
        BinaryGenotype {
            id: 3,
            value: false,
        },
        BinaryGenotype { id: 4, value: true },
        BinaryGenotype {
            id: 5,
            value: false,
        },
    ];

    let mut binary_chromosome = BinaryChromosome::new();
    use std::borrow::Cow;
    binary_chromosome.set_dna(Cow::Borrowed(&dna));
    assert_eq!(binary_chromosome.phenotype(), "01010");
}
