//! Configuration traits for the GA builder pattern.
//!
//! These traits define the fluent API used to configure a [`Ga`](crate::ga::Ga)
//! instance. Each trait groups related settings (selection, crossover, mutation,
//! stopping, niching, elitism), and [`ConfigurationT`] combines them all into a
//! single supertrait.

use crate::chromosomes::ChromosomeLength;
use crate::configuration::LocalSearchConfiguration;
use crate::configuration::ProblemSolving;
use crate::operations::{Crossover, Extension, Mutation, Selection, Survivor};

/// Configuration for parent selection.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Selection;
/// use genetic_algorithms::traits::{ConfigurationT, SelectionConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_selection_method(Selection::Tournament)
///     .with_number_of_couples(10);
/// ```
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
    /// Sets the epsilon tolerance for [`Selection::EpsilonLexicase`].
    fn with_epsilon_lexicase(self, epsilon: f64) -> Self;
}

/// Configuration for crossover operators.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Crossover;
/// use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_crossover_method(Crossover::BlendAlpha)
///     .with_blend_alpha(0.5)
///     .with_crossover_probability_max(0.9);
/// ```
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
    /// Overrides the UNDX orthogonal noise scale (σ_xi).
    /// Default: `0.35 / sqrt(n_parents - 1)`.
    fn with_undx_sigma_xi(self, value: f64) -> Self;
    /// Overrides the UNDX primary-direction noise scale (σ_eta).
    /// Default: `0.35 / sqrt(n_parents)`.
    fn with_undx_sigma_eta(self, value: f64) -> Self;
    /// Overrides the PCX directional noise scale (σ_eta). Default: `0.1`.
    fn with_pcx_sigma_eta(self, value: f64) -> Self;
    /// Overrides the PCX orthogonal noise scale (σ_zeta). Default: `0.1`.
    fn with_pcx_sigma_zeta(self, value: f64) -> Self;
}

/// Configuration for mutation operators.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::{Mutation, GaussianParams};
/// use genetic_algorithms::traits::{ConfigurationT, MutationConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: Some(0.1) }))
///     .with_mutation_probability_max(0.05);
/// ```
pub trait MutationConfig {
    /// Sets the maximum mutation probability (also the static probability when adaptive GA is off).
    fn with_mutation_probability_max(self, probability_max: f64) -> Self;
    /// Sets the minimum mutation probability (used only by adaptive GA).
    fn with_mutation_probability_min(self, probability_min: f64) -> Self;
    /// Sets the mutation method (e.g., `Mutation::Swap`, `Mutation::Gaussian(GaussianParams { sigma: Some(0.1) })`).
    ///
    /// Operator-specific parameters are now embedded directly in the variant:
    /// ```rust,no_run
    /// // no_run: API illustration — `ga` is a configured Ga instance
    /// use genetic_algorithms::operations::{Mutation, GaussianParams, CreepParams};
    /// // v3.0.0 — pass params inside the variant:
    /// // ga.with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: Some(0.05) }));
    /// // ga.with_mutation_method(Mutation::Creep(CreepParams { step: Some(0.1) }));
    /// ```
    fn with_mutation_method(self, method: Mutation) -> Self;
    /// Enables or disables dynamic mutation probability adjustment based on population cardinality.
    fn with_dynamic_mutation(self, enabled: bool) -> Self;
    /// Sets the target cardinality ratio for dynamic mutation (0.0..1.0).
    fn with_mutation_target_cardinality(self, target: f64) -> Self;
    /// Sets the probability step size for dynamic mutation adjustment.
    fn with_mutation_probability_step(self, step: f64) -> Self;
}

/// Configuration for survivor selection.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::{ConfigurationT, SurvivorConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_length_penalty(0.01); // parsimony pressure
/// ```
pub trait SurvivorConfig {
    /// Sets the parsimony pressure penalty coefficient for survivor selection.
    ///
    /// When non-zero, each chromosome's effective fitness during survivor selection
    /// is adjusted by `±(length_penalty × dna_length)` according to the
    /// `ProblemSolving` mode. The stored `fitness()` value is not mutated —
    /// only the comparison value is adjusted.
    ///
    /// Use `None` (or don't call this method) to disable parsimony pressure.
    fn with_length_penalty(self, penalty: f64) -> Self;
}

/// Configuration for stopping / termination criteria.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_max_generations(500)
///     .with_stagnation_limit(50)
///     .with_convergence_threshold(1e-6);
/// ```
pub trait StoppingConfig {
    /// Sets the maximum number of generations before the GA stops.
    fn with_max_generations(self, max_generations: usize) -> Self;
    /// Sets the target fitness value (used with [`ProblemSolving::FixedFitness`]).
    fn with_fitness_target(self, fitness_target: f64) -> Self;
    /// Stop after N consecutive generations without fitness improvement.
    fn with_stagnation_limit(self, n: usize) -> Self;
    /// Stop when the fitness standard deviation drops below `threshold`.
    fn with_convergence_threshold(self, threshold: f64) -> Self;
    /// Stop after `secs` elapsed wall-clock seconds.
    ///
    /// The field and builder are available on all targets. Only the call site in ga.rs is
    /// `#[cfg(not(target_arch = "wasm32"))]`-gated — on wasm32 the field is silently ignored
    /// (a warning is emitted at run start instead).
    fn with_max_duration_secs(self, secs: f64) -> Self;
}

