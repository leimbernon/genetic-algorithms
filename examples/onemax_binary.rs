/*!
# OneMax Binary Example — "Hello World" for Genetic Algorithms

This example demonstrates how to use the `genetic_algorithms` library to solve the classic OneMax problem:
maximize the number of `true` bits in a binary chromosome.

Features demonstrated:
- Binary chromosomes
- FixedFitness stopping mode
- `count_true` fitness helper
- RouletteWheel selection
- SinglePoint crossover
- BitFlip mutation
- Progress callback
- LogObserver lifecycle hooks

Run with:
```sh
cargo run --example onemax_binary
```
*/

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::fitness::count_true;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

fn main() {
    let _ = env_logger::try_init();
    // --- Problem parameters ---
    const N_BITS: usize = 100;
    const POP_SIZE: usize = 50;
    const MAX_GENERATIONS: usize = 1000;
    const FITNESS_TARGET: f64 = N_BITS as f64;

    // --- Fitness function: count the number of true bits ---
    // Uses the built-in helper for Binary chromosomes.
    let fitness_fn = |dna: &[Binary]| count_true(dna);

    // --- Build the GA configuration ---
    let mut ga = Ga::new()
        // Chromosome: Binary (bool) with N_BITS genes
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(N_BITS))
        .with_population_size(POP_SIZE)
        // Random initialization for Binary chromosomes
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        // Selection: Roulette Wheel (fitness-proportional)
        .with_selection_method(Selection::RouletteWheel)
        // Crossover: Single-point
        .with_crossover_method(Crossover::SinglePoint)
        // Mutation: Bit flip (default probability = 1.0; flips exactly one random bit per mutated child)
        .with_mutation_method(Mutation::BitFlip)
        // Survivor selection: Fitness-based
        .with_survivor_method(Survivor::Fitness)
        // Problem solving: maximize fitness, stop at target
        .with_problem_solving(ProblemSolving::FixedFitness)
        .with_fitness_target(FITNESS_TARGET)
        .with_max_generations(MAX_GENERATIONS)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Failed to build GA configuration");

    println!("== OneMax Binary Example ==");
    println!(
        "Chromosome: {} bits, Population: {}, Max generations: {}, Target fitness: {}",
        N_BITS, POP_SIZE, MAX_GENERATIONS, FITNESS_TARGET
    );
    println!("Operators: Selection=RouletteWheel, Crossover=SinglePoint, Mutation=BitFlip");
    println!("-------------------------------------------------------");

    // --- Run the GA with a callback to report progress ---
    let report_interval = 50;
    let result = ga.run_with_callback(
        Some(
            |gen: &usize,
             pop: &Population<BinaryChromosome>,
             _stats: &GenerationStats,
             _cause: &TerminationCause|
             -> std::ops::ControlFlow<()> {
                let avg_fitness =
                    pop.chromosomes.iter().map(|c| c.fitness()).sum::<f64>() / pop.size() as f64;
                println!(
                    "Generation {:4}: best = {:6.2}, avg = {:6.2}",
                    gen, pop.best_chromosome.fitness, avg_fitness
                );
                std::ops::ControlFlow::Continue(())
            },
        ),
        report_interval,
    );

    // --- Show the final result ---
    match result {
        Ok(population) => {
            println!("-------------------------------------------------------");
            println!(
                "Finished. Best fitness: {}",
                population.best_chromosome.fitness
            );
            if (population.best_chromosome.fitness - FITNESS_TARGET).abs() < f64::EPSILON {
                println!("SUCCESS: Found the global optimum (all bits are 1)!");
            } else {
                println!("Did not reach optimum. Try increasing generations or population size.");
            }
        }
        Err(e) => {
            println!("GA failed: {:?}", e);
        }
    }
}
