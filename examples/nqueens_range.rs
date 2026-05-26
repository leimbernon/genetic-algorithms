use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

const N: usize = 8; // Size of the chessboard (N-Queens problem)

fn fitness_fn(dna: &[RangeGenotype<i32>]) -> f64 {
    let mut conflicts = 0;

    for i in 0..N {
        for j in (i + 1)..N {
            if dna[i].value == dna[j].value
                || (dna[i].value - dna[j].value).abs() == (i as i32 - j as i32).abs()
            {
                conflicts += 1;
            }
        }
    }
    conflicts as f64 // Minimize the number of conflicts
}

fn report(
    generation: &usize,
    population: &Population<RangeChromosome<i32>>,
    _stats: &GenerationStats,
    termination_cause: &TerminationCause,
) -> std::ops::ControlFlow<()> {
    println!(
        "Generation: {} - Best Score: {} - Phenotype: {} - Termination Cause: {:?}",
        generation,
        population.best_chromosome.fitness,
        population.best_chromosome.phenotype(),
        termination_cause
    );
    std::ops::ControlFlow::Continue(())
}

fn main() {
    let alleles = vec![RangeGenotype::new(0, vec![(0, N as i32 - 1)], 0)];
    let alleles_clone = alleles.clone();
    let mut ga = Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(N))
        .with_population_size(100)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Value)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(5000)
        .with_fitness_target(0.0)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Invalid GA configuration");
    let population = ga.run_with_callback(Some(report), 100).unwrap();

    println!(
        "Best chromosome for N-Queens: {}",
        population.best_chromosome.phenotype()
    );
    println!("Starting generation of random chromosome");
}
