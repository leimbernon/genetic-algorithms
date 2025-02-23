#[cfg(test)]
mod structures;

use genetic_algorithms::{configuration::ProblemSolving, fitness::FitnessFnWrapper, operations::{Crossover, Mutation, Selection, Survivor}, population::Population, traits::{ChromosomeT, ConfigurationT}};
use genetic_algorithms::ga::TerminationCause;
use crate::structures::{Gene, Chromosome};
use genetic_algorithms::ga::Ga;
extern crate num_cpus;

fn fitness_fn(_dna: &[Gene]) -> f64 {
    0.0
}

#[test]
fn test_ga_start_maximize(){

    //Creates the population
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}];
    let dna_2 = vec![Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:1}];
    let dna_3 = vec![Gene{id:3}, Gene{id:4}, Gene{id:1}, Gene{id:2}];
    let dna_4 = vec![Gene{id:4}, Gene{id:1}, Gene{id:2}, Gene{id:3}];
    let dna_5 = vec![Gene{id:2}, Gene{id:1}, Gene{id:3}, Gene{id:4}];
    let dna_6 = vec![Gene{id:1}, Gene{id:3}, Gene{id:4}, Gene{id:2}];
    let dna_7 = vec![Gene{id:3}, Gene{id:4}, Gene{id:2}, Gene{id:1}];
    let dna_8 = vec![Gene{id:4}, Gene{id:2}, Gene{id:1}, Gene{id:3}];
    let dna_9 = vec![Gene{id:2}, Gene{id:1}, Gene{id:4}, Gene{id:3}];
    let dna_10 = vec![Gene{id:1}, Gene{id:4}, Gene{id:3}, Gene{id:2}];

    let chromosomes = vec![
        Chromosome{dna: dna_1, fitness: 1.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_2, fitness: 2.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_3, fitness: 3.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_4, fitness: 4.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_5, fitness: 5.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_6, fitness: 6.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_7, fitness: 7.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_8, fitness: 8.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_9, fitness: 9.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_10, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()},
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
                        .run();
    
    assert_eq!(population.chromosomes.len(), 10);
    assert_eq!(population.best_chromosome.get_fitness(), 20.0);

}

#[test]
fn test_ga_run_minimize(){

    //Creates the population
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}];
    let dna_2 = vec![Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:1}];
    let dna_3 = vec![Gene{id:3}, Gene{id:4}, Gene{id:1}, Gene{id:2}];
    let dna_4 = vec![Gene{id:4}, Gene{id:1}, Gene{id:2}, Gene{id:3}];
    let dna_5 = vec![Gene{id:2}, Gene{id:1}, Gene{id:3}, Gene{id:4}];
    let dna_6 = vec![Gene{id:1}, Gene{id:3}, Gene{id:4}, Gene{id:2}];
    let dna_7 = vec![Gene{id:3}, Gene{id:4}, Gene{id:2}, Gene{id:1}];
    let dna_8 = vec![Gene{id:4}, Gene{id:2}, Gene{id:1}, Gene{id:3}];
    let dna_9 = vec![Gene{id:2}, Gene{id:1}, Gene{id:4}, Gene{id:3}];
    let dna_10 = vec![Gene{id:1}, Gene{id:4}, Gene{id:3}, Gene{id:2}];

    let chromosomes = vec![
        Chromosome{dna: dna_1, fitness: 1.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_2, fitness: 2.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_3, fitness: 3.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_4, fitness: 4.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_5, fitness: 5.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_6, fitness: 6.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_7, fitness: 7.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_8, fitness: 8.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_9, fitness: 9.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_10, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()},
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
                    .run();
    
    assert_eq!(population.chromosomes.len(), 10);
    assert_eq!(population.best_chromosome.get_fitness(), 1.0);

}


#[test]
fn test_ga_run(){

    //Creates the population
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}];
    let dna_2 = vec![Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:1}];
    let dna_3 = vec![Gene{id:3}, Gene{id:4}, Gene{id:1}, Gene{id:2}];
    let dna_4 = vec![Gene{id:4}, Gene{id:1}, Gene{id:2}, Gene{id:3}];
    let dna_5 = vec![Gene{id:2}, Gene{id:1}, Gene{id:3}, Gene{id:4}];
    let dna_6 = vec![Gene{id:1}, Gene{id:3}, Gene{id:4}, Gene{id:2}];
    let dna_7 = vec![Gene{id:3}, Gene{id:4}, Gene{id:2}, Gene{id:1}];
    let dna_8 = vec![Gene{id:4}, Gene{id:2}, Gene{id:1}, Gene{id:3}];
    let dna_9 = vec![Gene{id:2}, Gene{id:1}, Gene{id:4}, Gene{id:3}];
    let dna_10 = vec![Gene{id:1}, Gene{id:4}, Gene{id:3}, Gene{id:2}];

    let chromosomes = vec![
        Chromosome{dna: dna_1, fitness: 1.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_2, fitness: 2.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_3, fitness: 3.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_4, fitness: 4.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_5, fitness: 5.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_6, fitness: 6.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_7, fitness: 7.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_8, fitness: 8.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_9, fitness: 9.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_10, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
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
                    .run();
    
    assert_eq!(population.chromosomes.len(), 10);
    
}

#[test]
fn test_parent_crossover_repeating_alleles(){

    //Setup the alleles and initialize the population randomly
    let binding =  vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4},
                                   Gene{id:5}, Gene{id:6}, Gene{id:7}, Gene{id:8}];
    let alleles = binding.as_slice();
    static GENES_PER_CHROMOSOME: i32 = 6;
    static POPULATION_SIZE: i32 = 100;
    static NEEDS_UNIQUE_IDS: bool = false;
    static ALLELES_CAN_BE_REPEATED: bool = true;
    static NUMBER_OF_THREADS: i32 = 8;


    let mut ga_instance = Ga::new();
    let ga: &mut Ga<Chromosome> = &mut ga_instance
                    .with_threads(NUMBER_OF_THREADS)
                    .with_fitness_fn(fitness_fn)
                    .with_population_size(POPULATION_SIZE)
                    .with_genes_per_chromosome(GENES_PER_CHROMOSOME)
                    .with_needs_unique_ids(NEEDS_UNIQUE_IDS)
                    .with_alleles_can_be_repeated(ALLELES_CAN_BE_REPEATED)
                    .with_alleles(alleles.to_vec())
                    .with_initialization_fn(genetic_algorithms::initializers::generic_random_initialization::<Chromosome>)
                    .initialization();

    //Once population has been initialized, we check for each chromosome in the population the number of genes in the dna
    for chromosome in &ga.population.chromosomes {
        assert!(chromosome.dna.len() == GENES_PER_CHROMOSOME.try_into().unwrap());
    }
}

