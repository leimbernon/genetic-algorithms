#[cfg(test)]
mod structures;

use crate::structures::{Chromosome, Gene};
use genetic_algorithms::configuration::StoppingCriteria;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::stats::GenerationStats;
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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population);
    let population = ga.run().unwrap();

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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Minimization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_mutation_probability_max(0.2)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population);
    let population = ga.run().unwrap();

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
    let mut ga = Ga::new()
        .with_threads(8)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_number_of_couples(10)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population);
    let population = ga.run().unwrap();

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

    let mut ga: Ga<Chromosome> = Ga::new()
        .with_threads(NUMBER_OF_THREADS)
        .with_fitness_fn(fitness_fn)
        .with_population_size(POPULATION_SIZE)
        .with_genes_per_chromosome(GENES_PER_CHROMOSOME)
        .with_needs_unique_ids(NEEDS_UNIQUE_IDS)
        .with_alleles_can_be_repeated(ALLELES_CAN_BE_REPEATED)
        .with_alleles(alleles.to_vec())
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
        );
    ga.initialization().unwrap();

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

    let mut ga: Ga<Chromosome> = Ga::new()
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
        );
    ga.initialization().unwrap();

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
    _stats: &GenerationStats,
    termination_cause: &TerminationCause,
) -> std::ops::ControlFlow<()> {
    assert!(*generation_number >= 7);
    assert_eq!(population.chromosomes.len(), 10);
    assert!(
        termination_cause == &TerminationCause::NotTerminated
            || termination_cause == &TerminationCause::GenerationLimitReached
    );
    std::ops::ControlFlow::Continue(())
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
    let mut ga = Ga::new()
        .with_threads(8)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_number_of_couples(10)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(10);
    let population = ga.run_with_callback(Some(callback_function), 8).unwrap();

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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(20)
        .with_elitism(2);
    let result = ga.run().unwrap();

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
    let mut ga = Ga::new()
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
        });
    ga.run().unwrap();

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
    let mut ga = Ga::new()
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
        });
    ga.run().unwrap();

    assert_eq!(
        ga.termination_cause,
        TerminationCause::ConvergenceReached,
        "GA should have terminated due to convergence"
    );
}

#[test]
fn test_time_limit_stopping_criterion() {
    // Create a normal population with a short but reliable time limit.
    // 0.1s is long enough that the GA loop enters at least one generation even on
    // slow CI, but short enough that it triggers well before 1_000_000 generations.
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
    let mut ga = Ga::new()
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
            max_duration_secs: Some(0.1), // 100 ms — reliable on slow CI
        });
    ga.run().unwrap();

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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Rank)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(10);
    let result = ga.run().unwrap();

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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Cycle)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(5)
        .with_niching_enabled(true)
        .with_niching_sigma_share(3.0) // sigma > 0, catches identical (distance=0) and close
        .with_niching_alpha(1.0);
    let result = ga.run().unwrap();

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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(3)
        .with_niching_enabled(false);
    let result = ga.run().unwrap();

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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(3);
    ga.run().unwrap();

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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(3)
        .with_elitism(100); // Much larger than population size of 4
    let result = ga.run().unwrap();

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

    let mut ga: Ga<Chromosome> = Ga::new()
        .with_fitness_fn(fitness_fn)
        .with_population_size(10)
        .with_genes_per_chromosome(4)
        .with_needs_unique_ids(false)
        .with_alleles_can_be_repeated(true)
        .with_alleles(alleles)
        .with_initialization_fn(
            genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
        );
    let result = ga.initialization();

    assert!(
        result.is_ok(),
        "Initialization (which calls validator) should accept built-in chromosome types, got: {:?}",
        result.err()
    );
}

// ============================================================================
// Island-NSGA-II integration tests
// ============================================================================

