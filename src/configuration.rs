//! Configuration — builder-based GA configuration types.
//!
//! This module defines the configuration structs used to parameterize every
//! aspect of the genetic algorithm: problem type, operator settings, stopping
//! criteria, logging, checkpointing, and more. The central type is
//! [`GaConfiguration`], which composes all sub-configs into a validated
//! configuration object.
//!
//! Most users interact with these types through the builder methods on [`Ga`]
//! (via the [`ConfigurationT`], [`SelectionConfig`], [`CrossoverConfig`], and
//! [`MutationConfig`] traits) rather than constructing them directly.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`GaConfiguration`] | Master configuration struct for the standard GA engine |
//! | [`ProblemSolving`] | Enum: Minimization or Maximization |
//! | [`CrossoverConfig`] | Validated crossover operator parameters |
//! | [`SelectionConfig`] | Validated selection operator parameters |
//! | [`MutationConfig`] | Validated mutation operator parameters |
//! | `SurvivorConfig` | Validated survivor operator parameters |
//!
//! # When to use
//! The configuration is created automatically when you call `.build()` on an
//! engine builder. Direct construction is only needed when inspecting or
//! serializing the configuration.
//!
//! [`Ga`]: crate::ga::Ga
//! [`ConfigurationT`]: crate::traits::ConfigurationT

use std::fmt;

use crate::chromosomes::ChromosomeLength;
use crate::extension::configuration::ExtensionConfiguration;
use crate::niching::configuration::NichingConfiguration;
use crate::operations::local_search::{
    HillClimbingConfig, LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode,
};
use crate::{
    operations::{Crossover, Extension, Mutation, Selection, Survivor},
    traits::{
        ConfigurationT, CrossoverConfig, ElitismConfig, ExtensionConfig, LocalSearchConfig,
        MutationConfig, NichingConfig, SelectionConfig, StoppingConfig, SurvivorConfig,
    },
};

/// Optimization direction for the genetic algorithm.
///
/// Determines how fitness values are compared when selecting the "best"
/// individual and when checking stopping conditions.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProblemSolving {
    /// Minimize fitness — lower values are better.
    Minimization,
    /// Maximize fitness — higher values are better.
    Maximization,
    /// Target a specific fitness value (set via [`LimitConfiguration::fitness_target`]).
    FixedFitness,
}
impl fmt::Display for ProblemSolving {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProblemSolving::Minimization => write!(f, "Minimization"),
            ProblemSolving::Maximization => write!(f, "Maximization"),
            ProblemSolving::FixedFitness => write!(f, "FixedFitness"),
        }
    }
}

/// Verbosity level for the GA's internal logging (backed by the `log` crate).
///
/// Default is [`LogLevel::Off`]. Set via [`ConfigurationT::with_logs`].
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogLevel {
    /// Disable all logging output.
    Off,
    /// Log only errors.
    Error,
    /// Log warnings and above.
    Warn,
    /// Log informational messages and above.
    Info,
    /// Log debug-level messages and above.
    Debug,
    /// Log everything, including fine-grained trace messages.
    Trace,
}

/// Configuration for the parent-selection operator.
///
/// Controls how many parent pairs are created each generation and which
/// selection strategy is used (tournament, roulette wheel, etc.).
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionConfiguration {
    pub number_of_couples: usize,
    pub method: Selection,
    /// Temperature parameter for Boltzmann selection. Controls selective pressure:
    /// high values → uniform selection, low values → strong selective pressure.
    /// Only used when `method` is `Selection::Boltzmann`. Default is `1.0`.
    pub boltzmann_temperature: f64,
    /// Niche radius for Clearing selection, measured in fitness space (`|f_a - f_b|`).
    /// Within each niche (defined by the best individual in that radius), all other
    /// individuals are cleared from the selection pool. Default is `0.1`.
    /// Only used when `method` is `Selection::Clearing`.
    pub niche_radius: f64,
}
impl Default for SelectionConfiguration {
    fn default() -> Self {
        SelectionConfiguration {
            number_of_couples: 0,
            method: Selection::Tournament,
            boltzmann_temperature: 1.0,
            niche_radius: 0.1,
        }
    }
}