#[test]
fn test_parent_crossover_without_repeating_alleles(){

    //Setup the alleles and initialize the population randomly
    let binding =  vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4},
                                   Gene{id:5}, Gene{id:6}, Gene{id:7}, Gene{id:8}];
    let alleles = binding.as_slice();
    static GENES_PER_CHROMOSOME: i32 = 6;
    static POPULATION_SIZE: i32 = 100;
    static NEEDS_UNIQUE_IDS: bool = false;
    static ALLELES_CAN_BE_REPEATED: bool = false;
    static NUMBER_OF_THREADS: i32 = 8;

    let mut ga_instance = Ga::new();
    let ga: &mut Ga<Chromosome> = &mut ga_instance
                    .with_threads(NUMBER_OF_THREADS)
                    .with_fitness_fn(fitness_fn)
                    .with_population_size(POPULATION_SIZE)
                    .with_genes_per_chromosome(GENES_PER_CHROMOSOME)
                    .with_needs_unique_ids(NEEDS_UNIQUE_IDS)
                    .with_alleles_can_be_repeated(ALLELES_CAN_BE_REPEATED)
                    .with_alleles(alleles.to_vec())
                    .with_initialization_fn(genetic_algorithms::initializers::generic_random_initialization_without_repetitions::<Chromosome>)
                    .initialization();

    //Once population has been initialized, we check for each chromosome we check that genes are not repeated
    for chromosome in &ga.population.chromosomes {
        let mut gene_ids = Vec::new();

        for gene in &chromosome.dna{
            if !gene_ids.is_empty(){
                assert!(!gene_ids.contains(&gene.id));
            }
            gene_ids.push(gene.id);
        }
    }
}

fn callback_function(generation_number: &i32, population: &Population<Chromosome>, termination_cause: &TerminationCause){
    assert!(*generation_number >= 7);
    assert_eq!(population.chromosomes.len(), 10);
    assert!(termination_cause == &TerminationCause::NotTerminated ||  termination_cause == &TerminationCause::GenerationLimitReached);
}

#[test]
fn test_callback_function(){
    //Creates the population
    let dna_1 = vec![Gene{id:1}, Gene{id:2}, Gene{id:3}, Gene{id:4}];
    let dna_2 = vec![Gene{id:2}, Gene{id:3}, Gene{id:4}, Gene{id:1}];
    let dna_3 = vec![Gene{id:3}, Gene{id:4}, Gene{id:1}, Gene{id:2}];
    let dna_4 = vec![Gene{id:4}, Gene{id:1}, Gene{id:2}, Gene{id:3}];
    let dna_5 = vec![Gene{id:2}, Gene{id:1}, Gene{id:3}, Gene{id:4}];
    let dna_6 = vec![Gene{id:1}, Gene{id:3}, Gene{id:4}, Gene{id:2}];
    let dna_7 = vec![Gene{id:3}, Gene{id:4}, Gene{id:2}, Gene{id:1}];
    let dna_8 = vec![Gene{id:4}, Gene{id:2}, Gene{id:1}, Gene{id:3}];
    let dna_9 = vec![Gene{id:2}, Gene{id:1}, Gene{id:4}, Gene{id:3}];
    let dna_10 = vec![Gene{id:1}, Gene{id:4}, Gene{id:3}, Gene{id:2}];

    let chromosome = vec![
        Chromosome{dna: dna_1, fitness: 1.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_2, fitness: 2.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_3, fitness: 3.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_4, fitness: 4.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_5, fitness: 5.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_6, fitness: 6.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_7, fitness: 7.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_8, fitness: 8.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
        Chromosome{dna: dna_9, fitness: 9.0, age: 0, fitness_fn: FitnessFnWrapper::default()},  
        Chromosome{dna: dna_10, fitness: 10.0, age: 0, fitness_fn: FitnessFnWrapper::default()}, 
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
        .run_with_callback(Some(callback_function), 8);

    assert_eq!(population.chromosomes.len(), 10);
}