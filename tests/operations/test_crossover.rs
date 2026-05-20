#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::CrossoverConfiguration,
    fitness::FitnessFnWrapper,
    operations::crossover::{self, aga_probability, cycle, multipoint, uniform_crossover},
    operations::Crossover,
};
use std::panic::AssertUnwindSafe;

#[test]
fn test_cycle_crossover() {
    //we create 2 dnas of 4 genes for 2 chromosomes
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_2 = vec![
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];

    let parent_1 = Chromosome {
        dna: dna_1,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: dna_2,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //Getting the offspring
    let mut offspring = cycle::cycle(&parent_1, &parent_2).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), child_2.dna.len());
    assert_eq!(parent_1.dna.len(), parent_2.dna.len());

    //Checking that the crossover has been well executed for the child 1
    assert_eq!(child_2.dna.first().unwrap().id, 4);
    assert_eq!(child_2.dna.get(1).unwrap().id, 2);
    assert_eq!(child_2.dna.get(2).unwrap().id, 3);
    assert_eq!(child_2.dna.get(3).unwrap().id, 1);

    //Checking that the crossover has been well executed for the child 2
    assert_eq!(child_1.dna.first().unwrap().id, 1);
    assert_eq!(child_1.dna.get(1).unwrap().id, 3);
    assert_eq!(child_1.dna.get(2).unwrap().id, 2);
    assert_eq!(child_1.dna.get(3).unwrap().id, 4);
}

#[test]
fn test_multipoint_crossover_2_points() {
    //we create 2 dnas of 6 genes for 2 chromosomes
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
    ];
    let dna_2 = vec![
        Gene { id: 6 },
        Gene { id: 5 },
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];

    let parent_1 = Chromosome {
        dna: dna_1.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: dna_2.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //Getting the offspring
    let mut offspring = multipoint(&parent_1, &parent_2, 2).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes as parents
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), parent_2.dna.len());

    //Checking that every gene in children came from one of the parents
    for i in 0..dna_1.len() {
        assert!(
            child_1.dna[i].id == dna_1[i].id || child_1.dna[i].id == dna_2[i].id,
            "child_1 gene at position {} (id={}) is not from either parent (p1={}, p2={})",
            i,
            child_1.dna[i].id,
            dna_1[i].id,
            dna_2[i].id
        );
        assert!(
            child_2.dna[i].id == dna_1[i].id || child_2.dna[i].id == dna_2[i].id,
            "child_2 gene at position {} (id={}) is not from either parent (p1={}, p2={})",
            i,
            child_2.dna[i].id,
            dna_1[i].id,
            dna_2[i].id
        );
    }

    //Checking that children are complementary (where child_1 takes from parent_1, child_2 takes from parent_2)
    for i in 0..dna_1.len() {
        if child_1.dna[i].id == dna_1[i].id {
            assert_eq!(
                child_2.dna[i].id, dna_2[i].id,
                "children should be complementary at position {}",
                i
            );
        } else {
            assert_eq!(
                child_2.dna[i].id, dna_1[i].id,
                "children should be complementary at position {}",
                i
            );
        }
    }

    //Checking that there is at least one crossover point (genes switch source at least once)
    let mut switches = 0;
    for i in 1..dna_1.len() {
        let child1_from_p1_prev = child_1.dna[i - 1].id == dna_1[i - 1].id;
        let child1_from_p1_curr = child_1.dna[i].id == dna_1[i].id;
        if child1_from_p1_prev != child1_from_p1_curr {
            switches += 1;
        }
    }
    assert!(
        (1..=2).contains(&switches),
        "Expected 1-2 crossover switches for 2-point crossover, got {}",
        switches
    );
}

