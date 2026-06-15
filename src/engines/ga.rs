//! # Standard Genetic Algorithm (Ga)
//!
//! ## Description
//!
//! The [`Ga`] struct is the primary single-population genetic algorithm orchestrator.
//! It implements the classic evolutionary cycle: initialization -> selection -> crossover
//! + mutation (parallelized via rayon) -> fitness evaluation -> survivor selection ->
//!   elitism -> statistics collection. Each generation, parents are selected via a
//!   configurable [`Selection`](crate::operations::Selection) operator, offspring are
//!   produced through [`Crossover`] and
//!   [`Mutation`], and the population is updated via a
//!   [`Survivor`](crate::operations::Survivor) strategy.
//!
//! Optionally supports:
//! - **Elitism** — preserve the N best individuals unchanged between generations
//! - **Niching / Fitness sharing** — maintain population diversity
//! - **Extension strategies** — re-introduce diversity when the population converges
//! - **Adaptive GA** — dynamically tune crossover/mutation rates based on diversity
//! - **Adaptive Operator Selection (AOS)** — choose from an operator portfolio dynamically
//! - **Memetic local search** — refine the best individuals each generation
//! - **Constraint handling** — penalty-based or repair-based constraint satisfaction
//! - **Observers** — lifecycle hooks for logging, tracing, metrics, and custom monitoring
//!
//! ## When to Use
//!
//! - **Problem type:** Single-objective — continuous, combinatorial, binary, or permutation
//! - **Number of objectives:** 1
//! - **Variable type:** Any (binary via [`chromosomes::Binary`](crate::chromosomes::Binary),
//!   real-valued via [`chromosomes::Range`](crate::chromosomes::Range), symbolic via
//!   [`chromosomes::ListChromosome`](crate::chromosomes::ListChromosome))
//! - **Key strength:** General-purpose; broadest operator library; best for learning
//! - **Key weakness:** Single-population convergence can be premature on multimodal landscapes
//!
//! ## Quick Reference
//!
//! ### Mandatory Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `population_size` | `usize` | Yes (via builder) | — | Number of chromosomes in the population |
//! | `max_generations` | `usize` | Yes (via builder) | — | Maximum number of generations |
//! | `genes_per_chromosome` | `usize` | Yes (via builder) | — | Length of each chromosome's DNA |
//! | `fitness_fn` | `FitnessFn<U>` | Yes (via builder) | — | Function evaluating a chromosome's fitness |
//!
//! ### Optional Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `selection` | `Selection` | No | `Tournament(3)` | Parent selection strategy |
//! | `crossover` | `Crossover` | No | `Uniform` | Offspring crossover strategy |
//! | `mutation` | `Mutation` | No | `Swap(0.05)` | Gene mutation strategy |
//! | `survivor` | `Survivor` | No | `Truncation` | Population replacement strategy |
//! | `elitism_count` | `usize` | No | `0` | Number of elite chromosomes preserved |
//! | `initialization_fn` | `InitializationFn<U>` | No | random init | Custom population initialization |
//! | `observer` | `Option<Arc<dyn GaObserver<U>>>` | No | `None` | Lifecycle observer |
//! | `max_duration` | `Option<Duration>` | No | `None` | Wall-clock time limit |
//! | `fitness_target` | `Option<f64>` | No | `None` | Stop when best fitness reaches this |
//! | `constraint_handling` | `Option<ConstraintHandling>` | No | `None` | Constraint handling strategy |
//! | `hall_of_fame_size` | `Option<usize>` | No | `None` | Archive size for best solutions |
//! | `aos_strategy` | `AosStrategy` | No | `ProbabilityMatching` | Adaptive operator selection |
//! | `local_search` | `Option<Arc<dyn LocalSearchOperator<U>>>` | No | `None` | Memetic local search operator |
//! | `adaptive_ga` | `bool` | No | `false` | Dynamic crossover/mutation tuning |
//! | `niching` | `Option<NichingConfiguration>` | No | `None` | Fitness sharing / diversity preservation |
//! | `extension` | `Option<ExtensionConfiguration>` | No | `None` | Diversity re-introduction strategy |
//! | `rng_seed` | `Option<u64>` | No | `None` | Reproducible RNG seed |
//!
//! ## Complete Example
//!
//! ```rust,ignore
//! use genetic_algorithms::chromosomes::Range as RangeChromosome;
//! use genetic_algorithms::ga::Ga;
//! use genetic_algorithms::genotypes::Range as RangeGenotype;
//! use genetic_algorithms::initializers::range_random_initialization;
//! use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
//! use genetic_algorithms::traits::{ChromosomeT, ConfigurationT, StoppingConfig};
//!
//! // Rastrigin function: f(x) = A*n + sum(x_i^2 - A*cos(2*pi*x_i))
//! fn rastrigin(dna: &[RangeGenotype<f64>]) -> f64 {
//!     let a = 10.0;
//!     let n = dna.len() as f64;
//!     a * n + dna.iter().map(|g| {
//!         g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos()
//!     }).sum::<f64>()
//! }
//!
//! let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0)];
//!
//! let mut ga = Ga::new()
//!     .with_population_size(200)
//!     .with_max_generations(500)
//!     .with_genes_per_chromosome(10)
//!     .with_fitness_fn(rastrigin)
//!     .with_initialization_fn(move |n, _, _| {
//!         range_random_initialization(n, Some(&alleles), Some(false))
//!     })
//!     .with_selection_method(Selection::Tournament)
//!     .with_crossover_method(Crossover::BlendAlpha)
//!     .with_mutation_method(Mutation::Gaussian)
//!     .with_survivor_method(Survivor::Fitness)
//!     .build()?;
//!
//! let population = ga.run()?;
//! println!("Best fitness: {:?}", population.best_chromosome.fitness);
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Configuration Tips
//!
//! - **Tournament size 3-5** is a good starting point; larger tournament = more selection pressure
//! - **Blend crossover (BLX-alpha)** works well for continuous problems; use **Uniform** for binary
//! - **Gaussian mutation** with sigma=0.1 and probability=0.1 is a solid default for real-valued genes
//! - Enable **elitism (1-2)** to prevent loss of the best solution found
//! - Attach a [`LogObserver`](crate::observer::LogObserver) during development for per-generation stats
//! - For multimodal problems, enable **niching** or use the **Island model** instead
//!
//! ## When to Choose This vs Differential Evolution
//!
//! | Factor | Ga | DeEngine |
//! |--------|-----|----------|
//! | Problem type | Any (binary, real, symbolic) | Continuous only |
//! | Convergence speed | Moderate | Fast on unimodal |
//! | Exploration | Good | Excellent |
//! | Operator flexibility | Very high (10+ each) | Limited |
//! | Ease of tuning | Moderate | Easy (F, CR) |
//!
//! See also: [`crate::island`] for multi-population island models, and
//! [`crate::nsga2`] for multi-objective optimization.
//!
//! ## References
//!
//! - Goldberg, D. E. (1989). *Genetic Algorithms in Search, Optimization, and Machine Learning.*
//! - Holland, J. H. (1975). *Adaptation in Natural and Artificial Systems.*

use crate::aos::AosState;
use crate::configuration::GaConfiguration;
use crate::constraints::{ConstraintHandling, PenaltyStrategy};
use crate::error::GaError;
use crate::hall_of_fame::{HallOfFame, HallOfFameConfig};
use crate::observer::{ExtensionEvent, GaObserver};
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, InitializationFn, MutationOperator};
use crate::validators::validator_factory as ValidatorFactory;
use crate::{
    configuration::{LimitConfiguration, LocalSearchConfiguration, ProblemSolving},
    operations::local_search::{LocalSearch, LocalSearchApplicationStrategy, LocalSearchMode},
    operations::{
        crossover, extension, mutation, selection, survivor, Crossover, Extension, Mutation,
    },
    population::Population,
    traits::{
        ConfigurationT, CrossoverConfig, ElitismConfig, ExtensionConfig, GeneT, LinearChromosome,
        LocalSearchConfig, LocalSearchOperator, MutationConfig, NichingConfig, VectorFitness,
        OperatorCompat, SelectionConfig, StoppingConfig, Strategy, SurvivorConfig,
    },
};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::fmt::Debug;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// ─── Type aliases (D-09) ──────────────────────────────────────────────────────

/// Constraint function: maps a chromosome's DNA slice to a violation score (0 = satisfied).
type ConstraintFn<G> = Arc<dyn Fn(&[G]) -> f64 + Send + Sync>;

/// Repair function: applies an in-place repair to a chromosome after mutation.
type RepairFn<U> = Arc<dyn Fn(&mut U) -> Result<(), GaError> + Send + Sync>;

/// AOS reward accumulator shared across rayon threads (Phase 43).
type RewardAccumulator = Option<Arc<Mutex<Vec<(usize, f64)>>>>;

/// Marker trait that resolves to `serde::Serialize` when the `serde` feature is
/// enabled, or to an auto-implemented blanket trait otherwise.
///
/// This allows the `Ga` impl block to conditionally require `Serialize` without
/// duplicating the entire implementation. Users never need to implement this
/// trait manually — it is automatically satisfied for all types (or all
/// `Serialize` types when the `serde` feature is active).
#[cfg(feature = "serde")]
pub trait MaybeSerialize: serde::Serialize {}
#[cfg(feature = "serde")]
impl<T: serde::Serialize> MaybeSerialize for T {}

/// Marker trait for conditional serialization support.
///
/// When the `serde` feature is enabled, this resolves to `serde::Serialize`.
/// When disabled, it is an auto-implemented blanket trait that does nothing.
/// This pattern allows generics to require serialization unconditionally
/// while only imposing the bound when the feature is active.
#[cfg(not(feature = "serde"))]
pub trait MaybeSerialize {}
#[cfg(not(feature = "serde"))]
impl<T> MaybeSerialize for T {}

/// Marker trait for conditional deserialization support.
///
/// When the `serde` feature is enabled, this resolves to
/// `for<'de> serde::Deserialize<'de>`. When disabled, it is an
/// auto-implemented blanket trait. Mirrors the `MaybeSerialize` pattern
/// for checkpoint loading.
#[cfg(not(feature = "serde"))]
pub trait MaybeDeserialize {}
#[cfg(not(feature = "serde"))]
impl<T> MaybeDeserialize for T {}

/// Marker trait that resolves to `serde::Deserialize` when the `serde` feature is
/// enabled, or to an auto-implemented blanket trait otherwise.
///
/// This mirrors the `MaybeSerialize` pattern for conditional deserialization
/// support (needed for checkpoint loading in `run_with_callback`).
#[cfg(feature = "serde")]
pub trait MaybeDeserialize: for<'de> serde::Deserialize<'de> {}
#[cfg(feature = "serde")]
impl<T: for<'de> serde::Deserialize<'de>> MaybeDeserialize for T {}

/// Indicates why a GA run terminated.
///
/// - `GenerationLimitReached`: the maximum number of generations was reached.
/// - `FitnessTargetReached`: a stopping criterion based on fitness was satisfied.
/// - `StagnationReached`: no fitness improvement for N consecutive generations.
/// - `ConvergenceReached`: fitness standard deviation dropped below threshold.
/// - `TimeLimitReached`: elapsed wall-clock time exceeded the configured limit.
/// - `CallbackRequested`: the user callback returned `ControlFlow::Break`.
/// - `NotTerminated`: internal state before the run finalizes or if a callback is invoked mid-run.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::TerminationCause;
///
/// let cause = TerminationCause::FitnessTargetReached;
/// assert_eq!(cause, TerminationCause::FitnessTargetReached);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TerminationCause {
    /// The maximum number of generations was reached.
    GenerationLimitReached,
    /// The specified fitness target was reached or surpassed.
    FitnessTargetReached,
    /// No fitness improvement for N consecutive generations.
    StagnationReached,
    /// Fitness standard deviation dropped below the convergence threshold.
    ConvergenceReached,
    /// Elapsed wall-clock time exceeded the configured limit.
    TimeLimitReached,
    /// The user callback returned `ControlFlow::Break`.
    CallbackRequested,
    /// Internal state before the run finalizes, or while a callback is running mid-run.
    NotTerminated,
}

