//! Configuration for the NSGA-III multi-objective genetic algorithm.

pub use crate::nsga2::configuration::ObjectiveDirection;

/// Configuration for the NSGA-III many-objective genetic algorithm.
///
/// NSGA-III replaces NSGA-II's crowding distance with reference-point
/// association on the unit hyperplane. Reference points are either
/// auto-generated via the Das-Dennis simplex lattice (set with
/// [`with_reference_points_auto`](Self::with_reference_points_auto)) or
/// user-supplied (set with [`with_reference_points`](Self::with_reference_points)).
///
/// Auto and custom are mutually exclusive — the last builder call wins.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};
///
/// let config = Nsga3Configuration::new()
///     .with_num_objectives(3)
///     .with_population_size(100)
///     .with_max_generations(200)
///     .with_reference_points_auto(12);
///
/// assert_eq!(config.num_objectives, 3);
/// assert!(config.effective_reference_points().is_some());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nsga3Configuration {
    /// Number of objective functions (NSGA-III targets >= 3 in practice).
    pub num_objectives: usize,
    /// Population size.
    pub population_size: usize,
    /// Maximum number of generations.
    pub max_generations: usize,
    /// Per-objective optimization direction. If empty, all objectives default to `Minimize`.
    /// When set, the length must match `num_objectives`.
    pub objective_directions: Vec<ObjectiveDirection>,
    /// Subdivision count for the Das-Dennis simplex lattice (auto reference-point mode).
    /// `Some(p)` means `with_reference_points_auto(p)` was the last reference-point call;
    /// `None` means custom points are in effect (or none configured).
    reference_points_auto_p: Option<usize>,
    /// User-supplied reference points (custom reference-point mode).
    /// `Some(points)` means `with_reference_points(points)` was the last reference-point call.
    reference_points_custom: Option<Vec<Vec<f64>>>,
}

impl Default for Nsga3Configuration {
    fn default() -> Self {
        Nsga3Configuration {
            num_objectives: 3,
            population_size: 100,
            max_generations: 200,
            objective_directions: Vec::new(),
            reference_points_auto_p: None,
            reference_points_custom: None,
        }
    }
}

impl Nsga3Configuration {
    /// Creates a new `Nsga3Configuration` with default values.
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

    /// Configures auto-generated reference points via the Das-Dennis simplex
    /// lattice with subdivision count `p`. Produces `C(p + M - 1, M - 1)` points.
    ///
    /// Calling this clears any previously-set custom reference points.
    /// The last reference-point builder call wins when both are chained.
    pub fn with_reference_points_auto(mut self, p: usize) -> Self {
        self.reference_points_auto_p = Some(p);
        self.reference_points_custom = None;
        self
    }

    /// Configures user-supplied reference points. Each inner `Vec<f64>` must
    /// have length equal to `num_objectives` (validated by the engine).
    ///
    /// Calling this clears any previously-set auto subdivision count.
    /// The last reference-point builder call wins when both are chained.
    pub fn with_reference_points(mut self, points: Vec<Vec<f64>>) -> Self {
        self.reference_points_custom = Some(points);
        self.reference_points_auto_p = None;
        self
    }

    /// Returns the materialised reference points, or `None` if neither builder was called.
    ///
    /// When `with_reference_points_auto(p)` is in effect, this calls the
    /// Das-Dennis generator each time it is invoked. When custom points are
    /// in effect, this clones the stored `Vec`.
    pub fn effective_reference_points(&self) -> Option<Vec<Vec<f64>>> {
        if let Some(p) = self.reference_points_auto_p {
            Some(crate::nsga3::das_dennis::generate_das_dennis(
                self.num_objectives,
                p,
            ))
        } else {
            self.reference_points_custom.clone()
        }
    }

    /// Returns the Das-Dennis subdivision count `p` if auto reference points are configured.
    ///
    /// Used by the engine's `validate()` to reject `p == 0` before materialising points.
    pub(crate) fn reference_points_auto_p(&self) -> Option<usize> {
        self.reference_points_auto_p
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
