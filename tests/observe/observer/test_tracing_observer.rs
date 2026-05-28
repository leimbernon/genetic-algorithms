#![cfg(feature = "observer-tracing")]
//! Integration tests for TracingObserver — covers TRAC-01, TRAC-02, TRAC-03.
//!
//! This file is cfg-gated on `observer-tracing`. Running `cargo test` (no features)
//! skips this file entirely, satisfying TRAC-02: default builds do not pull in
//! the tracing crate.

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::observer::{AllObserver, GaObserver};
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::{ChromosomeLength, CompositeObserver, TracingObserver};
use std::sync::Arc;

/// Build a standard 10-generation onemax GA with the given observer.
fn build_test_ga(
    observer: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync>,
) -> Ga<BinaryChromosome> {
    Ga::new()
        .with_chromosome_length(ChromosomeLength::Fixed(8))
        .with_population_size(50)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_observer(observer)
        .build()
        .unwrap()
}

/// TRAC-01: TracingObserver attaches to Ga<U> via `with_observer` and completes
/// a 10-generation run without panic or error.
#[test]
fn test_tracing_observer_attaches_and_runs() {
    let observer = Arc::new(TracingObserver::new());
    let mut ga = build_test_ga(observer);
    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA run with TracingObserver returned Err: {:?}",
        result.err()
    );
}

/// TRAC-01: TracingObserver satisfies Send + Sync — compile-time assertion.
///
/// If this test compiles, the bounds are satisfied. If TracingObserver were
/// not Send or not Sync, rustc would refuse to instantiate `assert_send_sync`.
#[test]
fn test_tracing_observer_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TracingObserver>();
}

/// TRAC-01: TracingObserver is re-exported from the crate root and can be
/// constructed via both `new()` and `Default::default()`.
#[test]
fn test_tracing_observer_crate_reexport() {
    // The import `use genetic_algorithms::TracingObserver` at file top already
    // proves the re-export compiles. Instantiate both constructors to be sure.
    let _obs = TracingObserver::new();
    let _obs2 = TracingObserver::default();
}

/// TRAC-03: LogTracer + TracingObserver coexist for 10 generations without
/// stack overflow or infinite recursion.
///
/// LogTracer routes `log` crate events into the active `tracing` subscriber.
/// TracingObserver deliberately emits zero `log::*` calls, so no feedback loop
/// can form. This test confirms that invariant holds end-to-end.
///
/// Uses `tracing::subscriber::with_default` (scoped) rather than
/// `set_global_default` to avoid poisoning the subscriber state for other tests.
#[test]
fn test_tracing_observer_with_logtracer_no_recursion() {
    // LogTracer::init() is global — ignore AlreadySet if another test ran first.
    let _ = tracing_log::LogTracer::init();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_test_writer()
        .finish();

    tracing::subscriber::with_default(subscriber, || {
        let observer = Arc::new(TracingObserver::new());
        let mut ga = build_test_ga(observer);
        let result = ga.run();
        assert!(
            result.is_ok(),
            "GA run with LogTracer + TracingObserver returned Err: {:?}",
            result.err()
        );
    });
}

/// COMP-01: TracingObserver can be placed inside CompositeObserver and run against Ga<U>.
#[test]
fn test_tracing_observer_in_composite() {
    // COMP-01: TracingObserver can be added to CompositeObserver
    let tracing_obs = Arc::new(TracingObserver::new());
    let composite = CompositeObserver::<BinaryChromosome>::new()
        .add(tracing_obs as Arc<dyn AllObserver<BinaryChromosome> + Send + Sync>);

    let composite_arc: Arc<dyn GaObserver<BinaryChromosome> + Send + Sync> = Arc::new(composite);
    let mut ga = build_test_ga(composite_arc);
    let result = ga.run();
    assert!(
        result.is_ok(),
        "GA run with TracingObserver in CompositeObserver returned Err: {:?}",
        result.err()
    );
    // If this compiles and runs, COMP-01 for TracingObserver is satisfied.
}