/// Generic Genetic Algorithm orchestrator.
///
/// Type parameter:
/// - `U`: Chromosome type implementing `ChromosomeT`.
///
/// Responsibilities:
/// - Manage configuration, alleles, population and termination state.
/// - Provide builder-like configuration methods (`ConfigurationT`) to compose the run.
/// - Coordinate the GA cycle: initialization, selection, crossover, mutation, survivor, evaluation.
pub struct Ga<U>
where
    U: LinearChromosome,
{
    /// Tunable GA configuration (limits, operators, logging, etc.).
    pub configuration: GaConfiguration,
    /// Alleles template for initialization functions (optional).
    pub alleles: Vec<U::Gene>,
    /// Current population.
    pub population: Population<U>,
    /// Termination cause after `run` or `run_with_callback`.
    pub termination_cause: TerminationCause,

    /// Initialization function to build chromosomes' DNA at startup.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Fitness function applied to chromosomes.
    pub fitness_fn: Option<Arc<FitnessFn<U::Gene>>>,

    /// Per-generation statistics collected during the run.
    stats: Vec<GenerationStats>,

    /// Current dynamic mutation probability, adjusted each generation when
    /// `dynamic_mutation` is enabled.
    dynamic_mutation_probability: f64,

    /// Optional LRU fitness cache size. When set, fitness evaluations are
    /// cached to avoid re-evaluating chromosomes with identical DNA.
    fitness_cache_size: Option<usize>,

    /// Shared handle to the active LRU fitness cache.
    ///
    /// Set during `build()` when `fitness_cache_size` is configured. `None`
    /// when no cache is in use. Wave 2 wires this to `GenerationStats` delta
    /// reporting. Reserved here to avoid a second struct-churn in Wave 2.
    fitness_cache: Option<std::sync::Arc<std::sync::Mutex<crate::fitness::cache::FitnessCache>>>,

    /// Optional batch fitness evaluator (D-03).
    ///
    /// When set, the engine evaluates an entire population slice in a single
    /// `evaluate_batch` call rather than calling `calculate_fitness` per
    /// chromosome. Mutually exclusive with `fitness_fn` (enforced in `build()`).
    /// Zero overhead when `None`.
    batch_evaluator: Option<Arc<dyn crate::fitness::BatchFitnessEvaluator<U> + Send + Sync>>,

    /// Optional surrogate model for offspring prescreening (D-04, D-06, D-08).
    ///
    /// When set, the engine predicts fitness scores for all offspring using this
    /// cheap surrogate model immediately after `parent_crossover()`. Only the
    /// top `max(1, floor(n * fraction))` offspring (by predicted score) are
    /// retained; the rest are **dropped permanently** (D-04) and never passed
    /// to [`FitnessCache`], [`BatchFitnessEvaluator`], repair, or constraint
    /// paths.
    ///
    /// Pipeline order (D-08): surrogate prescreening → cache check → batch
    /// evaluate. Both surrogate and `BatchFitnessEvaluator` may be configured
    /// simultaneously (D-09) — the surrogate runs first on the full offspring
    /// slice and reduces it before the batch evaluator is called.
    ///
    /// [`FitnessCache`]: crate::fitness::FitnessCache
    /// [`BatchFitnessEvaluator`]: crate::fitness::BatchFitnessEvaluator
    surrogate: Option<(Arc<dyn crate::fitness::SurrogateModel<U> + Send + Sync>, f64)>,

    /// Optional structured lifecycle observer. When `None` (the default),
    /// no hook calls or timing measurements are performed (zero overhead).
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

    /// Optional constraint violation functions.
    /// Each function returns a violation >= 0 for a DNA slice (0 means satisfied).
    constraint_fns: Option<Vec<ConstraintFn<U::Gene>>>,

    /// Strategy for applying penalty to infeasible solutions.
    penalty_strategy: PenaltyStrategy,

    /// Optional constraint handling method for comparisons.
    constraint_handling: Option<ConstraintHandling>,

    /// Optional repair operator. Applied after mutation, before fitness evaluation.
    repair_operator: Option<RepairFn<U>>,

    /// Current penalty coefficient (used by adaptive penalty).
    penalty_coefficient: f64,

    /// Generation counter for adaptive penalty tracking.
    /// Tracks how many generations the best individual has been feasible (positive)
    /// or infeasible (negative) within the current window.
    adaptive_penalty_counter: isize,

    /// Optional Hall of Fame / solution archive. When `None` (default), no
    /// archive is maintained and there is zero overhead.
    hall_of_fame: Option<HallOfFame<U>>,

    /// Optional user-provided seeds for warm-starting the population.
    /// When `Some(Vec<U>)`, the population is initialized with these chromosomes
    /// plus random fill to reach `population_size`. Seeds are NOT re-evaluated
    /// (fitness is trusted per D-07). When `None` (default), standard random
    /// initialization is used. Zero overhead when None.
    seeds: Option<Vec<U>>,

    /// Optional checkpoint file path for resuming a previous GA run.
    /// When `Some(path)`, the checkpoint is loaded at build-time and restores
    /// population, generation counter, and accumulated stats. The user's
    /// builder config wins for operator settings (hybrid config per D-04).
    /// When `None` (default), no checkpoint loading occurs. Zero overhead when None.
    checkpoint_path: Option<PathBuf>,

    /// Optional AOS crossover operator selection state (Phase 43).
    /// Runtime state wrapped in Mutex for safe shared access across rayon threads.
    /// When `Some(Mutex<AosState>)`, each offspring couple uses AOS to select the crossover operator.
    /// Default: None (standard single-operator dispatch).
    aos_crossover: Option<Mutex<AosState>>,

    /// Optional AOS mutation operator selection state (Phase 43).
    /// Runtime state wrapped in Mutex for safe shared access across rayon threads.
    /// When `Some(Mutex<AosState>)`, each offspring couple uses AOS to select the mutation operator.
    /// Default: None (standard single-operator dispatch).
    aos_mutation: Option<Mutex<AosState>>,
}

impl<U> Ga<U>
where
    U: LinearChromosome,
{
    /// Returns a read-only reference to the current configuration.
    pub fn configuration(&self) -> &GaConfiguration {
        &self.configuration
    }
}

impl<U> Default for Ga<U>
where
    U: LinearChromosome,
{
    fn default() -> Self {
        Ga {
            configuration: GaConfiguration {
                ..Default::default()
            },
            population: Population::new_empty(),
            alleles: Vec::new(),
            termination_cause: TerminationCause::NotTerminated,
            initialization_fn: None,
            fitness_fn: None,
            stats: Vec::new(),
            dynamic_mutation_probability: 1.0,
            fitness_cache_size: None,
            fitness_cache: None,
            batch_evaluator: None,
            surrogate: None,
            observer: None,
            constraint_fns: None,
            penalty_strategy: PenaltyStrategy::None,
            constraint_handling: None,
            repair_operator: None,
            penalty_coefficient: 0.0,
            adaptive_penalty_counter: 0,
            hall_of_fame: None,
            seeds: None,
            checkpoint_path: None,
            aos_crossover: None,
            aos_mutation: None,
        }
    }
}

impl<U> SelectionConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_number_of_couples(mut self, number_of_couples: usize) -> Self {
        self.configuration.selection_configuration.number_of_couples = number_of_couples;
        self
    }
    fn with_selection_method(mut self, selection_method: crate::operations::Selection) -> Self {
        self.configuration.selection_configuration.method = selection_method;
        self
    }
    fn with_niche_radius(mut self, niche_radius: f64) -> Self {
        self.configuration.selection_configuration.niche_radius = niche_radius;
        self
    }
    fn with_epsilon_lexicase(mut self, epsilon: f64) -> Self {
        self.configuration.selection_configuration.epsilon = epsilon;
        self
    }
}

impl<U> CrossoverConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_crossover_number_of_points(mut self, number_of_points: usize) -> Self {
        self.configuration.crossover_configuration.number_of_points = Some(number_of_points);
        self
    }
    fn with_crossover_probability_max(mut self, probability_max: f64) -> Self {
        self.configuration.crossover_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_crossover_probability_min(mut self, probability_min: f64) -> Self {
        self.configuration.crossover_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_crossover_method(mut self, method: crossover::Crossover) -> Self {
        self.configuration.crossover_configuration.method = method;
        self
    }
    fn with_sbx_eta(mut self, eta: f64) -> Self {
        self.configuration.crossover_configuration.sbx_eta = Some(eta);
        self
    }
    fn with_blend_alpha(mut self, alpha: f64) -> Self {
        self.configuration.crossover_configuration.blend_alpha = Some(alpha);
        self
    }
    fn with_undx_sigma_xi(mut self, value: f64) -> Self {
        self.configuration.crossover_configuration.undx_sigma_xi = Some(value);
        self
    }
    fn with_undx_sigma_eta(mut self, value: f64) -> Self {
        self.configuration.crossover_configuration.undx_sigma_eta = Some(value);
        self
    }
    fn with_pcx_sigma_eta(mut self, value: f64) -> Self {
        self.configuration.crossover_configuration.pcx_sigma_eta = Some(value);
        self
    }
    fn with_pcx_sigma_zeta(mut self, value: f64) -> Self {
        self.configuration.crossover_configuration.pcx_sigma_zeta = Some(value);
        self
    }
}

impl<U> MutationConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_mutation_probability_max(mut self, probability_max: f64) -> Self {
        self.configuration.mutation_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_mutation_probability_min(mut self, probability_min: f64) -> Self {
        self.configuration.mutation_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_mutation_method(mut self, method: crate::operations::Mutation) -> Self {
        self.configuration.mutation_configuration.method = method;
        self
    }
    fn with_dynamic_mutation(mut self, enabled: bool) -> Self {
        self.configuration.mutation_configuration.dynamic_mutation = enabled;
        self
    }
    fn with_mutation_target_cardinality(mut self, target: f64) -> Self {
        self.configuration.mutation_configuration.target_cardinality = Some(target);
        self
    }
    fn with_mutation_probability_step(mut self, step: f64) -> Self {
        self.configuration.mutation_configuration.probability_step = Some(step);
        self
    }
}

impl<U> StoppingConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_max_generations(mut self, max_generations: usize) -> Self {
        self.configuration.limit_configuration.max_generations = max_generations;
        self
    }
    fn with_fitness_target(mut self, fitness_target: f64) -> Self {
        self.configuration.limit_configuration.fitness_target = Some(fitness_target);
        self
    }
    fn with_stagnation_limit(mut self, n: usize) -> Self {
        self.configuration.stagnation_generations = Some(n);
        self
    }
    fn with_convergence_threshold(mut self, threshold: f64) -> Self {
        self.configuration.convergence_threshold = Some(threshold);
        self
    }
    fn with_max_duration_secs(mut self, secs: f64) -> Self {
        self.configuration.max_duration_secs = Some(secs);
        self
    }
}

impl<U> NichingConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_niching_enabled(mut self, enabled: bool) -> Self {
        self.configuration
            .niching_configuration
            .get_or_insert_with(crate::niching::configuration::NichingConfiguration::default)
            .enabled = enabled;
        self
    }
    fn with_niching_sigma_share(mut self, sigma_share: f64) -> Self {
        self.configuration
            .niching_configuration
            .get_or_insert_with(crate::niching::configuration::NichingConfiguration::default)
            .sigma_share = sigma_share;
        self
    }
    fn with_niching_alpha(mut self, alpha: f64) -> Self {
        self.configuration
            .niching_configuration
            .get_or_insert_with(crate::niching::configuration::NichingConfiguration::default)
            .alpha = alpha;
        self
    }
}

impl<U> ElitismConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_elitism(mut self, elitism_count: usize) -> Self {
        self.configuration.elitism_count = elitism_count;
        self
    }
}

impl<U> SurvivorConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_length_penalty(mut self, penalty: f64) -> Self {
        self.configuration.length_penalty = Some(penalty);
        self
    }
}

impl<U> ExtensionConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_extension_method(mut self, method: crate::operations::Extension) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(crate::extension::configuration::ExtensionConfiguration::default)
            .method = method;
        self
    }
    fn with_extension_diversity_threshold(mut self, threshold: f64) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(crate::extension::configuration::ExtensionConfiguration::default)
            .diversity_threshold = threshold;
        self
    }
    fn with_extension_survival_rate(mut self, rate: f64) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(crate::extension::configuration::ExtensionConfiguration::default)
            .survival_rate = rate;
        self
    }
    fn with_extension_mutation_rounds(mut self, rounds: usize) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(crate::extension::configuration::ExtensionConfiguration::default)
            .mutation_rounds = rounds;
        self
    }
    fn with_extension_elite_count(mut self, count: usize) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(crate::extension::configuration::ExtensionConfiguration::default)
            .elite_count = count;
        self
    }
}

impl<U> LocalSearchConfig for Ga<U>
where
    U: LinearChromosome,
{
    fn with_local_search_configuration(mut self, config: LocalSearchConfiguration) -> Self {
        self.configuration.local_search_configuration = Some(config);
        self
    }
}

impl<U> ConfigurationT for Ga<U>
where
    U: LinearChromosome,
{
    fn new() -> Self {
        Self::default()
    }
    fn with_adaptive_ga(mut self, adaptive_ga: bool) -> Self {
        self.configuration.adaptive_ga = adaptive_ga;
        self
    }
    fn with_threads(mut self, number_of_threads: usize) -> Self {
        self.configuration.number_of_threads = number_of_threads;
        self
    }
    fn with_survivor_method(mut self, method: crate::operations::Survivor) -> Self {
        self.configuration.survivor = method;
        self
    }

    //Limit configuration
    fn with_problem_solving(mut self, problem_solving: ProblemSolving) -> Self {
        self.configuration.limit_configuration.problem_solving = problem_solving;
        self
    }
    fn with_population_size(mut self, population_size: usize) -> Self {
        self.configuration.limit_configuration.population_size = population_size;

        // Setting the number of couples
        self.configuration.selection_configuration.number_of_couples =
            if self.configuration.selection_configuration.number_of_couples == 0 {
                self.configuration.limit_configuration.population_size / 2
            } else {
                self.configuration.selection_configuration.number_of_couples
            };

        self
    }
    fn with_chromosome_length(mut self, length: crate::chromosomes::ChromosomeLength) -> Self {
        self.configuration.limit_configuration.chromosome_length = length;
        self
    }

    //Save progress configuration
    fn with_save_progress(mut self, save_progress: bool) -> Self {
        self.configuration.save_progress_configuration.save_progress = save_progress;
        self
    }
    fn with_save_progress_interval(mut self, save_progress_interval: usize) -> Self {
        self.configuration
            .save_progress_configuration
            .save_progress_interval = save_progress_interval;
        self
    }
    fn with_save_progress_path(mut self, save_progress_path: String) -> Self {
        self.configuration
            .save_progress_configuration
            .save_progress_path = save_progress_path;
        self
    }

    fn with_rng_seed(mut self, seed: u64) -> Self {
        self.configuration.rng_seed = Some(seed);
        self
    }

    fn with_crossover_portfolio(mut self, portfolio: Vec<crate::operations::Crossover>) -> Self {
        self.configuration.crossover_portfolio = Some(portfolio);
        self
    }
    fn with_mutation_portfolio(mut self, portfolio: Vec<crate::operations::Mutation>) -> Self {
        self.configuration.mutation_portfolio = Some(portfolio);
        self
    }
    fn with_aos_strategy(mut self, strategy: crate::aos::AosStrategy) -> Self {
        self.configuration.aos_strategy = strategy;
        self
    }
    fn with_reward_window(mut self, window: usize) -> Self {
        self.configuration.aos_reward_window = window;
        self
    }
}

