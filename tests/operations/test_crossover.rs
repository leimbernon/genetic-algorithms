#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::CrossoverConfiguration,
    fitness::FitnessFnWrapper,
    operations::crossover::{self, aga_probability, cycle, multipoint, uniform_crossover},
    operations::Crossover,
};

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
    let mut offspring = multipoint(&parent_1, &parent_2, &2).unwrap();

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
    let mut offspring = multipoint(&parent_1, &parent_2, &4).unwrap();

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

    let mut offspring = multipoint(&parent_1, &parent_2, &1).unwrap();
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
        let mut offspring = multipoint(&parent_1, &parent_2, &3).unwrap();
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
