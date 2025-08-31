use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::ConfigurationT;

const N: i32 = 8; // Size of the chessboard (N-Queens problem)

fn fitness_fn(dna: &[RangeGenotype<i32>]) -> f64 {
    let mut conflicts = 0;

    for i in 0..N {
        for j in (i + 1)..N {
            if dna[i as usize].value == dna[j as usize].value || (dna[i as usize].value - dna[j as usize].value).abs() == (i - j).abs() {
                conflicts += 1;
            }
        }
    }
    conflicts as f64 // Minimize the number of conflicts
}

fn report(generation: &i32, population: &Population<RangeChromosome<i32>>, termination_cause: &TerminationCause) {
    println!("Generation: {} - Best Score: {} - Phenotype: {} - Termination Cause: {:?}", generation, population.best_chromosome.fitness, population.best_chromosome.phenotype(), termination_cause);
}

fn main() {
    let alleles = vec![RangeGenotype::new(0, vec![(0, N - 1)], 0)];
    let alleles_clone = alleles.clone();
    let mut ga = Ga::new();
    let population = ga
        .with_genes_per_chromosome(N)
        .with_population_size(100)
        .with_initialization_fn(move |genes_per_chromosome, _, _| range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false)))
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5000)
        .with_fitness_target(0.0)
        .run_with_callback(Some(report), 100);

    println!("Best chromosome for N-Queens: {}", population.best_chromosome.phenotype());
    println!("Starting generation of random chromosome");
}