//! `EdaEngine` — Estimation of Distribution Algorithm (UMDA variant).
//!
//! Implements the Univariate Marginal Distribution Algorithm (UMDA) as a new engine
//! following the established PSO/CMA engine pattern. Instead of crossover and mutation,
//! EDA learns a univariate probabilistic model from the best individuals each generation
//! and samples the entire new population from that model.
//!
//! # Model dispatch
//!
//! Two model strategies are supported, selected by the gene type's trait bounds:
//!
//! - **Bernoulli** (binary genes — `U::Gene` does NOT implement `RealGene`): estimates
//!   `p_i = fraction_of_selected_with_gene_i_equal_1` per position and samples
//!   Bernoulli(`p_i`) for each gene. Uses `gene.id() == 1` as the "one" indicator.
//!
//! - **Gaussian** (real-valued genes — `U::Gene: RealGene`): estimates `(mean_i, std_i)`
//!   per position from selected parents and samples `N(mean_i, std_i)` clamped to
//!   `gene.bounds()` when available.
//!
//! # WASM compatibility
//!
//! The core loop contains no unconditional `Instant::now()` calls.
//! Fitness evaluation of the new population uses `rayon` on native targets and
//! sequential iteration on `wasm32-unknown-unknown`.

use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use rand::Rng;

use crate::configuration::ProblemSolving;
use crate::error::GaError;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::rng::make_rng;
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, GeneT, LinearChromosome, RealGene};

use super::configuration::EdaConfiguration;

// ─── EdaModel ─────────────────────────────────────────────────────────────────

/// The probabilistic model learned by the EDA engine at the final generation.
///
/// Returned in [`EdaResult`] so callers can inspect the converged distribution.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::eda::EdaModel;
///
/// let model = EdaModel::Bernoulli(vec![0.5, 0.8, 0.2]);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdaModel {
    /// Bernoulli probability vector (one `f64` per gene position).
    ///
    /// After convergence, values should approach 0.0 or 1.0.
    /// Probabilities are clamped to `[0.01, 0.99]` to prevent degenerate distributions.
    Bernoulli(Vec<f64>),

    /// Gaussian univariate model (one mean and one std per gene position).
    ///
    /// Both `means` and `stds` have the same length as the chromosome.
    /// `stds` values have a floor of `1e-6` to prevent degenerate distributions.
    Gaussian {
        /// Per-position mean values.
        means: Vec<f64>,
        /// Per-position standard deviations.
        stds: Vec<f64>,
    },
}

// ─── EdaResult ────────────────────────────────────────────────────────────────

/// Result returned by [`EdaEngine::run`].
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::eda::{EdaConfiguration, EdaEngine, EdaResult};
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::GeneT;
///
/// let config = EdaConfiguration::default().with_max_generations(200);
/// let mut engine = EdaEngine::<Binary>::new(
///     config,
///     |n| vec![Binary::default(); n],
///     |dna| dna.iter().filter(|g| g.id() == 1).count() as f64,
/// );
/// let result: EdaResult<Binary> = engine.run().unwrap();
/// println!("Best fitness: {}", result.best_fitness);
/// ```
pub struct EdaResult<U: LinearChromosome> {
    /// Final population after the last generation.
    pub population: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
    /// The probabilistic model estimated at the final generation.
    pub learned_model: EdaModel,
}

// ─── EdaEngine (Bernoulli — binary genes) ─────────────────────────────────────

/// EDA engine for chromosomes whose gene type does **not** implement [`RealGene`].
///
/// Uses the classic UMDA Bernoulli model: estimates `p_i` per gene position from the
/// selected parents and samples Bernoulli(`p_i`) to produce offspring.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::eda::{EdaConfiguration, EdaEngine};
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::traits::GeneT;
///
/// let config = EdaConfiguration::default()
///     .with_max_generations(200)
///     .with_problem_solving(ProblemSolving::Maximization);
///
/// let mut engine = EdaEngine::<Binary>::new(
///     config,
///     |n| vec![Binary::default(); n],
///     |dna| dna.iter().filter(|g| g.id() == 1).count() as f64,
/// );
/// let result = engine.run();
/// println!("Generations: {}", result.generations);
/// ```
pub struct EdaEngine<U: LinearChromosome> {
    config: EdaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
    fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
}

