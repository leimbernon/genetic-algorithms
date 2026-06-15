//! Demonstrates Adaptive Operator Selection (AOS) with a crossover portfolio.
//!
//! Runs a GA on a simple minimization problem (sum of gene values) using a
//! portfolio of 3 crossover operators selected dynamically via Probability Matching.
//! The GA shows the final result including best fitness and termination cause.
//!
//! Usage:
//! ```bash
//! cargo run --example aos_demo
//! ```

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, LinearChromosome, MutationConfig, SelectionConfig, StoppingConfig,
};

fn main() {
    let _ = env_logger::try_init();
    println!("=== AOS Demo: Crossover Portfolio with Probability Matching ===\n");

    // Problem: minimize sum of 8 gene values (each in [0, 100])
    let n: i32 = 8;
    let alleles = vec![RangeGene::new(0, vec![(0_i32, 100_i32)], 0)];
    let alleles_clone = alleles.clone();

    let mut ga: Ga<RangeChromosome<i32>> = Ga::new()
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(n.try_into().unwrap()))
        .with_population_size(50)
        .with_max_generations(100)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(|dna: &[RangeGene<i32>]| {
            // Simple minimization: sum of gene values (target = 0)
            dna.iter().map(|g| g.value() as f64).sum()
        })
        .with_selection_method(Selection::Tournament)
        // Configure AOS crossover portfolio with 3 operators
        // (Uniform, SinglePoint, and BlendAlpha work with Range<i32> genes
        // where all genes share the same id — avoid Cycle/PMX/Order which
        // require unique gene IDs for permutation mapping)
        .with_crossover_portfolio(vec![
            Crossover::Uniform,
            Crossover::SinglePoint,
            Crossover::BlendAlpha,
        ])
        .with_mutation_method(Mutation::Swap)
        .with_mutation_probability_max(0.2)
        // Use Probability Matching with default parameters
        .with_aos_strategy(genetic_algorithms::aos::AosStrategy::pm_default())
        .with_reward_window(50) // exploration for first 25 generations
        .with_problem_solving(ProblemSolving::Minimization)
        .with_survivor_method(Survivor::Fitness)
        .with_alleles(alleles)
        .build()
        .expect("Failed to build GA with AOS configuration");

    let population = ga.run().expect("GA run failed");

    println!("Optimization complete!");
    println!("Best fitness: {:.4}", population.best_chromosome.fitness());

    // Show best solution (first few genes)
    let dna = population.best_chromosome.dna();
    let first_few: Vec<i32> = dna.iter().take(4).map(|g| g.value()).collect();
    println!("Best chromosome (first 4 of {} genes): {:?}", dna.len(), first_few);

    println!("\n=== AOS Demo Complete ===");
    println!("The GA dynamically selected among Uniform, SinglePoint, and BlendAlpha crossover");
    println!("operators based on recent fitness improvement (Probability Matching).");
}