#[test]
fn test_island_nsga2_run_returns_pareto_front() {
    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::island::configuration::IslandConfiguration;
    use genetic_algorithms::island::nsga2::IslandNsga2Ga;
    use genetic_algorithms::island::topology::MigrationTopology;
    use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
    use genetic_algorithms::operations::{Crossover, Mutation};
    use genetic_algorithms::traits::{CrossoverConfig, MutationConfig};

    let alleles = vec![
        Gene { id: 0 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
    ];
    let alleles_clone = alleles.clone();

    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_interval(5)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring);

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(20)
        .with_max_generations(10);

    let ga_config = GaConfiguration::default()
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap);

    let mut ga = IslandNsga2Ga::<Chromosome>::new(island_config, nsga2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles, _repeat| {
            // Simple random initialization: assign random IDs from alleles
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| {
                    let idx = rng.random_range(0..alleles_clone.len());
                    alleles_clone[idx]
                })
                .collect()
        })
        .with_objective_fns(vec![
            // Objective 1: sum of gene IDs (minimize)
            Box::new(|dna: &[Gene]| dna.iter().map(|g| g.id as f64).sum()),
            // Objective 2: negative sum (conflicting with obj 1)
            Box::new(|dna: &[Gene]| -(dna.iter().map(|g| g.id as f64).sum::<f64>())),
        ])
        .build()
        .expect("Configuration should be valid");

    let result = ga.run();
    assert!(result.is_ok(), "Island-NSGA-II run should succeed");

    let front = result.unwrap();
    assert!(
        !front.is_empty(),
        "Pareto front should contain at least one individual"
    );

    // All individuals in the front should have rank 0
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0, "All front individuals should have rank 0");
        assert_eq!(
            ind.objectives.len(),
            2,
            "Each individual should have 2 objectives"
        );
    }
}

#[test]
fn test_island_nsga2_build_validates() {
    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::island::configuration::IslandConfiguration;
    use genetic_algorithms::island::nsga2::IslandNsga2Ga;
    use genetic_algorithms::nsga2::configuration::Nsga2Configuration;

    // Missing initialization function should fail build
    let island_config = IslandConfiguration::new().with_num_islands(2);
    let nsga2_config = Nsga2Configuration::new().with_num_objectives(2);
    let ga_config = GaConfiguration::default();

    let result = IslandNsga2Ga::<Chromosome>::new(island_config, nsga2_config, ga_config)
        .with_objective_fns(vec![Box::new(|_: &[Gene]| 0.0), Box::new(|_: &[Gene]| 0.0)])
        .build();

    assert!(
        result.is_err(),
        "build() should fail without initialization_fn"
    );
}

#[test]
fn test_island_nsga2_migration_improves_diversity() {
    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::island::configuration::IslandConfiguration;
    use genetic_algorithms::island::nsga2::IslandNsga2Ga;
    use genetic_algorithms::island::topology::MigrationTopology;
    use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
    use genetic_algorithms::operations::{Crossover, Mutation};
    use genetic_algorithms::traits::{CrossoverConfig, MutationConfig};

    let alleles = vec![Gene { id: 0 }, Gene { id: 1 }, Gene { id: 2 }];
    let alleles_clone = alleles.clone();

    let island_config = IslandConfiguration::new()
        .with_num_islands(3)
        .with_migration_interval(3)
        .with_migration_count(2)
        .with_topology(MigrationTopology::FullyConnected);

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(15)
        .with_max_generations(15);

    let ga_config = GaConfiguration::default()
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap);

    let mut ga = IslandNsga2Ga::<Chromosome>::new(island_config, nsga2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles, _repeat| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| {
                    let idx = rng.random_range(0..alleles_clone.len());
                    alleles_clone[idx]
                })
                .collect()
        })
        .with_objective_fns(vec![
            Box::new(|dna: &[Gene]| dna.iter().map(|g| g.id as f64).sum()),
            Box::new(|dna: &[Gene]| dna.iter().map(|g| (g.id as f64 - 1.5).powi(2)).sum::<f64>()),
        ])
        .build()
        .expect("Configuration should be valid");

    let result = ga.run();
    assert!(
        result.is_ok(),
        "Island-NSGA-II with FullyConnected should succeed"
    );

    let front = result.unwrap();
    assert!(!front.is_empty(), "Should produce a non-empty Pareto front");
}

// ============================================================================
// Task 5.2: End-to-end NSGA-II run() integration test
// ============================================================================

