//! Configuration for the MOEA/D decomposition-based multi-objective genetic algorithm.

pub use crate::nsga2::configuration::ObjectiveDirection;

/// Scalarization function applied per sub-problem inside the MOEA/D
/// neighbourhood-replacement loop.
///
/// - `Tchebycheff` — classic MOEA/D scalarization (Zhang & Li 2007 eq. 4):
///   `g = max_i { w_i * |f_i - z*_i| }`.
/// - `Pbi { theta }` — penalty-based boundary intersection (eq. 5):
///   `g = d1 + theta * d2`. Common values for `theta` are 1.0–10.0; Zhang & Li
///   use 5.0.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarizationFn {
    /// Classic Tchebycheff: `g = max_i { w_i * |f_i - z*_i| }`.
    #[default]
    Tchebycheff,
    /// Penalty-based boundary intersection: `g = d1 + theta * d2`.
    Pbi { theta: f64 },
}

/// Configuration for the MOEA/D decomposition-based multi-objective genetic algorithm.
///
/// MOEA/D decomposes a multi-objective problem into N scalar sub-problems
/// using weight vectors; each sub-problem maintains a neighbourhood of
/// similar weight vectors, and offspring compete only within that
/// neighbourhood. Weight vectors are either auto-generated via the
/// Das-Dennis simplex lattice (set with
/// [`with_weight_vectors_auto`](Self::with_weight_vectors_auto)) or
/// user-supplied (set with [`with_weight_vectors`](Self::with_weight_vectors)).
///
/// Auto and custom are mutually exclusive — the last builder call wins.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::moead::configuration::{MoeaDConfiguration, ScalarizationFn};
///
/// let config = MoeaDConfiguration::new()
///     .with_num_objectives(3)
///     .with_population_size(91)
///     .with_max_generations(300)
///     .with_weight_vectors_auto(12)
///     .with_scalarization(ScalarizationFn::Tchebycheff)
///     .with_neighborhood_size(20)
///     .with_max_neighbor_replacements(2);
///
/// assert_eq!(config.num_objectives, 3);
/// assert!(config.effective_weight_vectors().is_some());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoeaDConfiguration {
    /// Number of objective functions.
    pub num_objectives: usize,
    /// Population size (typically equals weight-vector count N).
    pub population_size: usize,
    /// Maximum number of generations.
    pub max_generations: usize,
    /// Per-objective optimization direction. If empty, all objectives default to `Minimize`.
    /// When set, the length must match `num_objectives`.
    pub objective_directions: Vec<ObjectiveDirection>,
    /// Scalarization function (D-02). Default: [`ScalarizationFn::Tchebycheff`].
    pub scalarization: ScalarizationFn,
    /// Neighbourhood size T (D-08). Default: 20.
    pub neighborhood_size: usize,
    /// Maximum number of neighbour replacements per offspring (D-09). Default: 2.
    pub max_neighbor_replacements: usize,
    /// Subdivision count for the Das-Dennis simplex lattice (auto weight-vector mode).
    weight_vectors_auto_p: Option<usize>,
    /// User-supplied weight vectors (custom mode).
    weight_vectors_custom: Option<Vec<Vec<f64>>>,
}

impl Default for MoeaDConfiguration {
    fn default() -> Self {
        MoeaDConfiguration {
            num_objectives: 3,
            population_size: 100,
            max_generations: 200,
            objective_directions: Vec::new(),
            scalarization: ScalarizationFn::Tchebycheff,
            neighborhood_size: 20,
            max_neighbor_replacements: 2,
            weight_vectors_auto_p: None,
            weight_vectors_custom: None,
        }
    }
}

impl MoeaDConfiguration {
    /// Creates a new `MoeaDConfiguration` with default values.
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

    /// Sets the scalarization function (D-02). Default: [`ScalarizationFn::Tchebycheff`].
    pub fn with_scalarization(mut self, s: ScalarizationFn) -> Self {
        self.scalarization = s;
        self
    }

    /// Sets the neighbourhood size T (D-08). Default: 20.
    pub fn with_neighborhood_size(mut self, t: usize) -> Self {
        self.neighborhood_size = t;
        self
    }

    /// Sets the maximum number of neighbour replacements per offspring (D-09).
    /// Default: 2.
    pub fn with_max_neighbor_replacements(mut self, nr: usize) -> Self {
        self.max_neighbor_replacements = nr;
        self
    }

    /// Configures auto-generated weight vectors via the Das-Dennis simplex
    /// lattice with subdivision count `p`. Produces `C(p + M - 1, M - 1)` vectors.
    ///
    /// Calling this clears any previously-set custom weight vectors (D-07).
    pub fn with_weight_vectors_auto(mut self, p: usize) -> Self {
        self.weight_vectors_auto_p = Some(p);
        self.weight_vectors_custom = None;
        self
    }

    /// Configures user-supplied weight vectors. Each inner `Vec<f64>` must
    /// have length equal to `num_objectives` (validated by the engine).
    ///
    /// Calling this clears any previously-set auto subdivision count (D-07).
    pub fn with_weight_vectors(mut self, vecs: Vec<Vec<f64>>) -> Self {
        self.weight_vectors_custom = Some(vecs);
        self.weight_vectors_auto_p = None;
        self
    }

    /// Returns the materialised weight vectors, or `None` if neither builder was called.
    ///
    /// When `with_weight_vectors_auto(p)` is in effect, this calls the
    /// Das-Dennis generator each time it is invoked. When custom vectors are
    /// in effect, this clones the stored `Vec`.
    pub fn effective_weight_vectors(&self) -> Option<Vec<Vec<f64>>> {
        if let Some(p) = self.weight_vectors_auto_p {
            Some(crate::nsga3::das_dennis::generate_das_dennis(
                self.num_objectives,
                p,
            ))
        } else {
            self.weight_vectors_custom.clone()
        }
    }

    /// Returns the Das-Dennis subdivision count `p` if auto weight vectors are configured.
    pub(crate) fn weight_vectors_auto_p(&self) -> Option<usize> {
        self.weight_vectors_auto_p
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
