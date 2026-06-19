//! ZDT (Zitzler-Deb-Thiele) bi-objective benchmark functions.
//!
//! Provides ZDT1 through ZDT6 benchmark functions commonly used to evaluate
//! multi-objective optimization algorithms. All functions return two objectives
//! (`vec![f1, f2]`) from their `evaluate()` method.
//!
//! # Functions
//!
//! | Name | Domain | Default n | Pareto Front |
//! |------|--------|-----------|--------------|
//! | [`ZDT1`] | [0, 1]^n | 30 | Convex |
//! | [`ZDT2`] | [0, 1]^n | 30 | Non-convex |
//! | [`ZDT3`] | [0, 1]^n | 30 | Disconnected (5 segments) |
//! | [`ZDT4`] | \[0,1\] × \[-5,5\]\^{n-1} | 10 | Convex, multimodal |
//! | [`ZDT5`] | [0, 1]^n | 11 | Convex (continuous relaxation) |
//! | [`ZDT6`] | [0, 1]^n | 10 | Non-convex, biased density |
//!
//! Note: ZDT5 is a continuous relaxation of the original binary ZDT5 problem.
//! Each decision variable x_i in [0, 1] maps to an integer via
//! `z_i = floor(1 + k_i * x_i)` where `k_0 = 30` and `k_i = 5` for i >= 1.

use crate::benchmarks::BenchmarkFn;

// ── Helper: build uniform bounds ──────────────────────────────────

fn bounds_01(n: usize) -> Vec<(f64, f64)> {
    vec![(0.0, 1.0); n]
}

// ── ZDT1: Convex Pareto front ─────────────────────────────────────

/// ZDT1 benchmark function — convex Pareto front.
///
/// f1 = x_0
/// g = 1 + 9/(n-1) * sum(x_1..x_n)
/// f2 = g * (1 - sqrt(x_0 / g))
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZDT1 {
    n_vars: usize,
    bounds: Vec<(f64, f64)>,
}

impl ZDT1 {
    pub fn new(n_vars: usize) -> Self {
        Self {
            n_vars,
            bounds: bounds_01(n_vars),
        }
    }
}

impl Default for ZDT1 {
    fn default() -> Self {
        Self::new(30)
    }
}

impl BenchmarkFn for ZDT1 {
    fn name(&self) -> &'static str {
        "ZDT1"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0, 1.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "ZDT1::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let f1 = x[0];
        let g = 1.0 + 9.0 * x[1..].iter().sum::<f64>() / (self.n_vars - 1) as f64;
        let f2 = g * (1.0 - (x[0] / g).sqrt());
        vec![f1, f2]
    }
}

// ── ZDT2: Non-convex Pareto front ─────────────────────────────────

/// ZDT2 benchmark function — non-convex Pareto front.
///
/// f1 = x_0
/// g = 1 + 9/(n-1) * sum(x_1..x_n)
/// f2 = g * (1 - (x_0 / g)^2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZDT2 {
    n_vars: usize,
    bounds: Vec<(f64, f64)>,
}

impl ZDT2 {
    pub fn new(n_vars: usize) -> Self {
        Self {
            n_vars,
            bounds: bounds_01(n_vars),
        }
    }
}

impl Default for ZDT2 {
    fn default() -> Self {
        Self::new(30)
    }
}

impl BenchmarkFn for ZDT2 {
    fn name(&self) -> &'static str {
        "ZDT2"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0, 1.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "ZDT2::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let f1 = x[0];
        let g = 1.0 + 9.0 * x[1..].iter().sum::<f64>() / (self.n_vars - 1) as f64;
        let f2 = g * (1.0 - (x[0] / g).powi(2));
        vec![f1, f2]
    }
}

// ── ZDT3: Disconnected Pareto front ───────────────────────────────

/// ZDT3 benchmark function — disconnected Pareto front (5 segments).
///
/// f1 = x_0
/// g = 1 + 9/(n-1) * sum(x_1..x_n)
/// f2 = g * (1 - sqrt(x_0/g) - (x_0/g) * sin(10*pi*x_0))
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZDT3 {
    n_vars: usize,
    bounds: Vec<(f64, f64)>,
}

impl ZDT3 {
    pub fn new(n_vars: usize) -> Self {
        Self {
            n_vars,
            bounds: bounds_01(n_vars),
        }
    }
}

impl Default for ZDT3 {
    fn default() -> Self {
        Self::new(30)
    }
}

