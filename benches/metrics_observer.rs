//! Divan benchmark: MetricsObserver in island parallel execution (COMP-03).
//!
//! This bench is only compiled when `observer-metrics` feature is active
//! (enforced by `required-features` in Cargo.toml).
//!
//! If COMP-03 is violated (metrics calls inside par_iter closures), this will
//! panic or deadlock. Running in `--test` mode exercises correctness only.

use std::sync::Arc;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::{GaConfiguration, ProblemSolving};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::IslandGa;
use genetic_algorithms::observer::IslandGaObserver;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::MetricsObserver;

#[divan::bench]
fn bench_metrics_observer_island(bencher: divan::Bencher) {
    bencher.bench(|| {
        let observer = Arc::new(MetricsObserver::new("bench_run"));

        let island_config = IslandConfiguration::new()
            .with_num_islands(2)
            .with_migration_interval(3)
            .with_migration_count(1);

        let ga_config = GaConfiguration::new()
            .with_population_size(20)
            .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(8))
            .with_max_generations(10)
            .with_selection_method(Selection::Tournament)
            .with_crossover_method(Crossover::Uniform)
            .with_mutation_method(Mutation::BitFlip)
            .with_survivor_method(Survivor::Fitness)
            .with_problem_solving(ProblemSolving::Maximization);

        let mut island_ga = IslandGa::<BinaryChromosome>::new(island_config, ga_config)
            .with_initialization_fn(binary_random_initialization)
            .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
            .with_observer(observer as Arc<dyn IslandGaObserver<BinaryChromosome> + Send + Sync>)
            .build()
            .expect("IslandGa configuration should be valid");

        let _ = island_ga.run();
    });
}

fn main() {
    divan::main();
}
