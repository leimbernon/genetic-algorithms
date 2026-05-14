//! # Cellular Genetic Algorithm (CellularEngine)
//!
//! ## Description
//!
//! The Cellular Genetic Algorithm (cGA) places individuals on a **2D toroidal grid** where each
//! cell holds exactly one chromosome. Unlike a standard GA where the entire population is a single
//! breeding pool, cellular GAs restrict mating to **local neighborhoods** — individuals compete
//! and mate only with their immediate neighbors. This spatial structure creates slow information
//! diffusion across the grid, which naturally preserves population diversity and allows multiple
//! competing niches to emerge.
//!
//! Each generation, for every cell on the grid:
//!
//! 1. Collect neighbors according to the configured neighborhood topology (toroidal wrapping)
//! 2. Select a mate from the neighborhood using the configured Selection operator
//! 3. Apply Crossover and Mutation to produce an offspring
//! 4. Replace the cell if the offspring is fitter (greedy local replacement)
//!
//! In **synchronous** mode, all replacements are committed after the full sweep. In **asynchronous**
//! mode, replacements are applied immediately, allowing later cells to read offspring produced
//! earlier in the same generation (typically converges faster).
//!
//! ## When to Use
//!
//! - **Problem type:** Single-objective — continuous, binary, or permutation
//! - **Number of objectives:** 1
//! - **Variable type:** Any (requires [`ValueMutable`]
//!   for in-place mutation)
//! - **Key strength:** Excellent diversity preservation through spatial structure; natural
//!   parallelization (each cell evolves independently)
//! - **Key weakness:** Slower convergence than standard GA due to restricted gene flow;
//!   grid size introduces additional tuning parameters
//!
//! ## Quick Reference
//!
//! ### Mandatory Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `rows` | `usize` | Yes (via builder) | `10` | Number of rows in the toroidal grid |
//! | `cols` | `usize` | Yes (via builder) | `10` | Number of columns in the toroidal grid |
//! | `max_generations` | `usize` | Yes (via builder) | `1000` | Maximum number of generations |
//! | `init_fn` | `Fn(usize) -> Vec<U>` | Yes (constructor) | — | Population initialization closure |
//! | `fitness_fn` | `Fn(&[U::Gene]) -> f64` | Yes (constructor) | — | Fitness evaluation function |
//!
//! ### Optional Parameters
//!
//! | Parameter | Type | Required | Default | Description |
//! |-----------|------|----------|---------|-------------|
//! | `neighborhood` | `Neighborhood` | No | `Moore` | Cell neighborhood topology |
//! | `update_mode` | `UpdateMode` | No | `Asynchronous` | Synchronous or asynchronous updates |
//! | `selection` | `Selection` | No | `Tournament` | Mate selection from neighborhood |
//! | `crossover` | `Crossover` | No | `Uniform` | Offspring crossover strategy |
//! | `mutation` | `Mutation` | No | `Gaussian` | Gene mutation strategy |
//! | `mutation_sigma` | `Option<f64>` | No | `0.1` | Standard deviation for Gaussian mutation |
//! | `mutation_step` | `Option<f64>` | No | `None` | Step size for Creep mutation |
//! | `fitness_target` | `Option<f64>` | No | `None` | Stop when best fitness reaches this |
//! | `observer` | `Option<Arc<dyn GaObserver<U>>>` | No | `None` | Lifecycle observer |
//!
//! ### Neighborhood Topologies
//!
//! | Topology | Neighbors | Description |
//! |----------|-----------|-------------|
//! | [`VonNeumann`](Neighborhood::VonNeumann) | 4 | North, South, East, West (L1 distance <= 1) |
//! | [`Moore`](Neighborhood::Moore) | 8 | 3×3 square minus center (L-inf distance <= 1) |
//! | [`CompactR2`](Neighborhood::CompactR2) | 24 | 5×5 square minus center (L-inf distance <= 2) |
//! | [`Linear`](Neighborhood::Linear) | 2 | Left and right in row-major order |
//!
//! ## Complete Example
//!
//! ```rust,ignore
//! use genetic_algorithms::cellular::{
//!     CellularConfiguration, CellularEngine, Neighborhood, UpdateMode,
//! };
//! use genetic_algorithms::chromosomes::Range as RangeChromosome;
//! use genetic_algorithms::genotypes::Range as RangeGenotype;
//! use genetic_algorithms::operations::{Crossover, Mutation, Selection};
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
//! let config = CellularConfiguration::default()
//!     .with_grid(8, 8)
//!     .with_neighborhood(Neighborhood::Moore)
//!     .with_update_mode(UpdateMode::Asynchronous)
//!     .with_max_generations(200)
//!     .with_mutation_sigma(0.1)
//!     .with_selection(Selection::Tournament)
//!     .with_crossover(Crossover::Uniform)
//!     .with_mutation(Mutation::Gaussian);
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
//! let mut engine = CellularEngine::new(config, init_fn, fitness_fn);
//! let result = engine.run();
//! println!("Best fitness: {:?}", result.best_fitness);
//! ```
//!
//! ## Configuration Tips
//!
//! - **Moore neighborhood** is the best general-purpose starting point; larger neighborhoods
//!   (CompactR2) increase gene flow and speed convergence at the cost of diversity
//! - **Asynchronous update** typically converges faster than synchronous but may lose diversity
//! - Grid size should roughly equal the population size of a standard GA (e.g., 10×10 = 100 cells,
//!   15×15 = 225 cells). Smaller grids converge faster; larger grids preserve more diversity
//! - The **toroidal wrapping** ensures edge cells have the same number of neighbors as center cells
//!
//! ## When to Choose This vs Island Model
//!
//! | Factor | CellularEngine | IslandGa |
//! |--------|---------------|----------|
//! | Structure | 2D grid, fine-grained | Sub-populations, coarse-grained |
//! | Gene flow | Slow (local neighborhoods) | Periodic (migration events) |
//! | Diversity | Very high (spatial) | High (isolation) |
//! | Parallelism | Cell-level (fine-grained) | Island-level (coarse-grained) |
//! | Topology options | 4 neighborhood types | 5 migration topologies |
//!
//! ## References
//!
//! - Whitley, D. (1993). Cellular Genetic Algorithms. *Proceedings of the 5th International
//!   Conference on Genetic Algorithms*, 10–19.
//! - Alba, E., & Dorronsoro, B. (2008). *Cellular Genetic Algorithms*. Springer.
//! - Whitley, D., Rana, S., & Heckendorn, R. B. (1998). The Island Model Genetic Algorithm:
//!   On Separability, Population Size and Convergence. *Journal of Computing and Information
//!   Technology*.

