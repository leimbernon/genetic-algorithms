/// Direction of optimization for a single objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectiveDirection {
    /// Minimize this objective (lower is better).
    Minimize,
    /// Maximize this objective (higher is better).
    Maximize,
}

/// Configuration for the NSGA-II multi-objective genetic algorithm.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
///
/// let config = Nsga2Configuration::new()
///     .with_num_objectives(2)
///     .with_population_size(100)
///     .with_max_generations(500)
///     .with_objective_directions(vec![
///         ObjectiveDirection::Minimize,
///         ObjectiveDirection::Maximize,
///     ]);
///
/// assert_eq!(config.num_objectives, 2);
/// assert_eq!(config.population_size, 100);
/// assert_eq!(config.max_generations, 500);
/// assert_eq!(config.objective_directions.len(), 2);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nsga2Configuration {
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

impl Default for Nsga2Configuration {
    fn default() -> Self {
        Nsga2Configuration {
            num_objectives: 2,
            population_size: 100,
            max_generations: 200,
            objective_directions: Vec::new(),
        }
    }
}

impl Nsga2Configuration {
    /// Creates a new `Nsga2Configuration` with default values.
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
    ///
    /// The length of `directions` must match `num_objectives` at validation time.
    /// If not set, all objectives default to `Minimize`.
    pub fn with_objective_directions(mut self, directions: Vec<ObjectiveDirection>) -> Self {
        self.objective_directions = directions;
        self
    }

    /// Returns the effective directions, defaulting to `Minimize` for each objective
    /// when `objective_directions` is empty.
    pub fn effective_directions(&self) -> Vec<ObjectiveDirection> {
        if self.objective_directions.is_empty() {
            vec![ObjectiveDirection::Minimize; self.num_objectives]
        } else {
            self.objective_directions.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nsga2_configuration_default() {
        let config = Nsga2Configuration::default();
        assert_eq!(config.num_objectives, 2);
        assert_eq!(config.population_size, 100);
        assert_eq!(config.max_generations, 200);
        assert!(config.objective_directions.is_empty());
    }

    #[test]
    fn test_nsga2_configuration_builder() {
        let config = Nsga2Configuration::new()
            .with_num_objectives(3)
            .with_population_size(50)
            .with_max_generations(1000);

        assert_eq!(config.num_objectives, 3);
        assert_eq!(config.population_size, 50);
        assert_eq!(config.max_generations, 1000);
    }

    #[test]
    fn test_nsga2_configuration_directions() {
        let config = Nsga2Configuration::new()
            .with_num_objectives(2)
            .with_objective_directions(vec![
                ObjectiveDirection::Minimize,
                ObjectiveDirection::Maximize,
            ]);

        assert_eq!(config.objective_directions.len(), 2);
        assert_eq!(config.objective_directions[0], ObjectiveDirection::Minimize);
        assert_eq!(config.objective_directions[1], ObjectiveDirection::Maximize);
    }

    #[test]
    fn test_nsga2_configuration_effective_directions_default() {
        let config = Nsga2Configuration::new().with_num_objectives(3);
        let dirs = config.effective_directions();
        assert_eq!(dirs.len(), 3);
        assert!(dirs.iter().all(|d| *d == ObjectiveDirection::Minimize));
    }

    #[test]
    fn test_nsga2_configuration_effective_directions_explicit() {
        let config = Nsga2Configuration::new()
            .with_num_objectives(2)
            .with_objective_directions(vec![
                ObjectiveDirection::Maximize,
                ObjectiveDirection::Minimize,
            ]);
        let dirs = config.effective_directions();
        assert_eq!(dirs[0], ObjectiveDirection::Maximize);
        assert_eq!(dirs[1], ObjectiveDirection::Minimize);
    }
}