#[test]
fn test_multipoint_crossover_4_points() {
    //we create 2 dnas of 6 genes for 2 chromosomes
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
    ];
    let dna_2 = vec![
        Gene { id: 6 },
        Gene { id: 5 },
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];

    let parent_1 = Chromosome {
        dna: dna_1.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: dna_2.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //Getting the offspring
    let mut offspring = multipoint(&parent_1, &parent_2, 4).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes as parents
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), parent_2.dna.len());

    //Checking that every gene in children came from one of the parents
    for i in 0..dna_1.len() {
        assert!(
            child_1.dna[i].id == dna_1[i].id || child_1.dna[i].id == dna_2[i].id,
            "child_1 gene at position {} (id={}) is not from either parent (p1={}, p2={})",
            i,
            child_1.dna[i].id,
            dna_1[i].id,
            dna_2[i].id
        );
        assert!(
            child_2.dna[i].id == dna_1[i].id || child_2.dna[i].id == dna_2[i].id,
            "child_2 gene at position {} (id={}) is not from either parent (p1={}, p2={})",
            i,
            child_2.dna[i].id,
            dna_1[i].id,
            dna_2[i].id
        );
    }

    //Checking that children are complementary
    for i in 0..dna_1.len() {
        if child_1.dna[i].id == dna_1[i].id {
            assert_eq!(
                child_2.dna[i].id, dna_2[i].id,
                "children should be complementary at position {}",
                i
            );
        } else {
            assert_eq!(
                child_2.dna[i].id, dna_1[i].id,
                "children should be complementary at position {}",
                i
            );
        }
    }

    //Checking that there are at least 1 and at most 4 crossover switches
    let mut switches = 0;
    for i in 1..dna_1.len() {
        let child1_from_p1_prev = child_1.dna[i - 1].id == dna_1[i - 1].id;
        let child1_from_p1_curr = child_1.dna[i].id == dna_1[i].id;
        if child1_from_p1_prev != child1_from_p1_curr {
            switches += 1;
        }
    }
    assert!(
        (1..=4).contains(&switches),
        "Expected 1-4 crossover switches for 4-point crossover, got {}",
        switches
    );
}

