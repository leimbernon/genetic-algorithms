//! Configuration types for the Permutate engine.

use crate::configuration::ProblemSolving;

/// Configuration for `PermutateEngine`. Candidates must have fitness pre-evaluated before being
/// passed to the engine — `chromosome.fitness()` is called directly, not a fitness function.
#[derive(Debug, Clone)]
pub struct PermutateConfiguration {
    /// Maximum number of candidates to evaluate before stopping.
    pub safety_gate: usize,
    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,
    /// Optional fitness target — engine stops early when reached.
    pub fitness_target: Option<f64>,
}

impl Default for PermutateConfiguration {
    fn default() -> Self {
        Self {
            safety_gate: 100_000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
        }
    }
}

impl PermutateConfiguration {
    /// Builder: set the safety gate (maximum candidates evaluated).
    pub fn with_safety_gate(mut self, gate: usize) -> Self {
        self.safety_gate = gate;
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
