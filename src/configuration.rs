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
//! [`SelectionConfig`]: crate::traits::SelectionConfig
//! [`CrossoverConfig`]: crate::traits::CrossoverConfig
//! [`MutationConfig`]: crate::traits::MutationConfig

use std::fmt;

use crate::chromosomes::ChromosomeLength;
use crate::extension::configuration::ExtensionConfiguration;
use crate::niching::configuration::NichingConfiguration;
use crate::operations::local_search::{
    HillClimbingConfig, LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode,
};
use crate::operations::{Crossover, Mutation, Selection, Survivor};

/// Optimization direction for the genetic algorithm.
///
/// Determines how fitness values are compared when selecting the "best"
/// individual and when checking stopping conditions.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::traits::ConfigurationT;
///
/// let _ga = Ga::<Binary>::new()
///     .with_problem_solving(ProblemSolving::Minimization);
/// ```
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

/// Configuration for the parent-selection operator.
///
/// Controls how many parent pairs are created each generation and which
/// selection strategy is used (tournament, roulette wheel, etc.).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Selection;
/// use genetic_algorithms::traits::{ConfigurationT, SelectionConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_selection_method(Selection::Tournament)
///     .with_number_of_couples(20);
/// ```
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
    /// Epsilon tolerance for [`Selection::EpsilonLexicase`].
    /// Default `0.0` = use dynamic per-case MAD as tolerance.
    /// Any value `> 0.0` is treated as a fixed epsilon threshold.
    pub epsilon: f64,
}
impl Default for SelectionConfiguration {
    fn default() -> Self {
        SelectionConfiguration {
            number_of_couples: 0,
            method: Selection::Tournament,
            boltzmann_temperature: 1.0,
            niche_radius: 0.1,
            epsilon: 0.0,
        }
    }
}

/// Configuration for the crossover (recombination) operator.
///
/// Specifies the crossover method, probability bounds (for adaptive GA),
/// and method-specific parameters like SBX eta or BLX-alpha.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Crossover;
/// use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_crossover_method(Crossover::Sbx)
///     .with_sbx_eta(20.0)
///     .with_crossover_probability_max(0.9);
/// ```
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
    /// Override for the UNDX orthogonal noise scale (σ_xi).
    /// Default (when `None`): `0.35 / sqrt(n_parents - 1)`.
    /// Only consulted when `method == Crossover::Undx`.
    pub undx_sigma_xi: Option<f64>,
    /// Override for the UNDX primary-direction noise scale (σ_eta).
    /// Default (when `None`): `0.35 / sqrt(n_parents)`.
    /// Only consulted when `method == Crossover::Undx`.
    pub undx_sigma_eta: Option<f64>,
    /// Override for the PCX directional noise scale (σ_eta).
    /// Default (when `None`): `0.1`.
    /// Only consulted when `method == Crossover::Pcx`.
    pub pcx_sigma_eta: Option<f64>,
    /// Override for the PCX orthogonal noise scale (σ_zeta).
    /// Default (when `None`): `0.1`.
    /// Only consulted when `method == Crossover::Pcx`.
    pub pcx_sigma_zeta: Option<f64>,
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
            undx_sigma_xi: None,
            undx_sigma_eta: None,
            pcx_sigma_eta: None,
            pcx_sigma_zeta: None,
        }
    }
}

/// Configuration for the mutation operator.
///
/// Specifies the mutation method and probability bounds (for adaptive GA).
/// Operator-specific parameters (step size, sigma, eta, etc.) are now carried
/// directly by the [`Mutation`] variant — see the variant documentation for defaults.
///
/// **v3.0.0 breaking change:** The fields `step`, `sigma`, `polynomial_eta`,
/// `non_uniform_b`, `differential_f`, `cauchy_scale`, `levy_alpha`,
/// `self_adaptive_tau`, `self_adaptive_tau_prime`, `sigma_min`, and `sigma_max`
/// have been removed. Embed parameters in the variant directly, e.g.:
/// `Mutation::Gaussian { sigma: Some(0.05) }`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Mutation;
/// use genetic_algorithms::traits::{ConfigurationT, MutationConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_mutation_method(Mutation::Gaussian { sigma: Some(0.05) })
///     .with_mutation_probability_max(0.1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MutationConfiguration {
    pub probability_max: Option<f64>,
    pub probability_min: Option<f64>,
    pub method: Mutation,
    /// Enable dynamic mutation probability adjustment based on population cardinality.
    /// When enabled, mutation probability is adjusted each generation: increased when
    /// diversity is low and decreased when diversity is high.
    pub dynamic_mutation: bool,
    /// Target cardinality ratio (unique fitness values / population size) in `[0.0, 1.0]`.
    /// The dynamic mutation adjusts probability toward this target.
    pub target_cardinality: Option<f64>,
    /// Step size for dynamic mutation probability adjustment each generation.
    pub probability_step: Option<f64>,
}
impl Default for MutationConfiguration {
    fn default() -> Self {
        MutationConfiguration {
            probability_max: None,
            probability_min: None,
            method: Mutation::Swap,
            dynamic_mutation: false,
            target_cardinality: None,
            probability_step: None,
        }
    }
}

