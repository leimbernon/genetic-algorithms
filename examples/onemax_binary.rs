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

use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::fitness::count_true;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};

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
        .with_initialization_fn(|genes_per_chromosome, _, _| {
            // Each gene is randomly true/false
            (0..genes_per_chromosome)
                .map(|_| rand::random::<bool>())
                .collect()
        })
        .with_fitness_fn(fitness_fn)
        // Selection: Roulette Wheel (fitness-proportional)
        .with_selection_method(Selection::RouletteWheel)
        // Crossover: Single-point
        .with_crossover_method(Crossover::SinglePoint)
        // Mutation: Bit flip (default rate: 1/N)
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
    let result = ga.run_with_callback(
        |gen: usize,
         pop: &Vec<genetic_algorithms::chromosome::Chromosome<Binary>>,
         best: &genetic_algorithms::chromosome::Chromosome<Binary>| {
            if gen % report_interval == 0 || best.fitness() >= FITNESS_TARGET {
                let avg_fitness = pop.iter().map(|c| c.fitness()).sum::<f64>() / pop.len() as f64;
                println!(
                    "Generation {:4}: best = {:6.2}, avg = {:6.2}",
                    gen,
                    best.fitness(),
                    avg_fitness
                );
            }
        },
    );

    // --- Show the final result ---
    match result {
        Ok((final_gen, final_pop, best)) => {
            println!("-------------------------------------------------------");
            println!(
                "Finished at generation {}. Best fitness: {}",
                final_gen,
                best.fitness()
            );
            if (best.fitness() - FITNESS_TARGET).abs() < f64::EPSILON {
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
