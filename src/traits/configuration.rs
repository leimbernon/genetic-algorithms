use crate::configuration::{LogLevel, ProblemSolving, StoppingCriteria};
use crate::operations::{Crossover, Mutation, Selection, Survivor};

/// Configuration for parent selection.
pub trait SelectionConfig {
    fn with_number_of_couples(self, number_of_couples: usize) -> Self;
    fn with_selection_method(self, selection_method: Selection) -> Self;
}

/// Configuration for crossover operators.
pub trait CrossoverConfig {
    fn with_crossover_number_of_points(self, number_of_points: usize) -> Self;
    fn with_crossover_probability_max(self, probability_max: f64) -> Self;
    fn with_crossover_probability_min(self, probability_min: f64) -> Self;
    fn with_crossover_method(self, method: Crossover) -> Self;
    /// Sets the distribution index (eta) for SBX crossover.
    fn with_sbx_eta(self, eta: f64) -> Self;
    /// Sets the alpha parameter for BLX-α crossover.
    fn with_blend_alpha(self, alpha: f64) -> Self;
}

/// Configuration for mutation operators.
pub trait MutationConfig {
    fn with_mutation_probability_max(self, probability_max: f64) -> Self;
    fn with_mutation_probability_min(self, probability_min: f64) -> Self;
    fn with_mutation_method(self, method: Mutation) -> Self;
    /// Sets the step size for Creep mutation.
    fn with_mutation_step(self, step: f64) -> Self;
    /// Sets the sigma for Gaussian mutation.
    fn with_mutation_sigma(self, sigma: f64) -> Self;
}

/// Configuration for stopping / termination criteria.
pub trait StoppingConfig {
    fn with_max_generations(self, max_generations: usize) -> Self;
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
    fn with_elitism(self, elitism_count: usize) -> Self;
}

/// Full GA configuration supertrait.
///
/// Combines all focused sub-traits (`SelectionConfig`, `CrossoverConfig`,
/// `MutationConfig`, `StoppingConfig`, `NichingConfig`, `ElitismConfig`)
/// with general GA settings (population size, threading, logging, etc.).
pub trait ConfigurationT:
    SelectionConfig + CrossoverConfig + MutationConfig + StoppingConfig + NichingConfig + ElitismConfig
{
    fn new() -> Self;
    fn with_adaptive_ga(self, adaptive_ga: bool) -> Self;
    fn with_threads(self, number_of_threads: usize) -> Self;
    fn with_logs(self, log_level: LogLevel) -> Self;
    fn with_survivor_method(self, method: Survivor) -> Self;

    //Limit configuration
    fn with_problem_solving(self, problem_solving: ProblemSolving) -> Self;
    fn with_population_size(self, population_size: usize) -> Self;
    fn with_genes_per_chromosome(self, genes_per_chromosome: usize) -> Self;
    fn with_needs_unique_ids(self, needs_unique_ids: bool) -> Self;
    fn with_alleles_can_be_repeated(self, alleles_can_be_repeated: bool) -> Self;

    //Save progress configuration
    fn with_save_progress(self, save_progress: bool) -> Self;
    fn with_save_progress_interval(self, save_progress_interval: usize) -> Self;
    fn with_save_progress_path(self, save_progress_path: String) -> Self;

    /// Sets the RNG seed for reproducible runs.
    ///
    /// Two runs with the same seed (and thread count) produce identical results.
    fn with_rng_seed(self, seed: u64) -> Self;
}