impl BenchmarkFn for ZDT3 {
    fn name(&self) -> &'static str {
        "ZDT3"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0, 1.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "ZDT3::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let f1 = x[0];
        let g = 1.0 + 9.0 * x[1..].iter().sum::<f64>() / (self.n_vars - 1) as f64;
        let x0_g = x[0] / g;
        let f2 = g * (1.0 - x0_g.sqrt() - x0_g * (10.0 * std::f64::consts::PI * x[0]).sin());
        vec![f1, f2]
    }
}

// ── ZDT4: Multimodal convex Pareto front ──────────────────────────

/// ZDT4 benchmark function — convex front with many local fronts.
///
/// f1 = x_0
/// g = 1 + 10*(n-1) + sum(x_1..x_n, x_i^2 - 10*cos(4*pi*x_i))
/// f2 = g * (1 - sqrt(x_0 / g))
///
/// First variable in [0, 1]; remaining variables in [-5, 5].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZDT4 {
    n_vars: usize,
    bounds: Vec<(f64, f64)>,
}

impl ZDT4 {
    pub fn new(n_vars: usize) -> Self {
        let mut bounds = vec![(0.0, 1.0)]; // first variable
        bounds.extend(vec![(-5.0, 5.0); n_vars - 1]); // remaining
        Self { n_vars, bounds }
    }
}

impl Default for ZDT4 {
    fn default() -> Self {
        Self::new(10)
    }
}

impl BenchmarkFn for ZDT4 {
    fn name(&self) -> &'static str {
        "ZDT4"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0, 1.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "ZDT4::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let f1 = x[0];
        let g = 1.0
            + 10.0 * (self.n_vars - 1) as f64
            + x[1..]
                .iter()
                .map(|&xi| xi.powi(2) - 10.0 * (4.0 * std::f64::consts::PI * xi).cos())
                .sum::<f64>();
        let f2 = g * (1.0 - (x[0] / g).sqrt());
        vec![f1, f2]
    }
}

// ── ZDT5: Continuous relaxation ──────────────────────────────────

/// ZDT5 benchmark function — continuous relaxation of the original binary problem.
///
/// The original ZDT5 uses 80 binary decision variables with group encoding.
/// This implementation uses a continuous relaxation with 11 variables in [0, 1]:
///
/// z_0 = floor(1 + 30 * x_0)          // integer in [1, 31]
/// z_i = floor(1 + 5 * x_i) for i=1..10 // integer in [1, 5] each
/// u = z_0
/// v = sum(z_i for i=1..10)
/// g = 1 + v
/// f1 = 1 + u
/// f2 = g / f1
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZDT5 {
    n_vars: usize,
    bounds: Vec<(f64, f64)>,
}

impl ZDT5 {
    pub fn new(n_vars: usize) -> Self {
        Self {
            n_vars,
            bounds: bounds_01(n_vars),
        }
    }
}

impl Default for ZDT5 {
    fn default() -> Self {
        Self::new(11)
    }
}

impl BenchmarkFn for ZDT5 {
    fn name(&self) -> &'static str {
        "ZDT5"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![2.0, 5.5]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "ZDT5::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let z0 = (1.0 + 30.0 * x[0]).floor();
        let zi_sum: f64 = x[1..].iter().map(|&xi| (1.0 + 5.0 * xi).floor()).sum();
        let u = z0;
        let g = 1.0 + zi_sum;
        let f1 = 1.0 + u;
        let f2 = g / f1;
        vec![f1, f2]
    }
}

// ── ZDT6: Biased density, non-convex front ───────────────────────

/// ZDT6 benchmark function — non-convex front with biased density.
///
/// f1 = 1 - exp(-4*x_0) * sin^6(6*pi*x_0)
/// g = 1 + 9 * ((sum(x_1..x_n) / (n-1))^0.25)
/// f2 = g * (1 - (f1 / g)^2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZDT6 {
    n_vars: usize,
    bounds: Vec<(f64, f64)>,
}

impl ZDT6 {
    pub fn new(n_vars: usize) -> Self {
        Self {
            n_vars,
            bounds: bounds_01(n_vars),
        }
    }
}

impl Default for ZDT6 {
    fn default() -> Self {
        Self::new(10)
    }
}

impl BenchmarkFn for ZDT6 {
    fn name(&self) -> &'static str {
        "ZDT6"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0, 1.0]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "ZDT6::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let f1 = 1.0 - (-4.0 * x[0]).exp() * (6.0 * std::f64::consts::PI * x[0]).sin().powi(6);
        let g = 1.0 + 9.0 * (x[1..].iter().sum::<f64>() / (self.n_vars - 1) as f64).powf(0.25);
        let f2 = g * (1.0 - (f1 / g).powi(2));
        vec![f1, f2]
    }
}
