//! `CellularEngine` — the Cellular Genetic Algorithm execution loop.
//!
//! A Cellular GA places individuals on a 2D toroidal grid.  Each cell evolves
//! by interacting only with its local neighborhood, which promotes spatial
//! diversity and the emergence of multiple competing niches.
//!
//! # Algorithm
//!
//! 1. **Initialisation** — fill the grid with `rows × cols` individuals via
//!    the user-supplied `init_fn`; evaluate fitness for all of them.
//! 2. **Evolution loop** (up to `max_generations`) — for each cell: collect
//!    neighbors (toroidal wrapping), select a mate via the configured
//!    `Selection` operator, apply `Crossover` and `Mutation`, evaluate the
//!    offspring, and replace the cell if the offspring is fitter (greedy
//!    local replacement).  In *synchronous* mode replacements are committed
//!    after the full sweep; in *asynchronous* mode they are applied
//!    immediately.
//! 3. Return the final grid, best individual, and number of generations run.

use std::sync::Arc;

use crate::configuration::{CrossoverConfiguration, ProblemSolving};
use crate::operations::mutation::ValueMutable;
use crate::operations::crossover;
use crate::rng::make_rng;
use crate::traits::{FitnessFn, LinearChromosome, MutationOperator, SelectionOperator};
use rand::Rng;

use super::configuration::{CellularConfiguration, Neighborhood, UpdateMode};

/// Result returned by [`CellularEngine::run`].
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::cellular::{CellularConfiguration, CellularEngine, CellularResult};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// let config = CellularConfiguration::default();
/// let mut engine = CellularEngine::<RangeChromosome<f64>>::new(
///     config,
///     |n| vec![RangeChromosome::default(); n],
///     |dna| dna.iter().map(|g| g.value() * g.value()).sum(),
/// );
/// let result: CellularResult<RangeChromosome<f64>> = engine.run();
/// println!("Best fitness: {}", result.best_fitness);
/// ```
pub struct CellularResult<U: LinearChromosome> {
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
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::cellular::{CellularConfiguration, CellularEngine, Neighborhood, UpdateMode};
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::operations::Mutation;
///
/// let config = CellularConfiguration::default()
///     .with_grid(8, 8)
///     .with_neighborhood(Neighborhood::Moore)
///     .with_update_mode(UpdateMode::Asynchronous)
///     .with_max_generations(200)
///     .with_mutation(Mutation::Gaussian { sigma: Some(0.1) })
///     .with_problem_solving(ProblemSolving::Minimization);
///
/// let mut engine = CellularEngine::<RangeChromosome<f64>>::new(
///     config,
///     |n| vec![RangeChromosome::default(); n],
///     |dna| dna.iter().map(|g| g.value() * g.value()).sum(),
/// );
/// let result = engine.run();
/// println!("Generations: {}", result.generations);
/// ```
pub struct CellularEngine<U: LinearChromosome> {
    config: CellularConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
}

impl<U> CellularEngine<U>
where
    U: LinearChromosome,
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
        }
    }
}

impl<U> CellularEngine<U>
where
    U: LinearChromosome + Clone + ValueMutable + 'static,
{
    /// Run the Cellular GA and return the result.
    pub fn run(&mut self) -> CellularResult<U> {
        let rows = self.config.rows;
        let cols = self.config.cols;
        let pop_size = rows * cols;

        // ── Initialise ────────────────────────────────────────────────────────
        let mut pop: Vec<U> = (self.init_fn)(pop_size);
        for ind in &mut pop {
            let f = (self.fitness_fn)(ind.dna());
            ind.set_fitness(f);
        }

        // ── Best tracking ─────────────────────────────────────────────────────
        if pop.is_empty() {
            panic!("CellularEngine: grid must have at least 1 cell (rows > 0 && cols > 0)");
        }
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

        // ── Main loop ─────────────────────────────────────────────────────────
        for _gen in 0..self.config.max_generations {
            let source: Vec<U> = match self.config.update_mode {
                UpdateMode::Synchronous => pop.clone(), // read from snapshot
                UpdateMode::Asynchronous => vec![],     // unused; reads directly from pop
            };

            let is_sync = matches!(self.config.update_mode, UpdateMode::Synchronous);
            // For synchronous mode, collect replacements and apply after the sweep.
            let mut replacements: Vec<(usize, U)> = Vec::new();

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
                    // We ask for 1 couple with num_parents=2; take the second element of the
                    // group as the mate so we don't just pick the cell itself.
                    let pairs = self.config.selection.select(&local, 1, 1, 2);
                    let mate_local_idx = if let Some(group) = pairs.first() {
                        let a = group[0];
                        let b = group[1];
                        // Prefer the non-self member of the pair; fall back to `b`.
                        if a != 0 {
                            a
                        } else if b != 0 {
                            b
                        } else {
                            rng.random_range(1..local.len())
                        }
                    } else {
                        // Fallback: random neighbor
                        rng.random_range(1..local.len())
                    };

                    // Crossover cell with selected mate
                    let parent_cell = &src_pop[cell_idx];
                    let parent_mate = &local[mate_local_idx];
                    let mut offspring =
                        match crossover::factory(parent_cell, parent_mate, crossover_cfg) {
                            Ok(children) if !children.is_empty() => {
                                children.into_iter().next().unwrap()
                            }
                            _ => parent_cell.clone(),
                        };

                    // Mutate offspring
                    let _ = self.config.mutation.mutate(&mut offspring, &self.config.mutation);

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

            // Early stopping
            if let Some(target) = self.config.fitness_target {
                if self.reached_target(best_fitness, target) {
                    break;
                }
            }
        }

        CellularResult {
            population: pop,
            best,
            best_fitness,
            generations,
        }
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
