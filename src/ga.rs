//! Main genetic algorithm orchestrator.
//!
//! This module contains the [`Ga`] struct, the central entry point for
//! configuring and running a single-objective genetic algorithm. It coordinates
//! the full evolutionary cycle: initialization, selection, crossover, mutation,
//! survivor selection, and fitness evaluation.
//!
//! # Quick start
//!
//! ```ignore
//! use genetic_algorithms::prelude::*;
//!
//! let mut ga = Ga::new()
//!     .with_population_size(100)
//!     .with_max_generations(500)
//!     .with_genes_per_chromosome(8)
//!     .with_fitness_fn(|dna: &[MyGene]| { /* return fitness */ 0.0 })
//!     .with_initialization_fn(generic_random_initialization)
//!     .build()?;
//!
//! let population = ga.run()?;
//! println!("Best: {:?}", population.best_chromosome);
//! ```
//!
//! See also: [`crate::island`] for multi-population island models, and
//! [`crate::nsga2`] for multi-objective optimization.

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::observer::{ExtensionEvent, GaObserver};
#[allow(deprecated)]
use crate::reporter::Reporter;
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, InitializationFn};
use std::time::Duration;
use crate::validators::validator_factory as ValidatorFactory;
use crate::{
    configuration::{LimitConfiguration, LogLevel, ProblemSolving},
    operations::{crossover, extension, mutation, selection, survivor, Extension},
    population::Population,
    traits::{
        ChromosomeT, ConfigurationT, CrossoverConfig, ElitismConfig, ExtensionConfig, GeneT,
        MutationConfig, NichingConfig, SelectionConfig, StoppingConfig,
    },
};
use log::{debug, info, trace};
use rand::Rng;
use rayon::prelude::*;
use std::fmt::Debug;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Instant;

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

#[cfg(not(feature = "serde"))]
pub trait MaybeSerialize {}
#[cfg(not(feature = "serde"))]
impl<T> MaybeSerialize for T {}

/// Indicates why a GA run terminated.
///
/// - `GenerationLimitReached`: the maximum number of generations was reached.
/// - `FitnessTargetReached`: a stopping criterion based on fitness was satisfied.
/// - `StagnationReached`: no fitness improvement for N consecutive generations.
/// - `ConvergenceReached`: fitness standard deviation dropped below threshold.
/// - `TimeLimitReached`: elapsed wall-clock time exceeded the configured limit.
/// - `CallbackRequested`: the user callback returned `ControlFlow::Break`.
/// - `NotTerminated`: internal state before the run finalizes or if a callback is invoked mid-run.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TerminationCause {
    GenerationLimitReached,
    FitnessTargetReached,
    StagnationReached,
    ConvergenceReached,
    TimeLimitReached,
    CallbackRequested,
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
#[allow(deprecated)]
pub struct Ga<U>
where
    U: ChromosomeT,
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

    /// Optional lifecycle reporter. When `None` (the default), no hook
    /// calls are made and there is zero overhead.
    reporter: Option<Box<dyn Reporter<U> + Send>>,

    /// Optional structured lifecycle observer. When `None` (the default),
    /// no hook calls or timing measurements are performed (zero overhead).
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

#[allow(deprecated)]
impl<U> Default for Ga<U>
where
    U: ChromosomeT,
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
            reporter: None,
            observer: None,
        }
    }
}

impl<U> SelectionConfig for Ga<U>
where
    U: ChromosomeT,
{
    fn with_number_of_couples(mut self, number_of_couples: usize) -> Self {
        self.configuration.selection_configuration.number_of_couples = number_of_couples;
        self
    }
    fn with_selection_method(mut self, selection_method: crate::operations::Selection) -> Self {
        self.configuration.selection_configuration.method = selection_method;
        self
    }
}

impl<U> CrossoverConfig for Ga<U>
where
    U: ChromosomeT,
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
}