impl<U> Ga<U>
where
    U: LinearChromosome
        + Send
        + Sync
        + 'static
        + Clone
        + Debug
        + mutation::ValueMutable
        + MaybeSerialize
        + MaybeDeserialize
        + OperatorCompat,
    U::Gene: 'static + Debug,
{
    /// Validates configuration and adjusts defaults, returning a ready-to-run instance.
    ///
    /// Call this after setting all builder options and before calling `run()` or
    /// `initialization()`. It performs the following checks:
    ///
    /// - Auto-sets `number_of_couples` to `population_size / 2` if not explicitly set.
    /// - Validates that `FixedFitness` mode has a `fitness_target`.
    /// - Validates that adaptive GA has proper crossover probabilities.
    /// - Validates alleles vs chromosome length when alleles can be repeated.
    ///
    /// # Errors
    ///
    /// Returns `GaError::ConfigurationError` if any validation check fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut ga = Ga::new()
    ///     .with_population_size(100)
    ///     .with_genes_per_chromosome(8)
    ///     // ... other settings ...
    ///     .build()?;
    /// ga.run()?;
    /// ```
    pub fn build(mut self) -> Result<Self, GaError> {
        // Auto-set number_of_couples from population_size if not explicitly configured
        if self.configuration.selection_configuration.number_of_couples == 0
            && self.configuration.limit_configuration.population_size > 0
        {
            self.configuration.selection_configuration.number_of_couples =
                self.configuration.limit_configuration.population_size / 2;
        }

        // Validate configuration using the existing validator (config-only checks)
        ValidatorFactory::validate::<U>(
            Some(&self.configuration),
            None,
            if self.alleles.is_empty() {
                None
            } else {
                Some(&self.alleles)
            },
        )?;

        // Check operator compatibility for this chromosome type (OperatorCompat trait)
        crate::validators::generic_validator::operator_compat_check::<U>(&self.configuration)?;

        // Validate num_parents >= 3 for multi-parent crossover operators
        match self.configuration.crossover_configuration.method {
            crate::operations::Crossover::Undx { num_parents }
            | crate::operations::Crossover::Spx { num_parents }
            | crate::operations::Crossover::Pcx { num_parents }
                if num_parents < 3 =>
            {
                return Err(GaError::ConfigurationError(format!(
                    "Multi-parent crossover requires num_parents >= 3, got {}",
                    num_parents
                )));
            }
            _ => {}
        }

        // Enforce mutual exclusivity: fitness_fn and batch_evaluator cannot both be set (D-03)
        if self.fitness_fn.is_some() && self.batch_evaluator.is_some() {
            return Err(GaError::ConfigurationError(
                "Cannot use both fitness_fn and with_batch_evaluator() — they are mutually exclusive"
                    .to_string(),
            ));
        }

        // Validate surrogate prescreening_fraction: must be in (0.0, 1.0] (D-03, Pattern 7)
        if let Some((_, fraction)) = &self.surrogate {
            if *fraction <= 0.0 || *fraction > 1.0 {
                return Err(GaError::ConfigurationError(
                    "prescreening_fraction must be in (0.0, 1.0]".to_string(),
                ));
            }
        }

        // Wrap fitness function with LRU cache if configured
        if let Some(cache_size) = self.fitness_cache_size {
            if let Some(fitness_fn) = self.fitness_fn.take() {
                let (wrapped, cache_handle) = crate::fitness::cache::wrap_with_cache(
                    fitness_fn, cache_size,
                );
                self.fitness_fn = Some(wrapped);
                self.fitness_cache = Some(cache_handle);
            }
        }

        // Validate constraint configuration
        crate::constraints::validate_penalty_strategy(&self.penalty_strategy)?;

        // Validate mutual exclusivity of seeds and checkpoint (per discretion)
        if self.seeds.is_some() && self.checkpoint_path.is_some() {
            return Err(GaError::ConfigurationError(
                "Cannot use both with_seeds() and with_checkpoint() — they are mutually exclusive"
                    .to_string(),
            ));
        }

        // Validate seeds count does not exceed population_size (per discretion)
        if let Some(ref seeds) = self.seeds {
            let pop_size = self.configuration.limit_configuration.population_size;
            if seeds.len() > pop_size {
                return Err(GaError::ConfigurationError(format!(
                    "Number of seeds ({}) exceeds population_size ({}): with_seeds count must not exceed population_size",
                    seeds.len(),
                    pop_size,
                )));
            }
        }

        // Validate checkpoint file exists (per discretion: build-time validation)
        // Note: full checkpoint loading (population restore, etc.) happens at run time
        // to avoid requiring serde/Deserialize bounds at build().
        if let Some(ref checkpoint_path) = self.checkpoint_path {
            if !checkpoint_path.exists() {
                return Err(GaError::CheckpointError(format!(
                    "Checkpoint file not found: {}",
                    checkpoint_path.display(),
                )));
            }
        }

        // AOS portfolio validation (Phase 43)
        if let Some(ref xover_pf) = self.configuration.crossover_portfolio {
            if xover_pf.is_empty() {
                return Err(GaError::ConfigurationError(
                    "AOS crossover portfolio is empty: provide at least 2 operators".to_string(),
                ));
            }
            if xover_pf.len() == 1 {
                log::warn!(target: "ga_events", "AOS crossover portfolio has only 1 operator; portfolio mode is effectively the same as single-operator mode");
            }
        }
        if let Some(ref mut_pf) = self.configuration.mutation_portfolio {
            if mut_pf.is_empty() {
                return Err(GaError::ConfigurationError(
                    "AOS mutation portfolio is empty: provide at least 2 operators".to_string(),
                ));
            }
            if mut_pf.len() == 1 {
                log::warn!(target: "ga_events", "AOS mutation portfolio has only 1 operator; portfolio mode is effectively the same as single-operator mode");
            }
        }
        // Warn if both portfolio and single-operator are configured
        if self.configuration.crossover_portfolio.is_some()
            && self.configuration.crossover_configuration.method
                != crate::operations::Crossover::Uniform
        {
            // The default method is Uniform, so only warn if the user explicitly changed it
            log::warn!(target: "ga_events", "Both crossover portfolio and with_crossover_method() are configured. with_crossover_method() will be ignored when portfolio is set");
        }
        if self.configuration.mutation_portfolio.is_some()
            && self.configuration.mutation_configuration.method != crate::operations::Mutation::Swap
        {
            log::warn!(target: "ga_events", "Both mutation portfolio and with_mutation_method() are configured. with_mutation_method() will be ignored when portfolio is set");
        }

        Ok(self)
    }

    /// Sets the alleles (possible gene values) used during initialization.
    pub fn with_alleles(mut self, alleles: Vec<U::Gene>) -> Self {
        self.alleles = alleles;
        self
    }

    /// Sets an initial population instead of generating one from scratch.
    ///
    /// If `number_of_couples` has not been set, it defaults to half the population size.
    pub fn with_population(mut self, population: Population<U>) -> Self {
        //Checks if the number of couples is 0, sets the number of couples to the half of the population
        if self.configuration.selection_configuration.number_of_couples == 0 {
            self.configuration.selection_configuration.number_of_couples = population.size() / 2;
        }
        self.population = population;
        self
    }

    /// Sets the fitness function used to evaluate chromosomes.
    ///
    /// The closure receives a chromosome's DNA (a slice of genes) and must return
    /// a scalar `f64` fitness value.
    pub fn with_fitness_fn<F>(mut self, fitness_fn: F) -> Self
    where
        F: Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = Some(Arc::new(fitness_fn));
        self
    }

    /// Attaches a structured lifecycle observer that receives hooks during execution.
    ///
    /// The observer is stored as an `Arc` for thread-safe sharing (required by the
    /// island model). All hooks receive `&self`, so observers that need interior
    /// mutability should use `Mutex`, `AtomicU64`, or similar.
    ///
    /// See [`GaObserver`] for the hook contract.
    pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Enables an LRU fitness cache with the given capacity.
    ///
    /// When enabled, fitness evaluations are cached by DNA hash. Chromosomes
    /// with identical genes will reuse cached fitness values, avoiding
    /// redundant (and potentially expensive) fitness function calls.
    ///
    /// The cache is shared across all chromosomes and threads.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum number of entries in the cache. A typical value
    ///   is 2-10x the population size.
    pub fn with_fitness_cache_size(mut self, size: usize) -> Self {
        self.fitness_cache_size = Some(size);
        self
    }

    /// Sets a batch fitness evaluator as an alternative to `with_fitness_fn`.
    ///
    /// When configured, the engine evaluates an entire population slice in a
    /// single `evaluate_batch` call rather than calling `calculate_fitness` on
    /// each chromosome individually. This is useful for vectorised computation
    /// (e.g. GPU shaders, external simulators that amortise setup cost).
    ///
    /// **Mutually exclusive with `with_fitness_fn`.** Calling `build()` with
    /// both set returns `GaError::ConfigurationError` (D-03).
    ///
    /// # Arguments
    ///
    /// * `evaluator` - An `Arc`-wrapped implementation of `BatchFitnessEvaluator<U>`.
    pub fn with_batch_evaluator(
        mut self,
        evaluator: Arc<dyn crate::fitness::BatchFitnessEvaluator<U> + Send + Sync>,
    ) -> Self {
        self.batch_evaluator = Some(evaluator);
        self
    }

    /// Configures a surrogate model for offspring prescreening.
    ///
    /// Each generation, after `parent_crossover()` produces offspring, the
    /// engine calls `model.predict(&c)` for every offspring chromosome, sorts
    /// them by predicted score (descending), and retains only
    /// `max(1, floor(n * prescreening_fraction))` of the best-predicted
    /// offspring before any true fitness evaluation.
    ///
    /// # Valid range
    ///
    /// `prescreening_fraction` must be in the half-open interval `(0.0, 1.0]`.
    /// Calling `build()` with a value outside this range returns
    /// `GaError::ConfigurationError` (D-03).
    ///
    /// Use `1.0` to disable prescreening while keeping the surrogate wired
    /// (all offspring survive the filter).
    ///
    /// # Composition (D-09)
    ///
    /// The surrogate composes with [`BatchFitnessEvaluator`] — both can be set
    /// simultaneously. The surrogate runs first (reducing the offspring slice)
    /// before the batch evaluator is called on the survivors.
    ///
    /// # Arguments
    ///
    /// * `model` — An `Arc`-wrapped implementation of `SurrogateModel<U>`.
    /// * `prescreening_fraction` — Fraction of offspring to retain, in `(0.0, 1.0]`.
    ///
    /// [`BatchFitnessEvaluator`]: crate::fitness::BatchFitnessEvaluator
    pub fn with_surrogate(
        mut self,
        model: Arc<dyn crate::fitness::SurrogateModel<U> + Send + Sync>,
        prescreening_fraction: f64,
    ) -> Self {
        self.surrogate = Some((model, prescreening_fraction));
        self
    }

    // ---------------------------------------------------------------------------
    // Constraint handling builder methods
    // ---------------------------------------------------------------------------

    /// Sets one or more constraint violation functions.
    ///
    /// Each function receives a chromosome's DNA slice and returns a violation
    /// value >= 0, where 0 means the constraint is fully satisfied. Multiple
    /// constraint functions can be provided; the total violation is the sum
    /// of all individual violation values.
    ///
    /// Combined with `with_penalty_strategy()` to define how violations
    /// affect fitness, or with `with_constraint_handling()` for Deb's
    /// feasibility rules.
    pub fn with_constraint_fns<F>(mut self, fns: Vec<F>) -> Self
    where
        F: Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.constraint_fns = Some(fns.into_iter().map(|f| Arc::new(f) as Arc<_>).collect());
        self
    }

    /// Sets the penalty strategy for constraint handling.
    ///
    /// Use with `with_constraint_fns()`. The strategy determines how
    /// constraint violations are added to raw fitness.
    ///
    /// Default: `PenaltyStrategy::None` (no penalty applied).
    pub fn with_penalty_strategy(mut self, strategy: PenaltyStrategy) -> Self {
        self.penalty_strategy = strategy;
        self
    }

    /// Sets the constraint handling method for comparisons.
    ///
    /// Use with `with_constraint_fns()`. When set, the comparison behavior
    /// in selection, survivor, and elite operations is modified according
    /// to the chosen method (e.g., Deb's feasibility rules).
    pub fn with_constraint_handling(mut self, handling: ConstraintHandling) -> Self {
        self.constraint_handling = Some(handling);
        self
    }

    /// Sets the repair operator for fixing infeasible chromosomes.
    ///
    /// The repair operator is applied after mutation and before fitness
    /// evaluation, allowing chromosomes to be modified in-place to satisfy
    /// problem-specific constraints (e.g., knapsack capacity, TSP validity).
    ///
    /// The operator receives a mutable reference to the chromosome and must
    /// return `Ok(())` after modifying it to satisfy constraints.
    pub fn with_repair_operator<F>(mut self, operator: F) -> Self
    where
        F: Fn(&mut U) -> Result<(), GaError> + Send + Sync + 'static,
    {
        self.repair_operator = Some(Arc::new(operator));
        self
    }

    /// Sets the initialization function used to create chromosome DNA.
    ///
    /// The closure receives `(genes_per_chromosome, alleles, needs_unique_ids)`
    /// and must return a `Vec` of genes for one chromosome.
    pub fn with_initialization_fn<F>(mut self, initialization_fn: F) -> Self
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
        F: Fn(usize, Option<&[U::Gene]>) -> Vec<U::Gene> + Send + Sync + 'static,
    {
        self.initialization_fn = Some(Arc::new(initialization_fn));
        self
    }

    /// Configures a Hall of Fame / solution archive.
    ///
    /// When configured, the GA will maintain a bounded archive of the top-N
    /// unique solutions encountered across all generations. Accessible after
    /// `run()` completes via `.hall_of_fame()`.
    ///
    /// Uses `HallOfFameConfig` for capacity and diversity filtering settings.
    pub fn with_hall_of_fame(mut self, config: HallOfFameConfig) -> Self {
        self.hall_of_fame = Some(HallOfFame::new(config));
        self
    }

    /// Seeds the population with known solutions before the GA run.
    ///
    /// The provided chromosomes are placed at the front of the population
    /// (in order), and the remaining slots are filled via the configured
    /// `initialization_fn`. Random fill deduplicates against seed DNA
    /// (genotypic uniqueness check via gene.id() comparison).
    ///
    /// Seed fitness is trusted (per D-07) — seeds skip re-evaluation.
    /// Seeds are also eligible for Hall of Fame admission during initialization.
    ///
    /// # Errors at build time
    /// - If the number of seeds exceeds `population_size`, `build()` returns
    ///   `GaError::ConfigurationError`.
    /// - If both `with_seeds()` and `with_checkpoint()` are called, `build()`
    ///   returns `GaError::ConfigurationError` (mutual exclusivity per discretion).
    pub fn with_seeds(mut self, seeds: Vec<U>) -> Self {
        self.seeds = Some(seeds);
        self
    }

    /// Resumes a GA run from a previously saved checkpoint file.
    ///
    /// The checkpoint is loaded at build time, restoring the population,
    /// generation counter, and accumulated statistics from the checkpoint
    /// file. The user's builder-configured operator settings (selection,
    /// crossover, mutation, survivor) override any in the checkpoint
    /// (hybrid config per D-04).
    ///
    /// The user must still provide `fitness_fn` and `initialization_fn`
    /// in the builder chain (these are not serializable).
    ///
    /// # Generation counting (D-05)
    /// Uses absolute mode: the generation loop starts from
    /// `checkpoint.generation` and runs for `max_generations` additional
    /// generations. Upper bound = `checkpoint.generation + max_generations`.
    ///
    /// # Errors at build time
    /// - If the checkpoint file does not exist or cannot be deserialized,
    ///   `build()` returns `GaError::CheckpointError`.
    /// - If both `with_seeds()` and `with_checkpoint()` are called, `build()`
    ///   returns `GaError::ConfigurationError` (mutual exclusivity per discretion).
    pub fn with_checkpoint(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.checkpoint_path = Some(path.into());
        self
    }

    /// Configures a local search operator for memetic algorithm refinement.
    ///
    /// When configured, the local search operator is applied to offspring
    /// after crossover+mutation+fitness and after repair/constraints,
    /// before population merge and survivor selection.
    ///
    /// The provided operator variant is stored in the configuration and
    /// applied each generation according to the application strategy.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ga = Ga::<RangeChromosome<f64>>::new()
    ///     .with_local_search(LocalSearch::HillClimbing)
    ///     .with_local_search_configuration(LocalSearchConfiguration {
    ///         application_strategy: LocalSearchApplicationStrategy::BestN { n: 5 },
    ///         ..Default::default()
    ///     });
    /// ```
    pub fn with_local_search(mut self, method: LocalSearch) -> Self {
        self.configuration
            .local_search_configuration
            .get_or_insert_with(LocalSearchConfiguration::default)
            .method = method;
        self
    }

    /// Randomly initializes the population using the provided initialization function.
    ///
    /// Behavior:
    /// - Validates configuration and alleles before starting.
    /// - Creates and evaluates chromosomes in parallel using rayon.
    /// - Sets the internal `population` with the collected chromosomes.
    pub fn initialization(&mut self) -> Result<&mut Self, GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        // Before starting initialization, verify that initializer is set
        if self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        // Validate configuration
        ValidatorFactory::validate::<U>(Some(&self.configuration), None, Some(&self.alleles))?;

        // Delegate to seed-aware or standard init
        if self.seeds.is_some() {
            self.initialize_with_seeds()?;
        } else {
            self.initialize_random()?;
        }

        // Apply repair operator to initial population if configured
        if let Some(ref repair_op) = self.repair_operator {
            for c in self.population.chromosomes.iter_mut() {
                repair_op(c)?;
                c.calculate_fitness();
            }
        }

        Ok(self)
    }

    /// Creates a random initial population (no seeds).
    fn initialize_random(&mut self) -> Result<(), GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        let population_size = self.configuration.limit_configuration.population_size;
        let chromosome_length = self.configuration.limit_configuration.chromosome_length;
        let init_fn = self.initialization_fn.as_ref().unwrap();
        // In batch mode `fitness_fn` is None; pass None to initialize_chromosomes_par
        // so chromosomes start with default 0.0 fitness — batch_evaluate runs afterward
        // to assign correct values.
        let fitness_fn = self.fitness_fn.as_ref();

        let chromosomes = match chromosome_length {
            crate::chromosomes::ChromosomeLength::Fixed(length) => {
                crate::traits::initialize_chromosomes_par::<U>(
                    population_size,
                    length,
                    if self.alleles.is_empty() {
                        None
                    } else {
                        Some(&self.alleles)
                    },
                    init_fn,
                    fitness_fn,
                    0,
                )
            }
            crate::chromosomes::ChromosomeLength::Variable { min, max } => {
                // For variable-length chromosomes, each individual gets a random
                // length sampled uniformly from [min, max].
                // Decision: pass sampled length as genes_per_chromosome to init_fn
                // (per Phase 52 discussion log — zero changes to init_fn signature).
                let alleles_ref: Option<&[U::Gene]> = if self.alleles.is_empty() {
                    None
                } else {
                    Some(self.alleles.as_slice())
                };
                // In batch mode fitness_fn is None; ff_opt carries the optional reference
                let ff = fitness_fn.cloned();

                #[cfg(not(target_arch = "wasm32"))]
                let result: Vec<U> = (0..population_size)
                    .into_par_iter()
                    .map(|_| {
                        let len = {
                            let mut rng = crate::rng::make_rng();
                            rng.random_range(min..=max)
                        };
                        let genes = init_fn(len, alleles_ref);
                        let mut c = U::new();
                        c.set_dna(std::borrow::Cow::Owned(genes));
                        if let Some(ref ff_arc) = ff {
                            let ff_clone = std::sync::Arc::clone(ff_arc);
                            c.set_fitness_fn(move |dna| ff_clone(dna));
                        }
                        c.calculate_fitness();
                        c.set_age(0);
                        c
                    })
                    .collect();
                #[cfg(target_arch = "wasm32")]
                let result: Vec<U> = (0..population_size)
                    .map(|_| {
                        let len = {
                            let mut rng = crate::rng::make_rng();
                            rng.random_range(min..=max)
                        };
                        let genes = init_fn(len, alleles_ref);
                        let mut c = U::new();
                        c.set_dna(std::borrow::Cow::Owned(genes));
                        if let Some(ref ff_arc) = ff {
                            let ff_clone = std::sync::Arc::clone(ff_arc);
                            c.set_fitness_fn(move |dna| ff_clone(dna));
                        }
                        c.calculate_fitness();
                        c.set_age(0);
                        c
                    })
                    .collect();
                result
            }
        };

        // Set population directly (with_population is consuming, so we assign inline)
        let new_population = Population::new(chromosomes);
        if self.configuration.selection_configuration.number_of_couples == 0 {
            self.configuration.selection_configuration.number_of_couples =
                new_population.size() / 2;
        }
        self.population = new_population;

        Ok(())
    }

    /// Initializes the population with pre-evaluated seeds placed at the front.
    ///
    /// Seeds are moved into the population in order, then remaining slots are
    /// filled with randomly generated chromosomes. Fill chromosomes are
    /// genotypically deduplicated against all existing seeds (and prior fills),
    /// using the same DNA comparison pattern as HallOfFame.
    ///
    /// When a HallOfFame is configured, seeds and fill chromosomes are both
    /// evaluated for archive entry during initialization (generation 0).
    ///
    /// WASM compatible: seed placement and dedup are pure data operations.
    fn initialize_with_seeds(&mut self) -> Result<(), GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        if self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        let seeds = self.seeds.take().unwrap();
        let population_size = self.configuration.limit_configuration.population_size;
        let fill_count = population_size - seeds.len();
        let length = match self.configuration.limit_configuration.chromosome_length {
            crate::chromosomes::ChromosomeLength::Fixed(n) => n,
            crate::chromosomes::ChromosomeLength::Variable { .. } => {
                return Err(GaError::ConfigurationError(
                    "ChromosomeLength::Variable is not yet supported (Phase 52). Use ChromosomeLength::Fixed.".into(),
                ));
            }
        };
        let init_fn = self.initialization_fn.as_ref().unwrap();
        // In batch mode `fitness_fn` is None — chromosomes start with default fitness;
        // batch_evaluate runs afterward to assign correct values.
        let fitness_fn = self.fitness_fn.as_ref();

        // Step 1: Collect seed DNA for dedup comparison
        let seed_dnas: Vec<&[U::Gene]> = seeds.iter().map(|s| s.dna()).collect();

        // Step 2: Generate random fill with genotypic dedup against seeds
        let mut fill_chromosomes: Vec<U> = Vec::with_capacity(fill_count);
        // Use sequential generation for dedup (parallel would make retry logic complex)
        // Use a max retry bound to prevent infinite loop in degenerate cases
        let max_attempts = fill_count * 10;
        let mut attempts = 0;

        while fill_chromosomes.len() < fill_count && attempts < max_attempts {
            attempts += 1;

            // Generate one random chromosome using the initialization function
            let genes = init_fn(
                length,
                if self.alleles.is_empty() {
                    None
                } else {
                    Some(&self.alleles)
                },
            );
            let mut new_chromosome = U::new();
            new_chromosome.set_dna(std::borrow::Cow::Owned(genes));

            // Check genotypic uniqueness against seed DNAs
            let new_dna = new_chromosome.dna();
            let is_duplicate = seed_dnas.iter().any(|seed_dna| {
                let max_len = new_dna.len().max(seed_dna.len());
                if max_len == 0 {
                    return true;
                }
                (0..max_len).all(|i| {
                    let id_a = new_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                    let id_b = seed_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                    id_a == id_b
                })
            });

            if is_duplicate {
                continue; // Discard and retry
            }

            // Also dedup against already-generated fill chromosomes
            let is_fill_duplicate = fill_chromosomes.iter().any(|existing| {
                let existing_dna = existing.dna();
                let max_len = new_dna.len().max(existing_dna.len());
                if max_len == 0 {
                    return true;
                }
                (0..max_len).all(|i| {
                    let id_a = new_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                    let id_b = existing_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                    id_a == id_b
                })
            });

            if is_fill_duplicate {
                continue; // Discard and retry
            }

            // Set fitness function and evaluate (skipped in batch mode — fitness_fn is None)
            if let Some(ff) = fitness_fn {
                let ff_clone = Arc::clone(ff);
                new_chromosome.set_fitness_fn(move |genes| ff_clone(genes));
            }
            new_chromosome.calculate_fitness();
            new_chromosome.set_age(0);

            fill_chromosomes.push(new_chromosome);
        }

        if fill_chromosomes.len() < fill_count {
            return Err(GaError::InitializationError(format!(
                "Failed to generate {} unique random chromosomes (max attempts {} reached). \
                 Try reducing the number of seeds or increasing population_size.",
                fill_count, max_attempts,
            )));
        }

        // Step 3: Build population: seeds placed first, then fill
        let mut all_chromosomes: Vec<U> = Vec::with_capacity(population_size);
        all_chromosomes.extend(seeds); // Seeds first (trusted fitness)
        all_chromosomes.extend(fill_chromosomes); // Fill (evaluated)

        let new_population = Population::new(all_chromosomes);
        if self.configuration.selection_configuration.number_of_couples == 0 {
            self.configuration.selection_configuration.number_of_couples =
                new_population.size() / 2;
        }
        self.population = new_population;

        // Step 4: Admit seeds to Hall of Fame if configured (per D-08)
        if let Some(ref mut hof) = self.hall_of_fame {
            for c in self.population.chromosomes.iter() {
                hof.try_insert(c, 0); // Generation 0: initialization
            }
        }

        Ok(())
    }

    /// Runs the GA without callbacks and returns a reference to the final population.
    ///
    /// Equivalent to `run_with_callback(None, 0)`.
    pub fn run(&mut self) -> Result<&Population<U>, GaError> {
        self.run_with_callback(
            None::<
                fn(&usize, &Population<U>, &GenerationStats, &TerminationCause) -> ControlFlow<()>,
            >,
            0,
        )
    }

    /// Runs the GA and optionally invokes a callback every `generations_to_callback` generations.
    ///
    /// The callback receives the generation index, current population, per-generation statistics,
    /// and the current termination cause. If the callback returns `ControlFlow::Break(())`, the
    /// run terminates early with `TerminationCause::CallbackRequested`.
    ///
    /// Execution cycle per generation:
    /// 1) Selection of parents, 2) Crossover to produce offspring, 3) Mutation of offspring,
    /// 4) Survivor selection to prune population, 5) Best chromosome update, 6) Stop check.
    ///
    /// Logging is controlled by configuration log level; adaptive GA updates use f_avg and f_max.
    pub fn run_with_callback<F>(
        &mut self,
        callback: Option<F>,
        generations_to_callback: usize,
    ) -> Result<&Population<U>, GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone + MaybeDeserialize,
        F: Fn(&usize, &Population<U>, &GenerationStats, &TerminationCause) -> ControlFlow<()>,
    {
        //Before starting the run, we will check the conditions
        ValidatorFactory::validate::<U>(Some(&self.configuration), None, Some(&self.alleles))?;

        // Apply RNG seed if configured (must be done before any random operations)
        crate::rng::set_seed(self.configuration.rng_seed);

        // Checkpoint resumption: load checkpoint if configured
        // `checkpoint_generation` is only mutated inside the `#[cfg(feature = "serde")]` block.
        // Without serde the mut is valid but unused — suppress only on non-serde builds (Pitfall 4).
        #[cfg_attr(not(feature = "serde"), allow(unused_mut))]
        let mut checkpoint_generation: Option<usize> = None;
        if self.checkpoint_path.is_some() {
            #[cfg(feature = "serde")]
            {
                let path = self.checkpoint_path.take().unwrap();
                let ckpt = crate::checkpoint::load_checkpoint::<U>(&path).map_err(|e| {
                    GaError::CheckpointError(format!(
                        "Failed to load checkpoint '{}': {}",
                        path.display(),
                        e
                    ))
                })?;

                // Hybrid config override (D-04): builder operators win, checkpoint state wins
                // 1. Save builder's operator settings
                let builder_selection = self.configuration.selection_configuration.method;
                let builder_crossover = self.configuration.crossover_configuration.method;
                let builder_mutation = self.configuration.mutation_configuration.method.clone();
                let builder_survivor = self.configuration.survivor;
                let builder_problem_solving =
                    self.configuration.limit_configuration.problem_solving;

                // 2. Override configuration from checkpoint (but keep builder's max_generations)
                let builder_max_generations =
                    self.configuration.limit_configuration.max_generations;
                let builder_population_size =
                    self.configuration.limit_configuration.population_size;
                self.configuration = ckpt.configuration;

                // 3. Restore builder's operator settings (D-04)
                self.configuration.selection_configuration.method = builder_selection;
                self.configuration.crossover_configuration.method = builder_crossover;
                self.configuration.mutation_configuration.method = builder_mutation;
                self.configuration.survivor = builder_survivor;
                self.configuration.limit_configuration.problem_solving = builder_problem_solving;

                // 4. Restore builder's max_generations (user controls this, D-05)
                self.configuration.limit_configuration.max_generations = builder_max_generations;
                self.configuration.limit_configuration.population_size = builder_population_size;

                // 5. Restore checkpoint population and stats
                self.population = ckpt.population;
                self.stats = ckpt.stats; // Preserve accumulated stats (D-06)
                checkpoint_generation = Some(ckpt.generation);
            }
            #[cfg(not(feature = "serde"))]
            {
                return Err(GaError::CheckpointError(
                    "Checkpoint loading requires the 'serde' feature. Enable it in Cargo.toml: genetic_algorithms = { features = [\"serde\"] }".to_string(),
                ));
            }
        } else if self.population.size() == 0 && self.initialization_fn.is_some() {
            // Standard initialization (no checkpoint, no population)
            self.initialization()?;
            // D-02 / Pitfall 4: batch-evaluate initial population when batch mode is active
            if let Some(eval) = self.batch_evaluator.as_ref().map(Arc::clone) {
                let cache = self.fitness_cache.as_ref().map(Arc::clone);
                batch_evaluate(eval, cache, &mut self.population.chromosomes)?;
            }
        } else if self.population.size() == 0 && self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        //Initialize the adaptive ga
        if self.configuration.adaptive_ga {
            self.population.recalculate_aga();
        }

        // Initialize AOS state if portfolios are configured (Phase 43)
        if let Some(ref xover_pf) = self.configuration.crossover_portfolio {
            self.aos_crossover = Some(Mutex::new(AosState::new(
                xover_pf.len(),
                self.configuration.aos_strategy.clone(),
                self.configuration.aos_reward_window,
            )));
        }
        if let Some(ref mut_pf) = self.configuration.mutation_portfolio {
            self.aos_mutation = Some(Mutex::new(AosState::new(
                mut_pf.len(),
                self.configuration.aos_strategy.clone(),
                self.configuration.aos_reward_window,
            )));
        }

        // Initialize dynamic mutation probability
        if self.configuration.mutation_configuration.dynamic_mutation {
            self.dynamic_mutation_probability = self
                .configuration
                .mutation_configuration
                .probability_max
                .unwrap_or(1.0);
        }

        // D-06: bootstrap cache for batch-only-with-cache case (fitness_fn is absent,
        // so build() never called wrap_with_cache; create the cache handle here).
        if self.batch_evaluator.is_some() && self.fitness_cache.is_none() {
            if let Some(size) = self.fitness_cache_size {
                self.fitness_cache = Some(Arc::new(Mutex::new(
                    crate::fitness::cache::FitnessCache::new(size),
                )));
            }
        }

        //Best chromosome within the generations and population returned
        let initial_population_size = self.population.size();
        let mut age = 0usize;

        //Calculation of the fitness and the best chromosome
        self.population.fitness_calculation(
            self.configuration.number_of_threads,
            self.configuration.limit_configuration.problem_solving,
        );

        // Apply constraint processing to initial population if configured
        if self.constraint_fns.is_some() {
            self.process_constraints_population(0)?;
        }

        // Starting counting the generations for the callback
        let mut generation_callback_count = 0usize;

        // Reset per-generation stats (only when not resuming from checkpoint, D-06)
        if checkpoint_generation.is_none() {
            self.stats.clear();
        }

        // Determine if this is a maximization problem
        let is_maximization = matches!(
            self.configuration.limit_configuration.problem_solving,
            ProblemSolving::Maximization
        );

        // Compound stopping criteria tracking
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = Instant::now();
        #[cfg(target_arch = "wasm32")]
        if self.configuration.max_duration_secs.is_some() {
            log::warn!(target: "ga_events", "max_duration_secs is not supported on wasm32 — time limit will be ignored");
        }
        let mut best_fitness_so_far = self.population.best_chromosome.fitness();
        let mut stagnation_count: usize = 0;

        self.notify(|obs| obs.on_run_start());

        //We start the cycles
        // Absolute generation counting (D-05):
        // When resuming from checkpoint, the effective total generations is
        // checkpoint.generation + max_generations. The loop variable `i` is the
        // absolute generation number used in observer hooks, statistics, and HOF.
        let start_gen = checkpoint_generation.unwrap_or(0);
        let total_gens = if checkpoint_generation.is_some() {
            start_gen + self.configuration.limit_configuration.max_generations
        } else {
            self.configuration.limit_configuration.max_generations
        };

        for i in start_gen..total_gens {
            age += 1;
            // D-07: snapshot cache counters before this generation so we can compute deltas.
            let (prev_cache_hits, prev_cache_misses) = match &self.fitness_cache {
                Some(ch) => {
                    let c = ch.lock().expect("fitness cache lock poisoned");
                    (c.hits(), c.misses())
                }
                None => (0, 0),
            };

            self.notify(|obs| obs.on_generation_start(i));

            //1- Parent selection for reproduction
            let t_sel: Option<Instant> = if self.observer.is_some() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Some(Instant::now())
                }
                #[cfg(target_arch = "wasm32")]
                {
                    None
                }
            } else {
                None
            };
            // Derive num_parents from crossover method: UNDX/SPX/PCX carry their own
            // num_parents field; all other variants use standard 2-parent crossover.
            let num_parents = match self.configuration.crossover_configuration.method {
                crate::operations::Crossover::Undx { num_parents }
                | crate::operations::Crossover::Spx { num_parents }
                | crate::operations::Crossover::Pcx { num_parents } => num_parents,
                _ => 2,
            };
            let parents = selection::factory(
                &self.population.chromosomes,
                self.configuration.selection_configuration,
                self.configuration.number_of_threads,
                num_parents,
            )?;
            if let Some(t) = t_sel {
                self.notify(|obs| obs.on_selection_complete(i, t.elapsed(), parents.len()));
            }
            //2- Getting the offspring
            let dynamic_prob = if self.configuration.mutation_configuration.dynamic_mutation {
                Some(self.dynamic_mutation_probability)
            } else {
                None
            };
            let t_cx: Option<Instant> = if self.observer.is_some() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Some(Instant::now())
                }
                #[cfg(target_arch = "wasm32")]
                {
                    None
                }
            } else {
                None
            };
            // D-02: In batch mode, pass None for fitness_fn so parent_crossover does not
            // call calculate_fitness per-child; batch_evaluate runs on the returned offspring
            // slice instead.
            let crossover_fitness_fn = if self.batch_evaluator.is_some() {
                None
            } else {
                self.fitness_fn.clone()
            };
            let mut offspring = parent_crossover(
                &parents,
                &self.population.chromosomes,
                &self.configuration,
                ParentCrossoverParams {
                    age,
                    f_max: self.population.f_max,
                    f_avg: self.population.f_avg,
                    dynamic_mutation_prob: dynamic_prob,
                    generation: i,
                    best_fitness: best_fitness_so_far,
                    is_maximization,
                    fitness_fn: crossover_fitness_fn,
                    crossover_portfolio: self.configuration.crossover_portfolio.as_ref(),
                    mutation_portfolio: self.configuration.mutation_portfolio.as_ref(),
                    aos_crossover_state: self.aos_crossover.as_ref(),
                    aos_mutation_state: self.aos_mutation.as_ref(),
                },
            )?;
            // D-08: surrogate prescreening — runs BEFORE cache/batch/repair/constraints (Pitfall 1).
            // Retains only the top max(1, floor(n * fraction)) offspring by predicted score.
            // Rejected offspring are dropped permanently (D-04) and never evaluated further.
            // Sequential sort only — unconditionally WASM-safe (no parallelism, no cfg gate).
            let true_fitness_calls: Option<u64> = if let Some((ref surrogate, fraction)) = self.surrogate {
                if offspring.is_empty() {
                    Some(0)
                } else {
                    let mut scores: Vec<(usize, f64)> = offspring
                        .iter()
                        .enumerate()
                        .map(|(idx, c)| {
                            let raw = surrogate.predict(c);
                            let score = if raw.is_nan() { f64::NEG_INFINITY } else { raw };
                            (idx, score)
                        })
                        .collect();
                    // Sort descending: best-predicted offspring first.
                    scores.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                    // Retain at least 1; floor formula from D-03/SC-1d.
                    let keep = ((offspring.len() as f64 * fraction).floor() as usize).max(1);
                    scores.truncate(keep);
                    // Restore original index order so downstream code sees a stable slice.
                    scores.sort_unstable_by_key(|&(idx, _)| idx);
                    offspring = scores.into_iter().map(|(idx, _)| offspring[idx].clone()).collect();
                    Some(offspring.len() as u64)
                }
            } else {
                None
            };
            // D-02: batch-evaluate offspring before merge (replaces calculate_fitness per child)
            if let Some(eval) = self.batch_evaluator.as_ref().map(Arc::clone) {
                let cache = self.fitness_cache.as_ref().map(Arc::clone);
                batch_evaluate(eval, cache, &mut offspring)?;
            }
            if let Some(t) = t_cx {
                let elapsed = t.elapsed();
                let offspring_count = offspring.len();
                let pop_size = self.population.chromosomes.len();
                self.notify(|obs| obs.on_crossover_complete(i, elapsed, offspring_count));
                // NOTE: elapsed covers combined crossover+mutation+fitness time (EXT-01)
                self.notify(|obs| obs.on_mutation_complete(i, elapsed, pop_size));
                // NOTE: elapsed covers combined crossover+mutation+fitness time (EXT-01)
                self.notify(|obs| obs.on_fitness_evaluation_complete(i, elapsed, pop_size));
            }

            // Apply repair operator to offspring if configured
            if let Some(ref repair_op) = self.repair_operator {
                for c in offspring.iter_mut() {
                    repair_op(c)?;
                    c.calculate_fitness();
                }
            }

            // Apply constraint penalty to offspring if configured
            if let Some(ref constraint_fns) = self.constraint_fns {
                for c in offspring.iter_mut() {
                    let dna = c.dna();
                    let total_viol: f64 = constraint_fns.iter().map(|f| f(dna)).sum();
                    if total_viol > 0.0 {
                        match self.penalty_strategy {
                            PenaltyStrategy::None => {}
                            PenaltyStrategy::Static { coefficient } => {
                                c.set_fitness(c.fitness() + coefficient * total_viol);
                            }
                            PenaltyStrategy::Dynamic { c: dc, alpha, beta } => {
                                let penalized = crate::constraints::apply_dynamic_penalty(
                                    c.fitness(),
                                    total_viol,
                                    age,
                                    dc,
                                    alpha,
                                    beta,
                                );
                                c.set_fitness(penalized);
                            }
                            PenaltyStrategy::Adaptive { .. } => {
                                // Adaptive penalty uses the current coefficient
                                let coeff = if self.penalty_coefficient == 0.0 {
                                    0.0 // Will be initialized at generation boundary
                                } else {
                                    self.penalty_coefficient
                                };
                                c.set_fitness(c.fitness() + coeff * total_viol);
                            }
                        }
                    }
                }
            }

            //3a- Apply local search refinement to selected offspring before merge (Phase 45)
            if let Some(ref ls_config) = self.configuration.local_search_configuration {
                let strategy = ls_config.application_strategy;
                let mode = ls_config.mode;

                // Step 1: Should we apply local search this generation?
                let should_apply = match strategy {
                    LocalSearchApplicationStrategy::EveryNGenerations { interval } => {
                        interval == 0 || (i + 1) % interval == 0
                    }
                    _ => true,
                };

                if should_apply && !offspring.is_empty() {
                    // Step 2: Select candidates from offspring
                    let candidates: Vec<usize> = match strategy {
                        LocalSearchApplicationStrategy::AllOffspring => {
                            (0..offspring.len()).collect()
                        }
                        LocalSearchApplicationStrategy::BestN { n } => {
                            let mut indices: Vec<usize> = (0..offspring.len()).collect();
                            let ps = self.configuration.limit_configuration.problem_solving;
                            let k = n.min(indices.len());
                            if k > 0 {
                                indices.select_nth_unstable_by(k.saturating_sub(1), |&a, &b| {
                                    let (fa, fb) = (offspring[a].fitness(), offspring[b].fitness());
                                    match ps {
                                        ProblemSolving::Minimization
                                        | ProblemSolving::FixedFitness => {
                                            fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
                                        }
                                        ProblemSolving::Maximization => {
                                            fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
                                        }
                                    }
                                });
                            }
                            indices.truncate(k);
                            indices
                        }
                        LocalSearchApplicationStrategy::Probabilistic { probability } => {
                            let mut rng = crate::rng::make_rng();
                            (0..offspring.len())
                                .filter(|_| rng.random::<f64>() < probability)
                                .collect()
                        }
                        LocalSearchApplicationStrategy::EveryNGenerations { .. } => {
                            (0..offspring.len()).collect()
                        }
                    };

                    let is_baldwinian = matches!(mode, LocalSearchMode::Baldwinian);

                    // Save original DNA for Baldwinian restore if needed
                    let original_dnas: Vec<Vec<U::Gene>> = if is_baldwinian {
                        candidates
                            .iter()
                            .map(|&idx| offspring[idx].dna().to_vec())
                            .collect()
                    } else {
                        Vec::new()
                    };

                    let ff = Arc::clone(
                        self.fitness_fn
                            .as_ref()
                            .expect("Fitness function required when local search is configured"),
                    );
                    let search_method = ls_config.method;

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Extract candidates, process in parallel, reinsert
                        let mut selected: Vec<U> = candidates
                            .iter()
                            .map(|&idx| offspring[idx].clone())
                            .collect();
                        selected.par_iter_mut().for_each(|individual| {
                            let _ = search_method.improve(individual, ff.as_ref());
                        });
                        for (&idx, improved) in candidates.iter().zip(selected) {
                            offspring[idx] = improved;
                        }
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        candidates.iter().for_each(|&idx| {
                            let _ = search_method.improve(&mut offspring[idx], ff.as_ref());
                        });
                    }

                    // Baldwinian restore: restore original DNA, keep improved fitness
                    if is_baldwinian {
                        for (orig_pos, &idx) in candidates.iter().enumerate() {
                            if let Some(orig_dna) = original_dnas.get(orig_pos) {
                                let improved_fitness = offspring[idx].fitness();
                                offspring[idx].set_dna(std::borrow::Cow::Owned(orig_dna.clone()));
                                offspring[idx].set_fitness(improved_fitness);
                            }
                        }
                    }
                }
            }

            //3- Insert the children in the population
            self.population.add_chromosomes(&mut offspring);

            //3b- Hall of Fame update: evaluate all population chromosomes for archive entry
            if let Some(ref mut hof) = self.hall_of_fame {
                for c in self.population.chromosomes.iter() {
                    hof.try_insert(c, i as u64);
                }
            }

            //3.5- Elitism: preserve the top N individuals
            let elite = if self.configuration.elitism_count > 0 {
                extract_elite(
                    &self.population.chromosomes,
                    self.configuration.elitism_count,
                    self.configuration.limit_configuration.problem_solving,
                )
            } else {
                Vec::new()
            };

            //4- Survivor selection
            let t_surv: Option<Instant> = if self.observer.is_some() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Some(Instant::now())
                }
                #[cfg(target_arch = "wasm32")]
                {
                    None
                }
            } else {
                None
            };
            if let Some(penalty) = self.configuration.length_penalty {
                survivor::apply_parsimony_pressure(
                    self.configuration.survivor,
                    &mut self.population.chromosomes,
                    initial_population_size,
                    self.configuration.limit_configuration,
                    penalty,
                )?;
            } else {
                survivor::factory(
                    self.configuration.survivor,
                    &mut self.population.chromosomes,
                    initial_population_size,
                    self.configuration.limit_configuration,
                )?;
            }
            if let Some(t) = t_surv {
                let pop_size = self.population.chromosomes.len();
                self.notify(|obs| obs.on_survivor_selection_complete(i, t.elapsed(), pop_size));
            }

            // Reinsert elite individuals, replacing the worst survivors if needed
            if !elite.is_empty() {
                reinsert_elite(
                    &mut self.population.chromosomes,
                    elite,
                    self.configuration.limit_configuration.problem_solving,
                );
            }
            if self.configuration.adaptive_ga {
                self.population.recalculate_aga();
            }

            // Collect fitness values once per generation; reused by niching and stats.
            let mut fitness_values: Vec<f64> = self
                .population
                .chromosomes
                .iter()
                .map(|c| c.fitness())
                .collect();

            // Apply niching / fitness sharing if configured
            if let Some(ref niching_config) = self.configuration.niching_configuration {
                if niching_config.enabled {
                    // Extract DNA slices for distance computation
                    let dna_slices: Vec<&[U::Gene]> = self
                        .population
                        .chromosomes
                        .iter()
                        .map(|c| c.dna())
                        .collect();

                    // Compute fitness sharing on-the-fly (no O(n^2) matrix allocation)
                    crate::niching::sharing::apply_fitness_sharing_with_dna(
                        &mut fitness_values,
                        &dna_slices,
                        |dna_a: &[U::Gene], dna_b: &[U::Gene]| {
                            let max_len = dna_a.len().max(dna_b.len());
                            if max_len == 0 {
                                return 0.0;
                            }
                            let mut diff = 0usize;
                            for idx in 0..max_len {
                                let id_a = dna_a.get(idx).map(|g| g.id()).unwrap_or(-1);
                                let id_b = dna_b.get(idx).map(|g| g.id()).unwrap_or(-1);
                                if id_a != id_b {
                                    diff += 1;
                                }
                            }
                            diff as f64
                        },
                        niching_config.sigma_share,
                        niching_config.alpha,
                    );

                    // Write adjusted fitness back
                    for (chromosome, &shared_fitness) in self
                        .population
                        .chromosomes
                        .iter_mut()
                        .zip(fitness_values.iter())
                    {
                        chromosome.set_fitness(shared_fitness);
                    }
                }
            }

            //5- Sets the best chromosome (scan fitness_values, no second chromosome traversal)
            {
                let ps = self.configuration.limit_configuration.problem_solving;
                if !fitness_values.is_empty() {
                    let best_idx =
                        fitness_values
                            .iter()
                            .enumerate()
                            .fold(0usize, |best, (i, &f)| {
                                let best_f = fitness_values[best];
                                let is_better = match ps {
                                    ProblemSolving::Maximization | ProblemSolving::FixedFitness => {
                                        f > best_f
                                    }
                                    ProblemSolving::Minimization => f < best_f,
                                };
                                if is_better {
                                    i
                                } else {
                                    best
                                }
                            });

                    if !self.population.best_chromosome_is_set {
                        self.population.best_chromosome =
                            self.population.chromosomes[best_idx].clone();
                        self.population.best_chromosome_is_set = true;
                    } else {
                        let candidate = fitness_values[best_idx];
                        let current = self.population.best_chromosome.fitness();
                        let better = match ps {
                            ProblemSolving::Maximization | ProblemSolving::FixedFitness => {
                                candidate > current
                            }
                            ProblemSolving::Minimization => candidate < current,
                        };
                        if better {
                            self.population.best_chromosome =
                                self.population.chromosomes[best_idx].clone();
                        }
                    }
                }
            }

            // Collect per-generation statistics
            let mut gen_stats =
                GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);

            // Update dynamic mutation probability based on population diversity
            if self.configuration.mutation_configuration.dynamic_mutation {
                let target = self
                    .configuration
                    .mutation_configuration
                    .target_cardinality
                    .unwrap_or(0.5);
                let step = self
                    .configuration
                    .mutation_configuration
                    .probability_step
                    .unwrap_or(0.01);
                let p_max = self
                    .configuration
                    .mutation_configuration
                    .probability_max
                    .unwrap_or(1.0);
                let p_min = self
                    .configuration
                    .mutation_configuration
                    .probability_min
                    .unwrap_or(0.0);

                self.dynamic_mutation_probability = mutation::dynamic_probability(
                    self.dynamic_mutation_probability,
                    gen_stats.diversity,
                    target,
                    step,
                    p_max,
                    p_min,
                );

                // Set the field directly on gen_stats before push (no last_mut needed)
                gen_stats.dynamic_mutation_probability = Some(self.dynamic_mutation_probability);
            }

            // D-07: populate per-generation cache delta stats when a cache is active.
            if let Some(ref ch) = self.fitness_cache {
                let c = ch.lock().expect("fitness cache lock poisoned");
                gen_stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
                gen_stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
            }

            // D-08: populate true_fitness_calls — Some(n) when surrogate ran this generation,
            // None otherwise (mirrors the cache delta pattern above).
            gen_stats.true_fitness_calls = true_fitness_calls;

            // Apply extension strategy if configured and diversity is low
            if let Some(ref ext_config) = self.configuration.extension_configuration {
                if ext_config.method != Extension::Noop
                    && gen_stats.diversity < ext_config.diversity_threshold
                {
                    extension::factory(
                        ext_config.method,
                        &mut self.population.chromosomes,
                        initial_population_size,
                        self.configuration.limit_configuration.problem_solving,
                        ext_config,
                    )?;
                    self.notify(|obs| {
                        obs.on_extension_triggered(ExtensionEvent {
                            generation: i,
                            diversity: gen_stats.diversity,
                            extension_type: ext_config.method.as_str(),
                            threshold: ext_config.diversity_threshold,
                        })
                    });

                    // Regrow population if extension reduced it
                    if self.population.chromosomes.len() < initial_population_size {
                        if let Some(ref init_fn) = self.initialization_fn {
                            let deficit =
                                initial_population_size - self.population.chromosomes.len();
                            let chromosome_length =
                                self.configuration.limit_configuration.chromosome_length;
                            let alleles_ref: Option<&[U::Gene]> = if self.alleles.is_empty() {
                                None
                            } else {
                                Some(self.alleles.as_slice())
                            };
                            let ff = self.fitness_fn.as_ref().map(Arc::clone);

                            // For variable-length chromosomes, sample regrowth lengths from
                            // [min_observed, max_observed] of the surviving population.
                            // Decision: Phase 52 discussion log — adaptive range from survivors.
                            let (min_obs, max_obs): (usize, usize) = match chromosome_length {
                                crate::chromosomes::ChromosomeLength::Variable { min, max } => {
                                    let observed_min = self
                                        .population
                                        .chromosomes
                                        .iter()
                                        .map(|c| c.dna().len())
                                        .min()
                                        .unwrap_or(min);
                                    let observed_max = self
                                        .population
                                        .chromosomes
                                        .iter()
                                        .map(|c| c.dna().len())
                                        .max()
                                        .unwrap_or(max);
                                    // Clamp to configured bounds
                                    (observed_min.max(min), observed_max.min(max))
                                }
                                crate::chromosomes::ChromosomeLength::Fixed(n) => (n, n),
                            };

                            #[cfg(not(target_arch = "wasm32"))]
                            let new_chromosomes: Vec<U> = (0..deficit)
                                .into_par_iter()
                                .map(|_| {
                                    let len = if min_obs == max_obs {
                                        min_obs
                                    } else {
                                        let mut rng = crate::rng::make_rng();
                                        rng.random_range(min_obs..=max_obs)
                                    };
                                    let genes = init_fn(len, alleles_ref);
                                    let mut new_chromosome = U::new();
                                    new_chromosome.set_dna(std::borrow::Cow::Owned(genes));
                                    if let Some(ref ff) = ff {
                                        let ff_clone = Arc::clone(ff);
                                        new_chromosome.set_fitness_fn(move |genes| ff_clone(genes));
                                    }
                                    new_chromosome.calculate_fitness();
                                    new_chromosome.set_age(0);
                                    new_chromosome
                                })
                                .collect();
                            #[cfg(target_arch = "wasm32")]
                            let new_chromosomes: Vec<U> = (0..deficit)
                                .map(|_| {
                                    let len = if min_obs == max_obs {
                                        min_obs
                                    } else {
                                        let mut rng = crate::rng::make_rng();
                                        rng.random_range(min_obs..=max_obs)
                                    };
                                    let genes = init_fn(len, alleles_ref);
                                    let mut new_chromosome = U::new();
                                    new_chromosome.set_dna(std::borrow::Cow::Owned(genes));
                                    if let Some(ref ff) = ff {
                                        let ff_clone = Arc::clone(ff);
                                        new_chromosome.set_fitness_fn(move |genes| ff_clone(genes));
                                    }
                                    new_chromosome.calculate_fitness();
                                    new_chromosome.set_age(0);
                                    new_chromosome
                                })
                                .collect();
                            self.population.chromosomes.extend(new_chromosomes);
                        }
                    }

                    // Recalculate fitness for chromosomes marked with NaN
                    // (e.g., after MassDegeneration)
                    for c in self.population.chromosomes.iter_mut() {
                        if c.fitness().is_nan() {
                            c.calculate_fitness();
                        }
                    }
                }
            }

            // Move gen_stats into the history vec (no clone)
            self.stats.push(gen_stats);

            // Notify with the pushed stats entry that includes dynamic_mutation_probability
            // Snapshot into local to avoid nested borrow of self (notify takes &self)
            let notify_stats = self.stats.last().unwrap().clone();
            self.notify(|obs| obs.on_generation_end(&notify_stats));

            // Save checkpoint to disk if configured (requires serde feature)
            #[cfg(feature = "serde")]
            {
                let spc = &self.configuration.save_progress_configuration;
                if spc.save_progress
                    && spc.save_progress_interval > 0
                    && (i + 1) % spc.save_progress_interval == 0
                {
                    let ckpt = crate::checkpoint::Checkpoint {
                        population: self.population.clone(),
                        configuration: self.configuration.clone(),
                        generation: i,
                        stats: self.stats.clone(),
                    };
                    let path = std::path::Path::new(&spc.save_progress_path)
                        .join(format!("checkpoint_gen_{}.json", i + 1));
                    if let Err(e) = crate::checkpoint::save_checkpoint(&ckpt, &path) {
                        // Exception: this log::warn! cannot migrate to LogObserver because no
                        // on_checkpoint_failed hook exists (deferred per REQUIREMENTS.md EXT-02).
                        // It is feature-gated (#[cfg(feature = "serde")]) and only fires on I/O errors.
                        log::warn!("Failed to save checkpoint at generation {}: {}", i + 1, e);
                    }
                }
            }

            // If we want to perform a periodic callback
            if let Some(func) = &callback {
                if (generation_callback_count + 1) == generations_to_callback {
                    if func(
                        &i,
                        &self.population,
                        self.stats.last().unwrap(),
                        &self.termination_cause,
                    )
                    .is_break()
                    {
                        self.termination_cause = TerminationCause::CallbackRequested;
                        break;
                    }
                    generation_callback_count = 0;
                } else {
                    generation_callback_count += 1;
                }
            }

            //6- Identifies if the limit has been reached or not
            if limit_reached(
                self.configuration.limit_configuration,
                &self.population.chromosomes,
            ) {
                self.termination_cause = TerminationCause::FitnessTargetReached;
                if let Some(func) = &callback {
                    let _ = func(
                        &i,
                        &self.population,
                        self.stats.last().unwrap(),
                        &self.termination_cause,
                    );
                }
                break;
            }

            //7- Compound stopping criteria
            // Stagnation check
            let current_best = self.population.best_chromosome.fitness();
            let improved = match self.configuration.limit_configuration.problem_solving {
                ProblemSolving::Maximization => current_best > best_fitness_so_far,
                ProblemSolving::Minimization => current_best < best_fitness_so_far,
                _ => (current_best - best_fitness_so_far).abs() > f64::EPSILON,
            };
            if improved {
                best_fitness_so_far = current_best;
                stagnation_count = 0;
                self.notify(|obs| obs.on_new_best(i, &self.population.best_chromosome));
            } else {
                stagnation_count += 1;
                self.notify(|obs| obs.on_stagnation(i, stagnation_count));
            }

            if let Some(max_stagnation) =
                self.configuration.stagnation_generations
            {
                if stagnation_count >= max_stagnation {
                    self.termination_cause = TerminationCause::StagnationReached;
                    if let Some(func) = &callback {
                        let _ = func(
                            &i,
                            &self.population,
                            self.stats.last().unwrap(),
                            &self.termination_cause,
                        );
                    }
                    break;
                }
            }

            // Convergence check (fitness std dev below threshold)
            if let Some(threshold) = self.configuration.convergence_threshold {
                if self.stats.last().unwrap().fitness_std_dev < threshold {
                    self.termination_cause = TerminationCause::ConvergenceReached;
                    if let Some(func) = &callback {
                        let _ = func(
                            &i,
                            &self.population,
                            self.stats.last().unwrap(),
                            &self.termination_cause,
                        );
                    }
                    break;
                }
            }

            // Time limit check (not available on wasm32 — see warning emitted at run start)
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(max_secs) = self.configuration.max_duration_secs {
                if start_time.elapsed().as_secs_f64() >= max_secs {
                    self.termination_cause = TerminationCause::TimeLimitReached;
                    if let Some(func) = &callback {
                        let _ = func(
                            &i,
                            &self.population,
                            self.stats.last().unwrap(),
                            &self.termination_cause,
                        );
                    }
                    break;
                }
            }
        }

        // Set termination cause when generation limit is reached (regardless of callback)
        if self.termination_cause == TerminationCause::NotTerminated {
            self.termination_cause = TerminationCause::GenerationLimitReached;
        }

        self.notify(|obs| obs.on_run_end(self.termination_cause, &self.stats));

        // If we want to perform a callback and the generation limit was just reached
        if let Some(func) = &callback {
            if self.termination_cause == TerminationCause::GenerationLimitReached {
                let final_stats = self.stats.last().cloned().unwrap_or_else(|| {
                    GenerationStats::from_fitness_values(0, &[], is_maximization)
                });
                let _ = func(
                    &self.configuration.limit_configuration.max_generations,
                    &self.population,
                    &final_stats,
                    &self.termination_cause,
                );
            }
        }

        Ok(&self.population)
    }

    /// Returns per-generation statistics collected during the last run.
    ///
    /// The vector is populated during `run()` / `run_with_callback()` and cleared
    /// at the start of each new run. Each entry corresponds to one generation.
    pub fn stats(&self) -> &[GenerationStats] {
        &self.stats
    }

    /// Returns the Hall of Fame / solution archive, if configured.
    ///
    /// Returns `None` if no Hall of Fame was configured.
    /// Returns `Some(&HallOfFame<U>)` if configured, populated with the top-N
    /// unique solutions encountered across all generations during the run.
    pub fn hall_of_fame(&self) -> Option<&HallOfFame<U>> {
        self.hall_of_fame.as_ref()
    }

    // ---------------------------------------------------------------------------
    // Constraint handling helpers
    // ---------------------------------------------------------------------------

    /// Applies constraint violation penalty to the entire population.
    ///
    /// Called after initial fitness calculation and after offspring are added to
    /// the population each generation. Modifies fitness values in-place according
    /// to the configured penalty strategy or feasibility rules.
    fn process_constraints_population(&mut self, generation: usize) -> Result<(), GaError> {
        let constraint_fns = match self.constraint_fns {
            Some(ref fns) => fns,
            None => return Ok(()),
        };

        // Compute constraint violations for all chromosomes
        let violations: Vec<f64> = self
            .population
            .chromosomes
            .iter()
            .map(|c| {
                let dna = c.dna();
                constraint_fns.iter().map(|f| f(dna)).sum()
            })
            .collect();

        // Apply feasibility rules or penalty strategy
        match self.constraint_handling {
            Some(ConstraintHandling::FeasibilityRules) => {
                self.apply_feasibility_rules(&violations);
            }
            None => {
                self.apply_penalty_to_chromosomes(&violations, generation);
            }
        }

        Ok(())
    }

    /// Applies feasibility rules to modify fitness values so that feasible
    /// individuals always compare better than infeasible ones, and infeasible
    /// individuals are ordered by total violation.
    fn apply_feasibility_rules(&mut self, violations: &[f64]) {
        let (feasible_count, worst_feasible) = {
            let mut wf = f64::NEG_INFINITY;
            let mut bf = f64::INFINITY;
            let mut count = 0usize;
            for (c, &v) in self.population.chromosomes.iter().zip(violations.iter()) {
                if v <= 0.0 {
                    count += 1;
                    let f = c.fitness();
                    if f > wf {
                        wf = f;
                    }
                    if f < bf {
                        bf = f;
                    }
                }
            }
            (count, wf)
        };

        let is_maximization = matches!(
            self.configuration.limit_configuration.problem_solving,
            ProblemSolving::Maximization
        );

        for (c, &v) in self
            .population
            .chromosomes
            .iter_mut()
            .zip(violations.iter())
        {
            if v > 0.0 {
                // Infeasible — encode violation so that:
                // - All feasible beat all infeasible
                // - Within infeasible, lower violation is better
                if feasible_count > 0 {
                    if is_maximization {
                        // For maximization: feasible values are higher, so infeasible
                        // get fitness = worst_feasible - violation (lower = worse)
                        c.set_fitness(worst_feasible - v);
                    } else {
                        // For minimization: feasible values are lower, so infeasible
                        // get fitness = worst_feasible + violation (higher = worse)
                        c.set_fitness(worst_feasible + v);
                    }
                } else {
                    // No feasible solutions — sort by violation only
                    if is_maximization {
                        c.set_fitness(-v);
                    } else {
                        c.set_fitness(v);
                    }
                }
            }
        }
    }

    /// Applies the configured penalty strategy to all chromosomes.
    fn apply_penalty_to_chromosomes(&mut self, violations: &[f64], generation: usize) {
        match self.penalty_strategy {
            PenaltyStrategy::None => {}
            PenaltyStrategy::Static { coefficient } => {
                for (c, &v) in self
                    .population
                    .chromosomes
                    .iter_mut()
                    .zip(violations.iter())
                {
                    if v > 0.0 {
                        c.set_fitness(c.fitness() + coefficient * v);
                    }
                }
            }
            PenaltyStrategy::Dynamic { c, alpha, beta } => {
                for (chr, &v) in self
                    .population
                    .chromosomes
                    .iter_mut()
                    .zip(violations.iter())
                {
                    if v > 0.0 {
                        let raw = chr.fitness();
                        let penalized = crate::constraints::apply_dynamic_penalty(
                            raw, v, generation, c, alpha, beta,
                        );
                        chr.set_fitness(penalized);
                    }
                }
            }
            PenaltyStrategy::Adaptive {
                initial_coefficient,
                window_size,
            } => {
                let coeff = if self.penalty_coefficient == 0.0 {
                    initial_coefficient
                } else {
                    self.penalty_coefficient
                };
                // Track feasibility of best individual for adaptive adjustment
                if generation > 0 && generation % window_size == 0 {
                    let best_violation = violations
                        .iter()
                        .zip(self.population.chromosomes.iter())
                        .find(|(_, c)| {
                            c.fitness()
                                == self
                                    .population
                                    .chromosomes
                                    .iter()
                                    .map(|x| x.fitness())
                                    .fold(f64::NEG_INFINITY, |a, b| a.max(b))
                        })
                        .map(|(v, _)| *v)
                        .unwrap_or(0.0);
                    self.adaptive_penalty_counter = if best_violation <= 0.0 {
                        self.adaptive_penalty_counter + 1
                    } else {
                        self.adaptive_penalty_counter - 1
                    };
                    if self.adaptive_penalty_counter > 0 {
                        // Best has been feasible — increase penalty pressure
                        let new_coeff = self.penalty_coefficient * 1.1;
                        self.penalty_coefficient = new_coeff;
                    } else if self.adaptive_penalty_counter < 0 {
                        // Best has been infeasible — decrease penalty pressure
                        let new_coeff = (self.penalty_coefficient / 1.1).max(0.001);
                        self.penalty_coefficient = new_coeff;
                    }
                }

                for (c, &v) in self
                    .population
                    .chromosomes
                    .iter_mut()
                    .zip(violations.iter())
                {
                    if v > 0.0 {
                        c.set_fitness(c.fitness() + coeff * v);
                    }
                }
            }
        }
    }
}

