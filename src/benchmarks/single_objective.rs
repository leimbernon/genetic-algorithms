//! Single-objective benchmark functions.
//!
//! Provides standard scalar-valued benchmark functions commonly used to
//! evaluate single-objective optimization algorithms.
//!
//! # Functions
//!
//! | Name | Domain | Optimum | Notes |
//! |------|--------|---------|-------|
//! | [`Sphere`] | [-5.12, 5.12]^n | f(0,...,0) = 0.0 | Simple separable |
//! | [`Rastrigin`] | [-5.12, 5.12]^n | f(0,...,0) = 0.0 | Highly multimodal |
//! | [`Ackley`] | [-32, 32]^n | f(0,...,0) = 0.0 | Many local minima |

use crate::benchmarks::BenchmarkFn;

// ── Constant bounds ────────────────────────────────────────────────

const SPHERE_LOW: f64 = -5.12;
const SPHERE_HIGH: f64 = 5.12;

const RASTRIGIN_LOW: f64 = -5.12;
const RASTRIGIN_HIGH: f64 = 5.12;

const ACKLEY_LOW: f64 = -32.0;
const ACKLEY_HIGH: f64 = 32.0;

// ── Helper ─────────────────────────────────────────────────────────

/// Build a `Vec<(f64, f64)>` of `n` identical bound pairs.
fn repeat_bounds(low: f64, high: f64, n: usize) -> Vec<(f64, f64)> {
    vec![(low, high); n]
}

// ── Sphere ─────────────────────────────────────────────────────────

/// Sphere benchmark function.
///
/// `f(x) = sum(x_i ^ 2)`
///
/// Simple, convex, and separable. A baseline for optimization algorithms.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sphere {
    /// Number of decision variables.
    pub n: usize,
    /// Per-dimension bounds, length equals `n`.
    bounds: Vec<(f64, f64)>,
}

impl Sphere {
    /// Create a new `Sphere` function with `n` decision variables.
    pub fn new(n: usize) -> Self {
        Self {
            n,
            bounds: repeat_bounds(SPHERE_LOW, SPHERE_HIGH, n),
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::new(30)
    }
}

impl BenchmarkFn for Sphere {
    fn name(&self) -> &'static str {
        "Sphere"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n,
            "Sphere::evaluate called with {} variables, expected {}",
            x.len(),
            self.n
        );
        vec![x.iter().map(|xi| xi * xi).sum()]
    }
}

// ── Rastrigin ──────────────────────────────────────────────────────

/// Rastrigin benchmark function.
///
/// `f(x) = 10 * n + sum(x_i ^ 2 - 10 * cos(2 * pi * x_i))`
///
/// Highly multimodal with many regularly spaced local minima.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rastrigin {
    /// Number of decision variables.
    pub n: usize,
    /// Per-dimension bounds, length equals `n`.
    bounds: Vec<(f64, f64)>,
}

impl Rastrigin {
    /// Create a new `Rastrigin` function with `n` decision variables.
    pub fn new(n: usize) -> Self {
        Self {
            n,
            bounds: repeat_bounds(RASTRIGIN_LOW, RASTRIGIN_HIGH, n),
        }
    }
}

impl Default for Rastrigin {
    fn default() -> Self {
        Self::new(30)
    }
}

impl BenchmarkFn for Rastrigin {
    fn name(&self) -> &'static str {
        "Rastrigin"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n,
            "Rastrigin::evaluate called with {} variables, expected {}",
            x.len(),
            self.n
        );
        let sum: f64 = x
            .iter()
            .map(|xi| xi * xi - 10.0 * (2.0 * std::f64::consts::PI * xi).cos())
            .sum();
        vec![10.0 * self.n as f64 + sum]
    }
}

// ── Ackley ─────────────────────────────────────────────────────────

/// Ackley benchmark function.
///
/// `f(x) = -20 * exp(-0.2 * sqrt((1/n) * sum(x_i ^ 2)))`
///       ` - exp((1/n) * sum(cos(2 * pi * x_i))) + 20 + e`
///
/// Many local minima with a single global minimum at the origin.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ackley {
    /// Number of decision variables.
    pub n: usize,
    /// Per-dimension bounds, length equals `n`.
    bounds: Vec<(f64, f64)>,
}

impl Ackley {
    /// Create a new `Ackley` function with `n` decision variables.
    pub fn new(n: usize) -> Self {
        Self {
            n,
            bounds: repeat_bounds(ACKLEY_LOW, ACKLEY_HIGH, n),
        }
    }
}

impl Default for Ackley {
    fn default() -> Self {
        Self::new(30)
    }
}

impl BenchmarkFn for Ackley {
    fn name(&self) -> &'static str {
        "Ackley"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n,
            "Ackley::evaluate called with {} variables, expected {}",
            x.len(),
            self.n
        );
        let n = self.n as f64;
        let sum_sq: f64 = x.iter().map(|xi| xi * xi).sum();
        let sum_cos: f64 = x
            .iter()
            .map(|xi| (2.0 * std::f64::consts::PI * xi).cos())
            .sum();

        let term1 = -20.0 * (-0.2 * (sum_sq / n).sqrt()).exp();
        let term2 = -(sum_cos / n).exp();

        vec![term1 + term2 + 20.0 + std::f64::consts::E]
    }
}
