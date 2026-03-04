#[cfg(test)]
mod structures;

use crate::structures::{Chromosome, Gene};
use genetic_algorithms::{fitness::FitnessFnWrapper, population::Population};

#[test]
fn test_add_chromosomes_aga() {
    //Setup of the project
    let chromosome_1 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 20.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 40.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 120.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let mut chromosomes = vec![chromosome_1, chromosome_2, chromosome_3];
    let mut population = Population::new_empty();

    //We add the chromosomes in the population 1 by 1
    population.add_chromosomes(&mut chromosomes);
    population.recalculate_aga();

    //We check the computations
    assert_eq!(population.f_max, 120.0);
    assert_eq!(population.f_avg, 60.0);
    assert_eq!(population.size(), 3);
}

#[test]
fn test_add_chromosomes() {
    //Setup of the project
    let chromosome_1 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 20.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_2 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 40.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let chromosome_3 = Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 120.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };
    let mut chromosomes = vec![chromosome_1, chromosome_2, chromosome_3];
    let mut population = Population::new_empty();

    //We add the chromosomes in the population 1 by 1
    population.add_chromosomes(&mut chromosomes);

    //We check the computations
    assert_eq!(population.size(), 3);
}

// ==================== Phase 1 new tests ====================

// --- Task 1.7: Fitness sentinel — chromosomes with true fitness 0.0 are preserved ---

#[test]
fn test_fitness_zero_is_preserved() {
    // A chromosome with fitness explicitly set to 0.0 should NOT be recalculated.
    // The NaN sentinel means only NaN triggers recalculation.
    use genetic_algorithms::configuration::ProblemSolving;

    let chromosome = Chromosome {
        dna: vec![Gene { id: 5 }, Gene { id: 10 }],
        fitness: 0.0, // Explicitly 0.0 — this is a valid fitness
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    // If we call calculate_fitness, it would compute 0*0 + 10*1 = 10.0
    // But since fitness is 0.0 (not NaN), fitness_calculation should skip it.
    let mut population = Population::new(vec![chromosome.clone()]);
    population.fitness_calculation(1, ProblemSolving::Maximization);

    // Fitness should remain 0.0, NOT recalculated to 10.0
    assert_eq!(
        population.chromosomes[0].fitness, 0.0,
        "Chromosome with fitness 0.0 should not be recalculated (NaN sentinel)"
    );
}

#[test]
fn test_nan_fitness_triggers_recalculation() {
    use genetic_algorithms::configuration::ProblemSolving;

    // Chromosome with NaN fitness should be recalculated
    let chromosome = Chromosome {
        dna: vec![Gene { id: 5 }, Gene { id: 10 }],
        fitness: f64::NAN,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    let mut population = Population::new(vec![chromosome]);
    population.fitness_calculation(1, ProblemSolving::Maximization);

    // calculate_fitness computes sum(gene.id * index) = 5*0 + 10*1 = 10.0
    assert_eq!(
        population.chromosomes[0].fitness, 10.0,
        "Chromosome with NaN fitness should be recalculated"
    );
}

// --- Task 1.8: f_max initialized to NEG_INFINITY ---

#[test]
fn test_fmax_all_negative_fitness() {
    let mut chromosomes: Vec<Chromosome> = vec![
        Chromosome {
            dna: Vec::<Gene>::new(),
            fitness: -100.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: Vec::<Gene>::new(),
            fitness: -50.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: Vec::<Gene>::new(),
            fitness: -200.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];
    let mut population = Population::new_empty();
    population.add_chromosomes(&mut chromosomes);
    population.recalculate_aga();

    // f_max should be -50.0 (the largest of the negatives)
    assert_eq!(
        population.f_max, -50.0,
        "f_max should correctly identify the largest negative fitness"
    );
    // f_avg should be (-100 + -50 + -200) / 3 = -350/3
    assert!(
        (population.f_avg - (-350.0 / 3.0)).abs() < 1e-10,
        "f_avg should be the mean of negative fitness values"
    );
}

#[test]
fn test_fmax_mixed_negative_and_positive() {
    let mut chromosomes: Vec<Chromosome> = vec![
        Chromosome {
            dna: Vec::<Gene>::new(),
            fitness: -10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: Vec::<Gene>::new(),
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: Vec::<Gene>::new(),
            fitness: -30.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];
    let mut population = Population::new_empty();
    population.add_chromosomes(&mut chromosomes);
    population.recalculate_aga();

    assert_eq!(population.f_max, 5.0);
    assert!((population.f_avg - (-35.0 / 3.0)).abs() < 1e-10);
}

#[test]
fn test_fmax_single_chromosome() {
    let mut chromosomes = vec![Chromosome {
        dna: Vec::<Gene>::new(),
        fitness: 42.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    }];
    let mut population = Population::new_empty();
    population.add_chromosomes(&mut chromosomes);
    population.recalculate_aga();

    assert_eq!(population.f_max, 42.0);
    assert_eq!(population.f_avg, 42.0);
}