/// Configuration for the crossover (recombination) operator.
///
/// Specifies the crossover method, probability bounds (for adaptive GA),
/// and method-specific parameters like SBX eta or BLX-alpha.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CrossoverConfiguration {
    pub number_of_points: Option<usize>,
    pub probability_max: Option<f64>,
    pub probability_min: Option<f64>,
    pub method: Crossover,
    /// Distribution index for SBX crossover. Higher values produce children
    /// closer to parents. Typical range: 2–20. Default is 2.0.
    pub sbx_eta: Option<f64>,
    /// Alpha parameter for BLX-α crossover. Controls exploration range.
    /// Typical value: 0.5. Default is 0.5.
    pub blend_alpha: Option<f64>,
    /// Alpha parameter for Arithmetic crossover. Controls weighting between parents.
    /// α=0.5 gives uniform arithmetic crossover (midpoint). Default is 0.5.
    pub arithmetic_alpha: Option<f64>,
}
impl Default for CrossoverConfiguration {
    fn default() -> Self {
        CrossoverConfiguration {
            number_of_points: None,
            probability_max: None,
            probability_min: None,
            method: Crossover::Uniform,
            sbx_eta: None,
            blend_alpha: None,
            arithmetic_alpha: None,
        }
    }
}

/// Configuration for the mutation operator.
///
/// Specifies the mutation method, probability bounds (for adaptive GA),
/// and method-specific parameters like step size, sigma, or polynomial eta.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MutationConfiguration {
    pub probability_max: Option<f64>,
    pub probability_min: Option<f64>,
    pub method: Mutation,
    /// Step size for Creep mutation. Only used when method is `Mutation::Creep`.
    /// Default is 1.0.
    pub step: Option<f64>,
    /// Standard deviation for Gaussian mutation. Only used when method is `Mutation::Gaussian`.
    /// Default is 1.0.
    pub sigma: Option<f64>,
    /// Distribution index for Polynomial mutation. Higher values produce smaller
    /// perturbations. Typical range: 20–100. Default is 20.0.
    pub polynomial_eta: Option<f64>,
    /// Decay parameter for NonUniform mutation. Controls how fast mutation
    /// magnitude decreases over generations. Typical range: 2–5. Default is 2.0.
    pub non_uniform_b: Option<f64>,
    /// F scale factor for Differential mutation. Controls perturbation magnitude.
    /// Typical range: 0.4–1.0. Default is 0.5 when `None`.
    /// Only used when `method` is `Mutation::Differential`.
    pub differential_f: Option<f64>,
    /// Scale parameter (γ) for `Mutation::Cauchy`. Default is `1.0` when `None`.
    /// Only consulted when `method == Mutation::Cauchy`.
    pub cauchy_scale: Option<f64>,
    /// Stability index (α) for `Mutation::LevyFlight`. Valid range: (0.0, 2.0). Default is `1.5` when `None`.
    /// Only consulted when `method == Mutation::LevyFlight`.
    pub levy_alpha: Option<f64>,
    /// Enable dynamic mutation probability adjustment based on population cardinality.
    /// When enabled, mutation probability is adjusted each generation: increased when
    /// diversity is low and decreased when diversity is high.
    pub dynamic_mutation: bool,
    /// Target cardinality ratio (unique fitness values / population size) in `[0.0, 1.0]`.
    /// The dynamic mutation adjusts probability toward this target.
    pub target_cardinality: Option<f64>,
    /// Step size for dynamic mutation probability adjustment each generation.
    pub probability_step: Option<f64>,
    /// Chromosome length policy for `Mutation::Insertion` and `Mutation::Deletion`.
    ///
    /// Set to `Some(ChromosomeLength::Variable { min, max })` to enable
    /// length-changing mutations. `Mutation::Insertion` and `Mutation::Deletion`
    /// return `GaError::MutationError` when this is `None` or `Fixed`.
    pub chromosome_length: Option<ChromosomeLength>,
}
impl Default for MutationConfiguration {
    fn default() -> Self {
        MutationConfiguration {
            probability_max: None,
            probability_min: None,
            method: Mutation::Swap,
            step: None,
            sigma: None,
            polynomial_eta: None,
            non_uniform_b: None,
            differential_f: None,
            cauchy_scale: None,
            levy_alpha: None,
            dynamic_mutation: false,
            target_cardinality: None,
            probability_step: None,
            chromosome_length: None,
        }
    }
}

