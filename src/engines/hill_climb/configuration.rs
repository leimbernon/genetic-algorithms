//! Configuration types for the Hill-climbing engine.

use crate::configuration::ProblemSolving;

/// Search mode used by [`HillClimbEngine`](super::engine::HillClimbEngine).
#[derive(Debug, Clone, PartialEq)]
pub enum HillClimbMode {
    /// Accept the first neighbor with higher fitness; early exit from neighbor scan.
    Stochastic,
    /// Evaluate all neighbors; accept only the global best among them.
    SteepestAscent,
}

/// Configuration for a [`HillClimbEngine`](super::engine::HillClimbEngine) run.
#[derive(Clone)]
pub struct HillClimbConfiguration {
    /// Hill-climbing search mode.
    pub mode: HillClimbMode,
    /// Number of consecutive iterations without improvement before stopping.
    pub no_improvement_limit: usize,
    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,
    /// Optional fitness target — engine stops early when reached.
    pub fitness_target: Option<f64>,
}

impl Default for HillClimbConfiguration {
    fn default() -> Self {
        Self {
            mode: HillClimbMode::Stochastic,
            no_improvement_limit: 100,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
        }
    }
}

impl HillClimbConfiguration {
    /// Builder: set the search mode.
    pub fn with_mode(mut self, mode: HillClimbMode) -> Self {
        self.mode = mode;
        self
    }

    /// Builder: set the no-improvement iteration limit.
    pub fn with_no_improvement_limit(mut self, limit: usize) -> Self {
        self.no_improvement_limit = limit;
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