impl<U: LinearChromosome + Clone> EdaEngine<U> {
    /// Construct a new EDA engine (Bernoulli model for binary genes).
    ///
    /// * `config` — algorithm parameters.
    /// * `init_fn` — called once with `population_size`; must return that many initialised chromosomes.
    /// * `fitness_fn` — maps a DNA slice to a scalar fitness value.
    pub fn new(
        config: EdaConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            observer: None,
            fitness_cache: None,
        }
    }

    /// Alias for [`new`](Self::new) — explicit Bernoulli constructor for clarity.
    ///
    /// Call [`run`](Self::run) after construction to execute the Bernoulli UMDA loop.
    /// For Gaussian (continuous) optimization, use [`EdaRealEngine::new`].
    pub fn bernoulli(
        config: EdaConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self::new(config, init_fn, fitness_fn)
    }

    /// Attach a lifecycle observer (see [`GaObserver`] for available hooks).
    pub fn with_observer(mut self, obs: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op otherwise.
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Returns `true` if `candidate` is better than `current` under the configured direction.
    #[inline]
    fn is_better(&self, candidate: f64, current: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => candidate < current,
            ProblemSolving::Maximization => candidate > current,
            ProblemSolving::FixedFitness => {
                if let Some(t) = self.config.fitness_target {
                    (candidate - t).abs() < (current - t).abs()
                } else {
                    candidate < current
                }
            }
        }
    }

    /// Returns `true` if `fitness` satisfies the stopping `target`.
    fn reached_target(&self, fitness: f64, target: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => fitness <= target,
            ProblemSolving::Maximization => fitness >= target,
            ProblemSolving::FixedFitness => (fitness - target).abs() < 1e-6,
        }
    }

    /// Returns the index and fitness of the best individual in `pop`.
    fn find_best(&self, pop: &[U]) -> (usize, f64) {
        assert!(
            !pop.is_empty(),
            "EdaEngine::find_best called with empty population"
        );
        let mut best_idx = 0;
        let mut best_fit = pop[0].fitness();
        for (i, ind) in pop.iter().enumerate().skip(1) {
            if self.is_better(ind.fitness(), best_fit) {
                best_fit = ind.fitness();
                best_idx = i;
            }
        }
        (best_idx, best_fit)
    }

    /// Sample a new Bernoulli individual from `probs`.
    ///
    /// Gene `i` is set to id=1 if `rng.random::<f64>() < probs[i]`, else id=0.
    /// The gene factory method `U::Gene::new()` creates the default gene which is then
    /// replaced at each position using `set_dna`.
    fn sample_bernoulli<R: Rng>(template: &U, probs: &[f64], rng: &mut R) -> U {
        let new_dna: Vec<U::Gene> = template
            .dna()
            .iter()
            .zip(probs.iter())
            .enumerate()
            .map(|(i, (gene, &p))| {
                let new_id = if rng.random::<f64>() < p { 1 } else { 0 };
                // Create a new gene by cloning the template gene then setting its id.
                // `GeneT::set_id` mutates in place and returns `&mut Self`.
                let mut g = gene.clone();
                g.set_id(new_id);
                // Positional identity: encode position index using the original gene's id
                // if id == 0 (i.e., it is the position id). For binary chromosomes,
                // id encodes the Bernoulli outcome (0 or 1), but we also need to match
                // the pattern that `gene.id() == 1` = "one". The position is tracked via
                // the index `i`, not the gene id in binary chromosomes.
                let _ = i; // index tracked implicitly through zip ordering
                g
            })
            .collect();
        let mut offspring = template.clone();
        offspring.set_dna(Cow::Owned(new_dna));
        offspring
    }

    /// Run the EDA engine (Bernoulli model) and return the result.
    ///
    /// When `population_size` is 0, a default of 100 is used.
    pub fn run(&mut self) -> Result<EdaResult<U>, GaError>
    where
        U::Gene: Debug,
    {
        let mut rng = make_rng();
        let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);

        // D-05: bootstrap cache handle at run() start.
        if let Some(size) = self.config.fitness_cache_size {
            if self.fitness_cache.is_none() {
                let (wrapped_fn, cache_handle) =
                    crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), size);
                self.fitness_fn = wrapped_fn;
                self.fitness_cache = Some(cache_handle);
            }
        }

        // Three-way sort comparator that mirrors `is_better` for all ProblemSolving variants.
        // This ensures FixedFitness selects parents closest to the target, not lowest raw fitness.
        let cmp = |a_fit: f64, b_fit: f64| -> std::cmp::Ordering {
            match self.config.problem_solving {
                ProblemSolving::Maximization => b_fit
                    .partial_cmp(&a_fit)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProblemSolving::Minimization => a_fit
                    .partial_cmp(&b_fit)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProblemSolving::FixedFitness => {
                    let t = self.config.fitness_target.unwrap_or(0.0);
                    let da = (a_fit - t).abs();
                    let db = (b_fit - t).abs();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        };

        // Observer: run start
        self.notify(|obs| obs.on_run_start());

        // Determine population size (default 100 when 0)
        let pop_size = if self.config.population_size == 0 {
            100
        } else {
            self.config.population_size
        };

        // Build initial population
        let mut pop: Vec<U> = (self.init_fn)(pop_size.max(1));

        if pop.is_empty() {
            return Err(GaError::InitializationError(
                "EdaEngine: init_fn returned an empty population".to_string(),
            ));
        }

        // Evaluate initial population
        for ind in &mut pop {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        let (best_idx, mut best_fitness) = self.find_best(&pop);
        let mut best = pop[best_idx].clone();
        let dim = pop[0].dna().len();

        // Notify initial best
        self.notify(|obs| obs.on_new_best(0, &best));

        let mut termination_cause = TerminationCause::GenerationLimitReached;
        let mut all_stats: Vec<GenerationStats> = Vec::with_capacity(self.config.max_generations);
        let mut learned_model = EdaModel::Bernoulli(vec![0.5; dim]);
        let mut best_model = learned_model.clone();

        // Cache snapshot for per-generation delta stats.
        let (mut prev_cache_hits, mut prev_cache_misses) = match &self.fitness_cache {
            Some(ch) => {
                let c = ch
                    .lock()
                    .map_err(|_| GaError::InternalError("fitness cache mutex poisoned".to_string()))?;
                (c.hits(), c.misses())
            }
            None => (0, 0),
        };

        // Main loop
        for gen in 0..self.config.max_generations {
            self.notify(|obs| obs.on_generation_start(gen));

            // Determine number of selected parents (truncation selection)
            let n_selected = ((pop_size as f64 * self.config.selection_ratio).floor() as usize)
                .max(1)
                .min(pop.len());

            // Sort population to find top n_selected using three-way comparator
            let mut indices: Vec<usize> = (0..pop.len()).collect();
            indices.select_nth_unstable_by(n_selected - 1, |&a, &b| {
                cmp(pop[a].fitness(), pop[b].fitness())
            });
            let selected: Vec<&U> = indices[..n_selected].iter().map(|&i| &pop[i]).collect();

            // Estimate Bernoulli model from selected parents
            let probs = Self::estimate_bernoulli_ref(&selected, dim);
            learned_model = EdaModel::Bernoulli(probs.clone());

            // Sample new population from model
            let template = pop[0].clone();
            let mut new_pop: Vec<U> = (0..pop_size)
                .map(|_| Self::sample_bernoulli(&template, &probs, &mut rng))
                .collect();

            // Evaluate new population
            #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
            {
                use rayon::prelude::*;
                let fitness_fn = Arc::clone(&self.fitness_fn);
                let fitnesses: Vec<f64> = new_pop
                    .par_iter()
                    .map(|ind| fitness_fn(ind.dna()))
                    .collect();
                for (ind, f) in new_pop.iter_mut().zip(fitnesses.into_iter()) {
                    ind.set_fitness(f);
                }
            }
            #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
            {
                for ind in &mut new_pop {
                    let f = (self.fitness_fn)(ind.dna());
                    ind.set_fitness(f);
                }
            }

            pop = new_pop;

            // Update best
            let (gen_best_idx, gen_best_fit) = self.find_best(&pop);
            if self.is_better(gen_best_fit, best_fitness) {
                best_fitness = gen_best_fit;
                best = pop[gen_best_idx].clone();
                best_model = learned_model.clone();
                self.notify(|obs| obs.on_new_best(gen, &best));
            }

            // Generation stats
            let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
            let mut stats =
                GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
            if let Some(ref ch) = self.fitness_cache {
                let c = ch
                    .lock()
                    .map_err(|_| GaError::InternalError("fitness cache mutex poisoned".to_string()))?;
                stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
                stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
                prev_cache_hits = c.hits();
                prev_cache_misses = c.misses();
            }
            self.notify(|obs| obs.on_generation_end(&stats));
            all_stats.push(stats);

            // Early stopping
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    termination_cause = TerminationCause::FitnessTargetReached;
                    break;
                }
            }
        }

        // Observer: run end
        let generations = all_stats.len();
        let all_stats_ref = all_stats.as_slice();
        self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));

        Ok(EdaResult {
            population: pop,
            best,
            best_fitness,
            generations,
            learned_model: best_model,
        })
    }

    /// Estimate Bernoulli probabilities from a slice of references.
    fn estimate_bernoulli_ref(selected: &[&U], dim: usize) -> Vec<f64> {
        let n = selected.len() as f64;
        (0..dim)
            .map(|i| {
                let ones = selected
                    .iter()
                    .filter(|ind| ind.dna().get(i).map(|g| g.id() == 1).unwrap_or(false))
                    .count() as f64;
                (ones / n).clamp(0.01, 0.99)
            })
            .collect()
    }
}

