//! Configuration types for the Cellular Genetic Algorithm engine.

use crate::configuration::ProblemSolving;
use crate::operations::{Crossover, Mutation, Selection};

/// Neighborhood topology for the Cellular GA grid.
///
/// Defines which cells are considered "neighbors" of a given cell on the
/// 2D toroidal grid.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::cellular::Neighborhood;
///
/// let n = Neighborhood::Moore;
/// assert_eq!(n, Neighborhood::Moore);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Neighborhood {
    /// Von Neumann neighborhood — 4 cells (North, South, East, West).
    ///
    /// Uses L1 distance ≤ 1, so only axis-aligned neighbors are included.
    VonNeumann,
    /// Moore neighborhood — 8 cells (3×3 square minus center).
    ///
    /// Uses L∞ (Chebyshev) distance ≤ 1.
    Moore,
    /// Compact neighborhood with radius 2 — 24 cells (5×5 square minus center).
    ///
    /// Uses L∞ (Chebyshev) distance ≤ 2.
    CompactR2,
    /// Linear (ring) neighborhood — 2 cells (left and right in row-major order).
    ///
    /// Treats the grid as a 1-D ring ignoring the 2-D structure.
    Linear,
}

/// Update mode for the Cellular GA.
///
/// Controls whether cells read from the previous generation's state or the
/// current (partially updated) state when computing offspring.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::cellular::UpdateMode;
///
/// let mode = UpdateMode::Asynchronous;
/// assert_eq!(mode, UpdateMode::Asynchronous);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateMode {
    /// All cells read from the previous generation; writes are applied after
    /// the full sweep.  Produces a strictly generation-based evolution.
    Synchronous,
    /// Cells are updated in-place; later cells in the sweep may read offspring
    /// produced earlier in the same generation.  Typically converges faster.
    Asynchronous,
}

/// Configuration for a [`CellularEngine`](super::engine::CellularEngine) run.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::cellular::{CellularConfiguration, Neighborhood, UpdateMode};
/// use genetic_algorithms::configuration::ProblemSolving;
/// use genetic_algorithms::operations::{Crossover, Mutation, Selection};
///
/// let config = CellularConfiguration::default()
///     .with_grid(8, 8)
///     .with_neighborhood(Neighborhood::Moore)
///     .with_update_mode(UpdateMode::Asynchronous)
///     .with_max_generations(200)
///     .with_problem_solving(ProblemSolving::Minimization);
/// ```
#[derive(Debug, Clone)]
pub struct CellularConfiguration {
    /// Number of rows in the toroidal grid.
    pub rows: usize,
    /// Number of columns in the toroidal grid.
    pub cols: usize,
    /// Neighborhood topology used to determine cell interactions.
    pub neighborhood: Neighborhood,
    /// Whether to apply synchronous or asynchronous updates.
    pub update_mode: UpdateMode,
    /// Maximum number of generations before stopping.
    pub max_generations: usize,
    /// Selection method used to pick a mate from the neighborhood.
    pub selection: Selection,
    /// Crossover operator applied to the cell and its selected mate.
    pub crossover: Crossover,
    /// Mutation operator applied to each offspring.
    pub mutation: Mutation,
    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,
    /// Optional fitness target — engine stops early when reached.
    pub fitness_target: Option<f64>,
}

impl Default for CellularConfiguration {
    fn default() -> Self {
        Self {
            rows: 10,
            cols: 10,
            neighborhood: Neighborhood::Moore,
            update_mode: UpdateMode::Asynchronous,
            max_generations: 1000,
            selection: Selection::Tournament,
            crossover: Crossover::Uniform,
            mutation: Mutation::Gaussian { sigma: Some(0.1) },
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
        }
    }
}

impl CellularConfiguration {
    /// Builder: set grid dimensions.
    pub fn with_grid(mut self, rows: usize, cols: usize) -> Self {
        self.rows = rows;
        self.cols = cols;
        self
    }
    /// Builder: set neighborhood topology.
    pub fn with_neighborhood(mut self, n: Neighborhood) -> Self {
        self.neighborhood = n;
        self
    }
    /// Builder: set update mode.
    pub fn with_update_mode(mut self, m: UpdateMode) -> Self {
        self.update_mode = m;
        self
    }
    /// Builder: set maximum generations.
    pub fn with_max_generations(mut self, n: usize) -> Self {
        self.max_generations = n;
        self
    }
    /// Builder: set selection method.
    pub fn with_selection(mut self, s: Selection) -> Self {
        self.selection = s;
        self
    }
    /// Builder: set crossover operator.
    pub fn with_crossover(mut self, c: Crossover) -> Self {
        self.crossover = c;
        self
    }
    /// Builder: set mutation operator.
    pub fn with_mutation(mut self, m: Mutation) -> Self {
        self.mutation = m;
        self
    }
    /// Builder: set problem solving direction.
    pub fn with_problem_solving(mut self, ps: ProblemSolving) -> Self {
        self.problem_solving = ps;
        self
    }
    /// Builder: set fitness target for early stopping.
    pub fn with_fitness_target(mut self, t: f64) -> Self {
        self.fitness_target = Some(t);
        self
    }
    /// Returns the total grid population size (`rows * cols`).
    pub fn population_size(&self) -> usize {
        self.rows * self.cols
    }
}
