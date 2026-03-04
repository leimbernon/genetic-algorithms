#[cfg(test)]
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::operations::mutation::bit_flip::bit_flip;
use genetic_algorithms::traits::ChromosomeT;
use std::borrow::Cow;

#[test]
fn bit_flip_changes_exactly_one_gene() {
    let dna = vec![
        BinaryGenotype { id: 0, value: true },
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
    ];

    // Run multiple times to check that exactly 1 gene changes each time
    for _ in 0..50 {
        let mut chromosome = BinaryChromosome::new();
        chromosome.set_dna(Cow::Owned(dna.clone()));

        bit_flip(&mut chromosome);

        let result = chromosome.dna();
        let mut diff_count = 0;
        for i in 0..dna.len() {
            if result[i].value != dna[i].value {
                diff_count += 1;
            }
        }
        assert_eq!(diff_count, 1, "Bit flip should change exactly 1 gene");
    }
}

#[test]
fn bit_flip_flips_the_value() {
    // All true -> one should become false
    let dna = vec![
        BinaryGenotype { id: 0, value: true },
        BinaryGenotype { id: 1, value: true },
        BinaryGenotype { id: 2, value: true },
    ];

    let mut chromosome = BinaryChromosome::new();
    chromosome.set_dna(Cow::Owned(dna));

    bit_flip(&mut chromosome);

    let result = chromosome.dna();
    let false_count = result.iter().filter(|g| !g.value).count();
    assert_eq!(false_count, 1, "One gene should have been flipped to false");
}

#[test]
fn bit_flip_preserves_length_and_ids() {
    let dna = vec![
        BinaryGenotype { id: 0, value: true },
        BinaryGenotype {
            id: 1,
            value: false,
        },
        BinaryGenotype { id: 2, value: true },
        BinaryGenotype {
            id: 3,
            value: false,
        },
    ];

    let mut chromosome = BinaryChromosome::new();
    chromosome.set_dna(Cow::Owned(dna.clone()));

    bit_flip(&mut chromosome);

    let result = chromosome.dna();
    assert_eq!(result.len(), 4);
    for i in 0..4 {
        assert_eq!(result[i].id, dna[i].id, "IDs should be preserved");
    }
}

// ==================== Phase 5 edge-case tests ====================

#[test]
fn bit_flip_empty_chromosome_no_panic() {
    let mut chromosome = BinaryChromosome::new();
    chromosome.set_dna(Cow::Owned(vec![]));
    bit_flip(&mut chromosome);
    assert_eq!(chromosome.dna().len(), 0);
}

#[test]
fn bit_flip_single_gene_flips_it() {
    let dna = vec![BinaryGenotype { id: 0, value: true }];
    let mut chromosome = BinaryChromosome::new();
    chromosome.set_dna(Cow::Owned(dna));

    bit_flip(&mut chromosome);

    // Only gene must be flipped
    assert!(!chromosome.dna()[0].value, "Single gene should be flipped");
}

#[test]
fn bit_flip_all_false_flips_one_to_true() {
    let dna = vec![
        BinaryGenotype {
            id: 0,
            value: false,
        },
        BinaryGenotype {
            id: 1,
            value: false,
        },
        BinaryGenotype {
            id: 2,
            value: false,
        },
    ];
    let mut chromosome = BinaryChromosome::new();
    chromosome.set_dna(Cow::Owned(dna));

    bit_flip(&mut chromosome);

    let true_count = chromosome.dna().iter().filter(|g| g.value).count();
    assert_eq!(true_count, 1, "One gene should have been flipped to true");
}
