//! Configuration types for the Differential Evolution engine.

use crate::configuration::ProblemSolving;

/// DE mutation strategy.
///
/// Each variant corresponds to the standard DE/<base>/<num_diff> naming
/// convention where `<base>` is the vector used as the starting point and
/// `<num_diff>` is the number of difference vectors added.
#[derive(Debug, Clone, PartialEq)]
pub enum DeMutationStrategy {
    /// `DE/rand/1` — random base, one difference vector.
    ///
    /// `v = x_r1 + F * (x_r2 - x_r3)`
    Rand1,
    /// `DE/best/1` — current best as base, one difference vector.
    ///
    /// `v = x_best + F * (x_r1 - x_r2)`
    Best1,
    /// `DE/current-to-best/1` — blend current and best, one difference vector.
    ///
    /// `v = x_i + F * (x_best - x_i) + F * (x_r1 - x_r2)`
    CurrentToBest1,
    /// `DE/rand/2` — random base, two difference vectors.
    ///
    /// `v = x_r1 + F * (x_r2 - x_r3) + F * (x_r4 - x_r5)`
    Rand2,
    /// `DE/best/2` — current best as base, two difference vectors.
    ///
    /// `v = x_best + F * (x_r1 - x_r2) + F * (x_r3 - x_r4)`
    Best2,
}

/// Crossover mode used to combine the mutant vector with the target.
#[derive(Debug, Clone, PartialEq)]
pub enum DeCrossoverMode {
    /// Independent per-gene trial: each gene is taken from the mutant with
    /// probability `cr`, plus one mandatory gene at a random index.
    Binomial,
    /// Contiguous block starting at a random gene: copy from mutant while
    /// `rand() < cr`, wrapping around if necessary.
    Exponential,
}

/// Adaptive parameter control variant.
#[derive(Debug, Clone, PartialEq)]
pub enum DeAdaptive {
    /// Static `F` and `CR` — no adaptation.
    None,
    /// JADE: self-adaptive `F` and `CR` with an external archive of inferior
    /// solutions used as additional difference donors.
    ///
    /// * `p` — fraction of top individuals eligible as "best" in
    ///   `DE/current-to-pbest/1`; must be in `(0, 1]`.
    /// * `c` — learning rate for updating `μ_F` and `μ_CR`; must be in
    ///   `(0, 1]`.
    Jade { p: f64, c: f64 },
    /// L-SHADE: success-history based adaptive `F` and `CR` with a rolling
    /// memory of size `history_size`.
    LShade { history_size: usize },
}

/// Configuration for a [`DeEngine`](super::engine::DeEngine) run.
#[derive(Debug, Clone)]
pub struct DeConfiguration {
    /// Number of individuals in the population.
    pub population_size: usize,
    /// Maximum number of generations before stopping.
    pub max_generations: usize,
    /// Differential weight `F ∈ (0, 2]`.  Used when `adaptive` is `None`.
    pub mutation_factor: f64,
    /// Crossover probability `CR ∈ [0, 1]`.  Used when `adaptive` is `None`.
    pub crossover_rate: f64,
    /// Which mutation formula to apply.
    pub mutation_strategy: DeMutationStrategy,
    /// Binomial or exponential crossover.
    pub crossover_mode: DeCrossoverMode,
    /// Optional self-adaptive variant (JADE or L-SHADE).
    pub adaptive: DeAdaptive,
    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,
    /// Optional fitness target — engine stops early when reached.
    pub fitness_target: Option<f64>,
}

impl Default for DeConfiguration {
    fn default() -> Self {
        Self {
            population_size: 50,
            max_generations: 1000,
            mutation_factor: 0.8,
            crossover_rate: 0.9,
            mutation_strategy: DeMutationStrategy::Rand1,
            crossover_mode: DeCrossoverMode::Binomial,
            adaptive: DeAdaptive::None,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
        }
    }
}

impl DeConfiguration {
    /// Builder: set population size.
    pub fn with_population_size(mut self, n: usize) -> Self {
        self.population_size = n;
        self
    }
    /// Builder: set maximum generations.
    pub fn with_max_generations(mut self, n: usize) -> Self {
        self.max_generations = n;
        self
    }
    /// Builder: set differential weight `F`.
    pub fn with_mutation_factor(mut self, f: f64) -> Self {
        self.mutation_factor = f;
        self
    }
    /// Builder: set crossover rate `CR`.
    pub fn with_crossover_rate(mut self, cr: f64) -> Self {
        self.crossover_rate = cr;
        self
    }
    /// Builder: set mutation strategy.
    pub fn with_mutation_strategy(mut self, s: DeMutationStrategy) -> Self {
        self.mutation_strategy = s;
        self
    }
    /// Builder: set crossover mode.
    pub fn with_crossover_mode(mut self, m: DeCrossoverMode) -> Self {
        self.crossover_mode = m;
        self
    }
    /// Builder: set adaptive variant.
    pub fn with_adaptive(mut self, a: DeAdaptive) -> Self {
        self.adaptive = a;
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
