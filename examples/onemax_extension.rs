/*!
# OneMax with Extension Strategies — Population Diversity Control

This example demonstrates how extension strategies prevent premature convergence
in a genetic algorithm. It solves the OneMax problem (maximize the number of `true`
bits) but uses `MassDeduplication` to automatically restore diversity when the
population becomes too homogeneous.

Features demonstrated:
- Binary chromosomes with the OneMax fitness function
- Extension strategy: `MassDeduplication` to remove duplicate individuals
- Diversity threshold monitoring
- Progress callback showing fitness and population stats
- LogObserver lifecycle hooks

Run with:
```sh
cargo run --example onemax_extension
```
*/

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::fitness::count_true;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Extension, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, ExtensionConfig, LinearChromosome, MutationConfig,
    SelectionConfig, StoppingConfig,
};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

fn main() {
    // --- Problem parameters ---
    const N_BITS: usize = 64;
    const POP_SIZE: usize = 30;
    const MAX_GENERATIONS: usize = 500;
    const FITNESS_TARGET: f64 = N_BITS as f64;

    // --- Fitness function: count the number of true bits ---
    let fitness_fn = |dna: &[Binary]| count_true(dna);

    // --- Build the GA with extension strategy ---
    let mut ga = Ga::new()
        .with_genes_per_chromosome(N_BITS)
        .with_population_size(POP_SIZE)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        // Selection: Tournament (strong selective pressure → fast convergence → more duplicates)
        .with_selection_method(Selection::Tournament)
        // Crossover: Uniform
        .with_crossover_method(Crossover::Uniform)
        // Mutation: Bit flip
        .with_mutation_method(Mutation::BitFlip)
        // Survivor: Fitness-based
        .with_survivor_method(Survivor::Fitness)
        // Extension: MassDeduplication triggers when fitness std_dev drops below 1.0
        // This removes duplicate chromosomes and regrows the population with fresh individuals.
        .with_extension_method(Extension::MassDeduplication)
        .with_extension_diversity_threshold(1.0)
        .with_extension_elite_count(2)
        // Problem: maximize fitness, stop at target
        .with_problem_solving(ProblemSolving::FixedFitness)
        .with_fitness_target(FITNESS_TARGET)
        .with_max_generations(MAX_GENERATIONS)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Failed to build GA configuration");

    println!("== OneMax with Extension Strategies ==");
    println!(
        "Chromosome: {} bits, Population: {}, Max generations: {}",
        N_BITS, POP_SIZE, MAX_GENERATIONS
    );
    println!("Extension: MassDeduplication (threshold=1.0, elite=2)");
    println!("-------------------------------------------------------");

    // --- Run the GA with a progress callback ---
    let report_interval = 25;
    let result = ga.run_with_callback(
        Some(
            |gen: &usize,
             pop: &Population<BinaryChromosome>,
             stats: &GenerationStats,
             _cause: &TerminationCause|
             -> std::ops::ControlFlow<()> {
                let unique_count = count_unique_chromosomes(&pop.chromosomes);
                println!(
                    "Gen {:4}: best={:5.1}, avg={:5.1}, std_dev={:5.2}, unique={}/{}",
                    gen,
                    pop.best_chromosome.fitness,
                    stats.avg_fitness,
                    stats.fitness_std_dev,
                    unique_count,
                    pop.chromosomes.len()
                );
                std::ops::ControlFlow::Continue(())
            },
        ),
        report_interval,
    );

    // --- Show the final result ---
    match result {
        Ok(population) => {
            let best_fitness = population.best_chromosome.fitness;
            let termination = ga.termination_cause;
            println!("-------------------------------------------------------");
            println!("Finished. Best fitness: {}/{}", best_fitness, N_BITS);
            println!("Termination: {:?}", termination);
            if (best_fitness - FITNESS_TARGET).abs() < f64::EPSILON {
                println!("SUCCESS: Found the global optimum (all bits are 1)!");
            } else {
                println!(
                    "Reached {:.0}% of optimum.",
                    (best_fitness / FITNESS_TARGET) * 100.0
                );
            }
        }
        Err(e) => {
            println!("GA failed: {:?}", e);
        }
    }
}

/// Counts the number of unique chromosomes in the population by gene values.
fn count_unique_chromosomes(chromosomes: &[BinaryChromosome]) -> usize {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for c in chromosomes {
        let key: Vec<bool> = c.dna().iter().map(|g| g.value).collect();
        seen.insert(key);
    }
    seen.len()
}
