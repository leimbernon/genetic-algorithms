//! # Age-Layered Population Structure (AlpsEngine)
//!
//! ## Description
//!
//! ALPS (Age-Layered Population Structure) is a metaheuristic that maintains **multiple
//! sub-populations ("layers") ordered by individual age**. Layer 0 is the youngest, layer
//! `n_layers - 1` is the oldest. Each layer has a maximum-age threshold determined by the
//! chosen age scheme. When an individual's age exceeds its layer's threshold, it is promoted
//! to the next older layer (or discarded if already in the oldest layer).
//!
//! The key innovation of ALPS is that it prevents premature convergence by continuously
//! introducing fresh genetic material into layer 0 (either through periodic re-initialization
//! or injection) while allowing fully converged solutions in the older layers to maintain
//! exploitation pressure. The age of an individual is incremented each generation, so even
//! highly fit individuals are eventually promoted upward, making room for new exploration
//! in lower layers.
//!
//! **Cross-layer mating** is supported: each generation, the best individual from an older
//! adjacent layer may mate with individuals in the current layer (with configurable
//! probability), allowing beneficial traits to propagate upward.
//!
//! ## When to Use
//!
//! - **Problem type:** Single-objective — continuous, binary, or permutation
//! - **Number of objectives:** 1
//! - **Variable type:** Any (requires [`ValueMutable`](crate::operations::mutation::ValueMutable)
//!   for in-place mutation)
//! - **Key strength:** Excellent diversity maintenance; prevents premature convergence even
//!   on highly multimodal landscapes; anytime algorithm (useful solutions at any generation)
//! - **Key weakness:** Higher per-generation overhead than standard GA due to multiple layers;
//!   tuning layer count, age gap, and injection interval requires experimentation
//!
//! ## Quick Reference
//!
//! ### Mandatory Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `n_layers` | `usize` | Yes (via builder) | `6` | Number of age layers (minimum 2) |
//! | `layer_size` | `usize` | Yes (via builder) | `20` | Target individuals per layer |
//! | `max_generations` | `usize` | Yes (via builder) | `1000` | Maximum number of generations |
//! | `init_fn` | `Fn(usize) -> Vec<U>` | Yes (constructor) | — | Population initialization closure |
//! | `fitness_fn` | `Fn(&[U::Gene]) -> f64` | Yes (constructor) | — | Fitness evaluation function |
//!
//! ### Optional Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `age_scheme` | `AlpsAgeScheme` | No | `Fibonacci` | Layer age threshold calculation |
//! | `age_gap` | `usize` | No | `5` | Base age unit for threshold computation |
//! | `injection_interval` | `usize` | No | `10` | Generations between layer-0 reseeding (0 = disable) |
//! | `crossover` | `Crossover` | No | `Uniform` | Offspring crossover operator |
//! | `mutation` | `Mutation` | No | `Gaussian` | Gene mutation operator |
//! | `mutation_sigma` | `Option<f64>` | No | `0.1` | Standard deviation for Gaussian mutation |
//! | `mutation_step` | `Option<f64>` | No | `None` | Step size for Creep mutation |
//! | `fitness_target` | `Option<f64>` | No | `None` | Stop when best fitness reaches this |
//! | `observer` | `Option<Arc<dyn GaObserver<U>>>` | No | `None` | Lifecycle observer |
//!
//! ### Age Schemes
//!
//! | Scheme | Formula | Properties |
//! |--------|---------|------------|
//! | [`Linear`](AlpsAgeScheme::Linear) | `(i+1) * age_gap` | Even spacing; good default |
//! | [`Fibonacci`](AlpsAgeScheme::Fibonacci) | `fib(i+2) * age_gap` | Rapid turnover in lower layers, very stable upper layers |
//! | [`Polynomial`](AlpsAgeScheme::Polynomial) | `(i+1)^2 * age_gap` | Exponentially expanding windows; slow convergence |
//!
//! ## Complete Example
//!
//! ```rust,ignore
//! use genetic_algorithms::alps::{
//!     AlpsAgeScheme, AlpsConfiguration, AlpsEngine,
//! };
//! use genetic_algorithms::chromosomes::Range as RangeChromosome;
//! use genetic_algorithms::genotypes::Range as RangeGenotype;
//! use genetic_algorithms::traits::ChromosomeT;
//!
//! // Rastrigin function (minimize toward 0.0 at origin)
//! let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
//!     let a = 10.0;
//!     let n = dna.len() as f64;
//!     a * n + dna.iter().map(|g| {
//!         g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos()
//!     }).sum::<f64>()
//! };
//!
//! let config = AlpsConfiguration::default()
//!     .with_n_layers(5)
//!     .with_layer_size(20)
//!     .with_age_scheme(AlpsAgeScheme::Fibonacci)
//!     .with_age_gap(5)
//!     .with_injection_interval(10)
//!     .with_max_generations(500)
//!     .with_mutation_sigma(0.1);
//!
//! let init_fn = |n: usize| -> Vec<RangeChromosome<f64>> {
//!     (0..n).map(|_| {
//!         let mut c = RangeChromosome::new();
//!         for i in 0..10 {
//!             c.dna.push(RangeGenotype::new(i as i32, vec![(-5.12, 5.12)], 0.0));
//!         }
//!         c
//!     }).collect()
//! };
//!
//! let mut engine = AlpsEngine::new(config, init_fn, fitness_fn);
//! let result = engine.run();
//! println!("Best fitness: {:?}", result.best_fitness);
//! ```
//!
//! ## Configuration Tips
//!
//! - **Fibonacci age scheme** is the recommended starting point — it gives new layers room to explore
//!   while keeping older layers stable for exploitation
//! - **5-8 layers** is the typical range; more layers = better diversity but higher overhead
//! - **age_gap = 5** works well for most problems; increase for longer convergence times
//! - Set **injection_interval = 0** to disable re-initialization if you prefer purely
//!   age-driven exploration
//! - Cross-layer mating probability is fixed at 20% — this balances exploration vs exploitation
//!   without requiring additional tuning
//!
//! ## When to Choose This vs Standard GA
//!
//! | Factor | AlpsEngine | Ga |
//! |--------|------------|-----|
//! | Population structure | Multiple age layers | Single population |
//! | Diversity mechanism | Age-based promotion + injection | Niching / extension (configured) |
//! | Convergence protection | Strong (fresh injection prevents stagnation) | Moderate (extension strategy required) |
//! | Per-generation cost | Higher (multiple layers) | Lower (single population) |
//! | Best for | Multimodal landscapes, anytime algorithms | General single-objective optimization |
//!
//! ## References
//!
//! - Hornby, G. S. (2006). ALPS: The Age-Layered Population Structure for Reducing the Problem
//!   of Premature Convergence. *Proceedings of the 8th Annual Conference on Genetic and
//!   Evolutionary Computation (GECCO)*, 815–822.

