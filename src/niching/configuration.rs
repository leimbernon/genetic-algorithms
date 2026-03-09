/// Configuration for fitness sharing / niching.
///
/// Fitness sharing reduces the fitness of individuals that are too similar to others,
/// promoting diversity in the population.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::niching::configuration::NichingConfiguration;
///
/// let config = NichingConfiguration::new()
///     .with_enabled(true)
///     .with_sigma_share(0.5)
///     .with_alpha(1.0);
///
/// assert!(config.enabled);
/// assert!((config.sigma_share - 0.5).abs() < f64::EPSILON);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NichingConfiguration {
    /// Whether fitness sharing is enabled.
    pub enabled: bool,
    /// Sharing radius: individuals closer than `sigma_share` share fitness.
    pub sigma_share: f64,
    /// Alpha parameter controlling the shape of the sharing function.
    /// Higher values make the sharing function steeper.
    pub alpha: f64,
}

impl Default for NichingConfiguration {
    fn default() -> Self {
        NichingConfiguration {
            enabled: false,
            sigma_share: 1.0,
            alpha: 1.0,
        }
    }
}

impl NichingConfiguration {
    /// Creates a new `NichingConfiguration` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables fitness sharing.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the sharing radius.
    ///
    /// # Arguments
    ///
    /// * `sigma_share` - The sharing radius. Must be > 0.
    pub fn with_sigma_share(mut self, sigma_share: f64) -> Self {
        self.sigma_share = sigma_share;
        self
    }

    /// Sets the alpha parameter.
    ///
    /// # Arguments
    ///
    /// * `alpha` - Shape parameter for the sharing function. Must be > 0.
    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_niching_configuration_default() {
        let config = NichingConfiguration::default();
        assert!(!config.enabled);
        assert!((config.sigma_share - 1.0).abs() < f64::EPSILON);
        assert!((config.alpha - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_niching_configuration_builder() {
        let config = NichingConfiguration::new()
            .with_enabled(true)
            .with_sigma_share(2.5)
            .with_alpha(0.5);

        assert!(config.enabled);
        assert!((config.sigma_share - 2.5).abs() < f64::EPSILON);
        assert!((config.alpha - 0.5).abs() < f64::EPSILON);
    }
}
