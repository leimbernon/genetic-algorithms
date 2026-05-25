//! GpGa — the Genetic Programming engine.
//!
//! [`GpGa<N>`] executes the full GP generation loop over `GpChromosome<N>`:
//!
//! 1. Initialize population with a user-provided `init_fn` (default:
//!    [`ramped_half_and_half`])
//! 2. Evaluate fitness via `fitness_fn`
//! 3. For each generation:
//!    - parent selection (reuses `selection::factory`)
//!    - subtree crossover with up to 3 bloat-retry attempts
//!    - mutation per-offspring (stochastic)
//!    - fitness evaluation of offspring
//!    - survivor selection (reuses `survivor::factory`)
//!    - stats collection (including `avg_node_count`)
//!    - observer hooks
//!    - stopping criteria check
//! 4. Return [`GpResult<N>`] with best individual, best fitness, and the final
//!    population
//!
//! # WASM compatibility
//!
//! Fitness evaluation uses `par_iter_mut()` on non-WASM targets and `iter_mut()`
//! on WASM. No other rayon usage is present in this module.

use std::sync::Arc;
use std::time::Instant;

use rand::Rng;

use crate::error::GaError;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::operations::{selection, survivor};
use crate::rng::make_rng;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;

use super::chromosome::{GpChromosome, TreeChromosome};
use super::configuration::GpConfiguration;
use super::init::ramped_half_and_half;
use super::node::{GpNode, Node};

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

// ---------------------------------------------------------------------------
// Type aliases to satisfy clippy::type_complexity
// ---------------------------------------------------------------------------

/// Type alias for the fitness function stored in [`GpGa`].
type GpFitnessFn<N> = Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>;

/// Type alias for the initialization function stored in [`GpGa`].
type GpInitFn<N> = Arc<dyn Fn(usize, usize) -> Vec<GpChromosome<N>> + Send + Sync>;

// ---------------------------------------------------------------------------
// GpResult
// ---------------------------------------------------------------------------

/// Result returned by [`GpGa::run`].
pub struct GpResult<N: GpNode + Default> {
    /// Final population (all individuals evaluated).
    pub population: Vec<GpChromosome<N>>,
    /// The best individual found during the run.
    pub best: GpChromosome<N>,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
}

// ---------------------------------------------------------------------------
// GpGa
// ---------------------------------------------------------------------------

/// The Genetic Programming engine.
///
/// Parameterized on `N: GpNode` — the user's primitive-set enum. The engine
/// works exclusively with `GpChromosome<N>` internally.
///
/// # Example
///
/// ```ignore
/// use genetic_algorithms::gp::{GpChromosome, GpConfiguration, GpGa, MathNode};
///
/// let config = GpConfiguration::new()
///     .with_population_size(50)
///     .with_max_generations(20);
///
/// let mut engine = GpGa::with_ramped_half_and_half(config, |tree| {
///     // Walk `tree` and return a fitness value
///     0.0
/// });
/// let result = engine.run().unwrap();
/// ```
pub struct GpGa<N: GpNode + Default + Clone + Send + Sync + 'static> {
    config: GpConfiguration,
    fitness_fn: GpFitnessFn<N>,
    /// Called with `(population_size, init_max_depth)` — creates its own RNG internally.
    init_fn: GpInitFn<N>,
    observer: Option<Arc<dyn GaObserver<GpChromosome<N>> + Send + Sync>>,
}

