//! Configuration traits for the GA builder pattern.
//!
//! These traits define the fluent API used to configure a [`Ga`](crate::ga::Ga)
//! instance. Each trait groups related settings (selection, crossover, mutation,
//! stopping, niching, elitism), and [`ConfigurationT`] combines them all into a
//! single supertrait.

use crate::configuration::{LogLevel, ProblemSolving, StoppingCriteria};
use crate::operations::{Crossover, Extension, Mutation, Selection, Survivor};

/// Configuration for parent selection.
pub trait SelectionConfig {
    /// Sets how many parent pairs are formed each generation.
    fn with_number_of_couples(self, number_of_couples: usize) -> Self;
    /// Sets the selection strategy (e.g., tournament, roulette wheel).
    fn with_selection_method(self, selection_method: Selection) -> Self;
    /// Sets the niche radius for [`Selection::Clearing`] (fitness-space distance).
    ///
    /// Individuals within `niche_radius` of a niche winner are cleared from
    /// the mating pool each generation. Default is `0.1`.
    fn with_niche_radius(self, niche_radius: f64) -> Self;
}

/// Configuration for crossover operators.
pub trait CrossoverConfig {
    /// Sets the number of crossover points (for multi-point crossover).
    fn with_crossover_number_of_points(self, number_of_points: usize) -> Self;
    /// Sets the maximum crossover probability (also the static probability when adaptive GA is off).
    fn with_crossover_probability_max(self, probability_max: f64) -> Self;
    /// Sets the minimum crossover probability (used only by adaptive GA).
    fn with_crossover_probability_min(self, probability_min: f64) -> Self;
    /// Sets the crossover method (e.g., uniform, single-point, SBX).
    fn with_crossover_method(self, method: Crossover) -> Self;
    /// Sets the distribution index (eta) for SBX crossover.
    fn with_sbx_eta(self, eta: f64) -> Self;
    /// Sets the alpha parameter for BLX-α crossover.
    fn with_blend_alpha(self, alpha: f64) -> Self;
}

/// Configuration for mutation operators.
pub trait MutationConfig {
    /// Sets the maximum mutation probability (also the static probability when adaptive GA is off).
    fn with_mutation_probability_max(self, probability_max: f64) -> Self;
    /// Sets the minimum mutation probability (used only by adaptive GA).
    fn with_mutation_probability_min(self, probability_min: f64) -> Self;
    /// Sets the mutation method (e.g., swap, inversion, Gaussian).
    fn with_mutation_method(self, method: Mutation) -> Self;
    /// Sets the step size for Creep mutation.
    fn with_mutation_step(self, step: f64) -> Self;
    /// Sets the sigma for Gaussian mutation.
    fn with_mutation_sigma(self, sigma: f64) -> Self;
    /// Enables or disables dynamic mutation probability adjustment based on population cardinality.
    fn with_dynamic_mutation(self, enabled: bool) -> Self;
    /// Sets the target cardinality ratio for dynamic mutation (0.0..1.0).
    fn with_mutation_target_cardinality(self, target: f64) -> Self;
    /// Sets the probability step size for dynamic mutation adjustment.
    fn with_mutation_probability_step(self, step: f64) -> Self;
    /// Sets the F scale factor for Differential mutation (DE-style).
    /// Typical range: 0.4–1.0. Default is 0.5 when not set.
    fn with_differential_f(self, f: f64) -> Self where Self: Sized {
        let _ = f;
        self
    }
}

/// Configuration for stopping / termination criteria.
pub trait StoppingConfig {
    /// Sets the maximum number of generations before the GA stops.
    fn with_max_generations(self, max_generations: usize) -> Self;
    /// Sets the target fitness value (used with [`ProblemSolving::FixedFitness`]).
    fn with_fitness_target(self, fitness_target: f64) -> Self;
    /// Sets compound stopping criteria. These are checked in addition to
    /// max_generations and fitness_target.
    fn with_stopping_criteria(self, criteria: StoppingCriteria) -> Self;
}

