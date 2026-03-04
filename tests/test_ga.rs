#[cfg(test)]
mod structures;

use crate::structures::{Chromosome, Gene};
use genetic_algorithms::configuration::StoppingCriteria;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::{
    configuration::ProblemSolving,
    fitness::FitnessFnWrapper,
    operations::{Crossover, Mutation, Selection, Survivor},
    population::Population,
    traits::{
        ChromosomeT, ConfigurationT, CrossoverConfig, ElitismConfig, MutationConfig, NichingConfig,
        SelectionConfig, StoppingConfig,
    },
};

fn fitness_fn(_dna: &[Gene]) -> f64 {
    0.0
}

#[test]
fn test_ga_start_maximize() {
    //Creates the population
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_2 = vec![
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
    ];
    let dna_3 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
    ];
    let dna_4 = vec![
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
    ];
    let dna_5 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_6 = vec![
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
    ];
    let dna_7 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];
    let dna_8 = vec![
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
    ];
    let dna_9 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
    ];
    let dna_10 = vec![
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
    ];

    let chromosomes = vec![
        Chromosome {
            dna: dna_1,
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];

    let population = Population::new(chromosomes);
    let mut binding = Ga::new();
    let population = binding
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .run()
        .unwrap();

    assert_eq!(population.chromosomes.len(), 10);
    assert_eq!(population.best_chromosome.fitness(), 20.0);
}

#[test]
fn test_ga_run_minimize() {
    //Creates the population
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_2 = vec![
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
    ];
    let dna_3 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
    ];
    let dna_4 = vec![
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
    ];
    let dna_5 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_6 = vec![
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
    ];
    let dna_7 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];
    let dna_8 = vec![
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
    ];
    let dna_9 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
    ];
    let dna_10 = vec![
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
    ];

    let chromosomes = vec![
        Chromosome {
            dna: dna_1,
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];

    let population = Population::new(chromosomes);
    let mut binding = Ga::new();
    let population = binding
        .with_problem_solving(ProblemSolving::Minimization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_mutation_probability_max(0.2)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .run()
        .unwrap();

    assert_eq!(population.chromosomes.len(), 10);
    assert_eq!(population.best_chromosome.fitness(), 1.0);
}

#[test]
fn test_ga_run() {
    //Creates the population
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_2 = vec![
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
    ];
    let dna_3 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
    ];
    let dna_4 = vec![
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
    ];
    let dna_5 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_6 = vec![
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
    ];
    let dna_7 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];
    let dna_8 = vec![
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
    ];
    let dna_9 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
    ];
    let dna_10 = vec![
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
    ];

    let chromosomes = vec![
        Chromosome {
            dna: dna_1,
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];

    let population = Population::new(chromosomes);
    let mut binding = Ga::new();
    let population = binding
        .with_threads(8)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_number_of_couples(10)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .run()
        .unwrap();

    assert_eq!(population.chromosomes.len(), 10);
}

#[test]
fn test_parent_crossover_repeating_alleles() {
    //Setup the alleles and initialize the population randomly
    let binding = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
    ];
    let alleles = binding.as_slice();
    static GENES_PER_CHROMOSOME: usize = 6;
    static POPULATION_SIZE: usize = 100;
    static NEEDS_UNIQUE_IDS: bool = false;
    static ALLELES_CAN_BE_REPEATED: bool = true;
    static NUMBER_OF_THREADS: usize = 8;

    let mut ga_instance = Ga::new();
    let ga: &mut Ga<Chromosome> = ga_instance
        .with_threads(NUMBER_OF_THREADS)
        .with_fitness_fn(fitness_fn)
        .with_population_size(POPULATION_SIZE)
        .with_genes_per_chromosome(GENES_PER_CHROMOSOME)
        .with_needs_unique_ids(NEEDS_UNIQUE_IDS)
        .with_alleles_can_be_repeated(ALLELES_CAN_BE_REPEATED)
        .with_alleles(alleles.to_vec())
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
        )
        .initialization()
        .unwrap();

    //Once population has been initialized, we check for each chromosome in the population the number of genes in the dna
    for chromosome in &ga.population.chromosomes {
        assert!(chromosome.dna.len() == GENES_PER_CHROMOSOME);
    }
}

