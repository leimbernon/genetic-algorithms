//! `PsoEngine` — Particle Swarm Optimization execution engine.
//!
//! Implements the canonical PSO algorithm (Kennedy & Eberhart 1995) with the
//! inertia weight extension (Shi & Eberhart 1998) for real-valued continuous
//! optimization. The engine is generic over the chromosome type `U`; `U::Gene`
//! must implement [`RealGene`] so that velocity and position updates can be
//! performed on gene values.
//!
//! # WASM compatibility
//!
//! The core loop contains no `Instant::now()` calls and no parallel iteration.
//! The engine compiles safely for `wasm32-unknown-unknown`.

use std::sync::Arc;

use crate::configuration::ProblemSolving;
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::stats::GenerationStats;
use crate::traits::{FitnessFn, LinearChromosome, RealGene};

use super::configuration::PsoConfiguration;

// ─── PsoResult ────────────────────────────────────────────────────────────────

/// Result returned by [`PsoEngine::run`].
pub struct PsoResult<U: LinearChromosome> {
    /// Final population (all particles evaluated at end of run).
    pub population: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
}

// ─── PsoEngine ────────────────────────────────────────────────────────────────

/// PSO engine.
///
/// Generic over the chromosome type `U`; `U::Gene` must implement [`RealGene`]
/// so that velocity and position arithmetic can be performed on gene values.
///
/// # Example
///
/// ```ignore
/// use genetic_algorithms::pso::{PsoConfiguration, PsoEngine};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::genotypes::Range as RangeGene;
///
/// let config = PsoConfiguration::default()
///     .with_max_generations(500);
///
/// let mut engine = PsoEngine::new(
///     config,
///     |n| { /* return Vec<RangeChromosome<f64>> of length n */ todo!() },
///     |dna| dna.iter().map(|g| g.real_value().powi(2)).sum(),
/// );
/// let result = engine.run();
/// ```
pub struct PsoEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: PsoConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: LinearChromosome + Clone> PsoEngine<U>
where
    U::Gene: RealGene,
{
    /// Construct a new engine.
    ///
    /// * `config` — algorithm parameters.
    /// * `init_fn` — called once with `population_size`; must return that many
    ///   initialised chromosomes.
    /// * `fitness_fn` — maps a DNA slice to a scalar fitness value.
    pub fn new(
        config: PsoConfiguration,
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

    /// Returns `true` if `candidate` is better than `current` under the
    /// configured optimization direction.
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

    /// Returns `(index, fitness)` of the best individual in `pop`.
    fn find_best(&self, pop: &[U]) -> (usize, f64) {
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

    /// Run the PSO algorithm and return the result.
    ///
    /// When `population_size` is 0, a default of 30 particles is used
    /// (PSO literature standard).
    ///
    /// # Stub (Plan 02)
    ///
    /// This implementation builds the initial population, evaluates fitness,
    /// identifies the best individual, fires observer hooks, and returns a
    /// `PsoResult` with `generations = 0`.
    ///
    /// TODO(plan-03): replace stub with full PSO velocity-update loop.
    pub fn run(&mut self) -> PsoResult<U> {
        let is_maximization =
            matches!(self.config.problem_solving, ProblemSolving::Maximization);

        self.notify(|obs| obs.on_run_start());

        // ── Determine population size (default 30 when 0) ────────────────────
        let pop_size = if self.config.population_size == 0 {
            30
        } else {
            self.config.population_size
        };

        // ── Build initial population ──────────────────────────────────────────
        let mut pop: Vec<U> = (self.init_fn)(pop_size.max(1));

        // Guard: empty population from user's init_fn
        if pop.is_empty() {
            log::warn!(
                target: "pso_events",
                "PsoEngine: init_fn returned an empty population; returning empty result"
            );
            self.notify(|obs| {
                obs.on_run_end(TerminationCause::GenerationLimitReached, &[])
            });
            panic!("PsoEngine: init_fn returned an empty population");
        }

        // ── Evaluate initial population fitness ───────────────────────────────
        for ind in &mut pop {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        // ── Identify best from initial population ─────────────────────────────
        let (best_idx, best_fitness) = self.find_best(&pop);
        let best = pop[best_idx].clone();

        let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
        let _stats =
            GenerationStats::from_fitness_values(0, &fitness_values, is_maximization);

        // TODO(plan-03): replace stub with full PSO velocity-update loop.
        self.notify(|obs| {
            obs.on_run_end(TerminationCause::GenerationLimitReached, &[])
        });

        PsoResult {
            population: pop,
            best,
            best_fitness,
            generations: 0,
        }
    }
}