/// Core limits and problem parameters for the GA.
///
/// Defines population size, chromosome length, optimization direction,
/// generation cap, and whether alleles can repeat or require unique IDs.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LimitConfiguration {
    pub problem_solving: ProblemSolving,
    pub max_generations: usize,
    pub fitness_target: Option<f64>,
    pub population_size: usize,
    pub genes_per_chromosome: usize,
    pub needs_unique_ids: bool,
    pub alleles_can_be_repeated: bool,
}
impl Default for LimitConfiguration {
    fn default() -> Self {
        LimitConfiguration {
            problem_solving: ProblemSolving::Minimization,
            max_generations: 100,
            fitness_target: None,
            population_size: 0,
            genes_per_chromosome: 0,
            needs_unique_ids: false,
            alleles_can_be_repeated: false,
        }
    }
}

/// Checkpoint / save-progress configuration.
///
/// When enabled, the GA periodically serializes its state (population,
/// configuration, and statistics) to disk so a run can be resumed later.
/// Requires the `serde` feature.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SaveProgressConfiguration {
    pub save_progress: bool,
    pub save_progress_interval: usize,
    pub save_progress_path: String,
}

/// Compound stopping criteria for the GA.
///
/// Multiple criteria can be enabled simultaneously. The GA stops when **any** of them is met.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StoppingCriteria {
    /// Stop after N generations without fitness improvement.
    /// `None` means this criterion is disabled.
    pub stagnation_generations: Option<usize>,
    /// Stop when the fitness standard deviation drops below this threshold.
    /// `None` means this criterion is disabled.
    pub convergence_threshold: Option<f64>,
    /// Stop after the specified elapsed time (in seconds).
    /// `None` means this criterion is disabled.
    pub max_duration_secs: Option<f64>,
}

/// Configuration for local search refinement in memetic algorithms.
///
/// When `None` on GaConfiguration, no local search is performed (zero overhead).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocalSearchConfiguration {
    /// The local search operator variant (e.g., HillClimbing).
    pub method: LocalSearch,
    /// Which offspring receive local search refinement.
    pub application_strategy: LocalSearchApplicationStrategy,
    /// Lamarckian (update DNA+fitness) or Baldwinian (fitness only).
    pub mode: LocalSearchMode,
    /// HillClimbing-specific configuration parameters.
    pub hill_climbing: HillClimbingConfig,
}

impl Default for LocalSearchConfiguration {
    fn default() -> Self {
        Self {
            method: LocalSearch::HillClimbing,
            application_strategy: LocalSearchApplicationStrategy::AllOffspring,
            mode: LocalSearchMode::Lamarckian,
            hill_climbing: HillClimbingConfig::default(),
        }
    }
}

