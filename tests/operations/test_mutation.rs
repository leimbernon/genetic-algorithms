#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    fitness::FitnessFnWrapper,
    operations::mutation::{self, aga_probability, inversion, scramble, swap},
    operations::Mutation,
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 100.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
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
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 49.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
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
        fitness_values: vec![],
    };
    let mut chromosome = original.clone();
    scramble::scramble(&mut chromosome);
    assert_eq!(chromosome.dna.len(), 2);
    let mut ids: Vec<i32> = chromosome.dna.iter().map(|g| g.id).collect();
    ids.sort();
    assert_eq!(ids, vec![1, 2]);
}

// ==================== Phase 5 edge-case tests ====================

// --- Mutation factory paths for all variants ---

#[test]
fn test_mutation_factory_swap() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory(Mutation::Swap, &mut chromosome);
    assert!(result.is_ok());
    assert_eq!(chromosome.dna.len(), 3);
}

#[test]
fn test_mutation_factory_inversion() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory(Mutation::Inversion, &mut chromosome);
    assert!(result.is_ok());
    assert_eq!(chromosome.dna.len(), 3);
}

#[test]
fn test_mutation_factory_scramble() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory(Mutation::Scramble, &mut chromosome);
    assert!(result.is_ok());
    assert_eq!(chromosome.dna.len(), 3);
}

// --- factory_non_value error paths ---

#[test]
fn test_factory_non_value_swap() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory_non_value(Mutation::Swap, &mut chromosome);
    assert!(result.is_ok());
}

#[test]
fn test_factory_non_value_inversion() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory_non_value(Mutation::Inversion, &mut chromosome);
    assert!(result.is_ok());
}

#[test]
fn test_factory_non_value_scramble() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory_non_value(Mutation::Scramble, &mut chromosome);
    assert!(result.is_ok());
}

#[test]
fn test_factory_non_value_value_returns_error() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory_non_value(Mutation::Value, &mut chromosome);
    assert!(
        result.is_err(),
        "factory_non_value should reject Mutation::Value"
    );
}

#[test]
fn test_factory_non_value_bitflip_returns_error() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory_non_value(Mutation::BitFlip, &mut chromosome);
    assert!(
        result.is_err(),
        "factory_non_value should reject Mutation::BitFlip"
    );
}

#[test]
fn test_factory_non_value_creep_returns_error() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory_non_value(Mutation::Creep { step: None }, &mut chromosome);
    assert!(
        result.is_err(),
        "factory_non_value should reject Mutation::Creep"
    );
}

#[test]
fn test_factory_non_value_gaussian_returns_error() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let result = mutation::factory_non_value(Mutation::Gaussian { sigma: None }, &mut chromosome);
    assert!(
        result.is_err(),
        "factory_non_value should reject Mutation::Gaussian"
    );
}

// --- Mutation AGA probability edge cases ---

#[test]
fn test_mutation_aga_probability_at_avg() {
    // When larger_f == f_avg exactly
    let parent_1 = Chromosome {
        dna: vec![],
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let parent_2 = Chromosome {
        dna: vec![],
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let prob = aga_probability(&parent_1, &parent_2, 50.0, 0.9, 0.1);
    // larger_f (50) >= f_avg (50) => probability_min
    assert_eq!(prob, 0.1);
}

#[test]
fn test_mutation_aga_probability_equal_parents_below_avg() {
    let parent = Chromosome {
        dna: vec![],
        fitness: 25.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
        fitness_values: vec![],
    };
    let prob = aga_probability(&parent, &parent, 50.0, 0.9, 0.1);
    // larger_f = 25 < f_avg = 50 => probability_max
    assert_eq!(prob, 0.9);
}