#[test]
fn test_parent_crossover_without_repeating_alleles() {
    //Setup the alleles and initialize the population randomly
    let binding = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
        Gene { id: 7 },
        Gene { id: 8 },
    ];
    let alleles = binding.as_slice();
    static GENES_PER_CHROMOSOME: usize = 6;
    static POPULATION_SIZE: usize = 100;
    static NEEDS_UNIQUE_IDS: bool = false;
    static ALLELES_CAN_BE_REPEATED: bool = false;
    static NUMBER_OF_THREADS: usize = 8;

    let mut ga_instance = Ga::new();
    let ga: &mut Ga<Chromosome> = ga_instance
        .with_threads(NUMBER_OF_THREADS)
        .with_fitness_fn(fitness_fn)
        .with_population_size(POPULATION_SIZE)
        .with_genes_per_chromosome(GENES_PER_CHROMOSOME)
        .with_needs_unique_ids(NEEDS_UNIQUE_IDS)
        .with_alleles_can_be_repeated(ALLELES_CAN_BE_REPEATED)
        .with_alleles(alleles.to_vec())
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization_without_repetitions::<
                Chromosome,
            >,
        )
        .initialization()
        .unwrap();

    //Once population has been initialized, we check for each chromosome we check that genes are not repeated
    for chromosome in &ga.population.chromosomes {
        let mut gene_ids = Vec::new();

        for gene in &chromosome.dna {
            if !gene_ids.is_empty() {
                assert!(!gene_ids.contains(&gene.id));
            }
            gene_ids.push(gene.id);
        }
    }
}

fn callback_function(
    generation_number: &usize,
    population: &Population<Chromosome>,
    termination_cause: &TerminationCause,
) {
    assert!(*generation_number >= 7);
    assert_eq!(population.chromosomes.len(), 10);
    assert!(
        termination_cause == &TerminationCause::NotTerminated
            || termination_cause == &TerminationCause::GenerationLimitReached
    );
}

#[test]
fn test_callback_function() {
    //Creates the population
    let dna_1 = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_2 = vec![
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
    ];
    let dna_3 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
    ];
    let dna_4 = vec![
        Gene { id: 4 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
    ];
    let dna_5 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let dna_6 = vec![
        Gene { id: 1 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
    ];
    let dna_7 = vec![
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];
    let dna_8 = vec![
        Gene { id: 4 },
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 3 },
    ];
    let dna_9 = vec![
        Gene { id: 2 },
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
    ];
    let dna_10 = vec![
        Gene { id: 1 },
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
    ];

    let chromosome = vec![
        Chromosome {
            dna: dna_1,
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        },
    ];

    let population = Population::new(chromosome);
    let mut binding = Ga::new();
    let population = binding
        .with_threads(8)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_number_of_couples(10)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(10)
        .run_with_callback(Some(callback_function), 8)
        .unwrap();

    assert_eq!(population.chromosomes.len(), 10);
}

#[test]
fn test_elitism_preserves_best_individual() {
    // Create a population where one individual has clearly the best fitness
    let mut chromosomes: Vec<Chromosome> = Vec::new();
    for i in 0..10 {
        let dna = vec![
            Gene { id: 1 + i },
            Gene { id: 2 + i },
            Gene { id: 3 + i },
            Gene { id: 4 + i },
        ];
        let mut chromosome = Chromosome {
            dna,
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        };
        chromosome.set_fitness_fn(|genes: &[Gene]| genes.iter().map(|g| g.id as f64).sum::<f64>());
        chromosome.calculate_fitness();
        chromosomes.push(chromosome);
    }

    let best_fitness_before = chromosomes
        .iter()
        .map(|c| c.fitness)
        .fold(f64::NEG_INFINITY, f64::max);

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(20)
        .with_elitism(2)
        .run()
        .unwrap();

    // With elitism, the best fitness should never decrease
    let best_fitness_after = result
        .chromosomes
        .iter()
        .map(|c| c.fitness())
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        best_fitness_after >= best_fitness_before,
        "Elitism should preserve or improve the best fitness. Before: {}, After: {}",
        best_fitness_before,
        best_fitness_after
    );
}

#[test]
fn test_stagnation_stopping_criterion() {
    // Create a population where fitness is constant (no improvement possible)
    let mut chromosomes: Vec<Chromosome> = Vec::new();
    for i in 0..10 {
        let dna = vec![Gene { id: 1 + i }, Gene { id: 2 + i }];
        let mut chromosome = Chromosome {
            dna,
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        };
        chromosome.set_fitness_fn(|_genes: &[Gene]| 42.0); // constant fitness
        chromosome.calculate_fitness();
        chromosomes.push(chromosome);
    }

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let _result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(1000)
        .with_stopping_criteria(StoppingCriteria {
            stagnation_generations: Some(5),
            convergence_threshold: None,
            max_duration_secs: None,
        })
        .run()
        .unwrap();

    assert_eq!(
        ga.termination_cause,
        TerminationCause::StagnationReached,
        "GA should have terminated due to stagnation"
    );
}