/// Top-level configuration for a [`Ga`](crate::ga::Ga) run.
///
/// Aggregates all sub-configurations (selection, crossover, mutation,
/// limits, stopping criteria, niching, checkpointing) into a single struct
/// that is stored inside [`Ga`](crate::ga::Ga).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GaConfiguration {
    pub adaptive_ga: bool,
    pub number_of_threads: usize,
    pub limit_configuration: LimitConfiguration,
    pub selection_configuration: SelectionConfiguration,
    pub crossover_configuration: CrossoverConfiguration,
    pub mutation_configuration: MutationConfiguration,
    pub survivor: Survivor,
    pub log_level: LogLevel,
    pub save_progress_configuration: SaveProgressConfiguration,
    /// Number of best individuals to preserve unchanged between generations (elitism).
    /// Default is 0 (no elitism).
    pub elitism_count: usize,
    /// Compound stopping criteria. These are checked in addition to
    /// max_generations and fitness_target.
    pub stopping_criteria: StoppingCriteria,
    /// Optional niching / fitness sharing configuration.
    pub niching_configuration: Option<NichingConfiguration>,
    /// Optional extension configuration for population diversity control.
    pub extension_configuration: Option<ExtensionConfiguration>,
    /// Optional RNG seed for reproducible runs.
    ///
    /// When set, all random number generators in operators are seeded
    /// deterministically from this value. Two runs with the same seed
    /// (and the same thread count) will produce identical results.
    pub rng_seed: Option<u64>,
    /// Optional crossover operator portfolio for AOS.
    /// When `Some(Vec<Crossover>)`, AOS selects among these operators dynamically.
    /// Default: None (uses single crossover method).
    pub crossover_portfolio: Option<Vec<Crossover>>,
    /// Optional mutation operator portfolio for AOS.
    pub mutation_portfolio: Option<Vec<Mutation>>,
    /// The AOS strategy for portfolio selection.
    /// Default: AosStrategy::ProbabilityMatching.
    pub aos_strategy: crate::aos::AosStrategy,
    /// Sliding window size for AOS reward history.
    /// Default: 50. Exploration phase = window / 2 generations.
    pub aos_reward_window: usize,
    /// Optional local search configuration for memetic algorithms.
    /// When `None`, no local search is performed (zero overhead).
    pub local_search_configuration: Option<LocalSearchConfiguration>,
    /// Parsimony pressure penalty coefficient.
    ///
    /// When set, each chromosome's effective fitness during survivor selection is adjusted
    /// by `±(length_penalty × chromosome_dna_length)`. The stored `fitness()` value is
    /// **never** mutated — only the comparison value is adjusted.
    ///
    /// Sign convention (auto-adjusted per `ProblemSolving` mode):
    /// - **Maximization** — adjusted = fitness - (length_penalty × length)
    ///   (longer chromosomes appear worse)
    /// - **Minimization** — adjusted = fitness + (length_penalty × length)
    ///   (longer chromosomes appear worse)
    ///
    /// Set to `None` (default) to disable parsimony pressure.
    pub length_penalty: Option<f64>,
}
impl Default for GaConfiguration {
    fn default() -> Self {
        GaConfiguration {
            adaptive_ga: false,
            number_of_threads: 1,
            survivor: Survivor::Fitness,
            log_level: LogLevel::Off,
            limit_configuration: LimitConfiguration {
                ..Default::default()
            },
            selection_configuration: SelectionConfiguration {
                ..Default::default()
            },
            crossover_configuration: CrossoverConfiguration {
                ..Default::default()
            },
            mutation_configuration: MutationConfiguration {
                ..Default::default()
            },
            save_progress_configuration: SaveProgressConfiguration {
                ..Default::default()
            },
            elitism_count: 0,
            stopping_criteria: StoppingCriteria::default(),
            niching_configuration: None,
            extension_configuration: None,
            rng_seed: None,
            crossover_portfolio: None,
            mutation_portfolio: None,
            aos_strategy: crate::aos::AosStrategy::pm_default(),
            aos_reward_window: 50,
            local_search_configuration: None,
            length_penalty: None,
        }
    }
}

impl SelectionConfig for GaConfiguration {
    fn with_number_of_couples(mut self, number_of_couples: usize) -> Self {
        self.selection_configuration.number_of_couples = number_of_couples;
        self
    }
    fn with_selection_method(mut self, selection_method: Selection) -> Self {
        self.selection_configuration.method = selection_method;
        self
    }
    fn with_niche_radius(mut self, niche_radius: f64) -> Self {
        self.selection_configuration.niche_radius = niche_radius;
        self
    }
}

