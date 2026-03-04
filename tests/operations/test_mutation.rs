#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper,
    operations::mutation::{aga_probability, inversion, scramble, swap},
};

#[test]
fn test_swap_mutation() {
    //We create 1 dna for 1 chromosome
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
        Gene { id: 9 },
        Gene { id: 10 },
        Gene { id: 11 },
        Gene { id: 12 },
        Gene { id: 13 },
        Gene { id: 14 },
        Gene { id: 15 },
        Gene { id: 16 },
        Gene { id: 17 },
        Gene { id: 18 },
        Gene { id: 19 },
        Gene { id: 20 },
        Gene { id: 21 },
        Gene { id: 22 },
        Gene { id: 23 },
        Gene { id: 24 },
        Gene { id: 25 },
        Gene { id: 26 },
        Gene { id: 27 },
        Gene { id: 28 },
        Gene { id: 29 },
        Gene { id: 30 },
        Gene { id: 31 },
        Gene { id: 32 },
        Gene { id: 33 },
        Gene { id: 34 },
        Gene { id: 35 },
        Gene { id: 36 },
        Gene { id: 37 },
        Gene { id: 38 },
        Gene { id: 39 },
        Gene { id: 40 },
        Gene { id: 41 },
        Gene { id: 42 },
        Gene { id: 43 },
        Gene { id: 44 },
        Gene { id: 45 },
        Gene { id: 46 },
        Gene { id: 47 },
        Gene { id: 48 },
        Gene { id: 49 },
        Gene { id: 50 },
    ];

    let chromosome_1_copy = Chromosome {
        dna: dna_1,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    // Swap is stochastic — may pick the same index twice, leaving DNA unchanged.
    let mut mutated = false;
    for _ in 0..10 {
        let mut chromosome_1 = chromosome_1_copy.clone();
        swap::swap(&mut chromosome_1);
        assert_eq!(chromosome_1.dna.len(), chromosome_1_copy.dna.len());
        if chromosome_1 != chromosome_1_copy {
            mutated = true;
            break;
        }
    }
    assert!(
        mutated,
        "swap mutation did not change the chromosome after 10 attempts"
    );
}

#[test]
fn test_inversion_mutation() {
    //We create 1 dna for 1 chromosome
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
        Gene { id: 9 },
        Gene { id: 10 },
        Gene { id: 11 },
        Gene { id: 12 },
        Gene { id: 13 },
        Gene { id: 14 },
        Gene { id: 15 },
        Gene { id: 16 },
        Gene { id: 17 },
        Gene { id: 18 },
        Gene { id: 19 },
        Gene { id: 20 },
        Gene { id: 21 },
        Gene { id: 22 },
        Gene { id: 23 },
        Gene { id: 24 },
        Gene { id: 25 },
        Gene { id: 26 },
        Gene { id: 27 },
        Gene { id: 28 },
        Gene { id: 29 },
        Gene { id: 30 },
        Gene { id: 31 },
        Gene { id: 32 },
        Gene { id: 33 },
        Gene { id: 34 },
        Gene { id: 35 },
        Gene { id: 36 },
        Gene { id: 37 },
        Gene { id: 38 },
        Gene { id: 39 },
        Gene { id: 40 },
        Gene { id: 41 },
        Gene { id: 42 },
        Gene { id: 43 },
        Gene { id: 44 },
        Gene { id: 45 },
        Gene { id: 46 },
        Gene { id: 47 },
        Gene { id: 48 },
        Gene { id: 49 },
        Gene { id: 50 },
    ];

    let chromosome_1_copy = Chromosome {
        dna: dna_1,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    // Inversion is stochastic — may pick the same index twice (or adjacent), leaving DNA unchanged.
    let mut mutated = false;
    for _ in 0..10 {
        let mut chromosome_1 = chromosome_1_copy.clone();
        inversion::inversion(&mut chromosome_1);
        assert_eq!(chromosome_1.dna.len(), chromosome_1_copy.dna.len());
        if chromosome_1 != chromosome_1_copy {
            mutated = true;
            break;
        }
    }
    assert!(
        mutated,
        "inversion mutation did not change the chromosome after 10 attempts"
    );
}