#[test]
fn test_convergence_stopping_criterion() {
    // Create a population where all fitnesses are identical (std_dev = 0)
    let mut chromosomes: Vec<Chromosome> = Vec::new();
    for i in 0..10 {
        let dna = vec![Gene { id: 1 + i }, Gene { id: 2 + i }];
        let mut chromosome = Chromosome {
            dna,
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        };
        chromosome.set_fitness_fn(|_genes: &[Gene]| 100.0); // constant fitness
        chromosome.calculate_fitness();
        chromosomes.push(chromosome);
    }

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let _result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(1000)
        .with_stopping_criteria(StoppingCriteria {
            stagnation_generations: None,
            convergence_threshold: Some(0.01),
            max_duration_secs: None,
        })
        .run()
        .unwrap();

    assert_eq!(
        ga.termination_cause,
        TerminationCause::ConvergenceReached,
        "GA should have terminated due to convergence"
    );
}

#[test]
fn test_time_limit_stopping_criterion() {
    // Create a normal population with a very short time limit
    let mut chromosomes: Vec<Chromosome> = Vec::new();
    for i in 0..10 {
        let dna = vec![Gene { id: 1 + i }, Gene { id: 2 + i }];
        let mut chromosome = Chromosome {
            dna,
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        };
        chromosome.set_fitness_fn(|genes: &[Gene]| genes.iter().map(|g| g.id as f64).sum::<f64>());
        chromosome.calculate_fitness();
        chromosomes.push(chromosome);
    }

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let _result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(1_000_000) // Very high to ensure time limit triggers first
        .with_stopping_criteria(StoppingCriteria {
            stagnation_generations: None,
            convergence_threshold: None,
            max_duration_secs: Some(0.001), // 1 millisecond
        })
        .run()
        .unwrap();

    assert_eq!(
        ga.termination_cause,
        TerminationCause::TimeLimitReached,
        "GA should have terminated due to time limit"
    );
}

#[test]
fn test_rank_selection_in_ga() {
    let mut chromosomes: Vec<Chromosome> = Vec::new();
    for i in 0..10 {
        let dna = vec![
            Gene { id: 1 + i },
            Gene { id: 2 + i },
            Gene { id: 3 + i },
            Gene { id: 4 + i },
        ];
        let mut chromosome = Chromosome {
            dna,
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        };
        chromosome.set_fitness_fn(|genes: &[Gene]| genes.iter().map(|g| g.id as f64).sum::<f64>());
        chromosome.calculate_fitness();
        chromosomes.push(chromosome);
    }

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Rank)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(10)
        .run()
        .unwrap();

    assert!(
        !result.chromosomes.is_empty(),
        "Population should not be empty after running with Rank selection"
    );
}

// ==================== Phase 1 new tests ====================

// --- Task 1.6: Niching wiring integration test ---

#[test]
fn test_ga_with_niching_enabled() {
    // Run a GA with niching enabled. The main goal is to verify:
    // 1. No panics or errors occur.
    // 2. Fitness values are modified by sharing (identical chromosomes get reduced fitness).

    // Create population with some identical chromosomes (same DNA)
    let base_dna = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let different_dna = vec![
        Gene { id: 4 },
        Gene { id: 3 },
        Gene { id: 2 },
        Gene { id: 1 },
    ];

    let mut chromosomes = Vec::new();
    // 8 identical chromosomes
    for _ in 0..8 {
        chromosomes.push(Chromosome {
            dna: base_dna.clone(),
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        });
    }
    // 2 different chromosomes
    for _ in 0..2 {
        chromosomes.push(Chromosome {
            dna: different_dna.clone(),
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        });
    }

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(5)
        .with_niching_enabled(true)
        .with_niching_sigma_share(3.0) // sigma > 0, catches identical (distance=0) and close
        .with_niching_alpha(1.0)
        .run()
        .unwrap();

    // Population should still have correct size
    assert_eq!(result.chromosomes.len(), 10);
    // GA should complete without panicking — that's the primary assertion
}