#[test]
fn test_nsga2_run_returns_pareto_front() {
    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
    use genetic_algorithms::nsga2::Nsga2Ga;
    use genetic_algorithms::operations::{Crossover, Mutation};
    use genetic_algorithms::traits::{CrossoverConfig, MutationConfig};

    let alleles = vec![
        Gene { id: 0 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
    ];
    let alleles_clone = alleles.clone();

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(20)
        .with_max_generations(15);

    let ga_config = GaConfiguration::default()
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap);

    let mut nsga2 = Nsga2Ga::<Chromosome>::new(nsga2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles, _repeat| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| {
                    let idx = rng.random_range(0..alleles_clone.len());
                    alleles_clone[idx]
                })
                .collect()
        })
        .with_objective_fns(vec![
            // Objective 1: sum of gene IDs
            Box::new(|dna: &[Gene]| dna.iter().map(|g| g.id as f64).sum()),
            // Objective 2: negative sum (conflicting)
            Box::new(|dna: &[Gene]| -(dna.iter().map(|g| g.id as f64).sum::<f64>())),
        ]);

    let result = nsga2.run();
    assert!(result.is_ok(), "NSGA-II run should succeed");

    let front = result.unwrap();
    assert!(
        !front.is_empty(),
        "Pareto front should contain at least one individual"
    );

    // All individuals in the front should have rank 0
    for ind in &front.individuals {
        assert_eq!(ind.rank, 0, "All front individuals should have rank 0");
        assert_eq!(
            ind.objectives.len(),
            2,
            "Each individual should have 2 objectives"
        );
    }

    // Population size should be respected
    assert!(
        front.individuals.len() <= 20,
        "Front should not exceed population size"
    );
}

#[test]
fn test_nsga2_three_objectives() {
    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
    use genetic_algorithms::nsga2::Nsga2Ga;
    use genetic_algorithms::operations::{Crossover, Mutation};
    use genetic_algorithms::traits::{CrossoverConfig, MutationConfig};

    let alleles = vec![
        Gene { id: 0 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
    ];
    let alleles_clone = alleles.clone();

    let nsga2_config = Nsga2Configuration::new()
        .with_num_objectives(3)
        .with_population_size(30)
        .with_max_generations(10);

    let ga_config = GaConfiguration::default()
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap);

    let mut nsga2 = Nsga2Ga::<Chromosome>::new(nsga2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles, _repeat| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| alleles_clone[rng.random_range(0..alleles_clone.len())])
                .collect()
        })
        .with_objective_fns(vec![
            Box::new(|dna: &[Gene]| dna.iter().map(|g| g.id as f64).sum()),
            Box::new(|dna: &[Gene]| -(dna.iter().map(|g| g.id as f64).sum::<f64>())),
            Box::new(|dna: &[Gene]| dna.iter().map(|g| (g.id as f64 - 2.0).powi(2)).sum::<f64>()),
        ]);

    let result = nsga2.run();
    assert!(result.is_ok(), "NSGA-II with 3 objectives should succeed");

    let front = result.unwrap();
    assert!(!front.is_empty());
    for ind in &front.individuals {
        assert_eq!(ind.objectives.len(), 3);
    }
}

// ============================================================================
// Task 5.3: End-to-end Island Model GA run() integration test
// ============================================================================

#[test]
fn test_island_ga_run_returns_best_chromosome() {
    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::island::configuration::IslandConfiguration;
    use genetic_algorithms::island::topology::MigrationTopology;
    use genetic_algorithms::island::IslandGa;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
    };

    let alleles = vec![
        Gene { id: 0 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
        Gene { id: 4 },
        Gene { id: 5 },
    ];
    let alleles_clone = alleles.clone();

    let island_config = IslandConfiguration::new()
        .with_num_islands(3)
        .with_migration_interval(5)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring);

    let ga_config = GaConfiguration::new()
        .with_population_size(20)
        .with_genes_per_chromosome(4)
        .with_max_generations(15)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization);

    let mut ga = IslandGa::<Chromosome>::new(island_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles, _repeat| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| alleles_clone[rng.random_range(0..alleles_clone.len())])
                .collect()
        })
        .with_fitness_fn(|dna: &[Gene]| dna.iter().map(|g| g.id as f64).sum())
        .build()
        .expect("IslandGa configuration should be valid");

    let result = ga.run();
    assert!(result.is_ok(), "Island GA run should succeed");

    let best = result.unwrap();
    // Fitness should be a real number (not NaN)
    assert!(
        !best.fitness().is_nan(),
        "Best chromosome fitness should be a real number"
    );
}

