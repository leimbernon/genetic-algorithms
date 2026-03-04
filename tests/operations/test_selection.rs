#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::SelectionConfiguration,
    fitness::FitnessFnWrapper,
    operations::selection::{self, fitness_proportionate, random, tournament},
    operations::Selection,
};

#[test]
fn test_random_even_selection() {
    //We create 6 dna's for 6 chromosomes
    let dna_1 = vec![Gene { id: 1 }, Gene { id: 2 }];
    let dna_2 = vec![Gene { id: 3 }, Gene { id: 4 }];
    let dna_3 = vec![Gene { id: 5 }, Gene { id: 6 }];
    let dna_4 = vec![Gene { id: 7 }, Gene { id: 8 }];
    let dna_5 = vec![Gene { id: 9 }, Gene { id: 10 }];
    let dna_6 = vec![Gene { id: 11 }, Gene { id: 12 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_6 = Chromosome {
        dna: dna_6,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
        chromosome_6,
    ];
    let mating_population = random::random(&population);
    assert_eq!(mating_population.len(), 3);
}

#[test]
fn test_random_odd_selection() {
    //We create 6 dna's for 6 chromosomes
    let dna_1 = vec![Gene { id: 1 }, Gene { id: 2 }];
    let dna_2 = vec![Gene { id: 3 }, Gene { id: 4 }];
    let dna_3 = vec![Gene { id: 5 }, Gene { id: 6 }];
    let dna_4 = vec![Gene { id: 7 }, Gene { id: 8 }];
    let dna_5 = vec![Gene { id: 9 }, Gene { id: 10 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
    ];
    let mating_population = random::random(&population);
    assert_eq!(mating_population.len(), 2);
}

#[test]
fn test_roulette_wheel_selection() {
    //We create 6 dna's for 5 chromosomes
    let dna_1 = vec![Gene { id: 1 }, Gene { id: 2 }];
    let dna_2 = vec![Gene { id: 3 }, Gene { id: 4 }];
    let dna_3 = vec![Gene { id: 5 }, Gene { id: 6 }];
    let dna_4 = vec![Gene { id: 7 }, Gene { id: 8 }];
    let dna_5 = vec![Gene { id: 9 }, Gene { id: 10 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 20.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 30.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 40.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
    ];
    let mating_population = fitness_proportionate::roulette_wheel_selection(&population);
    assert_ne!(mating_population.len(), 0);
}

#[test]
fn test_stochastic_universal_sampling() {
    //We create 7 dna's for 7 chromosomes
    let dna_1 = vec![Gene { id: 1 }, Gene { id: 2 }];
    let dna_2 = vec![Gene { id: 3 }, Gene { id: 4 }];
    let dna_3 = vec![Gene { id: 5 }, Gene { id: 6 }];
    let dna_4 = vec![Gene { id: 7 }, Gene { id: 8 }];
    let dna_5 = vec![Gene { id: 9 }, Gene { id: 10 }];
    let dna_6 = vec![Gene { id: 11 }, Gene { id: 12 }];
    let dna_7 = vec![Gene { id: 13 }, Gene { id: 14 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 20.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 30.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 40.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_6 = Chromosome {
        dna: dna_6,
        fitness: 60.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_7 = Chromosome {
        dna: dna_7,
        fitness: 70.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
        chromosome_6,
        chromosome_7,
    ];

    // SUS is stochastic — may rarely produce 0 pairs. Retry a few times.
    let mut found = false;
    for _ in 0..10 {
        let mating_population =
            fitness_proportionate::stochastic_universal_sampling(&population, 3);
        if !mating_population.is_empty() {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "stochastic_universal_sampling produced no pairs after 10 attempts"
    );
}

#[test]
fn test_tournament_singlethread() {
    //We create 5 dna's for 5 chromosomes
    let dna_1 = vec![Gene { id: 1 }, Gene { id: 2 }];
    let dna_2 = vec![Gene { id: 3 }, Gene { id: 4 }];
    let dna_3 = vec![Gene { id: 5 }, Gene { id: 6 }];
    let dna_4 = vec![Gene { id: 7 }, Gene { id: 8 }];
    let dna_5 = vec![Gene { id: 9 }, Gene { id: 10 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 20.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 30.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 40.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
    ];
    let mating_population = tournament::tournament(&population, 2, 1);
    assert_eq!(mating_population.len(), 2);
    assert_ne!(mating_population.len(), 0);
}

#[test]
fn test_tournament_multithread() {
    //We create 5 dna's for 5 chromosomes
    let dna_1 = vec![Gene { id: 1 }, Gene { id: 2 }];
    let dna_2 = vec![Gene { id: 3 }, Gene { id: 4 }];
    let dna_3 = vec![Gene { id: 5 }, Gene { id: 6 }];
    let dna_4 = vec![Gene { id: 7 }, Gene { id: 8 }];
    let dna_5 = vec![Gene { id: 9 }, Gene { id: 10 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 20.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 30.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 40.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 50.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
    ];
    let mating_population = tournament::tournament(&population, 2, 2);
    assert_eq!(mating_population.len(), 2);
    assert_ne!(mating_population.len(), 0);
}

// ==================== Phase 1 new tests ====================

// --- Task 1.1: Roulette wheel selection tests ---

#[test]
fn test_roulette_wheel_favours_higher_fitness() {
    // One chromosome has overwhelmingly higher fitness than the others.
    // Over many runs, it should be selected far more often than the others.
    let mut chromosomes = Vec::new();
    for i in 0..5 {
        chromosomes.push(Chromosome {
            dna: vec![Gene { id: i }],
            fitness: if i == 4 { 1000.0 } else { 1.0 },
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        });
    }

    let mut high_fitness_count = 0;
    let runs = 100;
    for _ in 0..runs {
        let pairs = fitness_proportionate::roulette_wheel_selection(&chromosomes);
        for (a, b) in &pairs {
            if *a == 4 {
                high_fitness_count += 1;
            }
            if *b == 4 {
                high_fitness_count += 1;
            }
        }
    }
    // With fitness 1000 vs 4*1, chromosome 4 has ~99.6% selection probability.
    // Over 100 runs * 5 selections each = 500 total selections, expect > 400.
    assert!(
        high_fitness_count > 300,
        "High-fitness chromosome should be selected frequently, but was selected {} times out of {}",
        high_fitness_count,
        runs * 5
    );
}

#[test]
fn test_roulette_wheel_returns_correct_pair_count() {
    // 6 chromosomes => 6 selections => 3 pairs
    let chromosomes: Vec<Chromosome> = (0..6)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 10.0 + i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::roulette_wheel_selection(&chromosomes);
    assert_eq!(pairs.len(), 3);

    // 5 chromosomes => 5 selections => 2 pairs (odd, last dropped)
    let chromosomes: Vec<Chromosome> = (0..5)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 10.0 + i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::roulette_wheel_selection(&chromosomes);
    assert_eq!(pairs.len(), 2);
}

#[test]
fn test_roulette_wheel_zero_total_fitness() {
    // All fitness = 0.0 => total = 0 => guard returns empty
    let chromosomes: Vec<Chromosome> = (0..4)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::roulette_wheel_selection(&chromosomes);
    assert!(
        pairs.is_empty(),
        "Roulette wheel should return empty for zero total fitness"
    );
}

#[test]
fn test_roulette_wheel_negative_fitness() {
    // Negative fitness => total < 0 => guard returns empty
    let chromosomes: Vec<Chromosome> = (0..3)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: -10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::roulette_wheel_selection(&chromosomes);
    assert!(
        pairs.is_empty(),
        "Roulette wheel should return empty for negative total fitness"
    );
}

#[test]
fn test_roulette_wheel_all_equal_fitness() {
    // All equal fitness => uniform selection, should still return valid pairs
    let chromosomes: Vec<Chromosome> = (0..6)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 50.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::roulette_wheel_selection(&chromosomes);
    assert_eq!(pairs.len(), 3);
    for (a, b) in &pairs {
        assert!(*a < 6);
        assert!(*b < 6);
    }
}

// --- Task 1.2: SUS selection tests ---

#[test]
fn test_sus_returns_requested_couple_count() {
    let chromosomes: Vec<Chromosome> = (0..10)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 10.0 + i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    // Request 4 couples => 8 selections => 4 pairs
    let pairs = fitness_proportionate::stochastic_universal_sampling(&chromosomes, 4);
    assert_eq!(pairs.len(), 4);

    // All indices should be valid
    for (a, b) in &pairs {
        assert!(*a < 10);
        assert!(*b < 10);
    }
}

#[test]
fn test_sus_more_couples_than_chromosomes() {
    // Request more couples than chromosomes: should not panic, may reselect same individuals
    let chromosomes: Vec<Chromosome> = (0..3)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 10.0 + i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    let pairs = fitness_proportionate::stochastic_universal_sampling(&chromosomes, 10);
    assert_eq!(pairs.len(), 10);
    for (a, b) in &pairs {
        assert!(*a < 3);
        assert!(*b < 3);
    }
}

#[test]
fn test_sus_empty_population() {
    let chromosomes: Vec<Chromosome> = vec![];
    let pairs = fitness_proportionate::stochastic_universal_sampling(&chromosomes, 3);
    assert!(pairs.is_empty());
}

#[test]
fn test_sus_zero_couples() {
    let chromosomes: Vec<Chromosome> = (0..5)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::stochastic_universal_sampling(&chromosomes, 0);
    assert!(pairs.is_empty());
}

#[test]
fn test_sus_zero_fitness_population() {
    let chromosomes: Vec<Chromosome> = (0..5)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::stochastic_universal_sampling(&chromosomes, 3);
    assert!(
        pairs.is_empty(),
        "SUS should return empty for zero total fitness"
    );
}

#[test]
fn test_sus_all_equal_fitness() {
    // All equal fitness => SUS should distribute evenly
    let chromosomes: Vec<Chromosome> = (0..6)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 20.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();
    let pairs = fitness_proportionate::stochastic_universal_sampling(&chromosomes, 3);
    assert_eq!(pairs.len(), 3);
}

// --- Task 1.5: Random selection tests ---

#[test]
fn test_random_selection_can_select_last_individual() {
    // With only 2 chromosomes, the last one (index 1) must always be in the pair.
    let chromosomes: Vec<Chromosome> = (0..2)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    for _ in 0..20 {
        let pairs = random::random(&chromosomes);
        assert_eq!(pairs.len(), 1);
        let (a, b) = pairs[0];
        // Both indices 0 and 1 should appear
        assert!((a == 0 && b == 1) || (a == 1 && b == 0));
    }
}

#[test]
fn test_random_selection_all_individuals_selectable() {
    // With 4 chromosomes, over many runs, every index should appear at least once.
    let chromosomes: Vec<Chromosome> = (0..4)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    let mut seen = [false; 4];
    for _ in 0..100 {
        let pairs = random::random(&chromosomes);
        for (a, b) in &pairs {
            seen[*a] = true;
            seen[*b] = true;
        }
    }
    for (i, &was_seen) in seen.iter().enumerate() {
        assert!(
            was_seen,
            "Index {} was never selected by random selection after 100 runs",
            i
        );
    }
}

#[test]
fn test_random_selection_single_chromosome() {
    // Single chromosome => not enough for a pair
    let chromosomes = vec![Chromosome {
        dna: vec![Gene { id: 0 }],
        fitness: 0.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    }];
    let pairs = random::random(&chromosomes);
    assert!(
        pairs.is_empty(),
        "Random selection with 1 chromosome should produce no pairs"
    );
}

// ==================== Phase 2 new tests ====================

// --- Task 2.6: NaN fitness guard in selection factory ---

#[test]
fn test_selection_factory_rejects_nan_fitness() {
    // A population where one chromosome has NaN fitness should be rejected by the factory.
    let chromosomes: Vec<Chromosome> = vec![
        Chromosome {
            dna: vec![Gene { id: 1 }],
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: vec![Gene { id: 2 }],
            fitness: f64::NAN,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: vec![Gene { id: 3 }],
            fitness: 20.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];

    let config = SelectionConfiguration {
        method: Selection::Random,
        number_of_couples: 1,
    };

    let result = selection::factory(&chromosomes, config, 1);
    assert!(
        result.is_err(),
        "Selection factory should reject NaN fitness"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("NaN fitness"),
        "Error should mention NaN fitness, got: {}",
        err_msg
    );
}

#[test]
fn test_selection_factory_accepts_valid_fitness() {
    // All chromosomes have valid fitness — factory should succeed.
    let chromosomes: Vec<Chromosome> = (0..6)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 10.0 + i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    let config = SelectionConfiguration {
        method: Selection::Random,
        number_of_couples: 2,
    };

    let result = selection::factory(&chromosomes, config, 1);
    assert!(
        result.is_ok(),
        "Selection factory should accept valid fitness, got: {:?}",
        result.err()
    );
}
