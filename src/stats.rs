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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_from_empty() {
        let stats = GenerationStats::from_fitness_values(0, &[], false);
        assert_eq!(stats.population_size, 0);
        assert_eq!(stats.avg_fitness, 0.0);
    }

    #[test]
    fn test_stats_maximization() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = GenerationStats::from_fitness_values(1, &values, true);
        assert_eq!(stats.generation, 1);
        assert_eq!(stats.best_fitness, 5.0);
        assert_eq!(stats.worst_fitness, 1.0);
        assert!((stats.avg_fitness - 3.0).abs() < 1e-10);
        assert_eq!(stats.population_size, 5);
        assert!(stats.fitness_std_dev > 0.0);
    }

    #[test]
    fn test_stats_minimization() {
        let values = vec![10.0, 20.0, 30.0];
        let stats = GenerationStats::from_fitness_values(5, &values, false);
        assert_eq!(stats.best_fitness, 10.0);
        assert_eq!(stats.worst_fitness, 30.0);
        assert!((stats.avg_fitness - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_stats_single_value() {
        let values = vec![42.0];
        let stats = GenerationStats::from_fitness_values(0, &values, true);
        assert_eq!(stats.best_fitness, 42.0);
        assert_eq!(stats.worst_fitness, 42.0);
        assert_eq!(stats.avg_fitness, 42.0);
        assert_eq!(stats.fitness_std_dev, 0.0);
        assert_eq!(stats.population_size, 1);
    }
}
