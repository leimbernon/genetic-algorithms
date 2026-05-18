//! Configuration for the SPEA2 strength-Pareto evolutionary algorithm.
//!
//! SPEA2 (Zitzler, Laumanns & Thiele 2001) maintains a fixed-size external
//! archive of non-dominated solutions. Fitness is computed from raw strength
//! (domination count) plus density (k-nearest-neighbour distance), and the
//! archive is truncated using iterative nearest-neighbour removal when it
//! exceeds capacity.

pub use crate::nsga2::configuration::ObjectiveDirection;

/// Configuration for the SPEA2 strength-Pareto evolutionary algorithm.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};
///
/// let config = Spea2Configuration::new()
///     .with_num_objectives(2)
///     .with_population_size(100)
///     .with_archive_size(100)
///     .with_max_generations(250);
///
/// assert_eq!(config.num_objectives, 2);
/// assert_eq!(config.archive_size, 100);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spea2Configuration {
    /// Number of objective functions.
    pub num_objectives: usize,
    /// Population size (size of the main population per generation).
    pub population_size: usize,
    /// External archive size (default: equals population_size per canonical SPEA2).
    pub archive_size: usize,
    /// Maximum number of generations.
    pub max_generations: usize,
    /// Per-objective optimization direction. If empty, all objectives default to `Minimize`.
    /// When set, the length must match `num_objectives`.
    pub objective_directions: Vec<ObjectiveDirection>,
}

impl Default for Spea2Configuration {
    fn default() -> Self {
        Spea2Configuration {
            num_objectives: 2,
            population_size: 100,
            archive_size: 100,
            max_generations: 250,
            objective_directions: Vec::new(),
        }
    }
}

impl Spea2Configuration {
    /// Creates a new `Spea2Configuration` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of objectives.
    pub fn with_num_objectives(mut self, n: usize) -> Self {
        self.num_objectives = n;
        self
    }

    /// Sets the population size.
    pub fn with_population_size(mut self, size: usize) -> Self {
        self.population_size = size;
        self
    }

    /// Sets the external archive size (D-01). Default: equals population_size.
    ///
    /// `validate()` rejects `archive_size > population_size` or `archive_size == 0`.
    pub fn with_archive_size(mut self, size: usize) -> Self {
        self.archive_size = size;
        self
    }

    /// Sets the maximum number of generations.
    pub fn with_max_generations(mut self, gens: usize) -> Self {
        self.max_generations = gens;
        self
    }

    /// Sets the per-objective optimization directions.
    pub fn with_objective_directions(mut self, directions: Vec<ObjectiveDirection>) -> Self {
        self.objective_directions = directions;
        self
    }

    /// Returns the effective directions, defaulting to `Minimize` for each
    /// objective when `objective_directions` is empty.
    pub fn effective_directions(&self) -> Vec<ObjectiveDirection> {
        if self.objective_directions.is_empty() {
            vec![ObjectiveDirection::Minimize; self.num_objectives]
        } else {
            self.objective_directions.clone()
        }
    }
}