impl<U> MutationConfig for Ga<U>
where
    U: ChromosomeT,
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
    fn with_mutation_step(mut self, step: f64) -> Self {
        self.configuration.mutation_configuration.step = Some(step);
        self
    }
    fn with_mutation_sigma(mut self, sigma: f64) -> Self {
        self.configuration.mutation_configuration.sigma = Some(sigma);
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
    U: ChromosomeT,
{
    fn with_max_generations(mut self, max_generations: usize) -> Self {
        self.configuration.limit_configuration.max_generations = max_generations;
        self
    }
    fn with_fitness_target(mut self, fitness_target: f64) -> Self {
        self.configuration.limit_configuration.fitness_target = Some(fitness_target);
        self
    }
    fn with_stopping_criteria(mut self, criteria: crate::configuration::StoppingCriteria) -> Self {
        self.configuration.stopping_criteria = criteria;
        self
    }
}

impl<U> NichingConfig for Ga<U>
where
    U: ChromosomeT,
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
    U: ChromosomeT,
{
    fn with_elitism(mut self, elitism_count: usize) -> Self {
        self.configuration.elitism_count = elitism_count;
        self
    }
}

impl<U> ExtensionConfig for Ga<U>
where
    U: ChromosomeT,
{
    fn with_extension_method(mut self, method: crate::operations::Extension) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(
                crate::extension::configuration::ExtensionConfiguration::default,
            )
            .method = method;
        self
    }
    fn with_extension_diversity_threshold(mut self, threshold: f64) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(
                crate::extension::configuration::ExtensionConfiguration::default,
            )
            .diversity_threshold = threshold;
        self
    }
    fn with_extension_survival_rate(mut self, rate: f64) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(
                crate::extension::configuration::ExtensionConfiguration::default,
            )
            .survival_rate = rate;
        self
    }
    fn with_extension_mutation_rounds(mut self, rounds: usize) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(
                crate::extension::configuration::ExtensionConfiguration::default,
            )
            .mutation_rounds = rounds;
        self
    }
    fn with_extension_elite_count(mut self, count: usize) -> Self {
        self.configuration
            .extension_configuration
            .get_or_insert_with(
                crate::extension::configuration::ExtensionConfiguration::default,
            )
            .elite_count = count;
        self
    }
}