/// Evaluates `pop` using a batch evaluator, with optional LRU cache partitioning.
///
/// Free function used by the GA run loop to avoid Rust's borrow checker conflict
/// when `pop` is a field of the same struct that also owns `batch_evaluator` and
/// `fitness_cache`.
///
/// # Cases
///
/// - `cache_opt = None`: evaluates every chromosome via `evaluator.evaluate_batch(pop)`.
/// - `cache_opt = Some(cache)`: performs the D-06 hit/miss partition — only cache misses
///   are sent to `evaluate_batch`; hits are returned from the cache directly. The cache
///   `Mutex` is released before the `evaluate_batch` call (Pitfall 2 / T-60-05).
fn batch_evaluate<U>(
    evaluator: Arc<dyn crate::fitness::BatchFitnessEvaluator<U> + Send + Sync>,
    cache_opt: Option<Arc<std::sync::Mutex<crate::fitness::cache::FitnessCache>>>,
    pop: &mut [U],
) -> Result<(), GaError>
where
    U: LinearChromosome + Clone,
    U::Gene: Debug,
{
    if pop.is_empty() {
        return Ok(());
    }

    match cache_opt {
        None => {
            // Case B: evaluator set, no cache — batch-evaluate everything
            let values = evaluator.evaluate_batch(pop);
            debug_assert_eq!(
                values.len(),
                pop.len(),
                "evaluate_batch returned {} values for {} chromosomes (T-60-01)",
                values.len(),
                pop.len()
            );
            for (i, chromosome) in pop.iter_mut().enumerate() {
                chromosome.set_fitness(values[i]);
            }
        }
        Some(cache_handle) => {
            // Case C: evaluator + cache — D-06 partition algorithm
            let mut fitness_values: Vec<f64> = vec![0.0; pop.len()];
            let mut miss_indices: Vec<usize> = Vec::new();

            // Step 1: check cache for each chromosome; collect misses
            {
                let mut cache = cache_handle
                    .lock()
                    .expect("fitness cache lock poisoned");
                for (i, chromosome) in pop.iter().enumerate() {
                    let key = crate::fitness::cache::hash_dna(chromosome.dna());
                    match cache.get(key) {
                        Some(f) => fitness_values[i] = f,
                        None => miss_indices.push(i),
                    }
                }
            } // Lock released here (Pitfall 2 — never hold lock across evaluate_batch)

            if !miss_indices.is_empty() {
                // Step 2: clone only the miss chromosomes (D-01: evaluate_batch takes &[U])
                let miss_chromosomes: Vec<U> = miss_indices
                    .iter()
                    .map(|&orig_i| pop[orig_i].clone())
                    .collect();

                // Step 3: batch-evaluate misses
                let miss_values = evaluator.evaluate_batch(&miss_chromosomes);
                debug_assert_eq!(
                    miss_values.len(),
                    miss_indices.len(),
                    "evaluate_batch returned {} values for {} miss chromosomes (T-60-01)",
                    miss_values.len(),
                    miss_indices.len()
                );

                // Step 4: re-acquire cache and store miss results
                {
                    let mut cache = cache_handle
                        .lock()
                        .expect("fitness cache lock poisoned");
                    for (pos, &orig_i) in miss_indices.iter().enumerate() {
                        let f = miss_values[pos];
                        fitness_values[orig_i] = f;
                        let key = crate::fitness::cache::hash_dna(pop[orig_i].dna());
                        cache.put(key, f);
                    }
                } // Lock released
            }

            // Step 5: write all fitness values (hits + misses) back into the population
            for (i, chromosome) in pop.iter_mut().enumerate() {
                chromosome.set_fitness(fitness_values[i]);
            }
        }
    }

    Ok(())
}