impl<N> GpGa<N>
where
    N: GpNode + Default + Clone + Send + Sync + 'static,
{
    /// Constructs a new engine with a custom initialization function.
    ///
    /// * `config` — algorithm parameters (validated when `run()` is called).
    /// * `fitness_fn` — maps an expression tree to a scalar fitness value.
    /// * `init_fn` — called once with `(population_size, init_max_depth)`;
    ///   must return exactly `population_size` initialized chromosomes.
    ///   The closure is responsible for creating its own RNG (e.g., via `rng::make_rng()`).
    pub fn new(
        config: GpConfiguration,
        fitness_fn: impl Fn(&Node<N>) -> f64 + Send + Sync + 'static,
        init_fn: impl Fn(usize, usize) -> Vec<GpChromosome<N>> + Send + Sync + 'static,
    ) -> Self {
        GpGa {
            config,
            fitness_fn: Arc::new(fitness_fn) as GpFitnessFn<N>,
            init_fn: Arc::new(init_fn) as GpInitFn<N>,
            observer: None,
        }
    }

    /// Constructs a `GpGa` using the standard ramped half-and-half initializer.
    ///
    /// This is the primary constructor for most GP use cases. The population is
    /// built by [`ramped_half_and_half`] with depths in `2..=init_max_depth`.
    ///
    /// * `config` — algorithm parameters.
    /// * `fitness_fn` — maps an expression tree to a scalar fitness value.
    pub fn with_ramped_half_and_half(
        config: GpConfiguration,
        fitness_fn: impl Fn(&Node<N>) -> f64 + Send + Sync + 'static,
    ) -> Self {
        GpGa::new(config, fitness_fn, |pop_size, init_max_depth| {
            let mut rng = make_rng();
            ramped_half_and_half::<N>(pop_size, init_max_depth, &mut rng)
        })
    }

    /// Attaches a lifecycle observer (see [`GaObserver`] for available hooks).
    pub fn with_observer(
        mut self,
        obs: Arc<dyn GaObserver<GpChromosome<N>> + Send + Sync>,
    ) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op otherwise.
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<GpChromosome<N>>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Evaluates the fitness of every individual in `pop`.
    ///
    /// Uses `par_iter_mut()` on non-WASM targets (rayon) and `iter_mut()` on WASM.
    fn evaluate_population(&self, pop: &mut Vec<GpChromosome<N>>) {
        #[cfg(not(target_arch = "wasm32"))]
        pop.par_iter_mut().for_each(|chr| {
            let f = (self.fitness_fn)(chr.tree());
            chr.set_fitness(f);
        });

        #[cfg(target_arch = "wasm32")]
        pop.iter_mut().for_each(|chr| {
            let f = (self.fitness_fn)(chr.tree());
            chr.set_fitness(f);
        });
    }

    /// Returns the average node count across all individuals in `pop`.
    fn compute_avg_node_count(pop: &[GpChromosome<N>]) -> f64 {
        if pop.is_empty() {
            return 0.0;
        }
        pop.iter().map(|c| c.node_count() as f64).sum::<f64>() / pop.len() as f64
    }

    /// Returns `true` if `candidate` is better than `current` under the
    /// configured optimization direction.
    #[inline]
    fn is_better(&self, candidate: f64, current: f64) -> bool {
        if self.config.is_maximization {
            candidate > current
        } else {
            candidate < current
        }
    }

    /// Returns the index of the best individual in `pop`.
    fn find_best_index(pop: &[GpChromosome<N>], is_maximization: bool) -> usize {
        let mut best_idx = 0;
        for (i, chr) in pop.iter().enumerate().skip(1) {
            let better = if is_maximization {
                chr.fitness() > pop[best_idx].fitness()
            } else {
                chr.fitness() < pop[best_idx].fitness()
            };
            if better {
                best_idx = i;
            }
        }
        best_idx
    }

    /// Runs the GP algorithm and returns the result.
    ///
    /// # Errors
    ///
    /// Returns `Err(GaError::ConfigurationError)` if the configuration is
    /// invalid, or `Err(GaError::SelectionError)` if the population is too
    /// small for the configured selection method.
    pub fn run(&mut self) -> Result<GpResult<N>, GaError> {
        // Validate configuration upfront.
        self.config.build()?;

        self.notify(|obs| obs.on_run_start());

        let mut rng = make_rng();

        // ── Initialization ────────────────────────────────────────────────────
        let mut pop: Vec<GpChromosome<N>> =
            (self.init_fn)(self.config.population_size, self.config.init_max_depth);

        self.evaluate_population(&mut pop);

        // ── Initial best ──────────────────────────────────────────────────────
        let best_idx = Self::find_best_index(&pop, self.config.is_maximization);
        let mut best: GpChromosome<N> = pop[best_idx].clone();
        let mut best_fitness = best.fitness();
        let mut stagnation_count = 0usize;
        let mut termination_cause = TerminationCause::GenerationLimitReached;

        let mut all_stats: Vec<GenerationStats> = Vec::with_capacity(self.config.max_generations);

        // ── Main loop ─────────────────────────────────────────────────────────
        for gen in 0..self.config.max_generations {
            self.notify(|obs| obs.on_generation_start(gen));

            // Parent selection — reuse selection::factory from existing infra.
            let sel_cfg = self.config.effective_selection_config();
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
            let pairs = selection::factory(&pop, sel_cfg, 1)?;
            if let Some(t) = t_sel {
                self.notify(|obs| obs.on_selection_complete(gen, t.elapsed(), pairs.len()));
            }

            // Crossover + mutation → offspring
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
            let mut offspring: Vec<GpChromosome<N>> = Vec::with_capacity(pairs.len() * 2);

            let max_depth = self.config.max_depth;
            let max_node_count = self.config.max_node_count;

            for (i, j) in &pairs {
                // Crossover with bloat retry — T-53-08: hard cap of 3 retries.
                let mut crossover_result = None;
                for _ in 0..3 {
                    match self.config.crossover.apply(
                        &pop[*i],
                        &pop[*j],
                        max_depth,
                        max_node_count,
                        &mut rng,
                    ) {
                        Ok((c1, c2)) => {
                            crossover_result = Some((c1, c2));
                            break;
                        }
                        Err(e) => {
                            log::warn!(
                                target: "gp_events",
                                "Bloat rejected in crossover gen={}: {}",
                                gen,
                                e
                            );
                        }
                    }
                }

                // Fall back to the better parent copy on all-retry failure.
                let (mut c1, mut c2) = crossover_result.unwrap_or_else(|| {
                    let better = if self.is_better(pop[*i].fitness(), pop[*j].fitness()) {
                        pop[*i].clone()
                    } else {
                        pop[*j].clone()
                    };
                    (better.clone(), better)
                });

                // Apply each mutation with its configured probability.
                for (mutation, prob) in &self.config.mutations {
                    if rng.random::<f64>() < *prob {
                        if let Err(e) = mutation.apply(&mut c1, max_depth, max_node_count, &mut rng)
                        {
                            log::warn!(
                                target: "gp_events",
                                "Bloat rejected in mutation gen={}: {}",
                                gen,
                                e
                            );
                        }
                    }
                    if rng.random::<f64>() < *prob {
                        if let Err(e) = mutation.apply(&mut c2, max_depth, max_node_count, &mut rng)
                        {
                            log::warn!(
                                target: "gp_events",
                                "Bloat rejected in mutation gen={}: {}",
                                gen,
                                e
                            );
                        }
                    }
                }

                offspring.push(c1);
                offspring.push(c2);
            }

            if let Some(t) = t_cx {
                let elapsed = t.elapsed();
                let offspring_count = offspring.len();
                let pop_size = pop.len();
                self.notify(|obs| obs.on_crossover_complete(gen, elapsed, offspring_count));
                self.notify(|obs| obs.on_mutation_complete(gen, elapsed, pop_size));
            }

            // Evaluate offspring fitness before merging into population.
            let t_fit: Option<Instant> = if self.observer.is_some() {
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
            self.evaluate_population(&mut offspring);
            if let Some(t) = t_fit {
                let pop_size = offspring.len();
                self.notify(|obs| obs.on_fitness_evaluation_complete(gen, t.elapsed(), pop_size));
            }

            // Merge parents + offspring, then trim to population_size.
            pop.extend(offspring);
            let limit_cfg = self.config.limit_configuration();
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
            survivor::factory(
                self.config.survivor,
                &mut pop,
                self.config.population_size,
                limit_cfg,
            )?;
            if let Some(t) = t_surv {
                let pop_size = pop.len();
                self.notify(|obs| obs.on_survivor_selection_complete(gen, t.elapsed(), pop_size));
            }

            // ── Best update ───────────────────────────────────────────────────
            let gen_best_idx = Self::find_best_index(&pop, self.config.is_maximization);
            let gen_best_fitness = pop[gen_best_idx].fitness();

            if self.is_better(gen_best_fitness, best_fitness) {
                best = pop[gen_best_idx].clone();
                best_fitness = gen_best_fitness;
                stagnation_count = 0;
                let best_clone = best.clone();
                self.notify(|obs| obs.on_new_best(gen, best_clone));
            } else {
                stagnation_count += 1;
                let sc = stagnation_count;
                self.notify(|obs| obs.on_stagnation(gen, sc));
            }

            // ── Stats ─────────────────────────────────────────────────────────
            let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
            let mut stats = GenerationStats::from_fitness_values(
                gen,
                &fitness_values,
                self.config.is_maximization,
            );
            stats.avg_node_count = Self::compute_avg_node_count(&pop);
            all_stats.push(stats.clone());
            self.notify(|obs| obs.on_generation_end(&stats));

            // ── Stopping criteria ─────────────────────────────────────────────
            if let Some(target) = self.config.fitness_target {
                let reached = if self.config.is_maximization {
                    best_fitness >= target
                } else {
                    best_fitness <= target
                };
                if reached {
                    termination_cause = TerminationCause::FitnessTargetReached;
                    break;
                }
            }

            if let Some(max_stag) = self.config.max_stagnation {
                if stagnation_count >= max_stag {
                    termination_cause = TerminationCause::StagnationReached;
                    break;
                }
            }
        }

        self.notify(|obs| obs.on_run_end(termination_cause, &all_stats));

        let generations = all_stats.len();
        Ok(GpResult {
            population: pop,
            best,
            best_fitness,
            generations,
        })
    }
}