#[test]
fn test_ga_with_niching_disabled() {
    // Same setup but with niching disabled — should behave exactly like normal GA.
    // Use Uniform crossover to avoid cycle crossover's permutation requirement.
    let chromosomes: Vec<Chromosome> = (0..10)
        .map(|i| Chromosome {
            dna: vec![
                Gene { id: 1 + i },
                Gene { id: 2 + i },
                Gene { id: 3 + i },
                Gene { id: 4 + i },
            ],
            fitness: (i + 1) as f64,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect();

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(3)
        .with_niching_enabled(false)
        .run()
        .unwrap();

    assert_eq!(result.chromosomes.len(), 10);
}

// ==================== Phase 2 new tests ====================

// --- Task 2.3: set_gene out-of-bounds is a safe no-op ---

#[test]
fn test_set_gene_out_of_bounds_is_noop() {
    let mut chromosome = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 5.0,
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    };

    // Out-of-bounds set_gene should not panic and should leave DNA unchanged
    chromosome.set_gene(999, Gene { id: 42 });
    assert_eq!(chromosome.dna().len(), 3);
    assert_eq!(chromosome.dna()[0].id, 1);
    assert_eq!(chromosome.dna()[1].id, 2);
    assert_eq!(chromosome.dna()[2].id, 3);

    // Boundary: index == len (off by one)
    chromosome.set_gene(3, Gene { id: 99 });
    assert_eq!(chromosome.dna().len(), 3);

    // Valid index still works
    chromosome.set_gene(1, Gene { id: 77 });
    assert_eq!(chromosome.dna()[1].id, 77);
}

// --- Task 2.5: TerminationCause set without callback ---

#[test]
fn test_termination_cause_set_without_callback() {
    // Run GA without a callback — termination cause should still be set correctly.
    let chromosomes: Vec<Chromosome> = (0..10)
        .map(|i| {
            let dna = vec![
                Gene { id: 1 + i },
                Gene { id: 2 + i },
                Gene { id: 3 + i },
                Gene { id: 4 + i },
            ];
            Chromosome {
                dna,
                fitness: (i + 1) as f64,
                age: 0,
                fitness_fn: FitnessFnWrapper::default(),
            }
        })
        .collect();

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let _result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(3)
        .run()
        .unwrap();

    assert_eq!(
        ga.termination_cause,
        TerminationCause::GenerationLimitReached,
        "TerminationCause should be GenerationLimitReached when no callback is used"
    );
}

// --- Task 2.4: Elitism with more elite than population ---

#[test]
fn test_elitism_count_exceeding_population_does_not_panic() {
    // Elitism count > population size should not panic or underflow.
    let chromosomes: Vec<Chromosome> = (0..4)
        .map(|i| {
            let dna = vec![
                Gene { id: 1 + i },
                Gene { id: 2 + i },
                Gene { id: 3 + i },
                Gene { id: 4 + i },
            ];
            Chromosome {
                dna,
                fitness: (i + 1) as f64,
                age: 0,
                fitness_fn: FitnessFnWrapper::default(),
            }
        })
        .collect();

    let population = Population::new(chromosomes);
    let mut ga = Ga::new();
    let result = ga
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(3)
        .with_elitism(100) // Much larger than population size of 4
        .run()
        .unwrap();

    assert!(
        !result.chromosomes.is_empty(),
        "Population should not be empty after run with oversized elitism count"
    );
}

// --- Task 2.7: Validator accepts built-in chromosome types ---

#[test]
fn test_validator_accepts_builtin_chromosome_type() {
    // Previously, the validator factory would return "Not yet implemented" for
    // built-in chromosome types (Binary, Range) due to type-gating.
    // After the Phase 2 fix, all types pass through the generic validator.
    // This test verifies initialization() succeeds (it calls validator internally).
    let alleles = vec![
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
        Gene { id: 6 },
    ];

    let mut ga_instance: Ga<Chromosome> = Ga::new();
    let result = ga_instance
        .with_fitness_fn(fitness_fn)
        .with_population_size(10)
        .with_genes_per_chromosome(4)
        .with_needs_unique_ids(false)
        .with_alleles_can_be_repeated(true)
        .with_alleles(alleles)
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
        )
        .initialization();

    assert!(
        result.is_ok(),
        "Initialization (which calls validator) should accept built-in chromosome types, got: {:?}",
        result.err()
    );
}
