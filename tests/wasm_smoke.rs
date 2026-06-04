//! Smoke test for Phase 34 (WASM support).
//!
//! Verifies that a standard `Ga` run with `max_duration_secs` configured
//! terminates cleanly in a small number of generations and does not panic.
//! This exercises the non-wasm32 native path of the cfg-gated time-limit
//! check; the wasm32 path is verified by `cargo check --target
//! wasm32-unknown-unknown` in CI (`.github/workflows/wasm-check.yml`).

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::ChromosomeLength;

fn count_ones(genes: &[genetic_algorithms::genotypes::Binary]) -> f64 {
    genes.iter().filter(|g| g.value).count() as f64
}

#[test]
fn ga_runs_with_max_duration_secs() {
    // Build a tiny GA: 8 chromosomes of 8 bits, 5 generations, with a
    // generous max_duration_secs so the time-limit branch is exercised but
    // generation limit terminates first.
    let mut ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(8)
        .with_chromosome_length(ChromosomeLength::Fixed(8))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(count_ones)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(5)
        .with_max_duration_secs(60.0)
        .build()
        .expect("valid configuration");

    let result = ga.run();

    // Reaching this line proves no panic from Instant::now() (cfg-gated)
    // and no panic from rayon (cfg-gated). Any clean termination is acceptable.
    // Reaching this line proves no panic from Instant::now() (cfg-gated)
    // and no panic from rayon (cfg-gated). Any clean termination is acceptable.
    assert!(result.is_ok(), "GA run should complete without error");
}