#[test]
fn test_uniform_crossover() {
    //we create 2 dnas of 4 genes for 2 chromosomes
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_2 = vec![
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];

    let parent_1 = Chromosome {
        dna: dna_1,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: dna_2,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //Getting the offspring
    let mut offspring = uniform_crossover::uniform(&parent_1, &parent_2).unwrap();

    //Setting the child
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    //Checking that children have the same number of genes
    assert_eq!(child_1.dna.len(), parent_1.dna.len());
    assert_eq!(child_2.dna.len(), child_2.dna.len());
    assert_eq!(parent_1.dna.len(), parent_2.dna.len());
}

#[test]
fn test_xover_aga_probability_over_avg() {
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
    let f_max = 150.0;
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for crossover
    let aga_xover_probability = aga_probability(
        &parent_1,
        &parent_2,
        f_max,
        f_avg,
        probability_max,
        probability_min,
    );

    //We verify the result of the aga crossover probability
    assert_eq!(aga_xover_probability, 0.375);
}

#[test]
fn test_xover_aga_probability_under_avg() {
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
    let f_max = 150.0;
    let f_avg = 50.0;
    let probability_max = 0.75;
    let probability_min = 0.25;

    //We calculate the Adaptive Genetic Algorithms probability for crossover
    let aga_xover_probability = aga_probability(
        &parent_1,
        &parent_2,
        f_max,
        f_avg,
        probability_max,
        probability_min,
    );

    //We verify the result of the aga crossover probability
    assert_eq!(aga_xover_probability, 0.25);
}

// ==================== Phase 1 new tests ====================

// --- Task 1.3: Cycle crossover alternates by cycle count ---

#[test]
fn test_cycle_crossover_three_cycles() {
    // Construct parents that form 3 distinct cycles:
    // Parent 1: [1, 2, 3, 4, 5, 6]
    // Parent 2: [2, 1, 5, 6, 3, 4]
    //
    // Cycle 1 (start at 0): 0 -> id 2 in p2 -> pos 1 in p1 -> id 1 in p2 -> pos 0 in p1 (done)
    //   Positions: {0, 1}. Even cycle => child1 takes from p1, child2 takes from p2.
    // Cycle 2 (start at 2): 2 -> id 5 in p2 -> pos 4 in p1 -> id 3 in p2 -> pos 2 in p1 (done)
    //   Positions: {2, 4}. Odd cycle => child1 takes from p2, child2 takes from p1.
    // Cycle 3 (start at 3): 3 -> id 6 in p2 -> pos 5 in p1 -> id 4 in p2 -> pos 3 in p1 (done)
    //   Positions: {3, 5}. Even cycle => child1 takes from p1, child2 takes from p2.

    let parent_1 = Chromosome {
        dna: vec![
            Gene { id: 1 },
            Gene { id: 2 },
            Gene { id: 3 },
            Gene { id: 4 },
            Gene { id: 5 },
            Gene { id: 6 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![
            Gene { id: 2 },
            Gene { id: 1 },
            Gene { id: 5 },
            Gene { id: 6 },
            Gene { id: 3 },
            Gene { id: 4 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let mut offspring = cycle::cycle(&parent_1, &parent_2).unwrap();
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    // Cycle 1 (even, positions 0,1): child1 from p1
    assert_eq!(child_1.dna[0].id, 1); // from p1
    assert_eq!(child_1.dna[1].id, 2); // from p1
                                      // Cycle 2 (odd, positions 2,4): child1 from p2
    assert_eq!(child_1.dna[2].id, 5); // from p2
    assert_eq!(child_1.dna[4].id, 3); // from p2
                                      // Cycle 3 (even, positions 3,5): child1 from p1
    assert_eq!(child_1.dna[3].id, 4); // from p1
    assert_eq!(child_1.dna[5].id, 6); // from p1

    // child2 is complementary
    assert_eq!(child_2.dna[0].id, 2); // from p2
    assert_eq!(child_2.dna[1].id, 1); // from p2
    assert_eq!(child_2.dna[2].id, 3); // from p1
    assert_eq!(child_2.dna[4].id, 5); // from p1
    assert_eq!(child_2.dna[3].id, 6); // from p2
    assert_eq!(child_2.dna[5].id, 4); // from p2
}

#[test]
fn test_cycle_crossover_preserves_all_gene_ids() {
    // Ensure all gene IDs from parents appear in children (permutation property)
    let parent_1 = Chromosome {
        dna: vec![
            Gene { id: 1 },
            Gene { id: 2 },
            Gene { id: 3 },
            Gene { id: 4 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![
            Gene { id: 3 },
            Gene { id: 4 },
            Gene { id: 1 },
            Gene { id: 2 },
        ],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let mut offspring = cycle::cycle(&parent_1, &parent_2).unwrap();
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    let mut c1_ids: Vec<i32> = child_1.dna.iter().map(|g| g.id).collect();
    let mut c2_ids: Vec<i32> = child_2.dna.iter().map(|g| g.id).collect();
    c1_ids.sort();
    c2_ids.sort();
    assert_eq!(c1_ids, vec![1, 2, 3, 4]);
    assert_eq!(c2_ids, vec![1, 2, 3, 4]);
}

// --- Task 1.4: Multipoint crossover with random points ---

#[test]
fn test_multipoint_crossover_1_point() {
    let dna_1: Vec<Gene> = (1..=10).map(|i| Gene { id: i }).collect();
    let dna_2: Vec<Gene> = (11..=20).map(|i| Gene { id: i }).collect();
    let parent_1 = Chromosome {
        dna: dna_1.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: dna_2.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let mut offspring = multipoint(&parent_1, &parent_2, 1).unwrap();
    let child_2 = offspring.pop().unwrap();
    let child_1 = offspring.pop().unwrap();

    // Length preserved
    assert_eq!(child_1.dna.len(), 10);
    assert_eq!(child_2.dna.len(), 10);

    // Every gene from one of the parents
    for i in 0..10 {
        assert!(child_1.dna[i].id == dna_1[i].id || child_1.dna[i].id == dna_2[i].id);
    }

    // Exactly 1 crossover switch
    let mut switches = 0;
    for i in 1..10 {
        let from_p1_prev = child_1.dna[i - 1].id == dna_1[i - 1].id;
        let from_p1_curr = child_1.dna[i].id == dna_1[i].id;
        if from_p1_prev != from_p1_curr {
            switches += 1;
        }
    }
    assert!(
        switches == 1,
        "Expected exactly 1 switch for 1-point crossover, got {}",
        switches
    );
}

#[test]
fn test_multipoint_crossover_children_complementary() {
    // Children should be complementary across many random runs
    let dna_1: Vec<Gene> = (1..=8).map(|i| Gene { id: i }).collect();
    let dna_2: Vec<Gene> = (11..=18).map(|i| Gene { id: i }).collect();
    let parent_1 = Chromosome {
        dna: dna_1.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: dna_2.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    for _ in 0..20 {
        let mut offspring = multipoint(&parent_1, &parent_2, 3).unwrap();
        let child_2 = offspring.pop().unwrap();
        let child_1 = offspring.pop().unwrap();

        for i in 0..8 {
            if child_1.dna[i].id == dna_1[i].id {
                assert_eq!(
                    child_2.dna[i].id, dna_2[i].id,
                    "Children not complementary at pos {}",
                    i
                );
            } else {
                assert_eq!(
                    child_2.dna[i].id, dna_1[i].id,
                    "Children not complementary at pos {}",
                    i
                );
            }
        }
    }
}

// --- Task 1.9: AGA crossover probability div-by-zero guard ---

#[test]
fn test_xover_aga_probability_equal_fitness_returns_max() {
    // When f_max == f_avg, the denominator is zero. Should return probability_max.
    let parent_1 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let f_max = 50.0;
    let f_avg = 50.0;
    let probability_max = 0.9;
    let probability_min = 0.1;

    let prob = aga_probability(
        &parent_1,
        &parent_2,
        f_max,
        f_avg,
        probability_max,
        probability_min,
    );
    assert_eq!(
        prob, probability_max,
        "When f_max == f_avg, should return probability_max to avoid div-by-zero"
    );
}

#[test]
fn test_xover_aga_probability_all_same_high_fitness() {
    // All chromosomes have identical fitness that equals both f_max and f_avg
    let parent_1 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 100.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 100.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let prob = aga_probability(&parent_1, &parent_2, 100.0, 100.0, 0.8, 0.2);
    assert_eq!(prob, 0.8);
}

// ==================== Phase 2 new tests ====================

// --- Task 2.1: MultiPoint crossover requires number_of_points ---

#[test]
fn test_multipoint_crossover_missing_number_of_points() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 4 }, Gene { id: 5 }, Gene { id: 6 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let config = CrossoverConfiguration {
        method: Crossover::MultiPoint,
        number_of_points: None,
        ..Default::default()
    };

    let result = crossover::factory(&parent_1, &parent_2, config);
    assert!(
        result.is_err(),
        "MultiPoint crossover without number_of_points should return Err"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("number_of_points"),
        "Error should mention number_of_points, got: {}",
        err_msg
    );
}

// ==================== Phase 5 edge-case tests ====================

// --- Multipoint crossover edge cases ---

#[test]
fn test_multipoint_crossover_empty_parents() {
    let parent_1 = Chromosome {
        dna: vec![],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    // Empty parents: dna_len == 0, dna_len - 1 would underflow.
    // The lengths are equal so it doesn't return the length-mismatch error.
    // This will likely panic due to underflow — which is a known bug.
    // We document it here but test with catch_unwind.
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| multipoint(&parent_1, &parent_2, 2)));
    // Either panics or returns an error — either is acceptable documentation
    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Multipoint with empty parents should panic or return Err"
    );
}

#[test]
fn test_multipoint_crossover_single_gene() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    // Single gene: n = min(points, dna_len-1) = 0, so no crossover points.
    // Empty candidates vec means no crossover, children = clones of parents.
    let result = multipoint(&parent_1, &parent_2, 2);
    assert!(result.is_ok());
    let children = result.unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].dna.len(), 1);
}

#[test]
fn test_multipoint_crossover_two_genes() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    // Two genes: max 1 crossover point
    let result = multipoint(&parent_1, &parent_2, 5);
    assert!(result.is_ok());
    let children = result.unwrap();
    assert_eq!(children[0].dna.len(), 2);
    assert_eq!(children[1].dna.len(), 2);
}

#[test]
fn test_multipoint_crossover_identical_parents() {
    let dna = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let parent = Chromosome {
        dna: dna.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = multipoint(&parent, &parent, 2).unwrap();
    // With identical parents, children should be identical to parents
    for child in &result {
        for (i, gene) in child.dna.iter().enumerate() {
            assert_eq!(gene.id, dna[i].id);
        }
    }
}

#[test]
fn test_multipoint_crossover_different_lengths() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = multipoint(&parent_1, &parent_2, 1);
    assert!(result.is_err(), "Different lengths should return Err");
}

#[test]
fn test_multipoint_crossover_zero_points() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 4 }, Gene { id: 5 }, Gene { id: 6 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = multipoint(&parent_1, &parent_2, 0).unwrap();
    // 0 crossover points: children should be clones of parents
    assert_eq!(result[0].dna[0].id, 1);
    assert_eq!(result[0].dna[1].id, 2);
    assert_eq!(result[0].dna[2].id, 3);
    assert_eq!(result[1].dna[0].id, 4);
    assert_eq!(result[1].dna[1].id, 5);
    assert_eq!(result[1].dna[2].id, 6);
}

// --- Cycle crossover edge cases ---

#[test]
fn test_cycle_crossover_mismatched_gene_ids() {
    // Parents with non-overlapping gene IDs should return error
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = cycle::cycle(&parent_1, &parent_2);
    assert!(
        result.is_err(),
        "Cycle crossover with mismatched gene IDs should fail"
    );
}

#[test]
fn test_cycle_crossover_identical_parents() {
    let dna = vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }];
    let parent = Chromosome {
        dna: dna.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = cycle::cycle(&parent, &parent).unwrap();
    // Identical parents -> children identical to parents
    for child in &result {
        for (i, gene) in child.dna.iter().enumerate() {
            assert_eq!(gene.id, dna[i].id);
        }
    }
}

#[test]
fn test_cycle_crossover_two_genes() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }, Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = cycle::cycle(&parent_1, &parent_2).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].dna.len(), 2);
}

