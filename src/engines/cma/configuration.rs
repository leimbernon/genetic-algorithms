//! Configuration for the CMA-ES engine.

use super::restart::RestartStrategy;
use crate::configuration::ProblemSolving;

/// Configuration for a [`CmaEngine`](super::engine::CmaEngine) run.
#[derive(Debug, Clone)]
pub struct CmaConfiguration {
    /// Initial step size σ₀ (default 0.3).
    ///
    /// Controls the initial spread of the search. Typical values are in the
    /// range of 1/5 to 1/3 of the search range, but any positive value is
    /// valid. Set to a smaller value if the optimum is expected to be close
    /// to the initial mean.
    pub sigma0: f64,

    /// Population size λ.
    ///
    /// If 0, auto-computed as `4 + floor(3·ln(n))` when `run()` is called,
    /// where `n` is the problem dimension. This is Hansen's recommended
    /// default. Set explicitly for larger populations (better exploration)
    /// or smaller populations (faster convergence per generation).
    pub population_size: usize,

    /// Maximum number of generations before stopping.
    ///
    /// The engine stops after at most `max_generations` iterations even if
    /// no `fitness_target` has been reached.
    pub max_generations: usize,

    /// Whether to minimise or maximise fitness.
    ///
    /// Use `ProblemSolving::Minimization` for standard optimization problems
    /// (sphere, Rosenbrock, etc.) and `ProblemSolving::Maximization` for
    /// problems where higher fitness is better.
    pub problem_solving: ProblemSolving,

    /// Optional fitness target — engine stops early when reached.
    ///
    /// `None` means the engine runs until `max_generations` is exhausted.
    /// When `Some(t)`, the engine stops as soon as the best individual's
    /// fitness satisfies the stopping condition for the current
    /// `problem_solving` direction.
    pub fitness_target: Option<f64>,

    /// Covariance matrix cumulation `cc`. `None` = Hansen's auto formula.
    ///
    /// Controls how quickly the covariance matrix accumulates curvature
    /// information. Override only when you have domain-specific knowledge;
    /// the default auto-formula is typically near-optimal.
    pub cc: Option<f64>,

    /// Step-size control cumulation `cs`. `None` = Hansen's auto formula.
    ///
    /// Controls the time horizon for step-size adaptation. Override only
    /// when you have domain-specific knowledge.
    pub cs: Option<f64>,

    /// Rank-one update rate `c1`. `None` = Hansen's auto formula.
    ///
    /// Learning rate for the rank-one covariance matrix update. Must be
    /// in `(0, 1]` if set explicitly.
    pub c1: Option<f64>,

    /// Rank-mu update rate `cmu`. `None` = Hansen's auto formula.
    ///
    /// Learning rate for the rank-μ covariance matrix update. Must be
    /// in `(0, 1]` if set explicitly, and `c1 + cmu <= 1`.
    pub cmu: Option<f64>,

    /// Optional restart strategy (IPOP or BIPOP).
    ///
    /// `None` means no automatic restarts — the engine runs for `max_generations`
    /// and returns the best result found. When `Some(strategy)`, the engine monitors
    /// stagnation and triggers restarts according to the chosen strategy.
    pub restart_strategy: Option<RestartStrategy>,

    /// Fitness cache capacity in entries (D-05).
    ///
    /// When set, `run()` wraps the scalar `fitness_fn` with an LRU cache of this
    /// size. Has no effect when `batch_evaluator` is not also configured — in batch
    /// mode the cache is created directly inside `run()` for the D-06 partition.
    pub fitness_cache_size: Option<usize>,
}

impl Default for CmaConfiguration {
    fn default() -> Self {
        Self {
            sigma0: 0.3,
            population_size: 0,
            max_generations: 1000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
            cc: None,
            cs: None,
            c1: None,
            cmu: None,
            restart_strategy: None,
            fitness_cache_size: None,
        }
    }
}

impl CmaConfiguration {
    /// Auto-sized configuration for a problem of dimension `n`.
    ///
    /// Sets `population_size = 4 + floor(3·ln(n))` using Hansen's recommended
    /// default formula. All other fields take their `Default` values.
    ///
    /// For `n = 0`, `population_size` is clamped to 4 (sane minimum) instead
    /// of panicking on `ln(0)`.
    pub fn default_for_dim(n: usize) -> Self {
        let lambda = if n == 0 {
            4
        } else {
            4 + (3.0 * (n as f64).ln()).floor() as usize
        };
        Self {
            population_size: lambda,
            ..Self::default()
        }
    }

    /// Builder: set initial step size σ₀.
    ///
    /// Positive values only; typically 1/5 to 1/3 of the expected search range.
    pub fn with_sigma0(mut self, s: f64) -> Self {
        self.sigma0 = s;
        self
    }

    /// Builder: set population size λ.
    ///
    /// Use 0 to keep the auto-computed default (`4 + floor(3·ln(n))`).
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
    ///
    /// The engine stops as soon as the best fitness satisfies the target
    /// condition for the current `problem_solving` direction.
    pub fn with_fitness_target(mut self, t: f64) -> Self {
        self.fitness_target = Some(t);
        self
    }

    /// Builder: override the covariance matrix cumulation `cc`.
    ///
    /// Must be in `(0, 1]`. `None` is used if not set (auto-formula).
    pub fn with_cc(mut self, v: f64) -> Self {
        self.cc = Some(v);
        self
    }

    /// Builder: override the step-size control cumulation `cs`.
    ///
    /// Must be in `(0, 1]`. `None` is used if not set (auto-formula).
    pub fn with_cs(mut self, v: f64) -> Self {
        self.cs = Some(v);
        self
    }

    /// Builder: override the rank-one update rate `c1`.
    ///
    /// Must be in `(0, 1]`. `None` is used if not set (auto-formula).
    pub fn with_c1(mut self, v: f64) -> Self {
        self.c1 = Some(v);
        self
    }

    /// Builder: override the rank-μ update rate `cmu`.
    ///
    /// Must be in `(0, 1]`, and `c1 + cmu <= 1`. `None` is used if not set
    /// (auto-formula).
    pub fn with_cmu(mut self, v: f64) -> Self {
        self.cmu = Some(v);
        self
    }

    /// Builder: enable the fitness cache (D-05).
    ///
    /// Sets the LRU cache capacity to `size` entries. When the engine runs in scalar
    /// fitness mode, `fitness_fn` is wrapped with the cache. When combined with
    /// `with_batch_evaluator`, the cache is used for the D-06 miss/hit partition.
    pub fn with_fitness_cache(mut self, size: usize) -> Self {
        self.fitness_cache_size = Some(size);
        self
    }

    /// Builder: enable automatic restarts using an IPOP or BIPOP strategy.
    ///
    /// The engine will monitor stagnation (no fitness improvement for
    /// `stagnation_threshold` generations) and restart the CMA-ES run with a
    /// modified population size when stagnation is detected.
    ///
    /// Set to `RestartStrategy::Ipop` for simple population-doubling restarts or
    /// `RestartStrategy::Bipop` for alternating large/small restart phases.
    pub fn with_restart_strategy(mut self, strategy: RestartStrategy) -> Self {
        self.restart_strategy = Some(strategy);
        self
    }
}
