//! Local search refinement operators for memetic algorithms.
//!
//! This module provides the [`LocalSearch`] enum with operator implementations,
//! [`HillClimbingConfig`] for configuration, and supporting types for
//! application strategy and mode selection.

use crate::error::GaError;
use crate::traits::{LinearChromosome, LocalSearchOperator};
use rand::Rng;
use std::borrow::Cow;

/// Local search refinement strategies.
///
/// Determines which local search operator is applied to individuals during
/// the memetic algorithm loop. The enum implements
/// [`LocalSearchOperator`], so it can be passed directly to the GA
/// configuration.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::local_search::LocalSearch;
///
/// let op = LocalSearch::HillClimbing;
/// assert_eq!(op, LocalSearch::HillClimbing);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocalSearch {
    /// Random hill-climbing: perturb one random gene per iteration, accept
    /// on improvement. Requires `RangeChromosome<f64>`.
    HillClimbing,
}

impl LocalSearchOperator for LocalSearch {
    fn improve<U>(
        &self,
        individual: &mut U,
        fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
    ) -> Result<usize, GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        match self {
            LocalSearch::HillClimbing => {
                HillClimbingConfig::default().improve(individual, fitness_fn)
            }
        }
    }
}

/// Configuration for the HillClimbing local search operator.
///
/// # Defaults
///
/// | Parameter       | Default |
/// |-----------------|---------|
/// | `step_size`     | `0.1`   |
/// | `max_iterations`| `20`    |
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::local_search::HillClimbingConfig;
///
/// let config = HillClimbingConfig { step_size: 0.05, max_iterations: 50 };
/// assert_eq!(config.step_size, 0.05);
/// assert_eq!(config.max_iterations, 50);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HillClimbingConfig {
    /// Magnitude of random perturbation applied to a gene each iteration.
    pub step_size: f64,
    /// Maximum number of perturbation-accept cycles.
    pub max_iterations: usize,
}

impl Default for HillClimbingConfig {
    fn default() -> Self {
        Self {
            step_size: 0.1,
            max_iterations: 20,
        }
    }
}

/// Strategy for selecting which individuals receive local search refinement.
///
/// Controls the application frequency of the local search operator within
/// the memetic algorithm loop.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::local_search::LocalSearchApplicationStrategy;
///
/// let strategy = LocalSearchApplicationStrategy::BestN { n: 5 };
/// assert_eq!(strategy, LocalSearchApplicationStrategy::BestN { n: 5 });
/// let default = LocalSearchApplicationStrategy::default();
/// assert_eq!(default, LocalSearchApplicationStrategy::AllOffspring);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocalSearchApplicationStrategy {
    /// Apply local search to every offspring before survivor selection.
    #[default]
    AllOffspring,
    /// Apply local search only to the `n` best individuals.
    BestN {
        /// Number of top individuals to refine.
        n: usize,
    },
    /// Apply local search to each individual with the given probability.
    Probabilistic {
        /// Probability (0.0–1.0) that an individual receives refinement.
        probability: f64,
    },
    /// Apply local search every N generations to the whole population.
    EveryNGenerations {
        /// Generation interval between refinement rounds.
        interval: usize,
    },
}

/// Strategy for how local search improvements affect the genome.
///
/// - **Lamarckian**: improved genotypes replace the original genes (genetic
///   information acquired during the individual's lifetime is inherited).
/// - **Baldwinian**: only the fitness is updated; the original genotype is
///   retained (acquired improvements affect selection but are not inherited).
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::local_search::LocalSearchMode;
///
/// let mode = LocalSearchMode::Baldwinian;
/// assert_eq!(mode, LocalSearchMode::Baldwinian);
/// let default = LocalSearchMode::default();
/// assert_eq!(default, LocalSearchMode::Lamarckian);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocalSearchMode {
    /// Replace the chromosome's genes with the locally improved version.
    /// Genetic changes are inherited by offspring.
    #[default]
    Lamarckian,
    /// Keep the original chromosome genes but update the fitness value.
    /// Learning does not affect the genetic code.
    Baldwinian,
}

/// Create a [`LocalSearch`] operator instance (standard dispatch).
///
/// Returns the [`LocalSearch`] enum, which itself implements
/// [`LocalSearchOperator`] so it can be used wherever a boxed or generic
/// operator is expected.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::local_search::{factory, LocalSearch};
///
/// let op = factory(LocalSearch::HillClimbing);
/// assert_eq!(op, LocalSearch::HillClimbing);
/// ```
pub fn factory(op: LocalSearch) -> LocalSearch {
    op
}

/// Create a [`LocalSearch`] operator instance with a custom configuration.
///
/// Returns a [`HillClimbingConfig`] (which implements
/// [`LocalSearchOperator`]) when the variant is `HillClimbing`.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::local_search::{factory_with_config, HillClimbingConfig, LocalSearch};
///
/// let config = HillClimbingConfig { step_size: 0.02, max_iterations: 100 };
/// let result = factory_with_config(LocalSearch::HillClimbing, config);
/// assert_eq!(result.step_size, 0.02);
/// assert_eq!(result.max_iterations, 100);
/// ```
pub fn factory_with_config(op: LocalSearch, config: HillClimbingConfig) -> HillClimbingConfig {
    match op {
        LocalSearch::HillClimbing => config,
    }
}

