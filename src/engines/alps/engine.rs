//! `AlpsEngine` — the Age-Layered Population Structure execution loop.
//!
//! ALPS maintains several sub-populations ("layers") ordered by individual age.
//! Layer 0 is the youngest; layer `n_layers - 1` is the oldest.  Each layer
//! has a maximum-age threshold determined by the chosen age scheme.  When an
//! individual's age exceeds its layer's threshold, it is promoted to the next
//! older layer (or discarded if it is already in the oldest layer).
//!
//! # Algorithm
//!
//! 1. **Initialisation** — fill layer 0 with `layer_size` individuals; all
//!    other layers start empty.
//! 2. **Evolution loop** (up to `max_generations`) — each generation: evolve
//!    every layer (random pairing, optional cross-layer mating with the best
//!    individual from the adjacent older layer, crossover, mutation, survivor
//!    trimming); increment all individual ages; promote aged-out individuals to
//!    the next older layer (oldest-layer overflow is discarded); and every
//!    `injection_interval` generations reinitialise layer 0 with fresh random
//!    individuals.
//! 3. Return all layers, the overall best individual, and generation count.

use std::sync::Arc;

use crate::configuration::{CrossoverConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::operations::crossover;
use crate::operations::mutation::ValueMutable;
use crate::rng::make_rng;
use crate::traits::MutationOperator;
use crate::traits::{FitnessFn, LinearChromosome};
use rand::Rng;

use super::configuration::AlpsConfiguration;

/// Result returned by [`AlpsEngine::run`].
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::alps::{AlpsConfiguration, AlpsEngine, AlpsResult};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// let config = AlpsConfiguration::default();
/// let mut engine = AlpsEngine::<RangeChromosome<f64>>::new(
///     config,
///     |n| vec![RangeChromosome::default(); n],
///     |dna| dna.iter().map(|g| g.value() * g.value()).sum(),
/// ).unwrap();
/// let result: AlpsResult<RangeChromosome<f64>> = engine.run();
/// println!("Best fitness: {}", result.best_fitness);
/// ```
pub struct AlpsResult<U: LinearChromosome> {
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
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::alps::{AlpsAgeScheme, AlpsConfiguration, AlpsEngine};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::operations::{Mutation, GaussianParams};
///
/// let config = AlpsConfiguration::default()
///     .with_n_layers(5)
///     .with_layer_size(20)
///     .with_age_scheme(AlpsAgeScheme::Fibonacci)
///     .with_age_gap(5)
///     .with_injection_interval(10)
///     .with_max_generations(500)
///     .with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.1) }))
///     .with_problem_solving(ProblemSolving::Minimization)
///     .with_fitness_target(0.01);
///
/// let mut engine = AlpsEngine::<RangeChromosome<f64>>::new(
///     config,
///     |n| vec![RangeChromosome::default(); n],
///     |dna| dna.iter().map(|g| g.value() * g.value()).sum(),
/// ).unwrap();
/// let result = engine.run();
/// println!("Generations: {}", result.generations);
/// ```
pub struct AlpsEngine<U: LinearChromosome> {
    config: AlpsConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
}

impl<U: LinearChromosome> AlpsEngine<U> {
    /// Construct a new engine.
    ///
    /// * `config` — layer count, age scheme, operators, and stopping criteria.
    /// * `init_fn` — called with a count `n`; must return `n` initialised
    ///   chromosomes (fitness is ignored — the engine re-evaluates).
    /// * `fitness_fn` — maps a DNA slice to a scalar fitness value.
    ///
    /// # Errors
    ///
    /// Returns `GaError::ConfigurationError` if `config.layer_size == 0` or
    /// `config.n_layers == 0` — zero-size configs are rejected at construction
    /// time (D-07: validation moved into `new()`).
    pub fn new(
        config: AlpsConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Result<Self, GaError> {
        if config.layer_size == 0 {
            return Err(GaError::ConfigurationError(
                "AlpsEngine: layer_size must be > 0".to_string(),
            ));
        }
        if config.n_layers == 0 {
            return Err(GaError::ConfigurationError(
                "AlpsEngine: n_layers must be > 0".to_string(),
            ));
        }
        Ok(Self {
            config,
            init_fn: Arc::new(init_fn),
            fitness_fn: Arc::new(fitness_fn),
        })
    }
}

impl<U> AlpsEngine<U>
where
    U: LinearChromosome + Clone + ValueMutable + crate::traits::RealValuedMutation + 'static,
{
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
        let mut best_fitness = layers[0]
            .iter()
            .map(|u| u.fitness())
            .fold(
                f64::NAN,
                |acc, f| if self.is_better(f, acc) { f } else { acc },
            );
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

        // ── Main loop ─────────────────────────────────────────────────────────
        for gen in 0..self.config.max_generations {
            // --- Evolve each layer -------------------------------------------
            for layer_idx in 0..self.config.n_layers {
                if layers[layer_idx].is_empty() {
                    continue;
                }

                // Optionally bring in the best individual from the adjacent
                // older layer as an extra parent (cross-layer mating).
                let elder_best: Option<U> = if layer_idx + 1 < self.config.n_layers {
                    self.find_best(&layers[layer_idx + 1])
                        .map(|i| layers[layer_idx + 1][i].clone())
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

                    let mut offspring =
                        match crossover::factory(&layers[layer_idx][a], &parent_2, crossover_cfg) {
                            Ok(children) if !children.is_empty() => {
                                children.into_iter().next().unwrap()
                            }
                            _ => layers[layer_idx][a].clone(),
                        };

                    let _ = self
                        .config
                        .mutation
                        .mutate(&mut offspring, &self.config.mutation);

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

            generations += 1;

            // --- Early stopping ----------------------------------------------
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    break;
                }
            }
        }

        AlpsResult {
            layers,
            best,
            best_fitness,
            generations,
        }
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
                    a.fitness()
                        .partial_cmp(&b.fitness())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            _ => {
                pop.sort_unstable_by(|a, b| {
                    b.fitness()
                        .partial_cmp(&a.fitness())
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