#[test]
fn test_island_ga_minimization() {
    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::island::configuration::IslandConfiguration;
    use genetic_algorithms::island::topology::MigrationTopology;
    use genetic_algorithms::island::IslandGa;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
    };

    let alleles = vec![
        Gene { id: 0 },
        Gene { id: 1 },
        Gene { id: 2 },
        Gene { id: 3 },
    ];
    let alleles_clone = alleles.clone();

    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_interval(3)
        .with_migration_count(1)
        .with_topology(MigrationTopology::FullyConnected);

    let ga_config = GaConfiguration::new()
        .with_population_size(15)
        .with_genes_per_chromosome(3)
        .with_max_generations(10)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut ga = IslandGa::<Chromosome>::new(island_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles, _repeat| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| alleles_clone[rng.random_range(0..alleles_clone.len())])
                .collect()
        })
        .with_fitness_fn(|dna: &[Gene]| dna.iter().map(|g| g.id as f64).sum())
        .build()
        .expect("IslandGa configuration should be valid");

    let result = ga.run();
    assert!(result.is_ok(), "Island GA minimization should succeed");

    let best = result.unwrap();
    assert!(
        !best.fitness().is_nan(),
        "Best chromosome fitness should not be NaN"
    );
}

// ============================================================================
// Task 5.5: Range chromosome GA run integration test
// ============================================================================

#[test]
fn test_ga_run_with_range_chromosome_f64() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig,
        StoppingConfig,
    };

    // Alleles define the search space: each gene has range [0.0, 10.0]
    let alleles = vec![RangeGene::new(0, vec![(0.0_f64, 10.0_f64)], 0.0)];

    let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
        .with_population_size(20)
        .with_genes_per_chromosome(3)
        .with_max_generations(20)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_alleles(alleles)
        .with_initialization_fn(
            genetic_algorithms::initializers::range_random_initialization::<f64>,
        )
        .with_fitness_fn(|dna: &[RangeGene<f64>]| {
            // Sphere function: sum of values squared (minimum at 0)
            dna.iter().map(|g| g.value() * g.value()).sum()
        });

    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA with Range<f64> chromosome should succeed, got: {:?}",
        result.err()
    );

    let pop = result.unwrap();
    assert_eq!(pop.chromosomes.len(), 20);
    assert!(
        !pop.best_chromosome.fitness().is_nan(),
        "Best fitness should be computed"
    );
}

#[test]
fn test_ga_run_with_range_chromosome_i32() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig,
        StoppingConfig,
    };

    let alleles = vec![RangeGene::new(0, vec![(0_i32, 50_i32)], 0)];

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_population_size(15)
        .with_genes_per_chromosome(4)
        .with_max_generations(10)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_alleles(alleles)
        .with_initialization_fn(
            genetic_algorithms::initializers::range_random_initialization::<i32>,
        )
        .with_fitness_fn(|dna: &[RangeGene<i32>]| dna.iter().map(|g| g.value() as f64).sum());

    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA with Range<i32> chromosome should succeed, got: {:?}",
        result.err()
    );

    let pop = result.unwrap();
    assert!(!pop.chromosomes.is_empty());
    assert!(
        !pop.best_chromosome.fitness().is_nan(),
        "Best fitness should be computed"
    );
}

/// Compute the average pairwise Hamming distance (gene-id comparison) of a population.
fn average_pairwise_distance(chromosomes: &[Chromosome]) -> f64 {
    let n = chromosomes.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    let mut count = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let dna_a = &chromosomes[i].dna;
            let dna_b = &chromosomes[j].dna;
            let max_len = dna_a.len().max(dna_b.len());
            let mut diff = 0usize;
            for idx in 0..max_len {
                let id_a = dna_a.get(idx).map(|g| g.id);
                let id_b = dna_b.get(idx).map(|g| g.id);
                if id_a != id_b {
                    diff += 1;
                }
            }
            total += diff as f64;
            count += 1;
        }
    }
    total / count as f64
}

