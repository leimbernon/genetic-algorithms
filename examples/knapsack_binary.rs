use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::ga;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::ConfigurationT;

const EXCESS_WEIGHT_PENALTY: f64 = 1000.0;
const MAX_WEIGHT: f64 = 67.0;

#[derive(Clone)]
struct Item{
    weight: f64,
    value: f64,
}

const ITEMS: [Item; 10] =[Item{weight: 23.0, value: 505.0}, Item{weight: 26.0, value: 352.0},
                          Item{weight: 20.0, value: 458.0}, Item{weight: 18.0, value: 220.0},
                          Item{weight: 32.0, value: 354.0}, Item{weight: 27.0, value: 414.0},
                          Item{weight: 29.0, value: 498.0}, Item{weight: 26.0, value: 545.0},
                          Item{weight: 30.0, value: 473.0}, Item{weight: 27.0, value: 543.0}];


fn fitness_fn(dna: &[BinaryGenotype])->f64{
    let mut total_weight = 0.0;
    let mut total_value = 0.0;

    for (item,gene) in ITEMS.iter().zip(dna.iter()) {
        if gene.value {
            total_weight += item.weight;
            total_value += item.value;
        }
    }

    if total_weight > MAX_WEIGHT {
        total_value -= EXCESS_WEIGHT_PENALTY * (total_weight - MAX_WEIGHT);
    }

    total_value
}

fn report(generation: &i32, population: &Population<BinaryChromosome>, termination_cause: TerminationCause){
    println!("Generation: {} - Score: {} - Termination Cause: {:?}", generation, population.best_chromosome.fitness, termination_cause);
}

fn main(){

    let mut _population = ga::Ga::new()
        .with_genes_per_chromosome(10)
        .with_population_size(100)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(1000)
        .run_with_callback(Some(report), 10);
}