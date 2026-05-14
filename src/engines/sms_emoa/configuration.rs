//! Configuration for the SMS-EMOA indicator-based multi-objective genetic algorithm.
//!
//! SMS-EMOA (Beume, Naujoks & Emmerich 2007) is a steady-state (mu+1) MOEA
//! that removes one individual per generation based on hypervolume contribution.
//! The algorithm uses non-dominated sorting and, from the worst front, removes
//! the individual with the smallest contribution to the hypervolume of that front.

pub use crate::nsga2::configuration::ObjectiveDirection;

/// Configuration for the SMS-EMOA steady-state multi-objective genetic algorithm.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::sms_emoa::configuration::{SmsEmoaConfiguration, ObjectiveDirection};
///
/// let config = SmsEmoaConfiguration::new()
///     .with_num_objectives(2)
///     .with_population_size(100)
///     .with_max_generations(250);
///
/// assert_eq!(config.num_objectives, 2);
/// assert_eq!(config.population_size, 100);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmsEmoaConfiguration {
    /// Number of objective functions (SMS-EMOA requires >= 2).
    pub num_objectives: usize,
    /// Population size.
    pub population_size: usize,
    /// Maximum number of generations.
    pub max_generations: usize,
    /// Per-objective optimization direction. If empty, all objectives default to `Minimize`.
    /// When set, the length must match `num_objectives`.
    pub objective_directions: Vec<ObjectiveDirection>,
    /// Reference point for hypervolume calculation, strictly dominating all solutions.
    /// If `None`, the engine auto-computes it from the initial population extremes plus
    /// a margin of 1.0 per objective.
    pub hypervolume_reference_point: Option<Vec<f64>>,
}

impl Default for SmsEmoaConfiguration {
    fn default() -> Self {
        SmsEmoaConfiguration {
            num_objectives: 2,
            population_size: 100,
            max_generations: 250,
            objective_directions: Vec::new(),
            hypervolume_reference_point: None,
        }
    }
}

impl SmsEmoaConfiguration {
    /// Creates a new `SmsEmoaConfiguration` with default values.
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

    /// Sets the hypervolume reference point (strictly dominates all solutions).
    pub fn with_hypervolume_reference_point(mut self, reference_point: Vec<f64>) -> Self {
        self.hypervolume_reference_point = Some(reference_point);
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