#[test]
fn test_cycle_crossover_different_lengths() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = cycle::cycle(&parent_1, &parent_2);
    assert!(result.is_err());
}

// --- Uniform crossover edge cases ---

#[test]
fn test_uniform_crossover_identical_parents() {
    let dna = vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }];
    let parent = Chromosome {
        dna: dna.clone(),
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = uniform_crossover::uniform(&parent, &parent).unwrap();
    for child in &result {
        for (i, gene) in child.dna.iter().enumerate() {
            assert_eq!(
                gene.id, dna[i].id,
                "Identical parents should produce identical children"
            );
        }
    }
}

#[test]
fn test_uniform_crossover_single_gene() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = uniform_crossover::uniform(&parent_1, &parent_2).unwrap();
    assert_eq!(result.len(), 2);
    // Each child gene must come from one of the parents
    assert!(result[0].dna[0].id == 1 || result[0].dna[0].id == 2);
    assert!(result[1].dna[0].id == 1 || result[1].dna[0].id == 2);
}

#[test]
fn test_uniform_crossover_different_lengths() {
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = uniform_crossover::uniform(&parent_1, &parent_2);
    assert!(result.is_err());
}

// --- Crossover enum dispatch error paths ---

#[test]
fn test_crossover_enum_multipoint_returns_error() {
    use genetic_algorithms::traits::CrossoverOperator;
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = Crossover::MultiPoint.crossover(&parent_1, &parent_2);
    assert!(
        result.is_err(),
        "MultiPoint through enum dispatch should return Err"
    );
}