/// Apply random hill-climbing refinement to a single individual.
///
/// Each iteration:
/// 1. Pick a random gene dimension `j`.
/// 2. Perturb `gene[j].value` by `delta ~ Uniform(-step, step)`.
/// 3. Evaluate the new fitness. Accept the perturbation if it improves;
///    revert otherwise.
///
/// # Type Requirements
///
/// Requires `U` to be `RangeChromosome<f64>` (checked at runtime via
/// `Any` downcast). Returns [`GaError::LocalSearchError`] for unsupported
/// types.
impl LocalSearchOperator for HillClimbingConfig {
    fn improve<U>(
        &self,
        individual: &mut U,
        fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
    ) -> Result<usize, GaError>
    where
        U: LinearChromosome + Send + Sync + 'static + Clone,
    {
        // 1. Runtime type check — require RangeChromosome<f64>
        let any_ref: &dyn std::any::Any = individual;
        if !any_ref.is::<crate::chromosomes::Range<f64>>() {
            return Err(GaError::LocalSearchError(
                "HillClimbing local search requires RangeChromosome<f64>".to_string(),
            ));
        }

        let step = self.step_size;
        let mut rng = crate::rng::make_rng();
        let mut improvements = 0usize;

        for _ in 0..self.max_iterations {
            let dim = individual.dna().len();
            if dim == 0 {
                break;
            }
            let j = rng.random_range(0..dim);
            let delta: f64 = (rng.random::<f64>() * 2.0 - 1.0) * step;

            let original_dna = individual.dna().to_vec();

            // 2. Clone DNA vec, modify gene[j] via pointer cast
            let mut new_dna = individual.dna().to_vec();
            // SAFETY: Type verified above as RangeChromosome<f64>, so each
            // gene is RangeGene<f64> with value, ranges fields. Bounds
            // are enforced by the `dim` check and the `j` range.
            unsafe {
                use crate::genotypes::Range as RangeGene;
                let ptr = new_dna.as_mut_ptr() as *mut RangeGene<f64>;
                let gene = &mut *ptr.add(j);
                let new_val = gene.value + delta;
                if !gene.ranges.is_empty() {
                    gene.value = new_val.clamp(gene.ranges[0].0, gene.ranges[0].1);
                } else {
                    gene.value = new_val;
                }
            }
            individual.set_dna(Cow::Owned(new_dna));

            // 3. Evaluate and accept / revert
            let current_fitness = individual.fitness();
            let new_fitness = fitness_fn(individual.dna());
            if new_fitness < current_fitness {
                individual.set_fitness(new_fitness);
                improvements += 1;
            } else {
                individual.set_dna(Cow::Owned(original_dna));
            }
        }

        Ok(improvements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Range as RangeChromosome;
    use crate::genotypes::Range as RangeGene;
    use crate::traits::{ChromosomeT, LinearChromosome};

    /// Simple quadratic fitness: sum of squares.
    fn quadratic(dna: &[RangeGene<f64>]) -> f64 {
        dna.iter().map(|g| g.value * g.value).sum::<f64>()
    }

    #[test]
    fn test_hill_climbing_returns_improvements_count() {
        let genes: Vec<RangeGene<f64>> = (0..5)
            .map(|i| RangeGene::new(i, vec![(-10.0, 10.0)], 8.0))
            .collect();
        let mut chromo = RangeChromosome::<f64>::new();
        chromo.set_dna(Cow::Owned(genes));
        chromo.set_fitness(quadratic(chromo.dna()));

        let config = HillClimbingConfig {
            step_size: 1.0,
            max_iterations: 50,
        };
        let result = config.improve(&mut chromo, &quadratic);
        assert!(result.is_ok());
        let improvements = result.unwrap();
        assert!(improvements > 0, "expected at least one improvement");
        // Fitness should be lower (minimization) after hill climbing
        assert!(
            chromo.fitness() < 400.0,
            "fitness should improve: {}",
            chromo.fitness()
        );
    }

    #[test]
    fn test_hill_climbing_unsupported_type() {
        use crate::chromosomes::Binary as BinaryChromosome;
        use crate::genotypes::Binary as BinaryGene;

        let mut chromo = BinaryChromosome::new();
        let result = HillClimbingConfig::default().improve(&mut chromo, &|_: &[BinaryGene]| 0.0);
        assert!(result.is_err());
        match result {
            Err(GaError::LocalSearchError(msg)) => {
                assert!(msg.contains("HillClimbing"));
            }
            _ => panic!("expected LocalSearchError"),
        }
    }

    #[test]
    fn test_hill_climbing_empty_dna() {
        let mut chromo = RangeChromosome::<f64>::new();
        let result = HillClimbingConfig::default().improve(&mut chromo, &quadratic);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_local_search_application_strategy_default() {
        let strategy = LocalSearchApplicationStrategy::default();
        assert!(matches!(
            strategy,
            LocalSearchApplicationStrategy::AllOffspring
        ));
    }

    #[test]
    fn test_local_search_mode_default() {
        let mode = LocalSearchMode::default();
        assert!(matches!(mode, LocalSearchMode::Lamarckian));
    }

    #[test]
    fn test_factory_returns_enum() {
        let op = factory(LocalSearch::HillClimbing);
        assert_eq!(op, LocalSearch::HillClimbing);
    }

    #[test]
    fn test_factory_with_config() {
        let config = HillClimbingConfig {
            step_size: 0.01,
            max_iterations: 10,
        };
        let result = factory_with_config(LocalSearch::HillClimbing, config);
        // factory_with_config returns HillClimbingConfig which implements LocalSearchOperator
        let mut chromo = RangeChromosome::<f64>::new();
        let improve_result = result.improve(&mut chromo, &|_: &[RangeGene<f64>]| 0.0);
        assert!(improve_result.is_ok());
    }
}