use std::sync::Arc;

use crate::configuration::{CrossoverConfiguration, ProblemSolving};
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::operations::mutation::ValueMutable;
use crate::operations::{crossover, mutation};
use crate::rng::make_rng;
use crate::stats::GenerationStats;
use crate::traits::{ChromosomeT, FitnessFn};
use rand::Rng;
use log::warn;

use super::configuration::AlpsConfiguration;

/// Result returned by [`AlpsEngine::run`].
pub struct AlpsResult<U: ChromosomeT> {
    /// Final layer populations (index 0 = youngest).
    pub layers: Vec<Vec<U>>,
    /// The best individual found across all layers during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
}

/// ALPS (Age-Layered Population Structure) engine.
///
/// Evolves multiple age-layered sub-populations with cross-layer mating and
/// periodic reseeding of the youngest layer to maintain diversity.
///
/// # Example
///
/// ```ignore
/// use genetic_algorithms::alps::{AlpsAgeScheme, AlpsConfiguration, AlpsEngine};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::genotypes::Range as RangeGene;
/// use genetic_algorithms::configuration::ProblemSolving;
///
/// let config = AlpsConfiguration::default()
///     .with_n_layers(5)
///     .with_layer_size(20)
///     .with_age_scheme(AlpsAgeScheme::Fibonacci)
///     .with_age_gap(5)
///     .with_injection_interval(10)
///     .with_max_generations(500)
///     .with_mutation_sigma(0.1)
///     .with_problem_solving(ProblemSolving::Minimization)
///     .with_fitness_target(0.01);
///
/// let mut engine = AlpsEngine::new(
///     config,
///     |n| /* build n chromosomes */ todo!(),
///     |dna| dna.iter().map(|g| g.value() * g.value()).sum(),
/// );
/// let result = engine.run();
/// ```
pub struct AlpsEngine<U: ChromosomeT> {
    config: AlpsConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: ChromosomeT> AlpsEngine<U> {
    /// Construct a new engine.
    ///
    /// * `config` — layer count, age scheme, operators, and stopping criteria.
    /// * `init_fn` — called with a count `n`; must return `n` initialised
    ///   chromosomes (fitness is ignored — the engine re-evaluates).
    /// * `fitness_fn` — maps a DNA slice to a scalar fitness value.
    pub fn new(
        config: AlpsConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
            observer: None,
        }
    }

