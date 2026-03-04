#[cfg(test)]
use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{
    configuration::{LimitConfiguration, ProblemSolving},
    fitness::FitnessFnWrapper,
    operations::survivor::{self, age, fitness},
    operations::Survivor,
    traits::ChromosomeT,
};

#[test]
fn test_fitness_survivor_minization() {
    //We create 12 fitnesss for 12 chromosomes
    let dna_1 = vec![Gene { id: 1 }];
    let dna_2 = vec![Gene { id: 1 }];
    let dna_3 = vec![Gene { id: 1 }];
    let dna_4 = vec![Gene { id: 1 }];
    let dna_5 = vec![Gene { id: 1 }];
    let dna_6 = vec![Gene { id: 1 }];
    let dna_7 = vec![Gene { id: 1 }];
    let dna_8 = vec![Gene { id: 1 }];
    let dna_9 = vec![Gene { id: 1 }];
    let dna_10 = vec![Gene { id: 1 }];
    let dna_11 = vec![Gene { id: 1 }];
    let dna_12 = vec![Gene { id: 1 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.1,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 10.2,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 10.3,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 11.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 12.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_6 = Chromosome {
        dna: dna_6,
        fitness: 13.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_7 = Chromosome {
        dna: dna_7,
        fitness: 14.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_8 = Chromosome {
        dna: dna_8,
        fitness: 15.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_9 = Chromosome {
        dna: dna_9,
        fitness: 16.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_10 = Chromosome {
        dna: dna_10,
        fitness: 17.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_11 = Chromosome {
        dna: dna_11,
        fitness: 18.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_12 = Chromosome {
        dna: dna_12,
        fitness: 19.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let mut population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
        chromosome_6,
        chromosome_7,
        chromosome_8,
        chromosome_9,
        chromosome_10,
        chromosome_11,
        chromosome_12,
    ];

    fitness::fitness_based(
        &mut population,
        10,
        LimitConfiguration {
            problem_solving: ProblemSolving::Minimization,
            ..Default::default()
        },
    );

    //Tests that the population has 10 chromosomes
    assert_eq!(population.len(), 10);
    assert_eq!(population[0].fitness(), 17.0);
    assert_eq!(population[9].fitness(), 10.1);
}

#[test]
fn test_fitness_survivor_maximization() {
    //We create 12 fitnesss for 12 chromosomes
    let dna_1 = vec![Gene { id: 1 }];
    let dna_2 = vec![Gene { id: 1 }];
    let dna_3 = vec![Gene { id: 1 }];
    let dna_4 = vec![Gene { id: 1 }];
    let dna_5 = vec![Gene { id: 1 }];
    let dna_6 = vec![Gene { id: 1 }];
    let dna_7 = vec![Gene { id: 1 }];
    let dna_8 = vec![Gene { id: 1 }];
    let dna_9 = vec![Gene { id: 1 }];
    let dna_10 = vec![Gene { id: 1 }];
    let dna_11 = vec![Gene { id: 1 }];
    let dna_12 = vec![Gene { id: 1 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.1,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 10.2,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 10.3,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 11.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 12.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_6 = Chromosome {
        dna: dna_6,
        fitness: 13.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_7 = Chromosome {
        dna: dna_7,
        fitness: 14.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_8 = Chromosome {
        dna: dna_8,
        fitness: 15.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_9 = Chromosome {
        dna: dna_9,
        fitness: 16.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_10 = Chromosome {
        dna: dna_10,
        fitness: 17.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_11 = Chromosome {
        dna: dna_11,
        fitness: 18.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_12 = Chromosome {
        dna: dna_12,
        fitness: 19.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let mut population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
        chromosome_6,
        chromosome_7,
        chromosome_8,
        chromosome_9,
        chromosome_10,
        chromosome_11,
        chromosome_12,
    ];

    fitness::fitness_based(
        &mut population,
        10,
        LimitConfiguration {
            problem_solving: ProblemSolving::Maximization,
            ..Default::default()
        },
    );

    //Tests that the population has 10 chromosomes
    assert_eq!(population.len(), 10);
    assert_eq!(population[0].fitness(), 19.0);
    assert_eq!(population[9].fitness(), 10.3);
}

#[test]
fn test_age_based_survivor() {
    //We create 12 fitnesss for 12 chromosomes
    let dna_1 = vec![Gene { id: 1 }];
    let dna_2 = vec![Gene { id: 1 }];
    let dna_3 = vec![Gene { id: 1 }];
    let dna_4 = vec![Gene { id: 1 }];
    let dna_5 = vec![Gene { id: 1 }];
    let dna_6 = vec![Gene { id: 1 }];
    let dna_7 = vec![Gene { id: 1 }];
    let dna_8 = vec![Gene { id: 1 }];
    let dna_9 = vec![Gene { id: 1 }];
    let dna_10 = vec![Gene { id: 1 }];
    let dna_11 = vec![Gene { id: 1 }];
    let dna_12 = vec![Gene { id: 1 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.1,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 10.2,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 10.3,
        age: 1,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 11.0,
        age: 1,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 12.0,
        age: 3,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_6 = Chromosome {
        dna: dna_6,
        fitness: 13.0,
        age: 3,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_7 = Chromosome {
        dna: dna_7,
        fitness: 14.0,
        age: 2,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_8 = Chromosome {
        dna: dna_8,
        fitness: 15.0,
        age: 2,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_9 = Chromosome {
        dna: dna_9,
        fitness: 16.0,
        age: 2,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_10 = Chromosome {
        dna: dna_10,
        fitness: 17.0,
        age: 2,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_11 = Chromosome {
        dna: dna_11,
        fitness: 18.0,
        age: 1,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_12 = Chromosome {
        dna: dna_12,
        fitness: 19.0,
        age: 1,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let mut population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
        chromosome_6,
        chromosome_7,
        chromosome_8,
        chromosome_9,
        chromosome_10,
        chromosome_11,
        chromosome_12,
    ];

    age::age_based(&mut population, 6);

    //Tests that the population has 6 chromosomes
    assert_eq!(population.len(), 6);
    assert_eq!(population[0].age(), 3);
    assert_eq!(population[1].age(), 3);
    assert_eq!(population[2].age(), 2);
    assert_eq!(population[3].age(), 2);
    assert_eq!(population[4].age(), 2);
    assert_eq!(population[5].age(), 2);
}

#[test]
fn test_survivor_fitness_fixed() {
    //We create 12 fitnesss for 12 chromosomes
    let dna_1 = vec![Gene { id: 1 }];
    let dna_2 = vec![Gene { id: 1 }];
    let dna_3 = vec![Gene { id: 1 }];
    let dna_4 = vec![Gene { id: 1 }];
    let dna_5 = vec![Gene { id: 1 }];
    let dna_6 = vec![Gene { id: 1 }];
    let dna_7 = vec![Gene { id: 1 }];
    let dna_8 = vec![Gene { id: 1 }];
    let dna_9 = vec![Gene { id: 1 }];
    let dna_10 = vec![Gene { id: 1 }];
    let dna_11 = vec![Gene { id: 1 }];
    let dna_12 = vec![Gene { id: 1 }];

    //We create the chromosomes
    let chromosome_1 = Chromosome {
        dna: dna_1,
        fitness: 10.1,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: dna_2,
        fitness: 10.2,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: dna_3,
        fitness: 10.3,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_4 = Chromosome {
        dna: dna_4,
        fitness: 11.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_5 = Chromosome {
        dna: dna_5,
        fitness: 12.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_6 = Chromosome {
        dna: dna_6,
        fitness: 13.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_7 = Chromosome {
        dna: dna_7,
        fitness: 14.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_8 = Chromosome {
        dna: dna_8,
        fitness: 15.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_9 = Chromosome {
        dna: dna_9,
        fitness: 16.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_10 = Chromosome {
        dna: dna_10,
        fitness: 17.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_11 = Chromosome {
        dna: dna_11,
        fitness: 18.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_12 = Chromosome {
        dna: dna_12,
        fitness: 19.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    //We create the population and create the random mating
    let mut population = vec![
        chromosome_1,
        chromosome_2,
        chromosome_3,
        chromosome_4,
        chromosome_5,
        chromosome_6,
        chromosome_7,
        chromosome_8,
        chromosome_9,
        chromosome_10,
        chromosome_11,
        chromosome_12,
    ];

    fitness::fitness_based(
        &mut population,
        10,
        LimitConfiguration {
            problem_solving: ProblemSolving::FixedFitness,
            fitness_target: Some(14.5),
            ..Default::default()
        },
    );

    //Tests that the population has 10 chromosomes
    assert_eq!(population.len(), 10);
    assert_eq!(population[0].fitness(), 10.2);
    assert_eq!(population[9].fitness(), 15.0);
}

// ==================== Phase 2 new tests ====================

// --- Task 2.6: NaN fitness guard in survivor factory ---

#[test]
fn test_survivor_factory_rejects_nan_fitness() {
    let mut chromosomes: Vec<Chromosome> = vec![
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

    let result = survivor::factory(
        Survivor::Fitness,
        &mut chromosomes,
        2,
        LimitConfiguration {
            problem_solving: ProblemSolving::Maximization,
            ..Default::default()
        },
    );
    assert!(
        result.is_err(),
        "Survivor factory should reject NaN fitness"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("NaN fitness"),
        "Error should mention NaN fitness, got: {}",
        err_msg
    );
}

#[test]
fn test_survivor_factory_accepts_valid_fitness() {
    let mut chromosomes: Vec<Chromosome> = (0..5)
        .map(|i| Chromosome {
            dna: vec![Gene { id: i }],
            fitness: 10.0 + i as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    let result = survivor::factory(
        Survivor::Fitness,
        &mut chromosomes,
        3,
        LimitConfiguration {
            problem_solving: ProblemSolving::Maximization,
            ..Default::default()
        },
    );
    assert!(
        result.is_ok(),
        "Survivor factory should accept valid fitness, got: {:?}",
        result.err()
    );
    assert_eq!(chromosomes.len(), 3);
}