/// Checks termination limits according to `LimitConfiguration`.
///
/// - For Minimization: stops when any chromosome has fitness exactly `0.0`.
/// - For FixedFitness: stops when any chromosome has fitness exactly `fitness_target`.
fn limit_reached<U>(limit: LimitConfiguration, chromosomes: &[U]) -> bool
where
    U: LinearChromosome,
{
    let mut result = false;

    if limit.problem_solving == ProblemSolving::Minimization {
        //If the problem-solving is minimization, fitness must be 0
        for chromosome in chromosomes {
            if chromosome.fitness() == 0.0 {
                result = true;
                break;
            }
        }
    } else if limit.problem_solving == ProblemSolving::FixedFitness {
        //If the problem-solving is a fixed fitness
        if let Some(target) = limit.fitness_target {
            for chromosome in chromosomes {
                if chromosome.fitness() == target {
                    result = true;
                    break;
                }
            }
        }
    }

    result
}

/// AOS and fitness parameters bundled for `parent_crossover` (D-07).
///
/// Groups the Adaptive Operator Selection state, operator portfolios,
/// fitness-context values, and per-offspring age assignment — the args that
/// do not belong to the core population/configuration inputs — so the
/// function signature stays below Clippy's `too_many_arguments` limit (7).
struct ParentCrossoverParams<'a, U: LinearChromosome> {
    /// Age assigned to each produced offspring.
    age: usize,
    /// Population-level maximum fitness (used by AGA crossover probability).
    f_max: f64,
    /// Population-level average fitness (used by AGA probability formulas).
    f_avg: f64,
    /// Dynamic mutation probability override (None → use configured static probability).
    dynamic_mutation_prob: Option<f64>,
    /// Current generation index (used by AOS selection strategy).
    generation: usize,
    /// Current best fitness across the population (used by AOS reward).
    best_fitness: f64,
    /// Whether the problem is a maximization problem (for AOS reward sign).
    is_maximization: bool,
    /// Per-chromosome fitness function (None in batch mode).
    fitness_fn: Option<Arc<FitnessFn<U::Gene>>>,
    /// Optional AOS crossover portfolio.
    crossover_portfolio: Option<&'a Vec<Crossover>>,
    /// Optional AOS mutation portfolio.
    mutation_portfolio: Option<&'a Vec<Mutation>>,
    /// Optional shared AOS crossover state.
    aos_crossover_state: Option<&'a Mutex<AosState>>,
    /// Optional shared AOS mutation state.
    aos_mutation_state: Option<&'a Mutex<AosState>>,
}