/// Core limits and problem parameters for the GA.
///
/// Defines population size, chromosome length, optimization direction,
/// and generation cap. The `chromosome_length` field describes whether
/// chromosomes have a fixed or variable number of genes.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_population_size(100)
///     .with_max_generations(500)
///     .with_problem_solving(ProblemSolving::Minimization);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LimitConfiguration {
    pub problem_solving: ProblemSolving,
    pub max_generations: usize,
    pub fitness_target: Option<f64>,
    pub population_size: usize,
    pub chromosome_length: ChromosomeLength,
}
impl Default for LimitConfiguration {
    fn default() -> Self {
        LimitConfiguration {
            problem_solving: ProblemSolving::Minimization,
            max_generations: 100,
            fitness_target: None,
            population_size: 0,
            chromosome_length: ChromosomeLength::default(),
        }
    }
}

/// Checkpoint / save-progress configuration.
///
/// When enabled, the GA periodically serializes its state (population,
/// configuration, and statistics) to disk so a run can be resumed later.
/// Requires the `serde` feature.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::ConfigurationT;
///
/// let _ga = Ga::<Binary>::new()
///     .with_save_progress(true)
///     .with_save_progress_interval(100)
///     .with_save_progress_path("/tmp/ga_checkpoint".to_string());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SaveProgressConfiguration {
    pub save_progress: bool,
    pub save_progress_interval: usize,
    pub save_progress_path: String,
}

