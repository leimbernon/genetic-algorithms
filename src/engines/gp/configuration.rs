//! GP engine configuration.
//!
//! [`GpConfiguration`] holds all parameters for the `GpGa` engine:
//! population size, generation limit, tree depth limits, node count limits,
//! crossover and mutation operators, selection and survivor strategies, and
//! optimization direction.

use crate::configuration::{LimitConfiguration, SelectionConfiguration};
use crate::error::GaError;
use crate::operations::{Selection, Survivor};
use super::crossover::GpCrossover;
use super::mutation::GpMutation;

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
/// | `crossover` | `GpCrossover::SubtreeCrossover` |
/// | `mutations` | `[(SubtreeMutation { mutation_max_depth: 4 }, 0.1)]` |
/// | `selection` | `SelectionConfiguration::default()` (Tournament) |
/// | `survivor` | `Survivor::Fitness` (minimization) |
/// | `is_maximization` | `false` (minimization) |
/// | `max_stagnation` | `None` (disabled) |
/// | `fitness_target` | `None` (disabled) |
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
    /// Crossover operator used by the engine.
    pub(crate) crossover: GpCrossover,
    /// Mutation operators with per-application probabilities.
    ///
    /// Each element is `(operator, probability)`. On each offspring the engine
    /// rolls `rng.random::<f64>() < probability` for each operator in order.
    pub(crate) mutations: Vec<(GpMutation, f64)>,
    /// Parent selection configuration.
    pub(crate) selection: SelectionConfiguration,
    /// Survivor selection strategy.
    pub(crate) survivor: Survivor,
    /// When `true`, higher fitness is better (maximization).
    /// When `false` (default), lower fitness is better (minimization).
    pub(crate) is_maximization: bool,
    /// Stop after this many generations without improvement. `None` disables.
    pub(crate) max_stagnation: Option<usize>,
    /// Stop when the best fitness reaches this target. `None` disables.
    pub(crate) fitness_target: Option<f64>,
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
            crossover: GpCrossover::SubtreeCrossover,
            mutations: vec![(
                GpMutation::SubtreeMutation {
                    mutation_max_depth: 4,
                },
                0.1,
            )],
            selection: SelectionConfiguration {
                number_of_couples: 0, // 0 → engine uses population_size / 2
                method: Selection::Tournament,
                boltzmann_temperature: 1.0,
                niche_radius: 0.1,
            },
            survivor: Survivor::Fitness,
            is_maximization: false,
            max_stagnation: None,
            fitness_target: None,
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

    /// Returns `true` if this is a maximization problem.
    pub fn is_maximization(&self) -> bool {
        self.is_maximization
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

    /// Sets the crossover operator.
    pub fn with_crossover(mut self, crossover: GpCrossover) -> Self {
        self.crossover = crossover;
        self
    }

    /// Sets the mutation operators with per-application probabilities.
    ///
    /// Each element is `(operator, probability)`. The engine applies each
    /// mutation independently per offspring with the given probability.
    pub fn with_mutations(mut self, mutations: Vec<(GpMutation, f64)>) -> Self {
        self.mutations = mutations;
        self
    }

    /// Sets the parent selection configuration.
    pub fn with_selection_config(mut self, selection: SelectionConfiguration) -> Self {
        self.selection = selection;
        self
    }

    /// Sets the survivor selection strategy.
    pub fn with_survivor_config(mut self, survivor: Survivor) -> Self {
        self.survivor = survivor;
        self
    }

    /// Sets the optimization direction.
    ///
    /// `true` → maximization (higher fitness is better).
    /// `false` → minimization (lower fitness is better, the default).
    pub fn with_is_maximization(mut self, is_maximization: bool) -> Self {
        self.is_maximization = is_maximization;
        self
    }

    /// Stops the run after `n` consecutive generations without improvement.
    ///
    /// Set to `None` to disable stagnation stopping (the default).
    pub fn with_max_stagnation(mut self, max_stagnation: Option<usize>) -> Self {
        self.max_stagnation = max_stagnation;
        self
    }

    /// Stops the run when the best fitness reaches `target`.
    ///
    /// For minimization the run stops when `best_fitness <= target`.
    /// For maximization the run stops when `best_fitness >= target`.
    /// Set to `None` to disable (the default).
    pub fn with_fitness_target(mut self, target: Option<f64>) -> Self {
        self.fitness_target = target;
        self
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Returns the `LimitConfiguration` required by `survivor::factory`.
    pub(crate) fn limit_configuration(&self) -> LimitConfiguration {
        use crate::configuration::ProblemSolving;
        LimitConfiguration {
            problem_solving: if self.is_maximization {
                ProblemSolving::Maximization
            } else {
                ProblemSolving::Minimization
            },
            max_generations: self.max_generations,
            fitness_target: self.fitness_target,
            population_size: self.population_size,
            genes_per_chromosome: 1, // placeholder — GP has no linear genes
            needs_unique_ids: false,
            alleles_can_be_repeated: true,
        }
    }

    /// Returns the `SelectionConfiguration` with `number_of_couples` filled in.
    ///
    /// If `self.selection.number_of_couples` is 0 the engine defaults to
    /// `population_size / 2`.
    pub(crate) fn effective_selection_config(&self) -> SelectionConfiguration {
        let mut cfg = self.selection;
        if cfg.number_of_couples == 0 {
            cfg.number_of_couples = (self.population_size / 2).max(1);
        }
        cfg
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
    /// - `max_generations > 0`
    /// - `mutations` vec is not empty
    /// - each mutation probability is finite and in `[0.0, 1.0]`
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
        if self.max_generations == 0 {
            return Err(GaError::ConfigurationError(
                "max_generations must be greater than 0".to_string(),
            ));
        }
        if self.mutations.is_empty() {
            return Err(GaError::ConfigurationError(
                "mutations list must not be empty — provide at least one (GpMutation, probability) pair".to_string(),
            ));
        }
        for (i, (_, prob)) in self.mutations.iter().enumerate() {
            if !prob.is_finite() || *prob < 0.0 || *prob > 1.0 {
                return Err(GaError::ConfigurationError(format!(
                    "mutations[{}]: probability {} is not in [0.0, 1.0]",
                    i, prob
                )));
            }
        }
        Ok(())
    }
}