/// Performs parent crossover using the configured crossover and mutation strategies.
///
/// Behavior:
/// - Splits work among threads considering available parent pairs.
/// - Computes adaptive probabilities when enabled; otherwise uses static ones.
/// - Produces children, mutates them, computes their fitness, and returns the offspring.
fn parent_crossover<U>(
    parents: &[Vec<usize>],
    chromosomes: &[U],
    configuration: &GaConfiguration,
    params: ParentCrossoverParams<'_, U>,
) -> Result<Vec<U>, GaError>
where
    U: LinearChromosome + Send + Sync + 'static + Clone + mutation::ValueMutable,
{
    // Destructure the population-level and AOS params bundle (D-07)
    let ParentCrossoverParams {
        age,
        f_max,
        f_avg,
        dynamic_mutation_prob,
        generation,
        best_fitness,
        is_maximization,
        fitness_fn,
        crossover_portfolio,
        mutation_portfolio,
        aos_crossover_state,
        aos_mutation_state,
    } = params;

    /*
        Gets the static crossover probability config and the static mutation probability config
        This way we avoid of passing by these conditions at each thread if it's not necessary
    */
    let crossover_probability_config =
        if let Some(p) = configuration.crossover_configuration.probability_max {
            if !configuration.adaptive_ga {
                Some(p)
            } else {
                None
            }
        } else {
            Some(1.0)
        };

    let mutation_probability_config = if let Some(dp) = dynamic_mutation_prob {
        // Dynamic mutation overrides static probability
        Some(dp)
    } else if let Some(p) = configuration.mutation_configuration.probability_max {
        if !configuration.adaptive_ga {
            Some(p)
        } else {
            None
        }
    } else {
        Some(1.0)
    };

    // Create AOS reward accumulators (Phase 43)
    // These are shared across rayon threads via Arc<Mutex<Vec<(usize, f64)>>>
    let crossover_reward_acc: RewardAccumulator = if aos_crossover_state.is_some() {
        Some(Arc::new(Mutex::new(Vec::new())))
    } else {
        None
    };
    let mutation_reward_acc: RewardAccumulator = if aos_mutation_state.is_some() {
        Some(Arc::new(Mutex::new(Vec::new())))
    } else {
        None
    };

    // Shared per-group closure: produces children from one N-ary parent group.
    // cfg-gated only for the iterator kind (par_iter on native, iter on wasm32).
    let process_pair = |group: &Vec<usize>| -> Result<Vec<U>, GaError> {
        let mut rng = crate::rng::make_rng();

        // T-54-01: guard against out-of-bounds or too-small group (minimum 2 parents)
        if group.len() < 2 {
            return Err(GaError::SelectionError(format!(
                "Selection group has fewer than 2 parents (got {})",
                group.len()
            )));
        }
        let key = group[0];
        let value = group[1];

        // Getting the parent 1 and 2 for crossover
        let parent_1 = chromosomes.get(key).ok_or_else(|| {
            GaError::SelectionError(format!(
                "Selection returned out-of-bounds index {} (population size {})",
                key,
                chromosomes.len()
            ))
        })?;
        let parent_2 = chromosomes.get(value).ok_or_else(|| {
            GaError::SelectionError(format!(
                "Selection returned out-of-bounds index {} (population size {})",
                value,
                chromosomes.len()
            ))
        })?;

        // Select operators via AOS if portfolios are configured (Phase 43)
        // AOS returns (operator_index, operator_enum) for reward tracking
        let selected_crossover: Option<(usize, Crossover)> = if let (
            Some(portfolio),
            Some(aos_state),
        ) =
            (crossover_portfolio, aos_crossover_state)
        {
            let mut state = aos_state.lock().unwrap();
            let op_idx = state.select_operator(&mut rng, generation);
            Some((op_idx, portfolio[op_idx]))
        } else {
            None
        };

        let selected_mutation: Option<(usize, Mutation)> =
            if let (Some(portfolio), Some(aos_state)) = (mutation_portfolio, aos_mutation_state) {
                let mut state = aos_state.lock().unwrap();
                let op_idx = state.select_operator(&mut rng, generation);
                Some((op_idx, portfolio[op_idx].clone()))
            } else {
                None
            };

        // Making the crossover of the parents when the random number is below or equal to the given probability
        let crossover_probability = rng.random_range(0.0..1.0);
        let effective_crossover_prob = if let Some(p) = crossover_probability_config {
            p
        } else {
            crossover::aga_probability(
                parent_1,
                parent_2,
                f_max,
                f_avg,
                configuration
                    .crossover_configuration
                    .probability_max
                    .unwrap_or(1.0),
                configuration
                    .crossover_configuration
                    .probability_min
                    .unwrap_or(0.0),
            )
        };

        // Making the mutation of each child when the random number is below or equal the given probability
        let mut mutation_probability = rng.random_range(0.0..1.0);
        let effective_mutation_prob = if let Some(p) = mutation_probability_config {
            p
        } else {
            mutation::aga_probability(
                parent_1,
                parent_2,
                f_avg,
                configuration
                    .mutation_configuration
                    .probability_max
                    .unwrap_or(1.0),
                configuration
                    .mutation_configuration
                    .probability_min
                    .unwrap_or(0.0),
            )
        };

        let mut child_1: U;
        let mut child_2: U;

        if crossover_probability <= effective_crossover_prob {
            // Determine the effective crossover method (AOS-selected or user-configured)
            let effective_method = selected_crossover
                .map(|(_, op)| op)
                .unwrap_or(configuration.crossover_configuration.method);

            // Dispatch crossover by group size: groups of 2 use the standard 2-parent path;
            // larger groups use the multi-parent dispatch (UNDX/SPX/PCX via group.len() > 2).
            let mut children = if group.len() > 2 {
                // Multi-parent crossover path: collect all parents from the group
                let mut parent_refs: Vec<&U> = Vec::with_capacity(group.len());
                for &idx in group.iter() {
                    let p = chromosomes.get(idx).ok_or_else(|| {
                        GaError::SelectionError(format!(
                            "Selection returned out-of-bounds index {} (population size {})",
                            idx,
                            chromosomes.len()
                        ))
                    })?;
                    parent_refs.push(p);
                }
                let mut cx_config = configuration.crossover_configuration;
                cx_config.method = effective_method;
                // Returns 1 offspring per D-04 (single-offspring contract)
                crossover::factory_multi_parent_dispatch(&parent_refs, cx_config)?
            } else {
                // Standard 2-parent crossover path — all variants with group.len() == 2
                let mut cx_config = configuration.crossover_configuration;
                cx_config.method = effective_method;
                crossover::factory(parent_1, parent_2, cx_config)?
            };

            // factory_multi_parent_dispatch returns 1 child; factory returns 2.
            // For the 1-child path, child_1 gets the actual offspring; child_2 falls back to
            // parent_1.clone() (D-04 / Pitfall 1). For the 2-child path, both pops succeed.
            child_1 = children.pop().ok_or_else(|| {
                GaError::CrossoverError("Crossover returned no children".to_string())
            })?;
            child_2 = children.pop().unwrap_or_else(|| parent_1.clone());
        } else {
            child_1 = parent_1.clone();
            child_2 = parent_2.clone();
        }

        // Determine mutation method: AOS-selected or configured single operator
        let selected_mutation_idx = selected_mutation.as_ref().map(|(idx, _)| *idx);
        let mutation_method = selected_mutation
            .map(|(_, op)| op)
            .unwrap_or_else(|| configuration.mutation_configuration.method.clone());

        if mutation_probability <= effective_mutation_prob {
            match &mutation_method {
                Mutation::Differential { f } => {
                    let f_val = f.unwrap_or(0.5);
                    crate::operations::mutation::differential::differential_mutation(
                        &mut child_1,
                        chromosomes,
                        key,
                        f_val,
                    )?;
                }
                Mutation::Insertion | Mutation::Deletion => {
                    mutation::factory_with_chromosome_length(
                        mutation_method.clone(),
                        &mut child_1,
                        Some(configuration.limit_configuration.chromosome_length),
                        None,
                        None,
                    )?;
                }
                _ => {
                    mutation_method.mutate(&mut child_1, &mutation_method)?;
                }
            }
        }

        mutation_probability = rng.random_range(0.0..1.0);
        if mutation_probability <= effective_mutation_prob {
            match &mutation_method {
                Mutation::Differential { f } => {
                    let f_val = f.unwrap_or(0.5);
                    crate::operations::mutation::differential::differential_mutation(
                        &mut child_2,
                        chromosomes,
                        value,
                        f_val,
                    )?;
                }
                Mutation::Insertion | Mutation::Deletion => {
                    mutation::factory_with_chromosome_length(
                        mutation_method.clone(),
                        &mut child_2,
                        Some(configuration.limit_configuration.chromosome_length),
                        None,
                        None,
                    )?;
                }
                _ => {
                    mutation_method.mutate(&mut child_2, &mutation_method)?;
                }
            }
        }

        // Inject fitness function into children built via U::new() (which start with the
        // default no-op fitness fn). Children from parent.clone() (the else branch above)
        // already carry the correct fitness fn from their parent.
        if let Some(ref ff) = fitness_fn {
            let ff1 = Arc::clone(ff);
            child_1.set_fitness_fn(move |genes| ff1(genes));
            let ff2 = Arc::clone(ff);
            child_2.set_fitness_fn(move |genes| ff2(genes));
        }

        // Calculate the fitness of both children and set their age
        child_1.calculate_fitness();
        child_2.calculate_fitness();

        child_1.set_age(age);
        child_2.set_age(age);

        // Accumulate AOS rewards (Phase 43)
        // Crossover reward: compare parent vs child fitness
        if let Some(ref acc) = crossover_reward_acc {
            if let Some((c_op_idx, _)) = selected_crossover {
                let (p, c) = if is_maximization {
                    (child_1.fitness(), parent_1.fitness())
                } else {
                    (parent_1.fitness(), child_1.fitness())
                };
                let reward = crate::aos::compute_normalized_reward(p, c, best_fitness);
                acc.lock().unwrap().push((c_op_idx, reward));
            }
        }
        // Mutation reward: compare parent vs child fitness
        if let Some(ref acc) = mutation_reward_acc {
            if let Some(m_op_idx) = selected_mutation_idx {
                let (p, c) = if is_maximization {
                    (child_1.fitness(), parent_1.fitness())
                } else {
                    (parent_1.fitness(), child_1.fitness())
                };
                let reward = crate::aos::compute_normalized_reward(p, c, best_fitness);
                acc.lock().unwrap().push((m_op_idx, reward));
            }
        }

        Ok(vec![child_1, child_2])
    };

    // Use rayon to process parent pairs in parallel (sequential fallback on wasm32)
    #[cfg(not(target_arch = "wasm32"))]
    let results: Vec<Result<Vec<U>, GaError>> = parents.par_iter().map(process_pair).collect();
    #[cfg(target_arch = "wasm32")]
    let results: Vec<Result<Vec<U>, GaError>> = parents.iter().map(process_pair).collect();

    // Check for any errors and flatten the results
    let mut offspring = Vec::new();
    for result in results {
        offspring.extend(result?);
    }

    // Apply AOS reward updates after collecting all rewards (Phase 43)
    if let Some(acc) = crossover_reward_acc {
        let rewards = acc.lock().unwrap().drain(..).collect::<Vec<_>>();
        if !rewards.is_empty() {
            if let Some(aos_state) = aos_crossover_state {
                let mut state = aos_state.lock().unwrap();
                state.record_rewards(&rewards);
                state.update();
            }
        }
    }
    if let Some(acc) = mutation_reward_acc {
        let rewards = acc.lock().unwrap().drain(..).collect::<Vec<_>>();
        if !rewards.is_empty() {
            if let Some(aos_state) = aos_mutation_state {
                let mut state = aos_state.lock().unwrap();
                state.record_rewards(&rewards);
                state.update();
            }
        }
    }

    Ok(offspring)
}

