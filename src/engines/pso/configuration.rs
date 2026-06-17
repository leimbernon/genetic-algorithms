//! Configuration types for the PSO engine. See `PsoConfiguration` for the entry point.

use crate::configuration::ProblemSolving;

/// Inertia weight strategy for the PSO velocity update.
///
/// Controls how much of the particle's previous velocity is retained on each generation.
/// A higher inertia promotes global exploration; a lower inertia promotes local exploitation.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::pso::PsoInertia;
///
/// let inertia = PsoInertia::Constant(0.729);
/// let decay = PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 };
/// ```
#[derive(Debug, Clone)]
pub enum PsoInertia {
    /// Fixed inertia weight applied every generation.
    ///
    /// Standard values: 0.4–0.9. A value of 0.729 (Clerc's constriction coefficient)
    /// is a common choice when using `c1 = c2 = 1.49445`.
    Constant(f64),

    /// Linearly decreasing inertia from `w_start` to `w_end` over the run.
    ///
    /// Recommended defaults (Shi & Eberhart 1998): `w_start = 0.9`, `w_end = 0.4`.
    /// Starts with high inertia (broad exploration) and tapers to low inertia
    /// (fine-grained exploitation) as generations progress.
    LinearDecay {
        /// Inertia weight at generation 0 (start of the run).
        w_start: f64,
        /// Inertia weight at the final generation (end of the run).
        w_end: f64,
    },
}

/// Neighborhood topology for the PSO social influence term.
///
/// Determines which particles each particle is attracted toward.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::pso::PsoTopology;
///
/// let topology = PsoTopology::Ring { neighborhood_size: 2 };
/// ```
#[derive(Debug, Clone)]
pub enum PsoTopology {
    /// Global best (gbest) topology.
    ///
    /// Every particle is attracted toward the single best position ever found
    /// across the entire swarm. Fast convergence but prone to premature
    /// convergence on multimodal landscapes.
    Global,

    /// Ring (lbest) topology with a fixed neighborhood size.
    ///
    /// Each particle is attracted toward the best position found within its
    /// `neighborhood_size` nearest neighbors by index (ring-wrapped).
    /// Slower convergence than gbest but better exploration on multimodal problems.
    /// Common values: 3 (tight ring) or 5 (moderate neighborhood).
    Ring {
        /// Number of neighbors (excluding self) to include in each particle's neighborhood.
        ///
        /// The neighborhood of particle `i` is `floor(k/2)` left neighbors and
        /// `ceil(k/2)` right neighbors (ring-wrapped), giving `k+1` particles total
        /// including `i` itself. For example, `neighborhood_size = 2` gives a
        /// 3-particle neighborhood: `{ i-1, i, i+1 }`.
        /// Common values: 2 (tight ring) or 4 (moderate neighborhood).
        neighborhood_size: usize,
    },
}

/// Configuration for a [`PsoEngine`](super::engine::PsoEngine) run.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::pso::{PsoConfiguration, PsoInertia, PsoTopology};
/// use genetic_algorithms::configuration::ProblemSolving;
///
/// let config = PsoConfiguration::default()
///     .with_population_size(30)
///     .with_max_generations(1000)
///     .with_inertia(PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 })
///     .with_topology(PsoTopology::Global)
///     .with_problem_solving(ProblemSolving::Minimization);
/// ```
#[derive(Debug, Clone)]
pub struct PsoConfiguration {
    /// Number of particles in the swarm.
    ///
    /// If 0, a default of 30 is used when `run()` is called (PSO literature
    /// standard, dimension-independent). Set explicitly to tune exploration
    /// depth versus computational budget.
    pub population_size: usize,

    /// Maximum number of generations before stopping.
    ///
    /// The engine stops after at most `max_generations` iterations even if
    /// no `fitness_target` has been reached.
    pub max_generations: usize,

    /// Whether to minimise or maximise fitness.
    ///
    /// Use `ProblemSolving::Minimization` for standard optimization problems
    /// (sphere, Rastrigin, etc.) and `ProblemSolving::Maximization` for
    /// problems where higher fitness is better.
    pub problem_solving: ProblemSolving,

    /// Optional fitness target — engine stops early when reached.
    ///
    /// `None` means the engine runs until `max_generations` is exhausted.
    /// When `Some(t)`, the engine stops as soon as the best individual's
    /// fitness satisfies the stopping condition for the current
    /// `problem_solving` direction.
    pub fitness_target: Option<f64>,

    /// Inertia weight strategy.
    ///
    /// Controls how much of each particle's current velocity is retained on
    /// each generation update. Defaults to `LinearDecay { w_start: 0.9, w_end: 0.4 }`
    /// (Shi & Eberhart 1998 recommendation).
    pub inertia: PsoInertia,

    /// Cognitive coefficient `c1` (personal-best attraction strength).
    ///
    /// Scales the random pull toward a particle's personal best position.
    /// Standard value: 2.0 (Kennedy & Eberhart 1995).
    pub c1: f64,

    /// Social coefficient `c2` (neighborhood-best attraction strength).
    ///
    /// Scales the random pull toward the neighborhood's best known position.
    /// Standard value: 2.0 (Kennedy & Eberhart 1995).
    pub c2: f64,

    /// Neighborhood topology for the social influence term.
    ///
    /// `PsoTopology::Global` (gbest) is the default — all particles share the
    /// same best-known position. `PsoTopology::Ring` uses a smaller local
    /// neighborhood for improved multimodal exploration.
    pub topology: PsoTopology,
}

impl Default for PsoConfiguration {
    fn default() -> Self {
        Self {
            population_size: 30,
            max_generations: 1000,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
            inertia: PsoInertia::LinearDecay {
                w_start: 0.9,
                w_end: 0.4,
            },
            c1: 2.0,
            c2: 2.0,
            topology: PsoTopology::Global,
        }
    }
}

impl PsoConfiguration {
    /// Builder: set population size (number of particles).
    ///
    /// Use 0 to keep the default of 30.
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

    /// Builder: set inertia weight strategy.
    pub fn with_inertia(mut self, inertia: PsoInertia) -> Self {
        self.inertia = inertia;
        self
    }

    /// Builder: set cognitive coefficient `c1`.
    pub fn with_c1(mut self, v: f64) -> Self {
        self.c1 = v;
        self
    }

    /// Builder: set social coefficient `c2`.
    pub fn with_c2(mut self, v: f64) -> Self {
        self.c2 = v;
        self
    }

    /// Builder: set neighborhood topology.
    pub fn with_topology(mut self, topology: PsoTopology) -> Self {
        self.topology = topology;
        self
    }
}

/// Compute the inertia weight `w` for a given generation.
///
/// - `PsoInertia::Constant(w)` → returns `w` unconditionally.
/// - `PsoInertia::LinearDecay { w_start, w_end }` → linearly interpolates from
///   `w_start` at generation 0 to `w_end` at generation `max_generations - 1`.
///   When `max_generations <= 1` the function returns `w_end` to guard against
///   division by zero (Pitfall #5 from 57-RESEARCH.md).
pub(crate) fn inertia_weight(inertia: &PsoInertia, gen: usize, max_generations: usize) -> f64 {
    match inertia {
        PsoInertia::Constant(w) => *w,
        PsoInertia::LinearDecay { w_start, w_end } => {
            if max_generations <= 1 {
                *w_end
            } else {
                w_start + (w_end - w_start) * (gen as f64) / ((max_generations - 1) as f64)
            }
        }
    }
}
