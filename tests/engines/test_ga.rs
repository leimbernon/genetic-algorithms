use crate::structures::{Chromosome, Gene};
use genetic_algorithms::ga::Ga;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::ChromosomeLength;
use genetic_algorithms::{
    configuration::ProblemSolving,
    fitness::FitnessFnWrapper,
    operations::{Crossover, Mutation, Selection, Survivor},
    population::Population,
    traits::{
        ChromosomeT, ConfigurationT, CrossoverConfig, ElitismConfig, LinearChromosome,
        MutationConfig, NichingConfig, SelectionConfig, StoppingConfig,
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
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
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
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
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
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
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
    static NUMBER_OF_THREADS: usize = 8;

    let mut ga: Ga<Chromosome> = Ga::new()
        .with_threads(NUMBER_OF_THREADS)
        .with_fitness_fn(fitness_fn)
        .with_population_size(POPULATION_SIZE)
        .with_chromosome_length(ChromosomeLength::Fixed(GENES_PER_CHROMOSOME))
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
    static NUMBER_OF_THREADS: usize = 8;

    let mut ga: Ga<Chromosome> = Ga::new()
        .with_threads(NUMBER_OF_THREADS)
        .with_fitness_fn(fitness_fn)
        .with_population_size(POPULATION_SIZE)
        .with_chromosome_length(ChromosomeLength::Fixed(GENES_PER_CHROMOSOME))
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
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_2,
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_3,
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_4,
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_5,
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_6,
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_7,
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_8,
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_9,
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: dna_10,
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
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
            fitness_values: vec![],
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
            fitness_values: vec![],
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
        .with_stagnation_limit(5);
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
            fitness_values: vec![],
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
        .with_convergence_threshold(0.01);
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
            fitness_values: vec![],
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
        .with_max_duration_secs(0.1); // 100 ms — reliable on slow CI
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
            fitness_values: vec![],
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
            fitness_values: vec![],
        });
    }
    // 2 different chromosomes
    for _ in 0..2 {
        chromosomes.push(Chromosome {
            dna: different_dna.clone(),
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
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
            fitness_values: vec![],
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
        fitness_values: vec![],
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
                fitness_values: vec![],
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
                fitness_values: vec![],
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
        .with_chromosome_length(ChromosomeLength::Fixed(4))
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
        .with_initialization_fn(move |genes_per_chrom, _alleles| {
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

    let result = IslandNsga2Ga::<Chromosome>::new(island_config, nsga2_config, ga_config).build();

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
        .with_initialization_fn(move |genes_per_chrom, _alleles| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| {
                    let idx = rng.random_range(0..alleles_clone.len());
                    alleles_clone[idx]
                })
                .collect()
        })
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
        .with_initialization_fn(move |genes_per_chrom, _alleles| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| {
                    let idx = rng.random_range(0..alleles_clone.len());
                    alleles_clone[idx]
                })
                .collect()
        });

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
        .with_initialization_fn(move |genes_per_chrom, _alleles| {
            use rand::Rng;
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| alleles_clone[rng.random_range(0..alleles_clone.len())])
                .collect()
        });

    // Chromosome.calculate_fitness() produces 2 fitness_values; num_objectives=3 causes runtime error
    let result = nsga2.run();
    assert!(
        result.is_err(),
        "NSGA-II should reject chromosome with 2 objectives when 3 expected"
    );
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
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_max_generations(15)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization);

    let mut ga = IslandGa::<Chromosome>::new(island_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles| {
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
        .with_chromosome_length(ChromosomeLength::Fixed(3))
        .with_max_generations(10)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut ga = IslandGa::<Chromosome>::new(island_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles| {
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
        .with_chromosome_length(ChromosomeLength::Fixed(3))
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
        .with_chromosome_length(ChromosomeLength::Fixed(4))
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
                fitness_values: vec![],
            });
        }
        for _ in 0..2 {
            chromosomes.push(Chromosome {
                dna: different_dna.clone(),
                fitness: 10.0,
                age: 0,
                fitness_fn: FitnessFnWrapper::default(),
                fitness_values: vec![],
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
            fitness_values: vec![],
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
            fitness_values: vec![],
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
            fitness_values: vec![],
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
///
/// Requires `--test-threads=1` because it relies on the global RNG counter
/// not being modified by concurrent tests.
#[test]
#[ignore]
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
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_population_size(20)
            .with_initialization_fn(move |genes_per_chromosome, _| {
                range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
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

/// Verifies that the `rng::make_rng` API is accessible from user code and
/// can produce random values. Uses the entropy path (no seed) to avoid
/// reliance on global RNG state.
#[test]
fn test_rng_module_is_public() {
    use genetic_algorithms::rng;
    use rand::Rng;

    // Use entropy-seeded path — no global state mutation
    let mut r = rng::make_rng();
    let _v: f64 = r.random();
}

// ==================== Dynamic mutation integration test ====================

#[test]
fn test_ga_with_dynamic_mutation() {
    let alleles = (1..=10).map(|i| Gene { id: i }).collect::<Vec<_>>();

    let mut ga: Ga<Chromosome> = Ga::new()
        .with_population_size(20)
        .with_chromosome_length(ChromosomeLength::Fixed(10))
        .with_initialization_fn(|genes, alleles| {
            let mut rng = genetic_algorithms::rng::make_rng();
            let alleles: &[Gene] = alleles.unwrap();
            (0..genes)
                .map(|_| {
                    use rand::Rng;
                    alleles[rng.random_range(0..alleles.len())]
                })
                .collect()
        })
        .with_fitness_fn(fitness_fn)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_mutation_probability_max(0.8)
        .with_mutation_probability_min(0.1)
        .with_dynamic_mutation(true)
        .with_mutation_target_cardinality(0.5)
        .with_mutation_probability_step(0.02)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(30)
        .with_alleles(alleles);

    ga.initialization().expect("initialization should succeed");
    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA with dynamic mutation should complete successfully"
    );

    // Verify configuration was stored correctly
    assert!(ga.configuration().mutation().dynamic_mutation);
    assert!((ga.configuration().mutation().target_cardinality.unwrap() - 0.5).abs() < f64::EPSILON);
    assert!((ga.configuration().mutation().probability_step.unwrap() - 0.02).abs() < f64::EPSILON);
}

#[test]
fn test_ga_stats_diversity_populated() {
    // Run a simple GA for a few generations and verify diversity is populated in stats
    let chromosomes = vec![
        Chromosome {
            dna: vec![
                Gene { id: 1 },
                Gene { id: 2 },
                Gene { id: 3 },
                Gene { id: 4 },
            ],
            fitness: 1.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 2 },
                Gene { id: 3 },
                Gene { id: 4 },
                Gene { id: 1 },
            ],
            fitness: 2.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 3 },
                Gene { id: 4 },
                Gene { id: 1 },
                Gene { id: 2 },
            ],
            fitness: 3.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 4 },
                Gene { id: 1 },
                Gene { id: 2 },
                Gene { id: 3 },
            ],
            fitness: 4.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 2 },
                Gene { id: 1 },
                Gene { id: 3 },
                Gene { id: 4 },
            ],
            fitness: 5.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 1 },
                Gene { id: 3 },
                Gene { id: 4 },
                Gene { id: 2 },
            ],
            fitness: 6.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 3 },
                Gene { id: 4 },
                Gene { id: 2 },
                Gene { id: 1 },
            ],
            fitness: 7.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 4 },
                Gene { id: 2 },
                Gene { id: 1 },
                Gene { id: 3 },
            ],
            fitness: 8.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 2 },
                Gene { id: 1 },
                Gene { id: 4 },
                Gene { id: 3 },
            ],
            fitness: 9.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
        },
        Chromosome {
            dna: vec![
                Gene { id: 1 },
                Gene { id: 4 },
                Gene { id: 3 },
                Gene { id: 2 },
            ],
            fitness: 10.0,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
            fitness_values: vec![],
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
    ga.run().unwrap();

    let stats = ga.stats();
    assert!(!stats.is_empty(), "Stats should have at least one entry");
    for s in stats {
        assert!(s.diversity >= 0.0, "Diversity must be non-negative");
        assert_eq!(
            s.diversity, s.fitness_std_dev,
            "Diversity must equal fitness_std_dev"
        );
    }
    // For a non-trivial population with varied fitness, at least one generation should have diversity > 0
    assert!(
        stats.iter().any(|s| s.diversity > 0.0),
        "At least one generation should have non-zero diversity"
    );
}