/// Extracts the top `count` individuals from the population by fitness.
///
/// Only clones the selected elite individuals instead of the whole population.
fn extract_elite<U: LinearChromosome>(
    chromosomes: &[U],
    count: usize,
    problem_solving: ProblemSolving,
) -> Vec<U> {
    if count == 0 || chromosomes.is_empty() {
        return Vec::new();
    }
    let k = count.min(chromosomes.len());

    // Build index array and partially sort so the best `k` are at the front.
    let mut indices: Vec<usize> = (0..chromosomes.len()).collect();
    let cmp_fn = |a: &usize, b: &usize| {
        let cmp = chromosomes[*a]
            .fitness()
            .partial_cmp(&chromosomes[*b].fitness())
            .unwrap_or(std::cmp::Ordering::Equal);
        match problem_solving {
            ProblemSolving::Maximization => cmp.reverse(),
            _ => cmp,
        }
    };
    indices.select_nth_unstable_by(k - 1, cmp_fn);
    // The first `k` elements are the best (unordered among themselves).
    indices.truncate(k);

    indices.iter().map(|&i| chromosomes[i].clone()).collect()
}

/// Reinserts elite individuals into the population, replacing the worst if already at capacity.
fn reinsert_elite<U: LinearChromosome>(
    chromosomes: &mut [U],
    elite: Vec<U>,
    problem_solving: ProblemSolving,
) {
    let k = elite.len().min(chromosomes.len());
    if k == 0 {
        return;
    }

    // Partition so the k worst chromosomes end up at indices 0..k (O(n) instead of O(n log n)).
    // The comparator puts the worst individuals first:
    //   - Maximization: natural order (lower fitness first) = worst first
    //   - Minimization/FixedFitness: reversed order (higher fitness first) = worst first
    chromosomes.select_nth_unstable_by(k - 1, |a, b| {
        let cmp = a
            .fitness()
            .partial_cmp(&b.fitness())
            .unwrap_or(std::cmp::Ordering::Equal);
        match problem_solving {
            ProblemSolving::Maximization => cmp,
            _ => cmp.reverse(),
        }
    });

    // Overwrite the k worst slots with the elite individuals.
    for (i, elite_individual) in elite.into_iter().take(k).enumerate() {
        chromosomes[i] = elite_individual;
    }
}