impl<U> ConfigurationT for Ga<U>
where
    U: ChromosomeT,
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
    fn with_logs(mut self, log_level: LogLevel) -> Self {
        self.configuration.log_level = log_level;
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
    fn with_genes_per_chromosome(mut self, genes_per_chromosome: usize) -> Self {
        self.configuration.limit_configuration.genes_per_chromosome = genes_per_chromosome;
        self
    }
    fn with_needs_unique_ids(mut self, needs_unique_ids: bool) -> Self {
        self.configuration.limit_configuration.needs_unique_ids = needs_unique_ids;
        self
    }
    fn with_alleles_can_be_repeated(mut self, alleles_can_be_repeated: bool) -> Self {
        self.configuration
            .limit_configuration
            .alleles_can_be_repeated = alleles_can_be_repeated;
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
}

impl<U> Ga<U>
where
    U: ChromosomeT
        + Send
        + Sync
        + 'static
        + Clone
        + Debug
        + mutation::ValueMutable
        + MaybeSerialize,
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

        // Wrap fitness function with LRU cache if configured
        if let Some(cache_size) = self.fitness_cache_size {
            if let Some(fitness_fn) = self.fitness_fn.take() {
                self.fitness_fn =
                    Some(crate::fitness::cache::wrap_with_cache(fitness_fn, cache_size));
            }
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

    /// Attaches a lifecycle reporter that receives hooks during execution.
    ///
    /// See [`Reporter`](crate::reporter::Reporter) for the hook contract.
    #[allow(deprecated)]
    #[deprecated(
        since = "2.2.0",
        note = "use with_observer() instead. Reporter will be removed in v3.0.0."
    )]
    pub fn with_reporter(mut self, reporter: Box<dyn Reporter<U> + Send>) -> Self {
        self.reporter = Some(reporter);
        self
    }

    /// Attaches a structured lifecycle observer that receives hooks during execution.
    ///
    /// The observer is stored as an `Arc` for thread-safe sharing (required by the
    /// island model). All hooks receive `&self`, so observers that need interior
    /// mutability should use `Mutex`, `AtomicU64`, or similar.
    ///
    /// See [`GaObserver`](crate::observer::GaObserver) for the hook contract.
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

    /// Sets the initialization function used to create chromosome DNA.
    ///
    /// The closure receives `(genes_per_chromosome, alleles, needs_unique_ids)`
    /// and must return a `Vec` of genes for one chromosome.
    pub fn with_initialization_fn<F>(mut self, initialization_fn: F) -> Self
    where
        U: ChromosomeT + Send + Sync + 'static + Clone,
        F: Fn(usize, Option<&[U::Gene]>, Option<bool>) -> Vec<U::Gene> + Send + Sync + 'static,
    {
        self.initialization_fn = Some(Arc::new(initialization_fn));
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
        U: ChromosomeT + Send + Sync + 'static + Clone,
    {
        // Before starting initialization, we should verify that initializer is set
        if self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        //Before starting the run, we will check the conditions
        ValidatorFactory::validate::<U>(Some(&self.configuration), None, Some(&self.alleles))?;

        info!("Initialization started");

        let population_size = self.configuration.limit_configuration.population_size;
        let genes_per_chromosome = self.configuration.limit_configuration.genes_per_chromosome;
        let needs_unique_ids = self.configuration.limit_configuration.needs_unique_ids;
        let init_fn = self.initialization_fn.as_ref().unwrap();
        let fitness_fn = self.fitness_fn.as_ref().unwrap();

        let chromosomes = crate::traits::initialize_chromosomes_par::<U>(
            population_size,
            genes_per_chromosome,
            if self.alleles.is_empty() {
                None
            } else {
                Some(&self.alleles)
            },
            Some(needs_unique_ids),
            init_fn,
            Some(fitness_fn),
            0,
        );

        // Set population directly (with_population is consuming, so we assign inline)
        let new_population = Population::new(chromosomes);
        if self.configuration.selection_configuration.number_of_couples == 0 {
            self.configuration.selection_configuration.number_of_couples =
                new_population.size() / 2;
        }
        self.population = new_population;
        Ok(self)
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
    #[allow(deprecated)]
    pub fn run_with_callback<F>(
        &mut self,
        callback: Option<F>,
        generations_to_callback: usize,
    ) -> Result<&Population<U>, GaError>
    where
        U: ChromosomeT + Send + Sync + 'static + Clone,
        F: Fn(&usize, &Population<U>, &GenerationStats, &TerminationCause) -> ControlFlow<()>,
    {
        //Before starting the run, we will check the conditions
        ValidatorFactory::validate::<U>(Some(&self.configuration), None, Some(&self.alleles))?;

        // Apply RNG seed if configured (must be done before any random operations)
        crate::rng::set_seed(self.configuration.rng_seed);

        //If we want to initialize the population randomly
        if self.population.size() == 0 && self.initialization_fn.is_some() {
            self.initialization()?;
        } else if self.population.size() == 0 && self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        //We initialize the logger programmatically (no env::set_var, which is UB in multi-threaded context)
        let log_level = match self.configuration.log_level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        };
        let _ = env_logger::Builder::from_default_env()
            .filter_level(log_level)
            .try_init();

        //Initialize the adaptive ga
        if self.configuration.adaptive_ga {
            self.population.recalculate_aga();
        }

        // Initialize dynamic mutation probability
        if self.configuration.mutation_configuration.dynamic_mutation {
            self.dynamic_mutation_probability = self
                .configuration
                .mutation_configuration
                .probability_max
                .unwrap_or(1.0);
        }

        //Best chromosome within the generations and population returned
        let initial_population_size = self.population.size();
        let mut age = 0usize;

        //Calculation of the fitness and the best chromosome
        self.population.fitness_calculation(
            self.configuration.number_of_threads,
            self.configuration.limit_configuration.problem_solving,
        );

        // Starting counting the generations for the callback
        let mut generation_callback_count = 0usize;

        // Reset per-generation stats
        self.stats.clear();

        // Determine if this is a maximization problem
        let is_maximization = matches!(
            self.configuration.limit_configuration.problem_solving,
            ProblemSolving::Maximization
        );

        // Compound stopping criteria tracking
        let start_time = Instant::now();
        let mut best_fitness_so_far = self.population.best_chromosome.fitness();
        let mut stagnation_count: usize = 0;

        if let Some(ref mut r) = self.reporter {
            r.on_start();
        }
        self.notify(|obs| obs.on_run_start());

        //We start the cycles
        for i in 0..self.configuration.limit_configuration.max_generations {
            info!(target="ga_events", method="run"; "Generation number: {}", i+1);
            age += 1;
            self.notify(|obs| obs.on_generation_start(i));

            //1- Parent selection for reproduction
            let t_sel = if self.observer.is_some() { Some(Instant::now()) } else { None };
            let parents = selection::factory(
                &self.population.chromosomes,
                self.configuration.selection_configuration,
                self.configuration.number_of_threads,
            )?;
            if let Some(t) = t_sel {
                self.notify(|obs| obs.on_selection_complete(i, t.elapsed(), parents.len()));
            }
            debug!(target="ga_events", method="run"; "Parents selected for reproduction");

            //2- Getting the offspring
            let dynamic_prob = if self.configuration.mutation_configuration.dynamic_mutation {
                Some(self.dynamic_mutation_probability)
            } else {
                None
            };
            let t_cx = if self.observer.is_some() { Some(Instant::now()) } else { None };
            let mut offspring = parent_crossover(
                &parents,
                &self.population.chromosomes,
                &self.configuration,
                age,
                self.population.f_max,
                self.population.f_avg,
                dynamic_prob,
            )?;
            if let Some(t) = t_cx {
                let elapsed = t.elapsed();
                let offspring_count = offspring.len();
                let pop_size = self.population.chromosomes.len();
                self.notify(|obs| obs.on_crossover_complete(i, elapsed, offspring_count));
                self.notify(|obs| obs.on_mutation_complete(i, Duration::ZERO, pop_size));
                self.notify(|obs| obs.on_fitness_evaluation_complete(i, Duration::ZERO, pop_size));
            }
            debug!(target="ga_events", method="run"; "Offspring created");

            //3- Insert the children in the population
            self.population.add_chromosomes(&mut offspring);

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
            let t_surv = if self.observer.is_some() { Some(Instant::now()) } else { None };
            survivor::factory(
                self.configuration.survivor,
                &mut self.population.chromosomes,
                initial_population_size,
                self.configuration.limit_configuration,
            )?;
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

            // Apply niching / fitness sharing if configured
            if let Some(ref niching_config) = self.configuration.niching_configuration {
                if niching_config.enabled {
                    // Extract fitness values
                    let mut fitness_values: Vec<f64> = self
                        .population
                        .chromosomes
                        .iter()
                        .map(|c| c.fitness())
                        .collect();

                    // Extract DNA slices for distance computation
                    let dna_slices: Vec<&[U::Gene]> = self
                        .population
                        .chromosomes
                        .iter()
                        .map(|c| c.dna())
                        .collect();

                    // Compute distance matrix using gene ID comparison
                    let distances = crate::niching::sharing::compute_distance_matrix(
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
                    );

                    // Apply fitness sharing
                    crate::niching::sharing::apply_fitness_sharing(
                        &mut fitness_values,
                        &distances,
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

            debug!(target="ga_events", method="run"; "Survivors selected");

            //5- Sets the best chromosome (scan by index, clone only the winner)
            {
                let ps = self.configuration.limit_configuration.problem_solving;
                let chromosomes = &self.population.chromosomes;
                if let Some(best_idx) = best_chromosome_index(chromosomes, ps) {
                    if !self.population.best_chromosome_is_set {
                        self.population.best_chromosome =
                            self.population.chromosomes[best_idx].clone();
                        self.population.best_chromosome_is_set = true;
                    } else {
                        let candidate = self.population.chromosomes[best_idx].fitness();
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
            debug!(target="ga_events", method="run"; "Best chromosome calculated - generation {}", i+1);

            // Collect per-generation statistics
            let fitness_values: Vec<f64> = self
                .population
                .chromosomes
                .iter()
                .map(|c| c.fitness())
                .collect();
            let gen_stats =
                GenerationStats::from_fitness_values(i, &fitness_values, is_maximization);
            self.stats.push(gen_stats.clone());

            if let Some(ref mut r) = self.reporter {
                r.on_generation_complete(&gen_stats);
            }
            self.notify(|obs| obs.on_generation_end(&gen_stats));

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

                debug!(
                    target = "ga_events",
                    method = "run";
                    "Dynamic mutation: diversity={:.4}, probability={:.4}",
                    gen_stats.diversity,
                    self.dynamic_mutation_probability
                );
            }

            // Apply extension strategy if configured and diversity is low
            if let Some(ref ext_config) = self.configuration.extension_configuration {
                if ext_config.method != Extension::Noop
                    && gen_stats.diversity < ext_config.diversity_threshold
                {
                    info!(
                        target = "extension_events",
                        method = "run";
                        "Extension triggered: diversity={:.6} < threshold={:.6}",
                        gen_stats.diversity,
                        ext_config.diversity_threshold
                    );

                    extension::factory(
                        ext_config.method,
                        &mut self.population.chromosomes,
                        initial_population_size,
                        self.configuration.limit_configuration.problem_solving,
                        ext_config,
                    )?;
                    self.notify(|obs| obs.on_extension_triggered(ExtensionEvent {
                        generation: i,
                        diversity: gen_stats.diversity,
                        extension_type: ext_config.method.as_str(),
                    }));

                    // Regrow population if extension reduced it
                    if self.population.chromosomes.len() < initial_population_size {
                        if let Some(ref init_fn) = self.initialization_fn {
                            let deficit =
                                initial_population_size - self.population.chromosomes.len();
                            for _ in 0..deficit {
                                let alleles_ref = if self.alleles.is_empty() {
                                    None
                                } else {
                                    Some(self.alleles.as_slice())
                                };
                                let genes = init_fn(
                                    self.configuration
                                        .limit_configuration
                                        .genes_per_chromosome,
                                    alleles_ref,
                                    Some(
                                        self.configuration
                                            .limit_configuration
                                            .alleles_can_be_repeated,
                                    ),
                                );
                                let mut new_chromosome = U::new();
                                new_chromosome.set_dna(std::borrow::Cow::Owned(genes));
                                if let Some(ref ff) = self.fitness_fn {
                                    let ff_clone = Arc::clone(ff);
                                    new_chromosome
                                        .set_fitness_fn(move |genes| ff_clone(genes));
                                }
                                new_chromosome.calculate_fitness();
                                new_chromosome.set_age(0);
                                self.population.chromosomes.push(new_chromosome);
                            }
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
                        log::warn!("Failed to save checkpoint at generation {}: {}", i + 1, e);
                    }
                }
            }

            // If we want to perform a periodic callback
            if let Some(func) = &callback {
                if (generation_callback_count + 1) == generations_to_callback {
                    if func(&i, &self.population, &gen_stats, &self.termination_cause).is_break() {
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
                    let _ = func(&i, &self.population, &gen_stats, &self.termination_cause);
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
                if let Some(ref mut r) = self.reporter {
                    r.on_new_best(i, self.population.best_chromosome.clone());
                }
                self.notify(|obs| obs.on_new_best(i, self.population.best_chromosome.clone()));
            } else {
                stagnation_count += 1;
                self.notify(|obs| obs.on_stagnation(i, stagnation_count));
            }

            if let Some(max_stagnation) =
                self.configuration.stopping_criteria.stagnation_generations
            {
                if stagnation_count >= max_stagnation {
                    self.termination_cause = TerminationCause::StagnationReached;
                    if let Some(func) = &callback {
                        let _ = func(&i, &self.population, &gen_stats, &self.termination_cause);
                    }
                    break;
                }
            }

            // Convergence check (fitness std dev below threshold)
            if let Some(threshold) = self.configuration.stopping_criteria.convergence_threshold {
                if gen_stats.fitness_std_dev < threshold {
                    self.termination_cause = TerminationCause::ConvergenceReached;
                    if let Some(func) = &callback {
                        let _ = func(&i, &self.population, &gen_stats, &self.termination_cause);
                    }
                    break;
                }
            }

            // Time limit check
            if let Some(max_secs) = self.configuration.stopping_criteria.max_duration_secs {
                if start_time.elapsed().as_secs_f64() >= max_secs {
                    self.termination_cause = TerminationCause::TimeLimitReached;
                    if let Some(func) = &callback {
                        let _ = func(&i, &self.population, &gen_stats, &self.termination_cause);
                    }
                    break;
                }
            }
        }

        // Set termination cause when generation limit is reached (regardless of callback)
        if self.termination_cause == TerminationCause::NotTerminated {
            self.termination_cause = TerminationCause::GenerationLimitReached;
        }

        if let Some(ref mut r) = self.reporter {
            r.on_finish(self.termination_cause, &self.stats);
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
}

/// Checks termination limits according to `LimitConfiguration`.
///
/// - For Minimization: stops when any chromosome has fitness exactly `0.0`.
/// - For FixedFitness: stops when any chromosome has fitness exactly `fitness_target`.
fn limit_reached<U>(limit: LimitConfiguration, chromosomes: &[U]) -> bool
where
    U: ChromosomeT,
{
    debug!(target="ga_events", method="limit_reached"; "Started limit reached method");
    let mut result = false;

    if limit.problem_solving == ProblemSolving::Minimization {
        //If the problem-solving is minimization, fitness must be 0
        for chromosome in chromosomes {
            if chromosome.fitness() == 0.0 {
                trace!(target="ga_events", method="limit_reached"; "limit reached for minimization");
                result = true;
                break;
            }
        }
    } else if limit.problem_solving == ProblemSolving::FixedFitness {
        //If the problem-solving is a fixed fitness
        if let Some(target) = limit.fitness_target {
            for chromosome in chromosomes {
                if chromosome.fitness() == target {
                    trace!(target="ga_events", method="limit_reached"; "limit reached for fixed fitness");
                    result = true;
                    break;
                }
            }
        }
    }

    debug!(target="ga_events", method="limit_reached"; "Limit reached method finished");
    result
}

/// Performs parent crossover using the configured crossover and mutation strategies.
///
/// Behavior:
/// - Splits work among threads considering available parent pairs.
/// - Computes adaptive probabilities when enabled; otherwise uses static ones.
/// - Produces children, mutates them, computes their fitness, and returns the offspring.
fn parent_crossover<U>(
    parents: &[(usize, usize)],
    chromosomes: &[U],
    configuration: &GaConfiguration,
    age: usize,
    f_max: f64,
    f_avg: f64,
    dynamic_mutation_prob: Option<f64>,
) -> Result<Vec<U>, GaError>
where
    U: ChromosomeT + Send + Sync + 'static + Clone + mutation::ValueMutable,
{
    debug!(target="ga_events", method="parent_crossover"; "Started the parent crossover");

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

    // Use rayon to process parent pairs in parallel
    let results: Vec<Result<Vec<U>, GaError>> = parents
        .par_iter()
        .map(|(key, value)| {
            let mut rng = crate::rng::make_rng();

            // Getting the parent 1 and 2 for crossover
            let parent_1 = chromosomes.get(*key).ok_or_else(|| {
                GaError::SelectionError(format!(
                    "Selection returned out-of-bounds index {} (population size {})",
                    key,
                    chromosomes.len()
                ))
            })?.clone();
            let parent_2 = chromosomes.get(*value).ok_or_else(|| {
                GaError::SelectionError(format!(
                    "Selection returned out-of-bounds index {} (population size {})",
                    value,
                    chromosomes.len()
                ))
            })?.clone();

            // Making the crossover of the parents when the random number is below or equal to the given probability
            let crossover_probability = rng.random_range(0.0..1.0);
            let effective_crossover_prob =
                if let Some(p) = crossover_probability_config {
                    p
                } else {
                    crossover::aga_probability(
                        &parent_1,
                        &parent_2,
                        f_max,
                        f_avg,
                        configuration.crossover_configuration.probability_max.unwrap_or(1.0),
                        configuration.crossover_configuration.probability_min.unwrap_or(0.0),
                    )
                };

            // Making the mutation of each child when the random number is below or equal the given probability
            let mut mutation_probability = rng.random_range(0.0..1.0);
            let effective_mutation_prob =
                if let Some(p) = mutation_probability_config {
                    p
                } else {
                    mutation::aga_probability(
                        &parent_1,
                        &parent_2,
                        f_avg,
                        configuration.mutation_configuration.probability_max.unwrap_or(1.0),
                        configuration.mutation_configuration.probability_min.unwrap_or(0.0),
                    )
                };

            debug!(target="ga_events", method="parent_crossover"; "Processing parent pair");

            let mut child_1: U;
            let mut child_2: U;

            if crossover_probability <= effective_crossover_prob {
                let mut children = crossover::factory(&parent_1, &parent_2, configuration.crossover_configuration)?;
                child_2 = children.pop().ok_or_else(|| {
                    GaError::CrossoverError("Crossover returned fewer than 2 children".to_string())
                })?;
                child_1 = children.pop().ok_or_else(|| {
                    GaError::CrossoverError("Crossover returned fewer than 2 children".to_string())
                })?;
            } else {
                child_1 = parent_1;
                child_2 = parent_2;
            }

            debug!(target="ga_events", method="parent_crossover"; "mutation_probability_config {} - mutation probability {}", effective_mutation_prob, mutation_probability);

            if mutation_probability < effective_mutation_prob {
                mutation::factory_with_params(
                    configuration.mutation_configuration.method,
                    &mut child_1,
                    configuration.mutation_configuration.step,
                    configuration.mutation_configuration.sigma,
                )?;
            }

            mutation_probability = rng.random_range(0.0..1.0);
            if mutation_probability <= effective_mutation_prob {
                mutation::factory_with_params(
                    configuration.mutation_configuration.method,
                    &mut child_2,
                    configuration.mutation_configuration.step,
                    configuration.mutation_configuration.sigma,
                )?;
            }

            // Calculate the fitness of both children and set their age
            child_1.calculate_fitness();
            child_2.calculate_fitness();

            child_1.set_age(age);
            child_2.set_age(age);

            Ok(vec![child_1, child_2])
        })
        .collect();

    // Check for any errors and flatten the results
    let mut offspring = Vec::new();
    for result in results {
        offspring.extend(result?);
    }

    debug!(target="ga_events", method="parent_crossover"; "Parent crossover finished");
    Ok(offspring)
}

/// Extracts the top `count` individuals from the population by fitness.
///
/// Only clones the selected elite individuals instead of the whole population.
fn extract_elite<U: ChromosomeT>(
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
fn reinsert_elite<U: ChromosomeT>(
    chromosomes: &mut [U],
    elite: Vec<U>,
    problem_solving: ProblemSolving,
) {
    // Sort population so worst are at the end
    chromosomes.sort_by(|a, b| {
        let cmp = a
            .fitness()
            .partial_cmp(&b.fitness())
            .unwrap_or(std::cmp::Ordering::Equal);
        match problem_solving {
            ProblemSolving::Maximization => cmp.reverse(),
            _ => cmp,
        }
    });

    // Replace the worst individuals with elite (guard against more elite than population)
    let pop_len = chromosomes.len();
    let count = elite.len().min(pop_len);
    for (i, elite_individual) in elite.into_iter().take(count).enumerate() {
        let replace_idx = pop_len - 1 - i;
        chromosomes[replace_idx] = elite_individual;
    }
}

/// Finds the index of the best chromosome according to the problem objective.
///
/// Returns `None` for an empty slice.
fn best_chromosome_index<U: ChromosomeT>(
    chromosomes: &[U],
    problem_solving: ProblemSolving,
) -> Option<usize> {
    if chromosomes.is_empty() {
        return None;
    }
    let mut best = 0;
    let mut best_fit = chromosomes[0].fitness();
    for (i, c) in chromosomes.iter().enumerate().skip(1) {
        let f = c.fitness();
        let is_better = match problem_solving {
            ProblemSolving::Maximization | ProblemSolving::FixedFitness => f > best_fit,
            ProblemSolving::Minimization => f < best_fit,
        };
        if is_better {
            best = i;
            best_fit = f;
        }
    }
    Some(best)
}
