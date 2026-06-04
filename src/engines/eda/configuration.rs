//! Configuration for the EDA (UMDA) engine. See [`EdaConfiguration`] for the entry point.

use crate::configuration::ProblemSolving;

/// Configuration for an [`EdaEngine`](super::engine::EdaEngine) run.
///
/// The EDA engine implements the Univariate Marginal Distribution Algorithm (UMDA):
/// a probabilistic model-building GA that learns a univariate probability model from
/// the best individuals each generation and samples new offspring from that model.
///
/// # Model selection
///
/// The probabilistic model is selected automatically based on the gene type:
/// - `U::Gene: RealGene` → Gaussian univariate model (mean + std per position)
/// - Otherwise → Bernoulli model (probability per position; classic UMDA for binary genes)
#[derive(Debug, Clone)]
pub struct EdaConfiguration {
    /// Number of individuals in the population.
    ///
    /// If 0, a default of 100 is used when `run()` is called.
    /// EDA typically requires larger populations than standard GA to accurately
    /// estimate the probability model.
    pub population_size: usize,

    /// Maximum number of generations before stopping.
    ///
    /// The engine stops after at most `max_generations` iterations even if
    /// no `fitness_target` has been reached.
    pub max_generations: usize,

    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,

    /// Optional fitness target — engine stops early when reached.
    ///
    /// `None` means the engine runs until `max_generations` is exhausted.
    /// When `Some(t)`, the engine stops as soon as the best individual's
    /// fitness satisfies the stopping condition for the current
    /// `problem_solving` direction.
    pub fitness_target: Option<f64>,

    /// Fraction of the population used as selected parents for model estimation.
    ///
    /// At each generation the top `floor(population_size * selection_ratio)` individuals
    /// (by fitness) feed the model estimation step. Must be in `(0.0, 1.0]`.
    /// A minimum of 1 parent is enforced regardless of this value.
    ///
    /// Default: `0.5` (truncation selection keeping the best half).
    pub selection_ratio: f64,
}

impl Default for EdaConfiguration {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 500,
            problem_solving: ProblemSolving::Maximization,
            fitness_target: None,
            selection_ratio: 0.5,
        }
    }
}

impl EdaConfiguration {
    /// Builder: set population size.
    ///
    /// Use 0 to keep the default of 100.
    pub fn with_population_size(mut self, n: usize) -> Self {
        self.population_size = n;
        self
    }

    /// Builder: set maximum number of generations.
    pub fn with_max_generations(mut self, n: usize) -> Self {
        self.max_generations = n;
        self
    }

    /// Builder: set problem solving direction (minimization / maximization).
    pub fn with_problem_solving(mut self, ps: ProblemSolving) -> Self {
        self.problem_solving = ps;
        self
    }

    /// Builder: set fitness target for early stopping.
    pub fn with_fitness_target(mut self, t: f64) -> Self {
        self.fitness_target = Some(t);
        self
    }

    /// Builder: set parent selection ratio for model estimation.
    ///
    /// Must be in `(0.0, 1.0]`. Values outside this range are clamped.
    pub fn with_selection_ratio(mut self, ratio: f64) -> Self {
        self.selection_ratio = ratio.clamp(f64::MIN_POSITIVE, 1.0);
        self
    }
}
