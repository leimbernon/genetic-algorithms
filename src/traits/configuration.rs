use crate::configuration::{LogLevel, ProblemSolving, StoppingCriteria};
use crate::operations::{Crossover, Mutation, Selection, Survivor};

/// Configuration for parent selection.
pub trait SelectionConfig {
    fn with_number_of_couples(&mut self, number_of_couples: usize) -> &mut Self;
    fn with_selection_method(&mut self, selection_method: Selection) -> &mut Self;
}

/// Configuration for crossover operators.
pub trait CrossoverConfig {
    fn with_crossover_number_of_points(&mut self, number_of_points: usize) -> &mut Self;
    fn with_crossover_probability_max(&mut self, probability_max: f64) -> &mut Self;
    fn with_crossover_probability_min(&mut self, probability_min: f64) -> &mut Self;
    fn with_crossover_method(&mut self, method: Crossover) -> &mut Self;
    /// Sets the distribution index (eta) for SBX crossover.
    fn with_sbx_eta(&mut self, eta: f64) -> &mut Self;
    /// Sets the alpha parameter for BLX-α crossover.
    fn with_blend_alpha(&mut self, alpha: f64) -> &mut Self;
}

/// Configuration for mutation operators.
pub trait MutationConfig {
    fn with_mutation_probability_max(&mut self, probability_max: f64) -> &mut Self;
    fn with_mutation_probability_min(&mut self, probability_min: f64) -> &mut Self;
    fn with_mutation_method(&mut self, method: Mutation) -> &mut Self;
    /// Sets the step size for Creep mutation.
    fn with_mutation_step(&mut self, step: f64) -> &mut Self;
    /// Sets the sigma for Gaussian mutation.
    fn with_mutation_sigma(&mut self, sigma: f64) -> &mut Self;
}

/// Configuration for stopping / termination criteria.
pub trait StoppingConfig {
    fn with_max_generations(&mut self, max_generations: usize) -> &mut Self;
    fn with_fitness_target(&mut self, fitness_target: f64) -> &mut Self;
    /// Sets compound stopping criteria. These are checked in addition to
    /// max_generations and fitness_target.
    fn with_stopping_criteria(&mut self, criteria: StoppingCriteria) -> &mut Self;
}

/// Configuration for fitness sharing / niching.
pub trait NichingConfig {
    /// Enables or disables fitness sharing (niching).
    fn with_niching_enabled(&mut self, enabled: bool) -> &mut Self;
    /// Sets the sharing radius for fitness sharing.
    fn with_niching_sigma_share(&mut self, sigma_share: f64) -> &mut Self;
    /// Sets the alpha parameter for the sharing function shape.
    fn with_niching_alpha(&mut self, alpha: f64) -> &mut Self;
}

/// Configuration for elitism.
pub trait ElitismConfig {
    fn with_elitism(&mut self, elitism_count: usize) -> &mut Self;
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
    fn with_adaptive_ga(&mut self, adaptive_ga: bool) -> &mut Self;
    fn with_threads(&mut self, number_of_threads: usize) -> &mut Self;
    fn with_logs(&mut self, log_level: LogLevel) -> &mut Self;
    fn with_survivor_method(&mut self, method: Survivor) -> &mut Self;

    //Limit configuration
    fn with_problem_solving(&mut self, problem_solving: ProblemSolving) -> &mut Self;
    fn with_population_size(&mut self, population_size: usize) -> &mut Self;
    fn with_genes_per_chromosome(&mut self, genes_per_chromosome: usize) -> &mut Self;
    fn with_needs_unique_ids(&mut self, needs_unique_ids: bool) -> &mut Self;
    fn with_alleles_can_be_repeated(&mut self, alleles_can_be_repeated: bool) -> &mut Self;

    //Save progress configuration
    fn with_save_progress(&mut self, save_progress: bool) -> &mut Self;
    fn with_save_progress_interval(&mut self, save_progress_interval: usize) -> &mut Self;
    fn with_save_progress_path(&mut self, save_progress_path: String) -> &mut Self;
}
