/// Configuration for the NSGA-II multi-objective genetic algorithm.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::nsga2::configuration::Nsga2Configuration;
///
/// let config = Nsga2Configuration::new()
///     .with_num_objectives(2)
///     .with_population_size(100)
///     .with_max_generations(500);
///
/// assert_eq!(config.num_objectives, 2);
/// assert_eq!(config.population_size, 100);
/// assert_eq!(config.max_generations, 500);
/// ```
#[derive(Debug, Clone)]
pub struct Nsga2Configuration {
    /// Number of objective functions.
    pub num_objectives: usize,
    /// Population size.
    pub population_size: usize,
    /// Maximum number of generations.
    pub max_generations: usize,
}

impl Default for Nsga2Configuration {
    fn default() -> Self {
        Nsga2Configuration {
            num_objectives: 2,
            population_size: 100,
            max_generations: 200,
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
}