/// Configuration for fitness sharing / niching.
pub trait NichingConfig {
    /// Enables or disables fitness sharing (niching).
    fn with_niching_enabled(self, enabled: bool) -> Self;
    /// Sets the sharing radius for fitness sharing.
    fn with_niching_sigma_share(self, sigma_share: f64) -> Self;
    /// Sets the alpha parameter for the sharing function shape.
    fn with_niching_alpha(self, alpha: f64) -> Self;
}

/// Configuration for elitism.
pub trait ElitismConfig {
    /// Sets the number of top individuals preserved unchanged between generations.
    fn with_elitism(self, elitism_count: usize) -> Self;
}

/// Configuration for extension strategies (population diversity control).
pub trait ExtensionConfig {
    /// Sets the extension strategy method.
    fn with_extension_method(self, method: Extension) -> Self;
    /// Sets the diversity threshold (fitness standard deviation).
    /// Extension triggers when fitness_std_dev drops below this value.
    fn with_extension_diversity_threshold(self, threshold: f64) -> Self;
    /// Sets the survival rate for MassExtinction (0.0..1.0).
    fn with_extension_survival_rate(self, rate: f64) -> Self;
    /// Sets the number of mutation rounds for MassDegeneration.
    fn with_extension_mutation_rounds(self, rounds: usize) -> Self;
    /// Sets the number of elite individuals protected from the extension event.
    fn with_extension_elite_count(self, count: usize) -> Self;
}

/// Full GA configuration supertrait.
///
/// Combines all focused sub-traits (`SelectionConfig`, `CrossoverConfig`,
/// `MutationConfig`, `StoppingConfig`, `NichingConfig`, `ElitismConfig`)
/// with general GA settings (population size, threading, logging, etc.).
pub trait ConfigurationT:
    SelectionConfig
    + CrossoverConfig
    + MutationConfig
    + StoppingConfig
    + NichingConfig
    + ElitismConfig
    + ExtensionConfig
{
    /// Creates a new instance with default configuration values.
    fn new() -> Self;
    /// Enables or disables the adaptive GA (crossover/mutation probabilities adjust dynamically).
    fn with_adaptive_ga(self, adaptive_ga: bool) -> Self;
    /// Sets the number of threads used for parallel fitness evaluation.
    fn with_threads(self, number_of_threads: usize) -> Self;
    /// Sets the logging verbosity level.
    fn with_logs(self, log_level: LogLevel) -> Self;
    /// Sets the survivor-selection strategy (fitness-based or age-based).
    fn with_survivor_method(self, method: Survivor) -> Self;

    // --- Limit configuration ---

    /// Sets the optimization direction (minimization, maximization, or fixed-fitness target).
    fn with_problem_solving(self, problem_solving: ProblemSolving) -> Self;
    /// Sets the population size (number of individuals per generation).
    fn with_population_size(self, population_size: usize) -> Self;
    /// Sets the number of genes in each chromosome.
    fn with_genes_per_chromosome(self, genes_per_chromosome: usize) -> Self;
    /// If `true`, each chromosome is assigned a unique ID during initialization.
    fn with_needs_unique_ids(self, needs_unique_ids: bool) -> Self;
    /// If `true`, the same allele value may appear more than once in a chromosome.
    fn with_alleles_can_be_repeated(self, alleles_can_be_repeated: bool) -> Self;

    // --- Save progress configuration ---

    /// Enables or disables periodic checkpoint saving (requires `serde` feature).
    fn with_save_progress(self, save_progress: bool) -> Self;
    /// Sets how often (in generations) a checkpoint is written.
    fn with_save_progress_interval(self, save_progress_interval: usize) -> Self;
    /// Sets the directory path where checkpoint files are written.
    fn with_save_progress_path(self, save_progress_path: String) -> Self;

    /// Sets the RNG seed for reproducible runs.
    ///
    /// Two runs with the same seed (and thread count) produce identical results.
    fn with_rng_seed(self, seed: u64) -> Self;
}