impl CrossoverConfig for GaConfiguration {
    fn with_crossover_number_of_points(mut self, number_of_points: usize) -> Self {
        self.crossover_configuration.number_of_points = Some(number_of_points);
        self
    }
    fn with_crossover_probability_max(mut self, probability_max: f64) -> Self {
        self.crossover_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_crossover_probability_min(mut self, probability_min: f64) -> Self {
        self.crossover_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_crossover_method(mut self, method: Crossover) -> Self {
        self.crossover_configuration.method = method;
        self
    }
    fn with_sbx_eta(mut self, eta: f64) -> Self {
        self.crossover_configuration.sbx_eta = Some(eta);
        self
    }
    fn with_blend_alpha(mut self, alpha: f64) -> Self {
        self.crossover_configuration.blend_alpha = Some(alpha);
        self
    }
}

impl MutationConfig for GaConfiguration {
    fn with_mutation_probability_max(mut self, probability_max: f64) -> Self {
        self.mutation_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_mutation_probability_min(mut self, probability_min: f64) -> Self {
        self.mutation_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_mutation_method(mut self, method: Mutation) -> Self {
        self.mutation_configuration.method = method;
        self
    }
    fn with_mutation_step(mut self, step: f64) -> Self {
        self.mutation_configuration.step = Some(step);
        self
    }
    fn with_mutation_sigma(mut self, sigma: f64) -> Self {
        self.mutation_configuration.sigma = Some(sigma);
        self
    }
    fn with_dynamic_mutation(mut self, enabled: bool) -> Self {
        self.mutation_configuration.dynamic_mutation = enabled;
        self
    }
    fn with_mutation_target_cardinality(mut self, target: f64) -> Self {
        self.mutation_configuration.target_cardinality = Some(target);
        self
    }
    fn with_mutation_probability_step(mut self, step: f64) -> Self {
        self.mutation_configuration.probability_step = Some(step);
        self
    }
    fn with_differential_f(mut self, f: f64) -> Self {
        self.mutation_configuration.differential_f = Some(f);
        self
    }
    fn with_polynomial_eta(mut self, eta: f64) -> Self {
        self.mutation_configuration.polynomial_eta = Some(eta);
        self
    }
    fn with_cauchy_scale(mut self, scale: f64) -> Self {
        // A scale of 0 or negative makes the perturbation a no-op or invalid.
        debug_assert!(scale > 0.0, "cauchy_scale must be positive; got {}", scale);
        self.mutation_configuration.cauchy_scale = Some(scale);
        self
    }
    fn with_levy_alpha(mut self, alpha: f64) -> Self {
        debug_assert!(
            alpha > 0.0 && alpha < 2.0,
            "levy_alpha must be in (0.0, 2.0); got {}. Values outside this range are clamped.",
            alpha
        );
        self.mutation_configuration.levy_alpha = Some(alpha);
        self
    }
    fn with_chromosome_length(mut self, chromosome_length: ChromosomeLength) -> Self {
        self.mutation_configuration.chromosome_length = Some(chromosome_length);
        self
    }
}

impl StoppingConfig for GaConfiguration {
    fn with_max_generations(mut self, max_generations: usize) -> Self {
        self.limit_configuration.max_generations = max_generations;
        self
    }
    fn with_fitness_target(mut self, fitness_target: f64) -> Self {
        self.limit_configuration.fitness_target = Some(fitness_target);
        self
    }
    fn with_stopping_criteria(mut self, criteria: StoppingCriteria) -> Self {
        self.stopping_criteria = criteria;
        self
    }
}

impl NichingConfig for GaConfiguration {
    fn with_niching_enabled(mut self, enabled: bool) -> Self {
        self.niching_configuration
            .get_or_insert_with(NichingConfiguration::default)
            .enabled = enabled;
        self
    }
    fn with_niching_sigma_share(mut self, sigma_share: f64) -> Self {
        self.niching_configuration
            .get_or_insert_with(NichingConfiguration::default)
            .sigma_share = sigma_share;
        self
    }
    fn with_niching_alpha(mut self, alpha: f64) -> Self {
        self.niching_configuration
            .get_or_insert_with(NichingConfiguration::default)
            .alpha = alpha;
        self
    }
}

impl ElitismConfig for GaConfiguration {
    fn with_elitism(mut self, elitism_count: usize) -> Self {
        self.elitism_count = elitism_count;
        self
    }
}

impl SurvivorConfig for GaConfiguration {
    fn with_length_penalty(mut self, penalty: f64) -> Self {
        self.length_penalty = Some(penalty);
        self
    }
}

impl ExtensionConfig for GaConfiguration {
    fn with_extension_method(mut self, method: Extension) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .method = method;
        self
    }
    fn with_extension_diversity_threshold(mut self, threshold: f64) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .diversity_threshold = threshold;
        self
    }
    fn with_extension_survival_rate(mut self, rate: f64) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .survival_rate = rate;
        self
    }
    fn with_extension_mutation_rounds(mut self, rounds: usize) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .mutation_rounds = rounds;
        self
    }
    fn with_extension_elite_count(mut self, count: usize) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .elite_count = count;
        self
    }
}