    /// Attach a lifecycle observer. Zero overhead when not set.
    pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(observer);
        self
    }
}

impl<U> AlpsEngine<U>
where
    U: ChromosomeT + Clone + ValueMutable + 'static,
{
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Run the ALPS algorithm and return the result.
    pub fn run(&mut self) -> AlpsResult<U> {
        let max_ages = self.config.max_ages();
        let crossover_cfg = CrossoverConfiguration {
            method: self.config.crossover,
            ..CrossoverConfiguration::default()
        };

        // ── Initialise layer 0 ────────────────────────────────────────────────
        let mut layers: Vec<Vec<U>> = vec![vec![]; self.config.n_layers];
        layers[0] = self.fresh_individuals(self.config.layer_size);

        // ── Best tracking ─────────────────────────────────────────────────────
        // Use a direction-aware sentinel so prev_best_fitness is never NaN on
        // generation 0, which would cause is_better(real, NaN) → true and fire
        // on_new_best unconditionally on the very first generation.
        let mut best_fitness = match self.config.problem_solving {
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => f64::MAX,
            ProblemSolving::Maximization => f64::MIN,
        };
        for ind in &layers[0] {
            if self.is_better(ind.fitness(), best_fitness) {
                best_fitness = ind.fitness();
            }
        }
        let mut best = layers[0]
            .iter()
            .max_by(|a, b| {
                if self.is_better(a.fitness(), b.fitness()) {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            })
            .cloned()
            .unwrap_or_else(|| layers[0][0].clone());

        let mut rng = make_rng();
        let mut generations = 0usize;
        let mut target_reached = false;
        let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
        let mut stats_history: Vec<GenerationStats> = Vec::new();
        self.notify(|obs| obs.on_run_start());

        // ── Main loop ─────────────────────────────────────────────────────────
        for gen in 0..self.config.max_generations {
            self.notify(|obs| obs.on_generation_start(gen));
            // Snapshot best before any evolution this generation — on_new_best fires once per generation
            let prev_best_fitness = best_fitness;

            // --- Evolve each layer -------------------------------------------
            for layer_idx in 0..self.config.n_layers {
                if layers[layer_idx].is_empty() {
                    continue;
                }

                // Optionally bring in the best individual from the adjacent
                // older layer as an extra parent (cross-layer mating).
                let elder_best: Option<U> = if layer_idx + 1 < self.config.n_layers {
                    self.find_best(&layers[layer_idx + 1]).map(|i| layers[layer_idx + 1][i].clone())
                } else {
                    None
                };

                let layer_len = layers[layer_idx].len();
                let mut new_offspring: Vec<U> = Vec::with_capacity(layer_len);

                // Pair up individuals and produce offspring.
                for _ in 0..layer_len {
                    let a = rng.random_range(0..layer_len);
                    // Pick the second parent: either from the layer or the elder.
                    let parent_2 = if let Some(ref elder) = elder_best {
                        if rng.random::<f64>() < 0.2 {
                            // 20% chance to mate with the elder from the older layer
                            elder.clone()
                        } else {
                            let mut b = rng.random_range(0..layer_len);
                            while b == a && layer_len > 1 {
                                b = rng.random_range(0..layer_len);
                            }
                            layers[layer_idx][b].clone()
                        }
                    } else {
                        let mut b = rng.random_range(0..layer_len);
                        while b == a && layer_len > 1 {
                            b = rng.random_range(0..layer_len);
                        }
                        layers[layer_idx][b].clone()
                    };

                    let mut offspring = match crossover::factory(
                        &layers[layer_idx][a],
                        &parent_2,
                        crossover_cfg,
                    ) {
                        Ok(children) if !children.is_empty() => children.into_iter().next().unwrap(),
                        _ => layers[layer_idx][a].clone(),
                    };

                    let mutation_result = if self.config.mutation == crate::operations::Mutation::Cauchy {
                        mutation::factory_with_params(
                            self.config.mutation,
                            &mut offspring,
                            self.config.mutation_step,
                            None,
                        )
                    } else if self.config.mutation == crate::operations::Mutation::LevyFlight {
                        mutation::factory_with_params(
                            self.config.mutation,
                            &mut offspring,
                            None,
                            self.config.mutation_sigma,
                        )
                    } else {
                        mutation::factory_with_params(
                            self.config.mutation,
                            &mut offspring,
                            self.config.mutation_step,
                            self.config.mutation_sigma,
                        )
                    };
                    if let Err(e) = mutation_result {
                        warn!(target: "mutation_events", "Mutation error (skipped): {}", e);
                    }

                    let f = (self.fitness_fn)(offspring.dna());
                    offspring.set_fitness(f);
                    offspring.set_age(0);

                    if self.is_better(f, best_fitness) {
                        best_fitness = f;
                        best = offspring.clone();
                    }

                    new_offspring.push(offspring);
                }

                // Merge parents + offspring; keep best `layer_size`.
                layers[layer_idx].extend(new_offspring);
                self.keep_best(&mut layers[layer_idx], self.config.layer_size);
            }

            // --- Increment age of all survivors ------------------------------
            for layer in &mut layers {
                for ind in layer.iter_mut() {
                    ind.set_age(ind.age() + 1);
                }
            }

            // --- Promote aged-out individuals (youngest → oldest) ------------
            for layer_idx in 0..self.config.n_layers {
                let max_age = max_ages[layer_idx];
                let mut promoted: Vec<U> = Vec::new();
                layers[layer_idx].retain(|ind| {
                    if ind.age() > max_age {
                        promoted.push(ind.clone());
                        false
                    } else {
                        true
                    }
                });

                if layer_idx + 1 < self.config.n_layers && !promoted.is_empty() {
                    layers[layer_idx + 1].extend(promoted);
                    // Cap the receiving layer to avoid unbounded growth.
                    self.keep_best(&mut layers[layer_idx + 1], self.config.layer_size * 2);
                }
                // Overflow from the oldest layer is discarded.
            }

            // --- Track global best from all layers ---------------------------
            for layer in &layers {
                for ind in layer {
                    if self.is_better(ind.fitness(), best_fitness) {
                        best_fitness = ind.fitness();
                        best = ind.clone();
                    }
                }
            }
            if self.is_better(best_fitness, prev_best_fitness) {
                self.notify(|obs| obs.on_new_best(gen, best.clone()));
            }

            // --- Periodic injection into layer 0 -----------------------------
            if self.config.injection_interval > 0
                && gen > 0
                && gen % self.config.injection_interval == 0
            {
                layers[0] = self.fresh_individuals(self.config.layer_size);
                for ind in &layers[0] {
                    if self.is_better(ind.fitness(), best_fitness) {
                        best_fitness = ind.fitness();
                        best = ind.clone();
                    }
                }
            }

            // Observer: merged fitness stats across all layers (D-06)
            let fitness_values: Vec<f64> = layers
                .iter()
                .flat_map(|layer| layer.iter().map(|ind| ind.fitness()))
                .collect();
            if !fitness_values.is_empty() {
                let gen_stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
                stats_history.push(gen_stats);
                self.notify(|obs| obs.on_generation_end(stats_history.last().unwrap()));
            }

            generations += 1;

            // --- Early stopping ----------------------------------------------
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    target_reached = true;
                    break;
                }
            }
        }

        let cause = if target_reached {
            TerminationCause::FitnessTargetReached
        } else {
            TerminationCause::GenerationLimitReached
        };
        self.notify(|obs| obs.on_run_end(cause, &stats_history));

        AlpsResult { layers, best, best_fitness, generations }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Build `n` fresh individuals, evaluate their fitness, and set age to 0.
    fn fresh_individuals(&self, n: usize) -> Vec<U> {
        let mut inds = (self.init_fn)(n);
        for ind in &mut inds {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
            ind.set_age(0);
        }
        inds
    }

    /// Return the index of the best individual in `pop`, or `None` if empty.
    fn find_best(&self, pop: &[U]) -> Option<usize> {
        if pop.is_empty() {
            return None;
        }
        let mut best_idx = 0;
        let mut best_f = pop[0].fitness();
        for (i, ind) in pop.iter().enumerate().skip(1) {
            if self.is_better(ind.fitness(), best_f) {
                best_f = ind.fitness();
                best_idx = i;
            }
        }
        Some(best_idx)
    }

    /// Retain only the best `k` individuals in-place; stable sort by fitness.
    fn keep_best(&self, pop: &mut Vec<U>, k: usize) {
        if pop.len() <= k {
            return;
        }
        // Partial sort: bring the best k to the front.
        match self.config.problem_solving {
            ProblemSolving::Minimization => {
                pop.sort_unstable_by(|a, b| {
                    a.fitness().partial_cmp(&b.fitness()).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ProblemSolving::Maximization => {
                pop.sort_unstable_by(|a, b| {
                    b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ProblemSolving::FixedFitness => {
                let t = self.config.fitness_target.unwrap_or(0.0);
                pop.sort_unstable_by(|a, b| {
                    (a.fitness() - t).abs().partial_cmp(&(b.fitness() - t).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
        pop.truncate(k);
    }

    fn is_better(&self, candidate: f64, current: f64) -> bool {
        if current.is_nan() {
            return true;
        }
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
}