use std::sync::Arc;

use crate::configuration::{CrossoverConfiguration, ProblemSolving};
use crate::ga::TerminationCause;
use crate::observer::GaObserver;
use crate::operations::mutation::ValueMutable;
use crate::operations::{crossover, mutation};
use crate::stats::GenerationStats;
use crate::traits::SelectionOperator;
use crate::rng::make_rng;
use crate::traits::{ChromosomeT, FitnessFn};
use rand::Rng;
use log::warn;

use super::configuration::{CellularConfiguration, Neighborhood, UpdateMode};

/// Result returned by [`CellularEngine::run`].
pub struct CellularResult<U: ChromosomeT> {
    /// Final grid population (row-major, length = `rows × cols`).
    pub population: Vec<U>,
    /// The best individual found during the run.
    pub best: U,
    /// Fitness of the best individual.
    pub best_fitness: f64,
    /// Number of generations completed.
    pub generations: usize,
}

/// Cellular Genetic Algorithm engine.
///
/// Evolves a population laid out on a 2D toroidal grid using local
/// neighbourhood interactions.
///
/// # Example
///
/// ```ignore
/// use genetic_algorithms::cellular::{CellularConfiguration, CellularEngine, Neighborhood, UpdateMode};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::genotypes::Range as RangeGene;
/// use genetic_algorithms::operations::{Crossover, Mutation, Selection};
/// use genetic_algorithms::configuration::ProblemSolving;
///
/// let config = CellularConfiguration::default()
///     .with_grid(8, 8)
///     .with_neighborhood(Neighborhood::Moore)
///     .with_update_mode(UpdateMode::Asynchronous)
///     .with_max_generations(200)
///     .with_mutation_sigma(0.1)
///     .with_problem_solving(ProblemSolving::Minimization)
///     .with_fitness_target(0.01);
///
/// let mut engine = CellularEngine::new(
///     config,
///     |n| /* build n chromosomes */ todo!(),
///     |dna| dna.iter().map(|g| g.value() * g.value()).sum(),
/// );
/// let result = engine.run();
/// ```
pub struct CellularEngine<U: ChromosomeT> {
    config: CellularConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U> CellularEngine<U>
where
    U: ChromosomeT,
{
    /// Construct a new engine.
    ///
    /// * `config` — grid and algorithm parameters.
    /// * `init_fn` — called once with `rows * cols`; must return that many
    ///   initialised chromosomes (fitness is ignored — the engine re-evaluates).
    /// * `fitness_fn` — maps a DNA slice to a scalar fitness value.
    pub fn new(
        config: CellularConfiguration,
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

impl<U> CellularEngine<U>
where
    U: ChromosomeT + Clone + ValueMutable + 'static,
{
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Run the Cellular GA and return the result.
    pub fn run(&mut self) -> CellularResult<U> {
        let rows = self.config.rows;
        let cols = self.config.cols;
        let pop_size = rows * cols;

        assert!(
            rows * cols >= 2,
            "CellularEngine requires a grid with at least 2 cells"
        );

        // ── Initialise ────────────────────────────────────────────────────────
        let mut pop: Vec<U> = (self.init_fn)(pop_size);
        for ind in &mut pop {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        // ── Best tracking ─────────────────────────────────────────────────────
        let mut best_fitness = pop[0].fitness();
        let mut best = pop[0].clone();
        for ind in &pop {
            if self.is_better(ind.fitness(), best_fitness) {
                best_fitness = ind.fitness();
                best = ind.clone();
            }
        }

        let crossover_cfg = CrossoverConfiguration {
            method: self.config.crossover,
            ..CrossoverConfiguration::default()
        };

        let mut rng = make_rng();
        let mut generations = 0usize;
        let is_maximization = matches!(self.config.problem_solving, ProblemSolving::Maximization);
        let mut stats_history: Vec<GenerationStats> = Vec::new();
        self.notify(|obs| obs.on_run_start());

        // ── Main loop ─────────────────────────────────────────────────────────
        for gen in 0..self.config.max_generations {
            self.notify(|obs| obs.on_generation_start(gen));
            let source: Vec<U> = match self.config.update_mode {
                UpdateMode::Synchronous => pop.clone(), // read from snapshot
                UpdateMode::Asynchronous => vec![],     // unused; reads directly from pop
            };

            let is_sync = matches!(self.config.update_mode, UpdateMode::Synchronous);
            // For synchronous mode, collect replacements and apply after the sweep.
            let mut replacements: Vec<(usize, U)> = Vec::new();
            // Snapshot best before inner sweep — on_new_best fires once per generation
            let prev_best_fitness = best_fitness;

            for row in 0..rows {
                for col in 0..cols {
                    let cell_idx = row * cols + col;

                    // Collect neighbor indices
                    let neighbor_idxs = self.neighbors(row, col, rows, cols);
                    if neighbor_idxs.is_empty() {
                        continue;
                    }

                    // Read source population (snapshot for sync, live for async)
                    let src_pop: &[U] = if is_sync { &source } else { &pop };

                    // Build a local neighbor slice for selection (includes self so
                    // the operator always has at least 2 choices)
                    let mut local: Vec<U> = Vec::with_capacity(neighbor_idxs.len() + 1);
                    local.push(src_pop[cell_idx].clone()); // index 0 = self
                    for &ni in &neighbor_idxs {
                        local.push(src_pop[ni].clone());
                    }

                    // Select a mate from the neighborhood using the configured operator.
                    // We ask for 1 couple; take the second element of the pair as the
                    // mate so we don't just pick the cell itself.
                    let pairs = self.config.selection.select(&local, 1, 1);
                    let mate_local_idx = if let Some(&(a, b)) = pairs.first() {
                        // Prefer the non-self member of the pair; fall back to `b`.
                        if a != 0 { a } else if b != 0 { b } else {
                            rng.random_range(1..local.len())
                        }
                    } else {
                        // Fallback: random neighbor
                        rng.random_range(1..local.len())
                    };

                    // Crossover cell with selected mate
                    let parent_cell = &src_pop[cell_idx];
                    let parent_mate = &local[mate_local_idx];
                    let mut offspring = match crossover::factory(parent_cell, parent_mate, crossover_cfg) {
                        Ok(children) if !children.is_empty() => children.into_iter().next().unwrap(),
                        _ => parent_cell.clone(),
                    };

                    // Mutate offspring
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

                    // Evaluate
                    let offspring_fitness = (self.fitness_fn)(offspring.dna());
                    offspring.set_fitness(offspring_fitness);

                    // Greedy local replacement
                    if self.is_better(offspring_fitness, src_pop[cell_idx].fitness()) {
                        if self.is_better(offspring_fitness, best_fitness) {
                            best_fitness = offspring_fitness;
                            best = offspring.clone();
                        }
                        if is_sync {
                            replacements.push((cell_idx, offspring));
                        } else {
                            pop[cell_idx] = offspring;
                        }
                    }
                }
            }

            // Apply synchronous replacements
            if is_sync {
                for (idx, ind) in replacements {
                    pop[idx] = ind;
                }
            }

            generations += 1;

            // Observer: on_new_best fires ONCE per generation if global best improved
            if self.is_better(best_fitness, prev_best_fitness) {
                self.notify(|obs| obs.on_new_best(gen, best.clone()));
            }

            // Observer: on_generation_end with stats from full grid
            let fitness_values: Vec<f64> = pop.iter().map(|c| c.fitness()).collect();
            let gen_stats = GenerationStats::from_fitness_values(gen, &fitness_values, is_maximization);
            stats_history.push(gen_stats);
            self.notify(|obs| obs.on_generation_end(stats_history.last().unwrap()));

            // Early stopping
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    break;
                }
            }
        }

        let cause = if generations < self.config.max_generations {
            TerminationCause::FitnessTargetReached
        } else {
            TerminationCause::GenerationLimitReached
        };
        self.notify(|obs| obs.on_run_end(cause, &stats_history));

        CellularResult { population: pop, best, best_fitness, generations }
    }

    // ── Grid helpers ──────────────────────────────────────────────────────────

    /// Returns the neighbor indices for the cell at `(row, col)` in a
    /// `rows × cols` toroidal grid, according to the configured neighborhood.
    fn neighbors(&self, row: usize, col: usize, rows: usize, cols: usize) -> Vec<usize> {
        match &self.config.neighborhood {
            Neighborhood::VonNeumann => {
                let mut v = Vec::with_capacity(4);
                let r = row;
                let c = col;
                // North
                v.push(((r + rows - 1) % rows) * cols + c);
                // South
                v.push(((r + 1) % rows) * cols + c);
                // West
                v.push(r * cols + (c + cols - 1) % cols);
                // East
                v.push(r * cols + (c + 1) % cols);
                v.sort_unstable();
                v.dedup();
                v.retain(|&i| i != row * cols + col);
                v
            }
            Neighborhood::Moore => {
                let mut v = Vec::with_capacity(8);
                for dr in [-1i64, 0, 1] {
                    for dc in [-1i64, 0, 1] {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let nr = ((row as i64 + dr).rem_euclid(rows as i64)) as usize;
                        let nc = ((col as i64 + dc).rem_euclid(cols as i64)) as usize;
                        v.push(nr * cols + nc);
                    }
                }
                v.sort_unstable();
                v.dedup();
                v
            }
            Neighborhood::CompactR2 => {
                let mut v = Vec::with_capacity(24);
                for dr in [-2i64, -1, 0, 1, 2] {
                    for dc in [-2i64, -1, 0, 1, 2] {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let nr = ((row as i64 + dr).rem_euclid(rows as i64)) as usize;
                        let nc = ((col as i64 + dc).rem_euclid(cols as i64)) as usize;
                        v.push(nr * cols + nc);
                    }
                }
                v.sort_unstable();
                v.dedup();
                v.retain(|&i| i != row * cols + col);
                v
            }
            Neighborhood::Linear => {
                let idx = row * cols + col;
                let n = rows * cols;
                let left = (idx + n - 1) % n;
                let right = (idx + 1) % n;
                let mut v = vec![left, right];
                v.sort_unstable();
                v.dedup();
                v.retain(|&i| i != idx);
                v
            }
        }
    }

    // ── Fitness helpers ───────────────────────────────────────────────────────

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
}