#[test]
fn test_scramble_mutation() {
    //We create 1 dna for 1 chromosome
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
        Gene { id: 9 },
        Gene { id: 10 },
        Gene { id: 11 },
        Gene { id: 12 },
        Gene { id: 13 },
        Gene { id: 14 },
        Gene { id: 15 },
        Gene { id: 16 },
        Gene { id: 17 },
        Gene { id: 18 },
        Gene { id: 19 },
        Gene { id: 20 },
        Gene { id: 21 },
        Gene { id: 22 },
        Gene { id: 23 },
        Gene { id: 24 },
        Gene { id: 25 },
        Gene { id: 26 },
        Gene { id: 27 },
        Gene { id: 28 },
        Gene { id: 29 },
        Gene { id: 30 },
        Gene { id: 31 },
        Gene { id: 32 },
        Gene { id: 33 },
        Gene { id: 34 },
        Gene { id: 35 },
        Gene { id: 36 },
        Gene { id: 37 },
        Gene { id: 38 },
        Gene { id: 39 },
        Gene { id: 40 },
        Gene { id: 41 },
        Gene { id: 42 },
        Gene { id: 43 },
        Gene { id: 44 },
        Gene { id: 45 },
        Gene { id: 46 },
        Gene { id: 47 },
        Gene { id: 48 },
        Gene { id: 49 },
        Gene { id: 50 },
    ];

    //We create the chromosomes
    let chromosome_1_copy = Chromosome {
        dna: dna_1.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    // Scramble is stochastic — it may rarely leave DNA unchanged when random indices
    // happen to swap genes back to their original positions. We retry a few times to
    // avoid flaky failures while still verifying the mutation works.
    let mut mutated = false;
    for _ in 0..10 {
        let mut chromosome_1 = chromosome_1_copy.clone();
        scramble::scramble(&mut chromosome_1);
        // Invariant: DNA length must be preserved
        assert_eq!(chromosome_1.dna.len(), chromosome_1_copy.dna.len());
        if chromosome_1 != chromosome_1_copy {
            mutated = true;
            break;
        }
    }
    assert!(
        mutated,
        "scramble mutation did not change the chromosome after 10 attempts"
    );
}

#[test]
fn test_mutation_aga_probability_over_avg() {
    let parent_1 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 25.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 100.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for mutation
    let aga_mutation_probability = aga_probability(
        &parent_1,
        &parent_2,
        f_avg,
        probability_max,
        probability_min,
    );

    //We verify the result of the aga mutation probability
    assert_eq!(aga_mutation_probability, probability_min);
}

#[test]
fn test_mutation_aga_probability_under_avg() {
    let parent_1 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 25.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 49.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for mutation
    let aga_mutation_probability = aga_probability(
        &parent_1,
        &parent_2,
        f_avg,
        probability_max,
        probability_min,
    );

    //We verify the result of the aga mutation probability
    assert_eq!(aga_mutation_probability, probability_max);
}

// ==================== Phase 1 new tests ====================

// --- Task 1.10: Mutation operators don't panic on empty or single-gene DNA ---

#[test]
fn test_swap_empty_dna_no_panic() {
    let mut chromosome = Chromosome {
        dna: vec![],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    // Should return early without panicking
    swap::swap(&mut chromosome);
    assert!(chromosome.dna.is_empty());
}

#[test]
fn test_swap_single_gene_no_panic() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 42 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    swap::swap(&mut chromosome);
    assert_eq!(chromosome.dna.len(), 1);
    assert_eq!(chromosome.dna[0].id, 42);
}

#[test]
fn test_inversion_empty_dna_no_panic() {
    let mut chromosome = Chromosome {
        dna: vec![],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    inversion::inversion(&mut chromosome);
    assert!(chromosome.dna.is_empty());
}

#[test]
fn test_inversion_single_gene_no_panic() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 99 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    inversion::inversion(&mut chromosome);
    assert_eq!(chromosome.dna.len(), 1);
    assert_eq!(chromosome.dna[0].id, 99);
}

#[test]
fn test_scramble_empty_dna_no_panic() {
    let mut chromosome = Chromosome {
        dna: vec![],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    scramble::scramble(&mut chromosome);
    assert!(chromosome.dna.is_empty());
}

#[test]
fn test_scramble_single_gene_no_panic() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 7 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    scramble::scramble(&mut chromosome);
    assert_eq!(chromosome.dna.len(), 1);
    assert_eq!(chromosome.dna[0].id, 7);
}

#[test]
fn test_swap_two_genes() {
    // Minimum viable swap: 2 genes. Should either swap or stay same (both valid).
    let original = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let mut chromosome = original.clone();
    swap::swap(&mut chromosome);
    assert_eq!(chromosome.dna.len(), 2);
    // Genes should still be the same set
    let mut ids: Vec<i32> = chromosome.dna.iter().map(|g| g.id).collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);
}

#[test]
fn test_inversion_two_genes() {
    let original = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let mut chromosome = original.clone();
    inversion::inversion(&mut chromosome);
    assert_eq!(chromosome.dna.len(), 2);
    let mut ids: Vec<i32> = chromosome.dna.iter().map(|g| g.id).collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);
}

#[test]
fn test_scramble_two_genes() {
    let original = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let mut chromosome = original.clone();
    scramble::scramble(&mut chromosome);
    assert_eq!(chromosome.dna.len(), 2);
    let mut ids: Vec<i32> = chromosome.dna.iter().map(|g| g.id).collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);
}
