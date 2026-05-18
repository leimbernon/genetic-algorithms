//! Configuration for the IBEA indicator-based multi-objective genetic algorithm.
//!
//! IBEA (Zitzler & Kunzli 2004) is a multi-objective evolutionary algorithm that
//! uses a pairwise indicator function (additive epsilon, I_eps+) to assign fitness
//! values. Environmental selection iteratively removes the individual with the
//! lowest indicator fitness, recalculating fitnesses after each removal.

pub use crate::nsga2::configuration::ObjectiveDirection;

/// Configuration for the IBEA indicator-based multi-objective genetic algorithm.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::ibea::configuration::{IbeaConfiguration, ObjectiveDirection};
///
/// let config = IbeaConfiguration::new()
///     .with_num_objectives(2)
///     .with_population_size(100)
///     .with_max_generations(250);
///
/// assert_eq!(config.num_objectives, 2);
/// assert_eq!(config.population_size, 100);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IbeaConfiguration {
    /// Number of objective functions.
    pub num_objectives: usize,
    /// Population size.
    pub population_size: usize,
    /// Maximum number of generations.
    pub max_generations: usize,
    /// Per-objective optimization direction. If empty, all objectives default to `Minimize`.
    /// When set, the length must match `num_objectives`.
    pub objective_directions: Vec<ObjectiveDirection>,
}

impl Default for IbeaConfiguration {
    fn default() -> Self {
        IbeaConfiguration {
            num_objectives: 2,
            population_size: 100,
            max_generations: 250,
            objective_directions: Vec::new(),
        }
    }
}

impl IbeaConfiguration {
    /// Creates a new `IbeaConfiguration` with default values.
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
