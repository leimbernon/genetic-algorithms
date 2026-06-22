//! Integration test: a minimal GA can be built and run using only prelude imports
//! plus concrete chromosome/genotype types. Covers SC-3.

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::prelude::*;

fn count_ones(genes: &[Binary]) -> f64 {
    genes.iter().filter(|g| g.value).count() as f64
}

#[test]
fn test_prelude_minimal_ga() {
    let mut ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(4)
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(count_ones)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(1)
        .build()
        .expect("valid configuration");

    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA run should complete using prelude imports"
    );
}
