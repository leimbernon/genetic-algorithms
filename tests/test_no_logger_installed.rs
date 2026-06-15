//! Integration test asserting that `Ga::run()` never installs a logger on the
//! caller's behalf.
//!
//! This test requires the `logging` feature (uses the `log` crate directly).
#![cfg(feature = "logging")]
//!
//!
//! Strategy: The global `log` logger slot can be set exactly once per process.
//! If the library called `env_logger::Builder::try_init()` during `Ga::run()`,
//! the slot would be occupied, and our subsequent `log::set_logger` call would
//! return `Err`. If the library no longer installs a logger, `set_logger`
//! succeeds. A `PanicLogger` is used so that any stray library call to
//! `log::set_logger` causes a visible failure even if we somehow reach it first.
//!
//! Relies on the fact that integration test files are separate binaries (each
//! gets a clean global logger slot per Rust's testing model), so the slot is
//! always free at test start.

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::ChromosomeLength;

/// A logger that panics if any log record is emitted after installation.
///
/// Used to confirm the global logger slot is still free after `Ga::run()`.
/// We only install it after the run; if `set_logger` succeeds, the library
/// did not occupy the slot.
struct PanicLogger;

impl log::Log for PanicLogger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        panic!(
            "genetic_algorithms must not install or invoke a logger on the caller's behalf, \
             but got a log record: [{level}] {message}",
            level = record.level(),
            message = record.args()
        );
    }

    fn flush(&self) {}
}

static PANIC_LOGGER: PanicLogger = PanicLogger;

fn count_ones(genes: &[genetic_algorithms::genotypes::Binary]) -> f64 {
    genes.iter().filter(|g| g.value).count() as f64
}

#[test]
fn ga_does_not_install_logger() {
    // Build and run a minimal GA without any logger installed.
    // If the old auto-init code were still present, it would call
    // env_logger::Builder::from_default_env().try_init() here, occupying the
    // global logger slot.
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
    assert!(result.is_ok(), "GA run should complete without error");

    // After the run, the global logger slot must still be unoccupied.
    // set_logger returns Err(SetLoggerError) if a logger was already installed.
    // If this call succeeds, the library did NOT install a logger during run().
    log::set_logger(&PANIC_LOGGER).expect(
        "global logger slot should still be free after Ga::run() — \
         the library must not install env_logger or any other logger",
    );
    log::set_max_level(log::LevelFilter::Trace);
    // Reaching this point proves the auto-init is gone.
}
