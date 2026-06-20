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
use crate::traits::{FitnessFn, LinearChromosome, RealGene};

use super::configuration::{inertia_weight, PsoConfiguration, PsoTopology};

// ─── PsoResult ────────────────────────────────────────────────────────────────

/// Result returned by [`PsoEngine::run`].
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoResult};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::RealGene;
///
/// let config = PsoConfiguration::default();
/// let mut engine = PsoEngine::<RangeChromosome<f64>>::new(
///     config,
///     |n| vec![RangeChromosome::default(); n],
///     |dna| dna.iter().map(|g| g.real_value().powi(2)).sum(),
/// );
/// let result: PsoResult<RangeChromosome<f64>> = engine.run().unwrap();
/// println!("Best fitness: {}", result.best_fitness);
/// ```
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

// ─── PsoState ─────────────────────────────────────────────────────────────────

/// Private internal state for the PSO algorithm.
///
/// Allocated once before the run loop and updated in-place each generation.
/// Stores all swarm bookkeeping that is not part of the chromosome/population.
struct PsoState {
    /// Problem dimension (genes per chromosome).
    dim: usize,
    /// Number of particles.
    n_particles: usize,
    /// Velocities: `velocities[particle][gene]`.
    velocities: Vec<Vec<f64>>,
    /// Personal best positions: `pbest_positions[particle][gene]`.
    pbest_positions: Vec<Vec<f64>>,
    /// Personal best fitness values: `pbest_fitness[particle]`.
    pbest_fitness: Vec<f64>,
    /// Global best position (gbest topology).
    gbest_position: Vec<f64>,
    /// Global best fitness.
    gbest_fitness: f64,
    /// Index of the particle whose personal-best last updated `gbest`.
    gbest_owner: usize,
    /// Maximum allowed velocity per gene `v_max[gene] = hi - lo`.
    v_max: Vec<f64>,
}

impl PsoState {
    /// Construct the initial PSO state from the already-evaluated initial population.
    ///
    /// * `pop` — the evaluated initial population (fitness values set).
    /// * `best_idx` — index of the best individual in `pop` (pre-computed by the caller).
    /// * `rng` — RNG for initial velocity sampling.
    ///
    /// Initial velocity per gene `d`: uniform in `[-v_max[d], +v_max[d]]` where
    /// `v_max[d] = hi_d - lo_d` from `gene.bounds()` (D-02, D-08).
    fn new<U: LinearChromosome, R: Rng>(pop: &[U], best_idx: usize, rng: &mut R) -> Self
    where
        U::Gene: RealGene,
    {
        let n_particles = pop.len();
        let dim = pop[0].dna().len();

        // Derive v_max per gene from the first chromosome's gene bounds.
        let v_max: Vec<f64> = (0..dim)
            .map(|d| {
                pop[0]
                    .dna()
                    .get(d)
                    .and_then(|g| g.bounds())
                    .map(|(lo, hi)| hi - lo)
                    .unwrap_or(1.0)
            })
            .collect();

        // Sample initial velocities: uniform in [-v_max[d], +v_max[d]] per gene.
        let velocities: Vec<Vec<f64>> = (0..n_particles)
            .map(|_| {
                (0..dim)
                    .map(|d| rng.random::<f64>() * 2.0 * v_max[d] - v_max[d])
                    .collect()
            })
            .collect();

        // Personal best = initial positions and fitness values (Pitfall #2 fix).
        let pbest_positions: Vec<Vec<f64>> = pop
            .iter()
            .map(|ind| ind.dna().iter().map(|g| g.real_value()).collect())
            .collect();
        let pbest_fitness: Vec<f64> = pop.iter().map(|ind| ind.fitness()).collect();

        // Global best = best individual in the initial population.
        let gbest_position = pbest_positions[best_idx].clone();
        let gbest_fitness = pbest_fitness[best_idx];

        PsoState {
            dim,
            n_particles,
            velocities,
            pbest_positions,
            pbest_fitness,
            gbest_position,
            gbest_fitness,
            gbest_owner: best_idx,
            v_max,
        }
    }
}

// ─── PsoEngine ────────────────────────────────────────────────────────────────

