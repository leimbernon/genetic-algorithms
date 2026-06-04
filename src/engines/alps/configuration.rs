//! Configuration types for the ALPS (Age-Layered Population Structure) engine.

use crate::configuration::ProblemSolving;
use crate::operations::{Crossover, Mutation};

/// Age scheme that determines the maximum age threshold for each layer.
///
/// Each scheme produces a monotonically increasing sequence of age limits.
/// The base unit is `age_gap`, which is multiplied by the scheme's factor
/// for each layer index.
#[derive(Debug, Clone, PartialEq)]
pub enum AlpsAgeScheme {
    /// `max_age[i] = (i + 1) * age_gap`
    ///
    /// Even spacing between layers; good default.
    Linear,
    /// `max_age[i] = fibonacci(i + 2) * age_gap`
    ///
    /// Sequence: age_gap × (1, 2, 3, 5, 8, 13, …).  Lower layers turn over
    /// rapidly; upper layers are very stable.
    Fibonacci,
    /// `max_age[i] = (i + 1)² * age_gap`
    ///
    /// Sequence: age_gap × (1, 4, 9, 16, …).  Exponentially expanding
    /// age windows; helpful when convergence is slow.
    Polynomial,
}

/// Configuration for an [`AlpsEngine`](super::engine::AlpsEngine) run.
#[derive(Debug, Clone)]
pub struct AlpsConfiguration {
    /// Number of age layers (minimum 2).
    pub n_layers: usize,
    /// Target number of individuals per layer.
    pub layer_size: usize,
    /// Age scheme used to compute per-layer age thresholds.
    pub age_scheme: AlpsAgeScheme,
    /// Base age unit applied to the scheme's factors.
    ///
    /// Layer 0 accepts individuals up to age `age_gap × scheme_factor(0)`.
    pub age_gap: usize,
    /// How often (in generations) layer 0 is reseeded with fresh individuals.
    ///
    /// Set to `0` to disable injection.
    pub injection_interval: usize,
    /// Maximum number of generations before stopping.
    pub max_generations: usize,
    /// Crossover operator applied when producing offspring.
    pub crossover: Crossover,
    /// Mutation operator applied to offspring.
    pub mutation: Mutation,
    /// Optional step size for Creep mutation (deprecated — embed in `Mutation::Creep { step }`).
    #[deprecated(since = "3.0.0", note = "Use `Mutation::Creep { step: Some(value) }` instead")]
    pub mutation_step: Option<f64>,
    /// Optional standard deviation for Gaussian mutation (deprecated — embed in `Mutation::Gaussian { sigma }`).
    #[deprecated(since = "3.0.0", note = "Use `Mutation::Gaussian { sigma: Some(value) }` instead")]
    pub mutation_sigma: Option<f64>,
    /// Whether to minimise or maximise fitness.
    pub problem_solving: ProblemSolving,
    /// Optional fitness target — engine stops early when reached.
    pub fitness_target: Option<f64>,
}

#[allow(deprecated)]
impl Default for AlpsConfiguration {
    fn default() -> Self {
        Self {
            n_layers: 6,
            layer_size: 20,
            age_scheme: AlpsAgeScheme::Fibonacci,
            age_gap: 5,
            injection_interval: 10,
            max_generations: 1000,
            crossover: Crossover::Uniform,
            mutation: Mutation::Gaussian { sigma: Some(0.1) },
            mutation_step: None,
            mutation_sigma: None,
            problem_solving: ProblemSolving::Minimization,
            fitness_target: None,
        }
    }
}

impl AlpsConfiguration {
    /// Builder: set number of layers.
    pub fn with_n_layers(mut self, n: usize) -> Self {
        self.n_layers = n;
        self
    }
    /// Builder: set target individuals per layer.
    pub fn with_layer_size(mut self, n: usize) -> Self {
        self.layer_size = n;
        self
    }
    /// Builder: set age scheme.
    pub fn with_age_scheme(mut self, scheme: AlpsAgeScheme) -> Self {
        self.age_scheme = scheme;
        self
    }
    /// Builder: set base age unit.
    pub fn with_age_gap(mut self, gap: usize) -> Self {
        self.age_gap = gap;
        self
    }
    /// Builder: set injection interval (`0` to disable).
    pub fn with_injection_interval(mut self, interval: usize) -> Self {
        self.injection_interval = interval;
        self
    }
    /// Builder: set maximum generations.
    pub fn with_max_generations(mut self, n: usize) -> Self {
        self.max_generations = n;
        self
    }
    /// Builder: set crossover operator.
    pub fn with_crossover(mut self, c: Crossover) -> Self {
        self.crossover = c;
        self
    }
    /// Builder: set mutation operator.
    pub fn with_mutation(mut self, m: Mutation) -> Self {
        self.mutation = m;
        self
    }
    /// Builder: set mutation step size (for Creep mutation).
    /// **Deprecated** — use `Mutation::Creep { step: Some(value) }` with `with_mutation()`.
    #[allow(deprecated)]
    pub fn with_mutation_step(mut self, step: f64) -> Self {
        self.mutation_step = Some(step);
        self
    }
    /// Builder: set mutation sigma (for Gaussian mutation).
    /// **Deprecated** — use `Mutation::Gaussian { sigma: Some(value) }` with `with_mutation()`.
    #[allow(deprecated)]
    pub fn with_mutation_sigma(mut self, sigma: f64) -> Self {
        self.mutation_sigma = Some(sigma);
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

    /// Compute the maximum age threshold for each layer.
    ///
    /// Returns a `Vec<usize>` of length `n_layers` where `result[i]` is the
    /// maximum age an individual may have while remaining in layer `i`.
    pub fn max_ages(&self) -> Vec<usize> {
        (0..self.n_layers)
            .map(|i| {
                let factor = match self.age_scheme {
                    AlpsAgeScheme::Linear => i + 1,
                    AlpsAgeScheme::Fibonacci => fibonacci(i + 2),
                    AlpsAgeScheme::Polynomial => (i + 1) * (i + 1),
                };
                factor * self.age_gap
            })
            .collect()
    }
}

/// Returns the `n`-th Fibonacci number (1-indexed: fib(1)=1, fib(2)=1, fib(3)=2, …).
pub fn fibonacci(n: usize) -> usize {
    match n {
        0 => 0,
        1 | 2 => 1,
        _ => {
            let (mut a, mut b) = (1usize, 1usize);
            for _ in 2..n {
                (a, b) = (b, a.saturating_add(b));
            }
            b
        }
    }
}
