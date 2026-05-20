/*!
# Feature Selection with Adaptive GA

This example demonstrates how to use the `genetic_algorithms` library to solve a Feature Selection problem:
select the optimal subset of features from a 20-feature dataset where only 4 features (indices 0-3)
are truly relevant and 16 are noise.

Features demonstrated:
- Binary chromosomes (one gene per feature: 1 = selected, 0 = not selected)
- Adaptive GA (automatically adjusts crossover/mutation probabilities each generation)
- Maximization mode
- Tournament selection
- Uniform crossover
- BitFlip mutation
- Progress callback
- LogObserver lifecycle hooks

Run with:
```sh
cargo run --example feature_selection
```
*/

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, SelectionConfig,
    StoppingConfig,
};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

fn main() {
    // --- Problem parameters ---
    const NUM_FEATURES: usize = 20;
    const POP_SIZE: usize = 80;
    const MAX_GENERATIONS: usize = 200;
    const RELEVANT: &[usize] = &[0, 1, 2, 3];

    // --- Fitness function ---
    // Reward selecting relevant features; penalize selecting irrelevant (noise) features.
    let fitness_fn = |dna: &[Binary]| -> f64 {
        let relevant_selected = RELEVANT.iter().filter(|&&i| dna[i].value).count() as f64;
        let irrelevant_selected = dna
            .iter()
            .enumerate()
            .filter(|(i, g)| !RELEVANT.contains(i) && g.value)
            .count() as f64;
        relevant_selected - 0.5 * irrelevant_selected
    };

    // --- Build the GA configuration ---
    let mut ga = Ga::new()
        // Chromosome: Binary (bool) with NUM_FEATURES genes (one per feature)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(NUM_FEATURES))
        .with_population_size(POP_SIZE)
        // Random initialization for Binary chromosomes
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        // Selection: Tournament (good for noisy fitness landscapes)
        .with_selection_method(Selection::Tournament)
        // Crossover: Uniform (genes independently inherited from either parent)
        // Probability bounds are required by adaptive GA
        .with_crossover_probability_max(0.9)
        .with_crossover_probability_min(0.5)
        .with_crossover_method(Crossover::Uniform)
        // Mutation: Bit flip (flips a random bit to introduce diversity)
        .with_mutation_method(Mutation::BitFlip)
        // Survivor selection: Fitness-based
        .with_survivor_method(Survivor::Fitness)
        // Problem solving: maximize fitness
        .with_problem_solving(ProblemSolving::Maximization)
        // Adaptive GA: auto-adjusts crossover/mutation probabilities each generation
        .with_adaptive_ga(true)
        .with_max_generations(MAX_GENERATIONS)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Failed to build GA configuration");

    println!("== Feature Selection with Adaptive GA ==");
    println!(
        "Features: {} (relevant: {:?}), Population: {}, Max generations: {}",
        NUM_FEATURES, RELEVANT, POP_SIZE, MAX_GENERATIONS
    );
    println!("Operators: Selection=Tournament, Crossover=Uniform, Mutation=BitFlip");
    println!("Adaptive GA: enabled (auto-adjusts crossover/mutation probabilities)");
    println!("-------------------------------------------------------");

    // --- Run the GA with a callback to report progress ---
    let report_interval = 25;
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
                "Finished. Best fitness: {:.2}",
                population.best_chromosome.fitness
            );
            let mask: Vec<usize> = population
                .best_chromosome
                .dna()
                .iter()
                .enumerate()
                .filter(|(_, g)| g.value)
                .map(|(i, _)| i)
                .collect();
            println!("Selected features: {:?}", mask);
            println!("Expected relevant features: {:?}", RELEVANT);
            if RELEVANT.iter().all(|r| mask.contains(r)) {
                println!("SUCCESS: All relevant features were selected!");
            } else {
                println!(
                    "Not all relevant features found. Try increasing generations or population size."
                );
            }
        }
        Err(e) => {
            println!("GA failed: {:?}", e);
        }
    }
}