/// Configuration for fitness sharing / niching.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::{ConfigurationT, NichingConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_niching_enabled(true)
///     .with_niching_sigma_share(0.1)
///     .with_niching_alpha(1.0);
/// ```
pub trait NichingConfig {
    /// Enables or disables fitness sharing (niching).
    fn with_niching_enabled(self, enabled: bool) -> Self;
    /// Sets the sharing radius for fitness sharing.
    fn with_niching_sigma_share(self, sigma_share: f64) -> Self;
    /// Sets the alpha parameter for the sharing function shape.
    fn with_niching_alpha(self, alpha: f64) -> Self;
}

/// Configuration for elitism.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::{ConfigurationT, ElitismConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_elitism(2); // preserve top 2 individuals unchanged
/// ```
pub trait ElitismConfig {
    /// Sets the number of top individuals preserved unchanged between generations.
    fn with_elitism(self, elitism_count: usize) -> Self;
}

/// Configuration for extension strategies (population diversity control).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Extension;
/// use genetic_algorithms::traits::{ConfigurationT, ExtensionConfig};
///
/// let ga: Ga<Binary> = Ga::new()
///     .with_extension_method(Extension::MassExtinction)
///     .with_extension_diversity_threshold(0.01)
///     .with_extension_survival_rate(0.2);
/// ```
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

/// Configuration for local search / memetic algorithm refinement.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::local_search::{LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode};
/// use genetic_algorithms::configuration::LocalSearchConfiguration;
/// use genetic_algorithms::traits::{ConfigurationT, LocalSearchConfig};
///
/// let ls_config = LocalSearchConfiguration {
///     method: LocalSearch::HillClimbing,
///     application_strategy: LocalSearchApplicationStrategy::AllOffspring,
///     mode: LocalSearchMode::Lamarckian,
///     ..Default::default()
/// };
/// let ga: Ga<Binary> = Ga::new().with_local_search_configuration(ls_config);
/// ```
pub trait LocalSearchConfig {
    /// Configures the local search operator and application strategy.
    ///
    /// When configured, the local search operator refines selected offspring
    /// after crossover+mutation+fitness and after repair/constraints, before
    /// population merge and survivor selection.
    fn with_local_search_configuration(self, config: LocalSearchConfiguration) -> Self;
}

/// Full GA configuration supertrait.
///
/// Combines all focused sub-traits (`SelectionConfig`, `CrossoverConfig`,
/// `MutationConfig`, `StoppingConfig`, `NichingConfig`, `ElitismConfig`)
/// with general GA settings (population size, threading, logging, etc.).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::traits::{ConfigurationT, StoppingConfig, SelectionConfig, CrossoverConfig, MutationConfig};
///
/// let _ga: Ga<Binary> = Ga::new()
///     .with_population_size(100)
///     .with_max_generations(200)
///     .with_problem_solving(ProblemSolving::Minimization)
///     .with_selection_method(Selection::Tournament)
///     .with_crossover_method(Crossover::Uniform)
///     .with_mutation_method(Mutation::Swap)
///     .with_survivor_method(Survivor::Fitness)
///     .with_rng_seed(42);
/// ```
pub trait ConfigurationT:
    SelectionConfig
    + CrossoverConfig
    + MutationConfig
    + StoppingConfig
    + NichingConfig
    + ElitismConfig
    + ExtensionConfig
    + LocalSearchConfig
    + SurvivorConfig
{
    /// Creates a new instance with default configuration values.
    fn new() -> Self;
    /// Enables or disables the adaptive GA (crossover/mutation probabilities adjust dynamically).
    fn with_adaptive_ga(self, adaptive_ga: bool) -> Self;
    /// Sets the number of threads used for parallel fitness evaluation.
    fn with_threads(self, number_of_threads: usize) -> Self;
    /// Sets the survivor-selection strategy (fitness-based or age-based).
    fn with_survivor_method(self, method: Survivor) -> Self;

    // --- Limit configuration ---

    /// Sets the optimization direction (minimization, maximization, or fixed-fitness target).
    fn with_problem_solving(self, problem_solving: ProblemSolving) -> Self;
    /// Sets the population size (number of individuals per generation).
    fn with_population_size(self, population_size: usize) -> Self;
    /// Sets the chromosome length policy.
    ///
    /// Use [`ChromosomeLength::Fixed(n)`](ChromosomeLength::Fixed) for a fixed-size
    /// chromosome with `n` genes, or
    /// [`ChromosomeLength::Variable { min, max }`](ChromosomeLength::Variable) for
    /// variable-length chromosomes (introduced in Phase 52).
    fn with_chromosome_length(self, length: ChromosomeLength) -> Self;

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

    // --- Adaptive Operator Selection (AOS) configuration ---

    /// Sets a crossover operator portfolio for AOS.
    /// When configured, AOS selects among these operators dynamically
    /// instead of using the single crossover configured via with_crossover_method().
    fn with_crossover_portfolio(self, portfolio: Vec<Crossover>) -> Self;
    /// Sets a mutation operator portfolio for AOS.
    fn with_mutation_portfolio(self, portfolio: Vec<Mutation>) -> Self;
    /// Sets the AOS strategy for portfolio selection.
    /// Default: AosStrategy::ProbabilityMatching with literature-standard parameters.
    fn with_aos_strategy(self, strategy: crate::aos::AosStrategy) -> Self;
    /// Sets the sliding window size for AOS reward history (default: 50).
    /// The exploration phase lasts window_size / 2 generations.
    fn with_reward_window(self, window: usize) -> Self;
}