// ─── EdaRealEngine (Gaussian — real-valued genes) ─────────────────────────────

/// EDA engine for chromosomes whose gene type implements [`RealGene`].
///
/// Uses a Gaussian univariate model: estimates `(mean_i, std_i)` per gene position
/// from the selected parents and samples `N(mean_i, std_i)` clamped to `gene.bounds()`
/// for each offspring.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::eda::{EdaConfiguration, EdaRealEngine};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::RealGene;
///
/// let config = EdaConfiguration::default()
///     .with_max_generations(300)
///     .with_problem_solving(ProblemSolving::Minimization);
///
/// let mut engine = EdaRealEngine::<RangeChromosome<f64>>::new(
///     config,
///     |n| vec![RangeChromosome::default(); n],
///     |dna| dna.iter().map(|g| g.real_value().powi(2)).sum(),
/// );
/// let result = engine.run();
/// println!("Generations: {}", result.generations);
/// ```
pub struct EdaRealEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: EdaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
    fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
}

impl<U: LinearChromosome + Clone> EdaRealEngine<U>
where
    U::Gene: RealGene,
{
    /// Construct a new EDA engine (Gaussian model for real-valued genes).
    pub fn new(
        config: EdaConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            observer: None,
            fitness_cache: None,
        }
    }

    /// Attach a lifecycle observer.
    pub fn with_observer(mut self, obs: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    #[inline]
    fn is_better(&self, candidate: f64, current: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => candidate < current,
            ProblemSolving::Maximization => candidate > current,
            ProblemSolving::FixedFitness => {
                if let Some(t) = self.config.fitness_target {
                    (candidate - t).abs() < (current - t).abs()
                } else {
                    candidate < current
                }
            }
        }
    }

    fn reached_target(&self, fitness: f64, target: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => fitness <= target,
            ProblemSolving::Maximization => fitness >= target,
            ProblemSolving::FixedFitness => (fitness - target).abs() < 1e-6,
        }
    }

    fn find_best(&self, pop: &[U]) -> (usize, f64) {
        assert!(
            !pop.is_empty(),
            "EdaRealEngine::find_best called with empty population"
        );
        let mut best_idx = 0;
        let mut best_fit = pop[0].fitness();
        for (i, ind) in pop.iter().enumerate().skip(1) {
            if self.is_better(ind.fitness(), best_fit) {
                best_fit = ind.fitness();
                best_idx = i;
            }
        }
        (best_idx, best_fit)
    }

    /// Estimate Gaussian model from selected parents (slice of references).
    ///
    /// `mean_i = mean(gene_i.real_value())` over selected parents.
    /// `std_i = std_dev(gene_i.real_value())` clamped to floor `1e-6`.
    fn estimate_gaussian(selected: &[&U], dim: usize) -> (Vec<f64>, Vec<f64>) {
        let n = selected.len() as f64;
        let mut means = Vec::with_capacity(dim);
        let mut stds = Vec::with_capacity(dim);

        for i in 0..dim {
            let vals: Vec<f64> = selected
                .iter()
                .filter_map(|ind| ind.dna().get(i).map(|g| g.real_value()))
                .collect();
            let mean = vals.iter().sum::<f64>() / n;
            let variance = if n > 1.0 {
                vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)
            } else {
                0.0
            };
            let std = variance.sqrt().max(1e-6);
            means.push(mean);
            stds.push(std);
        }

        (means, stds)
    }

    /// Sample a new individual from the Gaussian model.
    ///
    /// Gene `i` is set to `clamp(N(mean_i, std_i), lo_i, hi_i)` using
    /// `gene.with_real_value(v)`.
    fn sample_gaussian<R: Rng>(template: &U, means: &[f64], stds: &[f64], rng: &mut R) -> U {
        // Box-Muller transform for N(0,1) sampling (no external dependency needed)
        let new_dna: Vec<U::Gene> = template
            .dna()
            .iter()
            .enumerate()
            .map(|(i, gene)| {
                let mean = means.get(i).copied().unwrap_or(0.0);
                let std = stds.get(i).copied().unwrap_or(1.0);

                // Box-Muller: two uniform samples → one normal sample
                let u1: f64 = rng.random::<f64>().max(1e-10);
                let u2: f64 = rng.random::<f64>();
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                let mut v = mean + std * z;

                // Clamp to gene bounds if available
                if let Some((lo, hi)) = gene.bounds() {
                    v = v.clamp(lo, hi);
                }

                gene.with_real_value(v)
            })
            .collect();

        let mut offspring = template.clone();
        offspring.set_dna(Cow::Owned(new_dna));
        offspring
    }

    /// Run the EDA engine (Gaussian model) and return the result.
    pub fn run(&mut self) -> Result<EdaResult<U>, GaError>
    where
        U::Gene: Debug,
    {
        let mut rng = make_rng();
        let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);

        // D-05: bootstrap cache handle at run() start.
        if let Some(size) = self.config.fitness_cache_size {
            if self.fitness_cache.is_none() {
                let (wrapped_fn, cache_handle) =
                    crate::fitness::cache::wrap_with_cache(Arc::clone(&self.fitness_fn), size);
                self.fitness_fn = wrapped_fn;
                self.fitness_cache = Some(cache_handle);
            }
        }

        // Three-way sort comparator that mirrors `is_better` for all ProblemSolving variants.
        let cmp = |a_fit: f64, b_fit: f64| -> std::cmp::Ordering {
            match self.config.problem_solving {
                ProblemSolving::Maximization => b_fit
                    .partial_cmp(&a_fit)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProblemSolving::Minimization => a_fit
                    .partial_cmp(&b_fit)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProblemSolving::FixedFitness => {
                    let t = self.config.fitness_target.unwrap_or(0.0);
                    let da = (a_fit - t).abs();
                    let db = (b_fit - t).abs();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        };

        self.notify(|obs| obs.on_run_start());

        let pop_size = if self.config.population_size == 0 {
            100
        } else {
            self.config.population_size
        };

        let mut pop: Vec<U> = (self.init_fn)(pop_size.max(1));

        if pop.is_empty() {
            return Err(GaError::InitializationError(
                "EdaRealEngine: init_fn returned an empty population".to_string(),
            ));
        }

        for ind in &mut pop {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        let (best_idx, mut best_fitness) = self.find_best(&pop);
        let mut best = pop[best_idx].clone();
        let dim = pop[0].dna().len();

        self.notify(|obs| obs.on_new_best(0, &best));

        let mut termination_cause = TerminationCause::GenerationLimitReached;
        let mut all_stats: Vec<GenerationStats> = Vec::with_capacity(self.config.max_generations);
        let mut learned_model = EdaModel::Gaussian {
            means: vec![0.0; dim],
            stds: vec![1.0; dim],
        };
        let mut best_model = learned_model.clone();

        // Cache snapshot for per-generation delta stats.
        let (mut prev_cache_hits, mut prev_cache_misses) = match &self.fitness_cache {
            Some(ch) => {
                let c = ch
                    .lock()
                    .map_err(|_| GaError::InternalError("fitness cache mutex poisoned".to_string()))?;
                (c.hits(), c.misses())
            }
            None => (0, 0),
        };

        for gen in 0..self.config.max_generations {
            self.notify(|obs| obs.on_generation_start(gen));

            let n_selected = ((pop_size as f64 * self.config.selection_ratio).floor() as usize)
                .max(1)
                .min(pop.len());

            let mut indices: Vec<usize> = (0..pop.len()).collect();
            indices.select_nth_unstable_by(n_selected - 1, |&a, &b| {
                cmp(pop[a].fitness(), pop[b].fitness())
            });
            let selected: Vec<&U> = indices[..n_selected].iter().map(|&i| &pop[i]).collect();

            // Estimate Gaussian model
            let (means, stds) = Self::estimate_gaussian(&selected, dim);
            learned_model = EdaModel::Gaussian {
                means: means.clone(),
                stds: stds.clone(),
            };

            // Sample new population
            let template = pop[0].clone();
            let mut new_pop: Vec<U> = (0..pop_size)
                .map(|_| Self::sample_gaussian(&template, &means, &stds, &mut rng))
                .collect();

            // Evaluate fitness
            #[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
            {
                use rayon::prelude::*;
                let fitness_fn = Arc::clone(&self.fitness_fn);
                let fitnesses: Vec<f64> = new_pop
                    .par_iter()
                    .map(|ind| fitness_fn(ind.dna()))
                    .collect();
                for (ind, f) in new_pop.iter_mut().zip(fitnesses.into_iter()) {
                    ind.set_fitness(f);
                }
            }
            #[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
            {
                for ind in &mut new_pop {
                    let f = (self.fitness_fn)(ind.dna());
                    ind.set_fitness(f);
                }
            }

            pop = new_pop;

            let (gen_best_idx, gen_best_fit) = self.find_best(&pop);
            if self.is_better(gen_best_fit, best_fitness) {
                best_fitness = gen_best_fit;
                best = pop[gen_best_idx].clone();
                best_model = learned_model.clone();
                self.notify(|obs| obs.on_new_best(gen, &best));
            }

            let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
            let mut stats =
                GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
            if let Some(ref ch) = self.fitness_cache {
                let c = ch
                    .lock()
                    .map_err(|_| GaError::InternalError("fitness cache mutex poisoned".to_string()))?;
                stats.cache_hits = Some(c.hits().saturating_sub(prev_cache_hits));
                stats.cache_misses = Some(c.misses().saturating_sub(prev_cache_misses));
                prev_cache_hits = c.hits();
                prev_cache_misses = c.misses();
            }
            self.notify(|obs| obs.on_generation_end(&stats));
            all_stats.push(stats);

            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    termination_cause = TerminationCause::FitnessTargetReached;
                    break;
                }
            }
        }

        let generations = all_stats.len();
        let all_stats_ref = all_stats.as_slice();
        self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));

        Ok(EdaResult {
            population: pop,
            best,
            best_fitness,
            generations,
            learned_model: best_model,
        })
    }
}