/// PSO engine.
///
/// Generic over the chromosome type `U`; `U::Gene` must implement [`RealGene`]
/// so that velocity and position arithmetic can be performed on gene values.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::pso::{PsoConfiguration, PsoEngine};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::RealGene;
///
/// let config = PsoConfiguration::default()
///     .with_max_generations(500);
///
/// let mut engine = PsoEngine::<RangeChromosome<f64>>::new(
///     config,
///     |n| vec![RangeChromosome::default(); n],
///     |dna| dna.iter().map(|g| g.real_value().powi(2)).sum(),
/// );
/// let result = engine.run();
/// println!("Generations: {}", result.generations);
/// ```
pub struct PsoEngine<U: LinearChromosome>
where
    U::Gene: RealGene,
{
    config: PsoConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
    fitness_cache: Option<Arc<Mutex<crate::fitness::cache::FitnessCache>>>,
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
            fitness_cache: None,
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

    /// Returns `true` if the `fitness` value satisfies the stopping `target`.
    fn reached_target(&self, fitness: f64, target: f64) -> bool {
        match self.config.problem_solving {
            ProblemSolving::Minimization => fitness <= target,
            ProblemSolving::Maximization => fitness >= target,
            ProblemSolving::FixedFitness => (fitness - target).abs() < 1e-6,
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

    /// Compute the lbest (ring topology) best position for gene `d` for particle `i`.
    ///
    /// Neighborhood of particle `i` with `neighborhood_size = k` (clamped to
    /// `n_particles - 1` if larger) includes particle `i` itself plus
    /// `floor(k/2)` left-side and `ceil(k/2)` right-side neighbors by index
    /// (ring-wrapped via `(i + n - offset) % n`).
    ///
    /// Returns `pbest_positions[winner][d]` where `winner` is the neighbor with
    /// the best personal-best fitness.
    fn lbest_position(
        &self,
        particle_i: usize,
        gene_d: usize,
        neighborhood_size: usize,
        state: &PsoState,
    ) -> f64 {
        let n = state.n_particles;
        // Clamp k to [1, n-1] so we never query 0 or n neighbors.
        let k = neighborhood_size.min(n - 1).max(1);
        let half_left = k / 2;
        let half_right = k.div_ceil(2);

        let mut best_idx = particle_i;
        let mut best_fit = state.pbest_fitness[particle_i];

        // Left-side neighbors: (i + n - 1) % n, (i + n - 2) % n, ...
        for offset in 1..=half_left {
            let j = (particle_i + n - offset) % n;
            if self.is_better(state.pbest_fitness[j], best_fit) {
                best_fit = state.pbest_fitness[j];
                best_idx = j;
            }
        }

        // Right-side neighbors: (i + 1) % n, (i + 2) % n, ...
        for offset in 1..=half_right {
            let j = (particle_i + offset) % n;
            if self.is_better(state.pbest_fitness[j], best_fit) {
                best_fit = state.pbest_fitness[j];
                best_idx = j;
            }
        }

        state.pbest_positions[best_idx][gene_d]
    }

    /// Run the PSO algorithm and return the result.
    ///
    /// When `population_size` is 0, a default of 30 particles is used
    /// (PSO literature standard).
    pub fn run(&mut self) -> Result<PsoResult<U>, GaError>
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

        // ── Observer: run start ───────────────────────────────────────────────
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
            return Err(GaError::InitializationError(
                "PsoEngine: init_fn returned an empty population".to_string(),
            ));
        }

        // ── Evaluate initial population fitness ───────────────────────────────
        for ind in &mut pop {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        // ── Identify best from initial population ─────────────────────────────
        let (best_idx, mut best_fitness) = self.find_best(&pop);
        let mut best = pop[best_idx].clone();

        // ── Initialise PSO internal state ─────────────────────────────────────
        let mut state = PsoState::new(&pop, best_idx, &mut rng);

        // ── Observer: initial best ────────────────────────────────────────────
        self.notify(|obs| obs.on_new_best(0, &best));

        let mut termination_cause = TerminationCause::GenerationLimitReached;
        let mut all_stats: Vec<GenerationStats> = Vec::with_capacity(self.config.max_generations);

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

        // ── Main loop ─────────────────────────────────────────────────────────
        for gen in 0..self.config.max_generations {
            self.notify(|obs| obs.on_generation_start(gen));

            let w = inertia_weight(&self.config.inertia, gen, self.config.max_generations);

            // ── Update each particle ──────────────────────────────────────────
            // `i` is required to cross-index state.velocities[i], state.pbest_positions[i],
            // state.pbest_fitness[i] — three independent vectors. Refactoring to
            // enumerate() over a single vec would break this contract.
            #[allow(clippy::needless_range_loop)]
            for i in 0..pop.len() {
                // Read current position.
                let x_curr: Vec<f64> = pop[i].dna().iter().map(|g| g.real_value()).collect();
                let mut new_positions = x_curr.clone();

                for d in 0..state.dim {
                    let r1: f64 = rng.random();
                    let r2: f64 = rng.random();

                    // Resolve social attractor (gbest or lbest).
                    let best_d = match &self.config.topology {
                        PsoTopology::Global => state.gbest_position[d],
                        PsoTopology::Ring { neighborhood_size } => {
                            self.lbest_position(i, d, *neighborhood_size, &state)
                        }
                    };

                    // Velocity update formula (Shi & Eberhart 1998).
                    let mut new_v = w * state.velocities[i][d]
                        + self.config.c1 * r1 * (state.pbest_positions[i][d] - x_curr[d])
                        + self.config.c2 * r2 * (best_d - x_curr[d]);

                    // Clamp velocity magnitude BEFORE position update (Pitfall #3).
                    new_v = new_v.clamp(-state.v_max[d], state.v_max[d]);

                    // Position update.
                    let mut new_x = x_curr[d] + new_v;

                    // Absorbing boundary: clamp position and zero velocity (D-07).
                    if let Some((lo, hi)) = pop[i].dna()[d].bounds() {
                        if new_x < lo {
                            new_x = lo;
                            new_v = 0.0;
                        } else if new_x > hi {
                            new_x = hi;
                            new_v = 0.0;
                        }
                    }

                    // Write back velocity and position.
                    state.velocities[i][d] = new_v;
                    new_positions[d] = new_x;
                }

                // Build new DNA from updated positions.
                let new_dna: Vec<U::Gene> = pop[i]
                    .dna()
                    .iter()
                    .enumerate()
                    .map(|(d, g)| g.with_real_value(new_positions[d]))
                    .collect();
                pop[i].set_dna(Cow::Owned(new_dna));

                // Evaluate fitness for updated particle.
                let new_fit = (self.fitness_fn)(pop[i].dna());
                pop[i].set_fitness(new_fit);

                // Personal best update: strictly improves (Pitfall #2 + strict rule).
                if self.is_better(new_fit, state.pbest_fitness[i]) {
                    state.pbest_fitness[i] = new_fit;
                    state.pbest_positions[i] = new_positions.clone();
                }
            }

            // ── Synchronous gbest update (after full particle sweep — Pitfall #4) ──
            for j in 0..state.n_particles {
                if self.is_better(state.pbest_fitness[j], state.gbest_fitness) {
                    state.gbest_fitness = state.pbest_fitness[j];
                    state.gbest_position = state.pbest_positions[j].clone();
                    state.gbest_owner = j;
                }
            }

            // Track engine-level best and notify observer when it improves.
            if self.is_better(state.gbest_fitness, best_fitness) {
                best_fitness = state.gbest_fitness;
                // Reconstruct best from the gbest owner's pbest position so that
                // `result.best` matches `result.best_fitness` exactly (CR-01).
                let owner = state.gbest_owner;
                let new_dna: Vec<U::Gene> = pop[owner]
                    .dna()
                    .iter()
                    .enumerate()
                    .map(|(d, g)| g.with_real_value(state.gbest_position[d]))
                    .collect();
                best = pop[owner].clone();
                best.set_dna(Cow::Owned(new_dna));
                best.set_fitness(state.gbest_fitness);
                self.notify(|obs| obs.on_new_best(gen, &best));
            }

            // ── Generation stats ──────────────────────────────────────────────
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

            // ── Early stopping ────────────────────────────────────────────────
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    termination_cause = TerminationCause::FitnessTargetReached;
                    break;
                }
            }
        }

        // ── Observer: run end ─────────────────────────────────────────────────
        let generations = all_stats.len();
        let all_stats_ref = all_stats.as_slice();
        self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));

        Ok(PsoResult {
            population: pop,
            best,
            best_fitness,
            generations,
        })
    }
}