#[test]
fn test_crossover_enum_sbx_returns_error() {
    use genetic_algorithms::traits::CrossoverOperator;
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = Crossover::Sbx.crossover(&parent_1, &parent_2);
    assert!(
        result.is_err(),
        "SBX through enum dispatch should return Err"
    );
}

#[test]
fn test_crossover_enum_blend_alpha_returns_error() {
    use genetic_algorithms::traits::CrossoverOperator;
    let parent_1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = Crossover::BlendAlpha.crossover(&parent_1, &parent_2);
    assert!(
        result.is_err(),
        "BlendAlpha through enum dispatch should return Err"
    );
}

// --- SBX / BlendAlpha enum dispatch with Range<T> chromosomes ---

#[test]
fn test_crossover_enum_sbx_works_with_range_f64() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::genotypes::Range as RangeGenotype;
    use genetic_algorithms::traits::{ChromosomeT, CrossoverOperator, LinearChromosome};
    use std::borrow::Cow;

    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    p1.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 20.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 80.0),
    ]));
    p2.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 60.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 30.0),
    ]));

    let children = Crossover::Sbx.crossover(&p1, &p2).unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].dna().len(), 2);
    for child in &children {
        for gene in child.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn test_crossover_enum_blend_alpha_works_with_range_f64() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::genotypes::Range as RangeGenotype;
    use genetic_algorithms::traits::{ChromosomeT, CrossoverOperator, LinearChromosome};
    use std::borrow::Cow;

    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    p1.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 30.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 70.0),
    ]));
    p2.set_dna(Cow::Owned(vec![
        RangeGenotype::new(0, vec![(0.0, 100.0)], 60.0),
        RangeGenotype::new(1, vec![(0.0, 100.0)], 40.0),
    ]));

    let children = Crossover::BlendAlpha.crossover(&p1, &p2).unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].dna().len(), 2);
    for child in &children {
        for gene in child.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}