/// Configuration for local search refinement in memetic algorithms.
///
/// When `None` on GaConfiguration, no local search is performed (zero overhead).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::configuration::LocalSearchConfiguration;
/// use genetic_algorithms::operations::local_search::{LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode};
///
/// let cfg = LocalSearchConfiguration {
///     method: LocalSearch::HillClimbing,
///     application_strategy: LocalSearchApplicationStrategy::AllOffspring,
///     mode: LocalSearchMode::Lamarckian,
///     ..Default::default()
/// };
/// ```
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
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
/// use genetic_algorithms::traits::{ConfigurationT, StoppingConfig, SelectionConfig, CrossoverConfig, MutationConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_population_size(100)
///     .with_max_generations(300)
///     .with_problem_solving(ProblemSolving::Minimization)
///     .with_selection_method(Selection::Tournament)
///     .with_crossover_method(Crossover::Uniform)
///     .with_mutation_method(Mutation::Swap)
///     .with_survivor_method(Survivor::Fitness);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GaConfiguration {
    pub(crate) adaptive_ga: bool,
    pub(crate) number_of_threads: usize,
    pub(crate) limit_configuration: LimitConfiguration,
    pub(crate) selection_configuration: SelectionConfiguration,
    pub(crate) crossover_configuration: CrossoverConfiguration,
    pub(crate) mutation_configuration: MutationConfiguration,
    pub(crate) survivor: Survivor,
    pub(crate) save_progress_configuration: SaveProgressConfiguration,
    /// Number of best individuals to preserve unchanged between generations (elitism).
    /// Default is 0 (no elitism).
    pub(crate) elitism_count: usize,
    /// Stop after N generations without fitness improvement.
    /// `None` means this criterion is disabled.
    pub(crate) stagnation_generations: Option<usize>,
    /// Stop when the fitness standard deviation drops below this threshold.
    /// `None` means this criterion is disabled.
    pub(crate) convergence_threshold: Option<f64>,
    /// Stop after the specified elapsed time (in seconds).
    /// `None` means this criterion is disabled.
    /// The field itself is un-gated; only the call site in ga.rs is `#[cfg(not(target_arch = "wasm32"))]`-gated.
    pub(crate) max_duration_secs: Option<f64>,
    /// Optional niching / fitness sharing configuration.
    pub(crate) niching_configuration: Option<NichingConfiguration>,
    /// Optional extension configuration for population diversity control.
    pub(crate) extension_configuration: Option<ExtensionConfiguration>,
    /// Optional RNG seed for reproducible runs.
    ///
    /// When set, all random number generators in operators are seeded
    /// deterministically from this value. Two runs with the same seed
    /// (and the same thread count) will produce identical results.
    pub(crate) rng_seed: Option<u64>,
    /// Optional crossover operator portfolio for AOS.
    /// When `Some(Vec<Crossover>)`, AOS selects among these operators dynamically.
    /// Default: None (uses single crossover method).
    pub(crate) crossover_portfolio: Option<Vec<Crossover>>,
    /// Optional mutation operator portfolio for AOS.
    pub(crate) mutation_portfolio: Option<Vec<Mutation>>,
    /// The AOS strategy for portfolio selection.
    /// Default: AosStrategy::ProbabilityMatching.
    pub(crate) aos_strategy: crate::aos::AosStrategy,
    /// Sliding window size for AOS reward history.
    /// Default: 50. Exploration phase = window / 2 generations.
    pub(crate) aos_reward_window: usize,
    /// Optional local search configuration for memetic algorithms.
    /// When `None`, no local search is performed (zero overhead).
    pub(crate) local_search_configuration: Option<LocalSearchConfiguration>,
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
    pub(crate) length_penalty: Option<f64>,
}
impl Default for GaConfiguration {
    fn default() -> Self {
        GaConfiguration {
            adaptive_ga: false,
            number_of_threads: 1,
            survivor: Survivor::Fitness,
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
            stagnation_generations: None,
            convergence_threshold: None,
            max_duration_secs: None,
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

impl GaConfiguration {
    // --- Sub-struct read-only accessors (D-09) ---

    /// Returns the limit configuration (population size, max generations, etc.).
    pub fn limit(&self) -> &LimitConfiguration {
        &self.limit_configuration
    }
    /// Returns the selection operator configuration.
    pub fn selection(&self) -> &SelectionConfiguration {
        &self.selection_configuration
    }
    /// Returns the crossover operator configuration.
    pub fn crossover(&self) -> &CrossoverConfiguration {
        &self.crossover_configuration
    }
    /// Returns the mutation operator configuration.
    pub fn mutation(&self) -> &MutationConfiguration {
        &self.mutation_configuration
    }
    /// Returns the survivor selection method.
    pub fn survivor(&self) -> Survivor {
        self.survivor
    }
    /// Returns the save-progress configuration.
    pub fn save_progress(&self) -> &SaveProgressConfiguration {
        &self.save_progress_configuration
    }
    /// Returns the optional extension configuration.
    pub fn extension(&self) -> Option<&ExtensionConfiguration> {
        self.extension_configuration.as_ref()
    }
    /// Returns whether adaptive GA is enabled.
    pub fn adaptive_ga(&self) -> bool {
        self.adaptive_ga
    }
    /// Returns the number of threads.
    pub fn number_of_threads(&self) -> usize {
        self.number_of_threads
    }
    /// Returns the elitism count.
    pub fn elitism_count(&self) -> usize {
        self.elitism_count
    }

    // --- Flat stopping-criteria accessors (D-08) ---

    /// Returns the stagnation-limit criterion: stop after N generations without improvement.
    pub fn stagnation_generations(&self) -> Option<usize> {
        self.stagnation_generations
    }
    /// Returns the convergence-threshold criterion: stop when fitness std dev drops below this.
    pub fn convergence_threshold(&self) -> Option<f64> {
        self.convergence_threshold
    }
    /// Returns the time-limit criterion (seconds). Un-gated; usage site in ga.rs is wasm-gated.
    pub fn max_duration_secs(&self) -> Option<f64> {
        self.max_duration_secs
    }
}

mod builders;
