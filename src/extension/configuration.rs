/// Configuration for extension strategies (population diversity control).
///
/// Extension strategies trigger when population diversity (measured by fitness
/// standard deviation) drops below a configurable threshold, applying a
/// corrective action to restore diversity.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::extension::configuration::ExtensionConfiguration;
/// use genetic_algorithms::operations::Extension;
///
/// let config = ExtensionConfiguration::new()
///     .with_method(Extension::MassExtinction)
///     .with_diversity_threshold(0.05)
///     .with_survival_rate(0.2)
///     .with_elite_count(2);
///
/// assert_eq!(config.method, Extension::MassExtinction);
/// assert!((config.diversity_threshold - 0.05).abs() < f64::EPSILON);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtensionConfiguration {
    /// The extension strategy to use.
    pub method: crate::operations::Extension,
    /// Diversity threshold (fitness standard deviation). The extension triggers
    /// when fitness_std_dev drops below this value.
    pub diversity_threshold: f64,
    /// For MassExtinction: fraction of population that survives the cull (0.0..1.0).
    pub survival_rate: f64,
    /// For MassDegeneration: number of mutation rounds applied to non-elite chromosomes.
    pub mutation_rounds: usize,
    /// Number of elite individuals protected from the extension event.
    pub elite_count: usize,
}

impl Default for ExtensionConfiguration {
    fn default() -> Self {
        ExtensionConfiguration {
            method: crate::operations::Extension::Noop,
            diversity_threshold: 0.01,
            survival_rate: 0.1,
            mutation_rounds: 3,
            elite_count: 1,
        }
    }
}

impl ExtensionConfiguration {
    /// Creates a new `ExtensionConfiguration` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the extension strategy.
    pub fn with_method(mut self, method: crate::operations::Extension) -> Self {
        self.method = method;
        self
    }

    /// Sets the diversity threshold (fitness standard deviation).
    pub fn with_diversity_threshold(mut self, threshold: f64) -> Self {
        self.diversity_threshold = threshold;
        self
    }

    /// Sets the survival rate for MassExtinction (0.0..1.0).
    pub fn with_survival_rate(mut self, rate: f64) -> Self {
        self.survival_rate = rate;
        self
    }

    /// Sets the number of mutation rounds for MassDegeneration.
    pub fn with_mutation_rounds(mut self, rounds: usize) -> Self {
        self.mutation_rounds = rounds;
        self
    }

    /// Sets the number of elite individuals protected from extension events.
    pub fn with_elite_count(mut self, count: usize) -> Self {
        self.elite_count = count;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::Extension;

    #[test]
    fn test_default() {
        let config = ExtensionConfiguration::default();
        assert_eq!(config.method, Extension::Noop);
        assert!((config.diversity_threshold - 0.01).abs() < f64::EPSILON);
        assert!((config.survival_rate - 0.1).abs() < f64::EPSILON);
        assert_eq!(config.mutation_rounds, 3);
        assert_eq!(config.elite_count, 1);
    }

    #[test]
    fn test_builder() {
        let config = ExtensionConfiguration::new()
            .with_method(Extension::MassExtinction)
            .with_diversity_threshold(0.05)
            .with_survival_rate(0.3)
            .with_mutation_rounds(5)
            .with_elite_count(2);

        assert_eq!(config.method, Extension::MassExtinction);
        assert!((config.diversity_threshold - 0.05).abs() < f64::EPSILON);
        assert!((config.survival_rate - 0.3).abs() < f64::EPSILON);
        assert_eq!(config.mutation_rounds, 5);
        assert_eq!(config.elite_count, 2);
    }
}