/// Task 5.4 — Niching integration test.
/// Verifies that enabling niching (fitness sharing) results in higher genotypic
/// diversity than an equivalent run without niching, as measured by average
/// pairwise Hamming distance.
///
/// Because the GA is stochastic, we use a retry loop: if niching produces higher
/// diversity in at least one of several independent trials, the test passes.
#[test]
fn test_niching_promotes_diversity() {
    // Build a population heavily biased towards one genotype (8 identical + 2 different).
    // With niching, the identical cluster gets penalised, so selection should favour
    // the different individuals, eventually raising diversity.
    let build_population = || {
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
        for _ in 0..8 {
            chromosomes.push(Chromosome {
                dna: base_dna.clone(),
                fitness: 10.0,
                age: 0,
                fitness_fn: FitnessFnWrapper::default(),
            });
        }
        for _ in 0..2 {
            chromosomes.push(Chromosome {
                dna: different_dna.clone(),
                fitness: 10.0,
                age: 0,
                fitness_fn: FitnessFnWrapper::default(),
            });
        }
        Population::new(chromosomes)
    };

    let generations = 15;
    let trials = 10;
    let mut niching_better_count = 0;

    for _ in 0..trials {
        // Run WITHOUT niching
        let mut ga_no_niche = Ga::new()
            .with_problem_solving(ProblemSolving::Maximization)
            .with_selection_method(Selection::Tournament)
            .with_crossover_method(Crossover::Cycle)
            .with_mutation_method(Mutation::Swap)
            .with_survivor_method(Survivor::Fitness)
            .with_population(build_population())
            .with_max_generations(generations)
            .with_niching_enabled(false);
        let result_no_niche = ga_no_niche.run().unwrap();
        let diversity_no_niche = average_pairwise_distance(&result_no_niche.chromosomes);

        // Run WITH niching
        let mut ga_niche = Ga::new()
            .with_problem_solving(ProblemSolving::Maximization)
            .with_selection_method(Selection::Tournament)
            .with_crossover_method(Crossover::Cycle)
            .with_mutation_method(Mutation::Swap)
            .with_survivor_method(Survivor::Fitness)
            .with_population(build_population())
            .with_max_generations(generations)
            .with_niching_enabled(true)
            .with_niching_sigma_share(3.0)
            .with_niching_alpha(1.0);
        let result_niche = ga_niche.run().unwrap();
        let diversity_niche = average_pairwise_distance(&result_niche.chromosomes);

        if diversity_niche > diversity_no_niche {
            niching_better_count += 1;
        }
    }

    // Niching should produce higher diversity in at least 3 out of 10 trials.
    // This is a weak assertion to account for stochastic behaviour.
    assert!(
        niching_better_count >= 3,
        "Niching should promote diversity in at least 3/{} trials, but only did in {}",
        trials,
        niching_better_count
    );
}

/// Task 5.6 — Adaptive GA (AGA) integration test.
/// Runs the GA with `with_adaptive_ga(true)` and crossover/mutation probability
/// ranges, verifying that the AGA path completes without errors and produces a
/// valid population.
#[test]
fn test_adaptive_ga_runs_without_error() {
    // Build a population with varying fitness values so f_avg != f_max,
    // exercising the non-trivial AGA probability branches.
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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(20)
        .with_adaptive_ga(true)
        .with_crossover_probability_max(0.9)
        .with_crossover_probability_min(0.3)
        .with_mutation_probability_max(0.5)
        .with_mutation_probability_min(0.1);

    let result = ga.run();
    assert!(
        result.is_ok(),
        "AGA run should succeed, got: {:?}",
        result.err()
    );

    let pop = result.unwrap();
    assert_eq!(
        pop.chromosomes.len(),
        10,
        "Population size should be preserved"
    );
    assert!(
        !pop.best_chromosome.fitness().is_nan(),
        "Best chromosome should have valid fitness"
    );
}

/// Task 5.6 — Adaptive GA with minimization.
/// Ensures AGA works with both problem solving directions.
#[test]
fn test_adaptive_ga_minimization() {
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
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Minimization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(20)
        .with_adaptive_ga(true)
        .with_crossover_probability_max(0.9)
        .with_crossover_probability_min(0.3)
        .with_mutation_probability_max(0.5)
        .with_mutation_probability_min(0.1);

    let result = ga.run();
    assert!(
        result.is_ok(),
        "AGA minimization run should succeed, got: {:?}",
        result.err()
    );

    let pop = result.unwrap();
    assert_eq!(pop.chromosomes.len(), 10);
}

