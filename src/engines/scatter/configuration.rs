//! Configuration for the Scatter Search engine.

use crate::configuration::ProblemSolving;

/// Configuration for a [`ScatterEngine`](super::engine::ScatterEngine) run.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::scatter::ScatterConfiguration;
/// use genetic_algorithms::configuration::ProblemSolving;
///
/// let config = ScatterConfiguration::default()
///     .with_population_size(50)
///     .with_reference_set_size(10)
///     .with_max_iterations(100)
///     .with_problem_solving(ProblemSolving::Minimization);
/// ```
#[derive(Debug, Clone)]
pub struct ScatterConfiguration {
    /// Total number of solutions generated during diversification.
    pub population_size: usize,
    /// Size of the reference set `b` (typically 10–20).  Must be < `population_size`.
    pub reference_set_size: usize,
    /// Maximum number of scatter-search iterations.
    pub max_iterations: usize,
    /// Whether to apply local search to each candidate after combination.
    pub local_search: bool,
    /// Maximum steps for the built-in local search (hill-climbing).
    pub local_search_steps: usize,
    /// Step size for the built-in local search perturbation.
    pub local_search_step_size: f64,
    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,
    /// Optional fitness target for early stopping.
    pub fitness_target: Option<f64>,
}

impl Default for ScatterConfiguration {
    fn default() -> Self {
        Self {
            population_size: 50,
            reference_set_size: 10,
            max_iterations: 100,
            local_search: false,
            local_search_steps: 20,
            local_search_step_size: 0.1,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
        }
    }
}

impl ScatterConfiguration {
    /// Builder: set total population (diversification pool) size.
    pub fn with_population_size(mut self, n: usize) -> Self {
        self.population_size = n;
        self
    }
    /// Builder: set reference set size.
    pub fn with_reference_set_size(mut self, b: usize) -> Self {
        self.reference_set_size = b;
        self
    }
    /// Builder: set maximum iterations.
    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }
    /// Builder: enable or disable local search post-processing.
    pub fn with_local_search(mut self, enabled: bool) -> Self {
        self.local_search = enabled;
        self
    }
    /// Builder: set local search step count.
    pub fn with_local_search_steps(mut self, steps: usize) -> Self {
        self.local_search_steps = steps;
        self
    }
    /// Builder: set local search perturbation magnitude.
    pub fn with_local_search_step_size(mut self, s: f64) -> Self {
        self.local_search_step_size = s;
        self
    }
    /// Builder: set problem solving direction.
    pub fn with_problem_solving(mut self, ps: ProblemSolving) -> Self {
        self.problem_solving = ps;
        self
    }
    /// Builder: set fitness target for early stopping.
    pub fn with_fitness_target(mut self, t: f64) -> Self {
        self.fitness_target = Some(t);
        self
    }
}