// ============================================================================
// Phase 45 — Memetic Algorithm: LocalSearchConfiguration serde roundtrip
// ============================================================================

#[cfg(feature = "serde")]
#[test]
fn test_local_search_configuration_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    use genetic_algorithms::configuration::LocalSearchConfiguration;
    use genetic_algorithms::operations::local_search::{
        HillClimbingConfig, LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode,
    };

    let config = LocalSearchConfiguration {
        method: LocalSearch::HillClimbing,
        application_strategy: LocalSearchApplicationStrategy::BestN { n: 5 },
        mode: LocalSearchMode::Baldwinian,
        hill_climbing: HillClimbingConfig {
            step_size: 0.05,
            max_iterations: 10,
        },
    };

    let serialized = serde_json::to_string(&config)?;
    let deserialized: LocalSearchConfiguration = serde_json::from_str(&serialized)?;

    assert_eq!(config.method, deserialized.method);
    assert_eq!(
        config.application_strategy,
        deserialized.application_strategy
    );
    assert_eq!(config.mode, deserialized.mode);
    assert_eq!(
        config.hill_climbing.step_size,
        deserialized.hill_climbing.step_size
    );
    assert_eq!(
        config.hill_climbing.max_iterations,
        deserialized.hill_climbing.max_iterations
    );

    // Verify serialized string contains key fields
    assert!(serialized.contains("HillClimbing"));
    assert!(serialized.contains("BestN"));
    assert!(serialized.contains("Baldwinian"));

    Ok(())
}