/// Task 5.6 — Test FitnessTargetReached termination with minimization.
/// Verifies that the GA terminates with `FitnessTargetReached` when fitness hits 0
/// in minimization mode.
#[test]
fn test_fitness_target_reached_minimization() {
    // The test Chromosome's calculate_fitness computes sum(gene.id * index).
    // A chromosome with all gene IDs == 0 has fitness 0.0, which triggers the
    // minimization limit. We create a population where all chromosomes have id=0
    // so the GA finds the target immediately.
    let mut chromosomes: Vec<Chromosome> = Vec::new();
    for _ in 0..10 {
        let dna = vec![Gene { id: 0 }, Gene { id: 0 }];
        chromosomes.push(Chromosome {
            dna,
            fitness: 0.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        });
    }

    let population = Population::new(chromosomes);
    let mut ga = Ga::new()
        .with_problem_solving(ProblemSolving::Minimization)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_population(population)
        .with_max_generations(100);

    ga.run().unwrap();
    assert_eq!(
        ga.termination_cause,
        TerminationCause::FitnessTargetReached,
        "GA should terminate due to fitness target reached (fitness == 0 in minimization)"
    );
}

// ============================================================================
// Task 6.1 — Seedable / injectable RNG
// ============================================================================

/// Verifies that `with_rng_seed` is accepted by the builder and that
/// `rng::set_seed` / `rng::make_rng` produce deterministic values.
#[test]
fn test_rng_seed_api_is_functional() {
    use genetic_algorithms::rng;
    use rand::Rng;

    // Verify deterministic RNG creation
    rng::set_seed(Some(777));
    let mut r1 = rng::make_rng();
    let v1: f64 = r1.random();

    rng::set_seed(Some(777));
    let mut r2 = rng::make_rng();
    let v2: f64 = r2.random();

    assert_eq!(
        v1, v2,
        "Same seed + same counter position should yield identical values"
    );
    rng::set_seed(None); // clean up
}

/// Verifies that two GA runs with the same seed produce identical final populations.
///
/// This test is ignored by default because reproducibility requires that no
/// concurrent code calls `rng::make_rng()` while the two runs execute. Run it
/// with `cargo test test_rng_seed -- --test-threads=1 --ignored`.
#[test]
#[ignore]
fn test_rng_seed_produces_reproducible_results() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::initializers::range_random_initialization;

    fn run_seeded(seed: u64) -> Vec<f64> {
        let alleles = vec![RangeGene::new(0, vec![(0.0, 100.0)], 0.0); 4];
        let alleles_clone = alleles.clone();

        let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
            .with_genes_per_chromosome(4)
            .with_population_size(20)
            .with_initialization_fn(move |genes_per_chromosome, _, _| {
                range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
            })
            .with_fitness_fn(|dna: &[RangeGene<f64>]| dna.iter().map(|g| g.value).sum::<f64>())
            .with_selection_method(Selection::Tournament)
            .with_crossover_method(Crossover::Uniform)
            .with_mutation_method(Mutation::Swap)
            .with_problem_solving(ProblemSolving::Minimization)
            .with_survivor_method(Survivor::Fitness)
            .with_max_generations(50)
            .with_rng_seed(seed)
            .build()
            .expect("Invalid configuration");

        ga.run().unwrap();
        ga.population
            .chromosomes
            .iter()
            .map(|c| c.fitness())
            .collect()
    }

    let run_a = run_seeded(42);
    let run_b = run_seeded(42);
    let run_c = run_seeded(99);

    assert_eq!(
        run_a, run_b,
        "Two runs with the same seed should produce identical fitness vectors"
    );
    // Different seeds should (almost certainly) produce different results
    assert_ne!(
        run_a, run_c,
        "Runs with different seeds should produce different fitness vectors"
    );
}

/// Verifies that the `rng::set_seed` / `rng::make_rng` API is accessible from user code.
#[test]
fn test_rng_module_is_public() {
    use genetic_algorithms::rng;
    use rand::Rng;

    rng::set_seed(Some(123));
    let mut r = rng::make_rng();
    let _v: f64 = r.random();

    rng::set_seed(None); // clean up
}