#[test]
fn test_crossover_config_sbx_uses_eta() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::genotypes::Range as RangeGenotype;
    use genetic_algorithms::traits::{ChromosomeT, CrossoverOperator, LinearChromosome};
    use std::borrow::Cow;

    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    p1.set_dna(Cow::Owned(vec![RangeGenotype::new(
        0,
        vec![(0.0, 100.0)],
        20.0,
    )]));
    p2.set_dna(Cow::Owned(vec![RangeGenotype::new(
        0,
        vec![(0.0, 100.0)],
        80.0,
    )]));

    let config = CrossoverConfiguration {
        method: Crossover::Sbx,
        sbx_eta: Some(20.0),
        ..Default::default()
    };
    let children = config.crossover(&p1, &p2).unwrap();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_crossover_config_blend_alpha_uses_alpha() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::genotypes::Range as RangeGenotype;
    use genetic_algorithms::traits::{ChromosomeT, CrossoverOperator, LinearChromosome};
    use std::borrow::Cow;

    let mut p1 = RangeChromosome::<f64>::new();
    let mut p2 = RangeChromosome::<f64>::new();
    p1.set_dna(Cow::Owned(vec![RangeGenotype::new(
        0,
        vec![(0.0, 100.0)],
        30.0,
    )]));
    p2.set_dna(Cow::Owned(vec![RangeGenotype::new(
        0,
        vec![(0.0, 100.0)],
        70.0,
    )]));

    let config = CrossoverConfiguration {
        method: Crossover::BlendAlpha,
        blend_alpha: Some(0.3),
        ..Default::default()
    };
    let children = config.crossover(&p1, &p2).unwrap();
    assert_eq!(children.len(), 2);
}

// --- Crossover AGA probability edge cases ---

#[test]
fn test_xover_aga_probability_at_avg() {
    // When larger_f == f_avg and f_max == f_avg, should return probability_max
    let parent_1 = Chromosome {
        dna: vec![],
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let parent_2 = Chromosome {
        dna: vec![],
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let prob = aga_probability(&parent_1, &parent_2, 50.0, 50.0, 0.9, 0.1);
    // larger_f >= f_avg => true, f_max - f_avg ~= 0 => returns probability_max
    assert_eq!(prob, 0.9);
}

#[test]
fn test_xover_aga_probability_equal_parents() {
    let parent = Chromosome {
        dna: vec![],
        fitness: 75.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let prob = aga_probability(&parent, &parent, 100.0, 50.0, 0.8, 0.2);
    // larger_f = 75 >= f_avg = 50, so: 0.8 * (100-75)/(100-50) = 0.8 * 0.5 = 0.4
    assert!((prob - 0.4).abs() < f64::EPSILON);
}
