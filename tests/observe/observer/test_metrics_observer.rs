#![cfg(feature = "observer-metrics")]
// Unit tests migrated from src/observer/metrics_observer.rs are appended below.
//! Integration tests for MetricsObserver — covers COMP-02 and COMP-03.
//!
//! This file is cfg-gated on `observer-metrics`. Running `cargo test` (no
//! features) skips this file entirely, satisfying COMP-02: default builds do
//! not pull in the metrics crate.

use std::sync::Arc;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::IslandGa;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::ChromosomeLength;
use genetic_algorithms::MetricsObserver;

// ============================================================================
// COMP-02 Test 1: MetricsObserver attaches and runs 10 generations without panic
// ============================================================================

/// COMP-02: MetricsObserver attaches to Ga<U> via `with_observer` and completes
/// a 10-generation run without panic or error. No recorder needed — noop
/// recorder is zero-cost.
#[test]
fn test_metrics_observer_attaches_and_runs() {
    let obs = Arc::new(MetricsObserver::new("test_run"));

    let mut ga = Ga::<BinaryChromosome>::new()
        .with_chromosome_length(ChromosomeLength::Fixed(8))
        .with_population_size(20)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_observer(obs as Arc<dyn GaObserver<BinaryChromosome> + Send + Sync>)
        .build()
        .unwrap();

    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA run with MetricsObserver returned Err: {:?}",
        result.err()
    );
}

// ============================================================================
// COMP-02 Test 2: MetricsObserver is Send + Sync — compile-time assertion
// ============================================================================

/// COMP-02: MetricsObserver satisfies Send + Sync.
///
/// If this test compiles, the bounds are satisfied.
#[test]
fn test_metrics_observer_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MetricsObserver>();
}

// ============================================================================
// COMP-02 Test 3: MetricsObserver::default() works and coerces to GaObserver
// ============================================================================

/// COMP-02: MetricsObserver::default() constructs successfully and coerces to
/// a GaObserver trait object.
#[test]
fn test_metrics_observer_default() {
    let obs = MetricsObserver::default();
    let _arc: Arc<dyn GaObserver<BinaryChromosome>> = Arc::new(obs);
}

// ============================================================================
// COMP-03 Test 4: MetricsObserver in island parallel execution produces no panics
// ============================================================================

/// COMP-03: MetricsObserver attaches to IslandGa<U> with 2 islands and runs
/// 10 generations without panic, data race, or error.
///
/// This is the primary COMP-03 verification: metric calls in sequential
/// per-island hooks (on_island_generation_end) produce no data races.
#[test]
fn test_metrics_observer_island_no_panic() {
    let obs = Arc::new(MetricsObserver::new("island_test"));

    let island_config = IslandConfiguration::new()
        .with_num_islands(2)
        .with_migration_interval(3)
        .with_migration_count(1);

    use genetic_algorithms::configuration::GaConfiguration;
    use genetic_algorithms::observer::IslandGaObserver;

    let ga_config = GaConfiguration::new()
        .with_population_size(10)
        .with_chromosome_length(ChromosomeLength::Fixed(8))
        .with_max_generations(10)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization);

    let mut island_ga = IslandGa::<BinaryChromosome>::new(island_config, ga_config)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_observer(obs as Arc<dyn IslandGaObserver<BinaryChromosome> + Send + Sync>)
        .build()
        .expect("IslandGa configuration should be valid");

    let result = island_ga.run();
    assert!(
        result.is_ok(),
        "IslandGa run with MetricsObserver returned Err: {:?}",
        result.err()
    );
}

// ── Unit tests migrated from src/observer/metrics_observer.rs ────────────────

#[test]
fn new_stores_run_id() {
    let obs = MetricsObserver::new("test_run");
    assert_eq!(obs.run_id(), "test_run");
}

#[test]
fn default_uses_default_run_id() {
    let obs = MetricsObserver::default();
    assert_eq!(obs.run_id(), "default");
}

#[test]
fn implements_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MetricsObserver>();
}
