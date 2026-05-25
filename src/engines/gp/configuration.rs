//! GP engine configuration shell.
//!
//! [`GpConfiguration`] holds tree-specific parameters for the `GpGa` engine:
//! population size, generation limit, tree depth limits, and node count limits.
//! Crossover, mutation, and selection configuration will be added in Wave 2.

use crate::error::GaError;

/// Configuration for the GP engine.
///
/// All limits are validated at [`GpConfiguration::build()`] time.
/// Build failures return [`GaError::ConfigurationError`].
///
/// # Defaults
///
/// | Parameter | Default |
/// |-----------|---------|
/// | `population_size` | 100 |
/// | `max_generations` | 50 |
/// | `init_max_depth` | 4 |
/// | `max_depth` | 8 |
/// | `max_node_count` | 200 |
#[derive(Debug, Clone)]
pub struct GpConfiguration {
    pub(crate) population_size: usize,
    pub(crate) max_generations: usize,
    /// Maximum depth used during ramped half-and-half initialisation.
    /// Typically shallower than `max_depth`.
    pub(crate) init_max_depth: usize,
    /// Hard depth limit enforced after crossover and mutation.
    pub(crate) max_depth: usize,
    /// Hard node-count limit enforced after crossover and mutation.
    pub(crate) max_node_count: usize,
}

impl Default for GpConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

impl GpConfiguration {
    /// Creates a `GpConfiguration` with sensible defaults.
    pub fn new() -> Self {
        GpConfiguration {
            population_size: 100,
            max_generations: 50,
            init_max_depth: 4,
            max_depth: 8,
            max_node_count: 200,
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Returns the population size.
    pub fn population_size(&self) -> usize {
        self.population_size
    }

    /// Returns the maximum number of generations.
    pub fn max_generations(&self) -> usize {
        self.max_generations
    }

    /// Returns the maximum depth used during initialisation.
    pub fn init_max_depth(&self) -> usize {
        self.init_max_depth
    }

    /// Returns the hard tree depth limit.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the hard node count limit.
    pub fn max_node_count(&self) -> usize {
        self.max_node_count
    }

    // -----------------------------------------------------------------------
    // Builder methods
    // -----------------------------------------------------------------------

    /// Sets the population size.
    pub fn with_population_size(mut self, size: usize) -> Self {
        self.population_size = size;
        self
    }

    /// Sets the maximum number of generations.
    pub fn with_max_generations(mut self, max: usize) -> Self {
        self.max_generations = max;
        self
    }

    /// Sets the maximum depth used during initialisation.
    pub fn with_init_max_depth(mut self, depth: usize) -> Self {
        self.init_max_depth = depth;
        self
    }

    /// Sets the hard tree depth limit.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets the hard node count limit.
    pub fn with_max_node_count(mut self, count: usize) -> Self {
        self.max_node_count = count;
        self
    }

    // -----------------------------------------------------------------------
    // Validation
    // -----------------------------------------------------------------------

    /// Validates this configuration.
    ///
    /// Returns `Ok(())` when all constraints are satisfied, or
    /// `Err(GaError::ConfigurationError)` with a descriptive message on the
    /// first failing constraint.
    ///
    /// # Constraints
    ///
    /// - `max_depth > 0`
    /// - `max_depth <= 1_000` (hard cap to prevent memory exhaustion, T-53-02)
    /// - `max_node_count >= max_depth` (a chain of depth D needs D nodes, T-53-03)
    /// - `max_node_count <= 100_000` (hard cap, T-53-03)
    /// - `init_max_depth > 0`
    /// - `init_max_depth <= max_depth`
    /// - `population_size > 0`
    pub fn build(&self) -> Result<(), GaError> {
        if self.max_depth == 0 {
            return Err(GaError::ConfigurationError(
                "max_depth must be greater than 0".to_string(),
            ));
        }
        if self.max_depth > 1_000 {
            return Err(GaError::ConfigurationError(format!(
                "max_depth {} exceeds the hard cap of 1000; set a smaller limit to prevent memory exhaustion",
                self.max_depth
            )));
        }
        if self.max_node_count < self.max_depth {
            return Err(GaError::ConfigurationError(format!(
                "max_node_count ({}) must be >= max_depth ({}) — a right-spine chain of depth D requires D nodes",
                self.max_node_count, self.max_depth
            )));
        }
        if self.max_node_count > 100_000 {
            return Err(GaError::ConfigurationError(format!(
                "max_node_count {} exceeds the hard cap of 100000; set a smaller limit to prevent memory exhaustion",
                self.max_node_count
            )));
        }
        if self.init_max_depth == 0 {
            return Err(GaError::ConfigurationError(
                "init_max_depth must be greater than 0".to_string(),
            ));
        }
        if self.init_max_depth > self.max_depth {
            return Err(GaError::ConfigurationError(format!(
                "init_max_depth ({}) must be <= max_depth ({})",
                self.init_max_depth, self.max_depth
            )));
        }
        if self.population_size == 0 {
            return Err(GaError::ConfigurationError(
                "population_size must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}