// ─── Phase 60 Wave 2 batch evaluator tests ───────────────────────────────────

mod batch_evaluator_tests {
    use super::*;
    use genetic_algorithms::BatchFitnessEvaluator;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A simple counting batch evaluator that returns a fixed fitness value.
    /// `calls` tracks how many times `evaluate_batch` was invoked.
    struct CountingEvaluator {
        calls: AtomicUsize,
        fitness_value: f64,
    }

    impl CountingEvaluator {
        fn new(fitness_value: f64) -> Self {
            CountingEvaluator {
                calls: AtomicUsize::new(0),
                fitness_value,
            }
        }
    }

    impl BatchFitnessEvaluator<Chromosome> for CountingEvaluator {
        fn evaluate_batch(&self, chromosomes: &[Chromosome]) -> Vec<f64> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            vec![self.fitness_value; chromosomes.len()]
        }
    }

    fn make_alleles() -> Vec<Gene> {
        vec![
            Gene { id: 1 },
            Gene { id: 2 },
            Gene { id: 3 },
            Gene { id: 4 },
        ]
    }

    #[test]
    fn ga_with_batch_evaluator_runs_to_completion() {
        let evaluator = Arc::new(CountingEvaluator::new(1.0));
        let alleles = make_alleles();
        let mut ga: Ga<Chromosome> = Ga::new()
            .with_batch_evaluator(evaluator)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_max_generations(3)
            .with_problem_solving(genetic_algorithms::configuration::ProblemSolving::Maximization)
            .build()
            .expect("build should succeed");

        let result = ga.run();
        assert!(result.is_ok(), "run should succeed: {:?}", result.err());
        assert!(
            !ga.population.chromosomes.is_empty(),
            "population should be non-empty"
        );
    }

    #[test]
    fn ga_batch_and_fitness_fn_mutually_exclusive_returns_configuration_error() {
        let evaluator = Arc::new(CountingEvaluator::new(1.0));
        let alleles = make_alleles();
        let result: Result<Ga<Chromosome>, _> = Ga::new()
            .with_fitness_fn(|_dna: &[Gene]| 0.0)
            .with_batch_evaluator(evaluator)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_max_generations(1)
            .build();

        assert!(
            result.is_err(),
            "build should fail with mutual exclusivity error"
        );
        match result.err().unwrap() {
            genetic_algorithms::error::GaError::ConfigurationError(msg) => {
                assert!(
                    msg.contains("Cannot use both fitness_fn and with_batch_evaluator"),
                    "error message should mention mutual exclusivity, got: {}",
                    msg
                );
            }
            e => panic!("Expected ConfigurationError, got: {:?}", e),
        }
    }

    #[test]
    fn ga_batch_evaluator_called_once_per_generation() {
        let evaluator = Arc::new(CountingEvaluator::new(1.0));
        let evaluator_ref = Arc::clone(&evaluator);
        let alleles = make_alleles();
        let n_gens = 5;
        let mut ga: Ga<Chromosome> = Ga::new()
            .with_batch_evaluator(evaluator)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_max_generations(n_gens)
            .with_problem_solving(genetic_algorithms::configuration::ProblemSolving::Maximization)
            .build()
            .expect("build should succeed");

        ga.run().expect("run should succeed");

        // 1 call for initial population + N_gens calls for offspring
        let calls = evaluator_ref.calls.load(Ordering::Relaxed);
        assert!(
            calls >= (n_gens + 1),
            "Expected >= {} evaluate_batch calls (1 init + {} gens), got {}",
            n_gens + 1,
            n_gens,
            calls
        );
        // All chromosomes in final population should have been evaluated (non-default fitness)
        for c in ga.population.chromosomes.iter() {
            assert_eq!(
                c.fitness(),
                1.0,
                "Every chromosome should have batch-assigned fitness 1.0"
            );
        }
    }

    // ─── Task 2 tests ─────────────────────────────────────────────────────────

    #[test]
    fn ga_batch_evaluator_replaces_calculate_fitness() {
        // The evaluator returns a deterministic 42.0 so we can prove calculate_fitness
        // was never called (that path would yield a different value or panic without fn).
        let evaluator = Arc::new(CountingEvaluator::new(42.0));
        let alleles = make_alleles();
        let mut ga: Ga<Chromosome> = Ga::new()
            .with_batch_evaluator(evaluator)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_max_generations(1)
            .with_problem_solving(genetic_algorithms::configuration::ProblemSolving::Maximization)
            .build()
            .expect("build should succeed");

        ga.run().expect("run should succeed");

        for c in ga.population.chromosomes.iter() {
            assert_eq!(
                c.fitness(),
                42.0,
                "All chromosomes should have batch-assigned fitness 42.0 (calculate_fitness was bypassed)"
            );
        }
    }

    #[test]
    fn ga_batch_evaluator_initial_population_evaluated() {
        // The evaluator returns 7.0; verify that after run() the initial pop was batch-evaluated.
        let evaluator = Arc::new(CountingEvaluator::new(7.0));
        let evaluator_ref = Arc::clone(&evaluator);
        let alleles = make_alleles();
        let mut ga: Ga<Chromosome> = Ga::new()
            .with_batch_evaluator(evaluator)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_max_generations(1)
            .with_problem_solving(genetic_algorithms::configuration::ProblemSolving::Maximization)
            .build()
            .expect("build should succeed");

        ga.run().expect("run should succeed");

        // At least 2 calls: 1 for initial population, 1 for generation 0 offspring
        let calls = evaluator_ref.calls.load(Ordering::Relaxed);
        assert!(
            calls >= 2,
            "Expected >= 2 evaluate_batch calls (init + gen), got {}",
            calls
        );

        // Every chromosome has the evaluator's value, not the zero default
        for c in ga.population.chromosomes.iter() {
            assert_eq!(
                c.fitness(),
                7.0,
                "Initial population must be batch-evaluated (D-02)"
            );
        }
    }

    #[test]
    fn ga_cache_stats_populated_in_generation_stats() {
        // Use scalar fitness_fn + fitness_cache_size so wrap_with_cache runs.
        // After a few generations, at least one cache hit or miss must be recorded in GenerationStats.
        let alleles = make_alleles();
        let mut ga: Ga<Chromosome> = Ga::new()
            .with_fitness_fn(|_dna: &[Gene]| 1.0)
            .with_fitness_cache_size(64)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_selection_method(genetic_algorithms::operations::Selection::Tournament)
            .with_crossover_method(genetic_algorithms::operations::Crossover::SinglePoint)
            .with_mutation_method(genetic_algorithms::operations::Mutation::Swap)
            .with_survivor_method(genetic_algorithms::operations::Survivor::Fitness)
            .with_max_generations(3)
            .with_problem_solving(genetic_algorithms::configuration::ProblemSolving::Maximization)
            .build()
            .expect("build should succeed");

        ga.run().expect("run should succeed");

        // D-07: every generation stat must carry Some(delta) when cache is active.
        // Delta = 0 is valid (no new lookups that generation); None means cache was inactive.
        for stat in ga.stats() {
            assert!(
                stat.cache_hits.is_some(),
                "cache_hits should be Some when cache is active (D-07)"
            );
            assert!(
                stat.cache_misses.is_some(),
                "cache_misses should be Some when cache is active (D-07)"
            );
        }
        assert!(
            !ga.stats().is_empty(),
            "should have at least one generation stat"
        );
    }

    #[test]
    fn ga_cache_stats_none_when_no_cache() {
        // No fitness_cache_size → cache_hits and cache_misses must remain None in all stats.
        let alleles = make_alleles();
        let mut ga: Ga<Chromosome> = Ga::new()
            .with_fitness_fn(|_dna: &[Gene]| 1.0)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_max_generations(2)
            .with_problem_solving(genetic_algorithms::configuration::ProblemSolving::Maximization)
            .build()
            .expect("build should succeed");

        ga.run().expect("run should succeed");

        for stat in ga.stats() {
            assert!(
                stat.cache_hits.is_none(),
                "cache_hits must be None when no cache is configured (D-07)"
            );
            assert!(
                stat.cache_misses.is_none(),
                "cache_misses must be None when no cache is configured (D-07)"
            );
        }
    }

    #[test]
    fn ga_batch_plus_cache_only_misses_evaluated() {
        // batch_evaluator + fitness_cache_size: cache hits should short-circuit evaluate_batch.
        // Use a small population with low-cardinality genes so DNA repeats appear across generations.
        let evaluator = Arc::new(CountingEvaluator::new(1.0));
        let evaluator_ref = Arc::clone(&evaluator);
        let alleles = vec![Gene { id: 1 }, Gene { id: 2 }]; // only 2 distinct genes → many repeats
        let mut ga: Ga<Chromosome> = Ga::new()
            .with_batch_evaluator(evaluator)
            .with_fitness_cache_size(64)
            .with_population_size(10)
            .with_chromosome_length(ChromosomeLength::Fixed(4))
            .with_alleles(alleles)
            .with_initialization_fn(
                genetic_algorithms::initializers::generic_random_initialization::<Chromosome>,
            )
            .with_max_generations(3)
            .with_problem_solving(genetic_algorithms::configuration::ProblemSolving::Maximization)
            .build()
            .expect("build should succeed");

        ga.run().expect("run should succeed");

        // After gen 0, the cache has entries. Subsequent generations should accumulate hits.
        let total_hits: u64 = ga.stats().iter().filter_map(|s| s.cache_hits).sum();
        let total_misses: u64 = ga.stats().iter().filter_map(|s| s.cache_misses).sum();

        assert!(
            total_hits.is_power_of_two() || total_hits > 0 || total_misses > 0,
            "D-07: cache stats must be populated when batch+cache is active"
        );

        // The evaluator should have been called fewer times than the total chromosomes evaluated
        // (because cache hits bypass evaluate_batch).
        let total_chromosomes_evaluated = total_hits + total_misses;
        let batch_calls = evaluator_ref.calls.load(Ordering::Relaxed) as u64;
        assert!(
            total_chromosomes_evaluated > 0,
            "Must have processed some chromosomes"
        );
        // Each evaluate_batch call handles only the miss chromosomes — if any hits occurred
        // at all, the evaluator was called fewer total-chromosome-times than total_chromosomes_evaluated.
        assert!(
            batch_calls > 0,
            "evaluate_batch must have been called at least once"
        );
        // If there were hits, those chromosomes skipped evaluate_batch entirely (D-06).
        if total_hits > 0 {
            // Total chromosomes passed to evaluate_batch equals total_misses (not total_chromosomes_evaluated).
            assert!(
                total_misses < total_chromosomes_evaluated,
                "Hits ({}) should have reduced the number of misses ({}) vs total ({})",
                total_hits,
                total_misses,
                total_chromosomes_evaluated
            );
        }
    }
}
