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

/// Entry point for the OneMax Binary example.
fn main() {
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
        .with_genes_per_chromosome(N_BITS)
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
    let result = ga.run_with_callback(Some(
        |gen: &usize, pop: &Population<BinaryChromosome>, stats: &GenerationStats| {
            if gen % report_interval == 0 || gen == 1 {
                let best = pop.best_chromosome();
                println!(
                    "[Gen {:4}] Best fitness: {:>5.1} | Avg: {:>5.1} | Worst: {:>5.1}",
                    gen, stats.best_fitness, stats.mean_fitness, stats.worst_fitness
                );
                if let Some(best) = best {
                    println!(
                        "  Best DNA: {}",
                        best.dna()
                            .iter()
                            .map(|b| if *b { '1' } else { '0' })
                            .collect::<String>()
                    );
                }
            }
        },
    ));

    // --- Report final result ---
    println!("-------------------------------------------------------");
    match result.termination_cause {
        TerminationCause::FitnessTargetReached => {
            println!(
                "SUCCESS: Optimum found at generation {}! Fitness = {}",
                result.generations,
                result.best_chromosome.fitness()
            );
        }
        TerminationCause::MaxGenerationsReached => {
            println!(
                "FAIL: Max generations reached. Best fitness = {}",
                result.best_chromosome.fitness()
            );
        }
        _ => {
            println!(
                "Terminated: {:?}. Best fitness = {}",
                result.termination_cause,
                result.best_chromosome.fitness()
            );
        }
    }
    println!("Best chromosome DNA:");
    println!(
        "{}",
        result
            .best_chromosome
            .dna()
            .iter()
            .map(|b| if *b { '1' } else { '0' })
            .collect::<String>()
    );
    println!("-------------------------------------------------------");
    println!("OneMax Binary example complete.");
}