impl LocalSearchConfig for GaConfiguration {
    fn with_local_search_configuration(mut self, config: LocalSearchConfiguration) -> Self {
        self.local_search_configuration = Some(config);
        self
    }
}

impl ConfigurationT for GaConfiguration {
    fn new() -> Self {
        Self::default()
    }
    fn with_adaptive_ga(mut self, adaptive_ga: bool) -> Self {
        self.adaptive_ga = adaptive_ga;
        self
    }
    fn with_threads(mut self, number_of_threads: usize) -> Self {
        self.number_of_threads = number_of_threads;
        self
    }
    fn with_logs(mut self, log_level: LogLevel) -> Self {
        self.log_level = log_level;
        self
    }
    fn with_survivor_method(mut self, method: Survivor) -> Self {
        self.survivor = method;
        self
    }

    //Limit configuration
    fn with_problem_solving(mut self, problem_solving: ProblemSolving) -> Self {
        self.limit_configuration.problem_solving = problem_solving;
        self
    }
    fn with_population_size(mut self, population_size: usize) -> Self {
        self.limit_configuration.population_size = population_size;
        self
    }
    fn with_genes_per_chromosome(mut self, genes_per_chromosome: usize) -> Self {
        self.limit_configuration.genes_per_chromosome = genes_per_chromosome;
        self
    }
    fn with_needs_unique_ids(mut self, needs_unique_ids: bool) -> Self {
        self.limit_configuration.needs_unique_ids = needs_unique_ids;
        self
    }
    fn with_alleles_can_be_repeated(mut self, alleles_can_be_repeated: bool) -> Self {
        self.limit_configuration.alleles_can_be_repeated = alleles_can_be_repeated;
        self
    }

    //Save progress configuration
    fn with_save_progress(mut self, save_progress: bool) -> Self {
        self.save_progress_configuration.save_progress = save_progress;
        self
    }
    fn with_save_progress_interval(mut self, save_progress_interval: usize) -> Self {
        self.save_progress_configuration.save_progress_interval = save_progress_interval;
        self
    }
    fn with_save_progress_path(mut self, save_progress_path: String) -> Self {
        self.save_progress_configuration.save_progress_path = save_progress_path;
        self
    }

    fn with_rng_seed(mut self, seed: u64) -> Self {
        self.rng_seed = Some(seed);
        self
    }

    fn with_crossover_portfolio(mut self, portfolio: Vec<Crossover>) -> Self {
        self.crossover_portfolio = Some(portfolio);
        self
    }
    fn with_mutation_portfolio(mut self, portfolio: Vec<Mutation>) -> Self {
        self.mutation_portfolio = Some(portfolio);
        self
    }
    fn with_aos_strategy(mut self, strategy: crate::aos::AosStrategy) -> Self {
        self.aos_strategy = strategy;
        self
    }
    fn with_reward_window(mut self, window: usize) -> Self {
        self.aos_reward_window = window;
        self
    }
}
