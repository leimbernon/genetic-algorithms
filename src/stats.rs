//! Stats — per-generation statistics for tracking GA convergence.
//!
//! The [`GenerationStats`] struct captures fitness metrics (best, worst,
//! average, standard deviation) at the end of each generation. These
//! statistics are used internally for convergence-based stopping criteria and
//! are also passed to user callbacks during `run_with_callback`.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`GenerationStats`] | Per-generation snapshot of fitness metrics and diversity |
//! | [`GenerationStats::best_fitness`] | Best fitness value in the generation |
//! | [`GenerationStats::avg_fitness`] | Average fitness value population-wide |
//!
//! # When to use
//! Accessible from observer hooks and callbacks during the GA run. Use
//! [`GenerationStats`] to monitor convergence or to implement custom
//! stopping criteria.

/// Per-generation statistics for tracking GA convergence and behavior.
///
/// Collected at the end of each generation and optionally passed to callbacks.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenerationStats {
    /// Generation number (0-based).
    pub generation: usize,
    /// Best (minimum or maximum depending on problem) fitness in this generation.
    pub best_fitness: f64,
    /// Worst fitness in this generation.
    pub worst_fitness: f64,
    /// Average fitness across the population.
    pub avg_fitness: f64,
    /// Standard deviation of fitness values.
    pub fitness_std_dev: f64,
    /// Population size at the end of this generation.
    pub population_size: usize,
    /// Population diversity: standard deviation of fitness values.
    /// Equal to `fitness_std_dev` in v2.2. Higher values = more diverse.
    #[cfg_attr(feature = "serde", serde(default))]
    pub diversity: f64,
    /// Current dynamic mutation probability, if dynamic mutation is enabled.
    /// `None` when dynamic mutation is disabled.
    #[cfg_attr(feature = "serde", serde(default))]
    pub dynamic_mutation_probability: Option<f64>,
    /// Average node count across the surviving population (GP only).
    ///
    /// Set to `0.0` for non-GP engines. Used by `GpGa` for bloat monitoring
    /// (CHR-05). The `serde(default)` attribute ensures backward-compatible
    /// deserialization of checkpoints created before this field was added.
    #[cfg_attr(feature = "serde", serde(default))]
    pub avg_node_count: f64,
    /// Number of LRU fitness cache hits in this generation.
    ///
    /// `None` when no fitness cache is configured. When set, represents the
    /// delta hits accumulated during this generation's fitness evaluations.
    #[cfg_attr(feature = "serde", serde(default))]
    pub cache_hits: Option<u64>,
    /// Number of LRU fitness cache misses in this generation.
    ///
    /// `None` when no fitness cache is configured. When set, represents the
    /// delta misses accumulated during this generation's fitness evaluations.
    #[cfg_attr(feature = "serde", serde(default))]
    pub cache_misses: Option<u64>,
}

impl GenerationStats {
    /// Computes statistics from a slice of fitness values.
    ///
    /// `is_maximization` controls which value is "best" vs "worst".
    pub fn from_fitness_values(
        generation: usize,
        fitness_values: &[f64],
        is_maximization: bool,
    ) -> Self {
        let n = fitness_values.len();
        if n == 0 {
            return GenerationStats {
                generation,
                best_fitness: 0.0,
                worst_fitness: 0.0,
                avg_fitness: 0.0,
                fitness_std_dev: 0.0,
                population_size: 0,
                diversity: 0.0,
                dynamic_mutation_probability: None,
                avg_node_count: 0.0,
                cache_hits: None,
                cache_misses: None,
            };
        }

        let sum: f64 = fitness_values.iter().sum();
        let avg = sum / n as f64;

        let variance = fitness_values
            .iter()
            .map(|f| (f - avg).powi(2))
            .sum::<f64>()
            / n as f64;
        let std_dev = variance.sqrt();

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &f in fitness_values {
            if f < min {
                min = f;
            }
            if f > max {
                max = f;
            }
        }

        let (best, worst) = if is_maximization {
            (max, min)
        } else {
            (min, max)
        };

        GenerationStats {
            generation,
            best_fitness: best,
            worst_fitness: worst,
            avg_fitness: avg,
            fitness_std_dev: std_dev,
            population_size: n,
            diversity: std_dev,
            dynamic_mutation_probability: None,
            avg_node_count: 0.0,
            cache_hits: None,
            cache_misses: None,
        }
    }
}