impl<U> Strategy<U> for Ga<U>
where
    U: LinearChromosome
        + Send
        + Sync
        + 'static
        + Clone
        + Debug
        + mutation::ValueMutable
        + MaybeSerialize
        + MaybeDeserialize
        + OperatorCompat,
    U::Gene: 'static + Debug,
{
    fn run(&mut self) -> Result<(), GaError> {
        Ga::run(self).map(|_| ())
    }

    fn best(&self) -> Option<&U> {
        if self.population.best_chromosome_is_set {
            Some(&self.population.best_chromosome)
        } else {
            None
        }
    }
}

impl<U> Ga<U>
where
    U: LinearChromosome
        + VectorFitness
        + Send
        + Sync
        + 'static
        + Clone
        + Debug
        + mutation::ValueMutable
        + MaybeSerialize
        + MaybeDeserialize
        + OperatorCompat,
    U::Gene: 'static + Debug,
{
    /// Selects parents using lexicase or epsilon-lexicase selection.
    ///
    /// Call this instead of the standard `run()` selection step when `U:` [`VectorFitness`]
    /// and `Selection::Lexicase` or `Selection::EpsilonLexicase` is configured.
    /// Also syncs each chromosome's scalar fitness to the mean of its case scores (D-04).
    ///
    /// # Errors
    ///
    /// Returns `GaError::SelectionError` if the population is too small,
    /// case fitness is unset, or any NaN case scores are found.
    /// Returns `GaError::ConfigurationError` if the configured selection method is
    /// not `Lexicase` or `EpsilonLexicase`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Selection::Lexicase / EpsilonLexicase reach run() only when U does not implement
    /// // VectorFitness; factory() returns GaError::ConfigurationError for those variants per D-06.
    /// // Users with VectorFitness chromosomes call select_parents_lexicase() directly.
    /// let pairs = ga.select_parents_lexicase()?;
    /// ```
    pub fn select_parents_lexicase(&mut self) -> Result<Vec<Vec<usize>>, GaError> {
        crate::operations::selection::factory_lexicase(
            &mut self.population.chromosomes,
            self.configuration.selection_configuration,
            self.configuration.number_of_threads,
        )
    }
}
