//! DTLZ (Deb-Thiele-Laumanns-Zitzler) many-objective benchmark functions.
//!
//! Provides DTLZ1 through DTLZ7 benchmark functions commonly used to evaluate
//! many-objective optimization algorithms. All functions take `(n_vars, n_obj)`
//! parameters where `n_vars` is the number of decision variables and `n_obj` is
//! the number of objectives.
//!
//! # Functions
//!
//! | Name | Type | g(x) Shape | Notes |
//! |------|------|------------|-------|
//! | [`DTLZ1`] | Linear hyperplane | Multimodal Rastrigin-like | 3^k local fronts |
//! | [`DTLZ2`] | Sphere surface | Quadratic | Unit sphere Pareto front |
//! | [`DTLZ3`] | Sphere surface | Multimodal Rastrigin-like | 3^k local fronts |
//! | [`DTLZ4`] | Sphere surface | Quadratic | Biased density (alpha=100) |
//! | [`DTLZ5`] | Degenerate curve | Quadratic | Reduced dimensionality front |
//! | [`DTLZ6`] | Degenerate curve | Cubic root | Reduced dimensionality front |
//! | [`DTLZ7`] | Disconnected | Linear | M-1 linear + 1 disconnected |

use crate::benchmarks::BenchmarkFn;

// ── Helper ─────────────────────────────────────────────────────────

/// Build uniform [0, 1] bounds for `n` variables.
fn bounds_01(n: usize) -> Vec<(f64, f64)> {
    vec![(0.0, 1.0); n]
}

// ── DTLZ1: Linear hyperplane front, multimodal ────────────────────

/// DTLZ1 benchmark function — linear hyperplane Pareto front.
///
/// g = 100 * (k + sum((xi - 0.5)^2 - cos(20*pi*(xi - 0.5)) for xi in X_K))
/// f_i follow the standard multi-dimensional product formula.
///
/// Has 3^k local Pareto-optimal fronts (Rastrigin-like g function).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DTLZ1 {
    n_vars: usize,
    n_obj: usize,
    bounds: Vec<(f64, f64)>,
}

impl DTLZ1 {
    pub fn new(n_vars: usize, n_obj: usize) -> Self {
        Self {
            n_vars,
            n_obj,
            bounds: bounds_01(n_vars),
        }
    }
}

impl BenchmarkFn for DTLZ1 {
    fn name(&self) -> &'static str {
        "DTLZ1"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0; self.n_obj]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "DTLZ1::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..]; // distance variables

        // g(x) = 100 * (k + sum((xi - 0.5)^2 - cos(20*pi*(xi - 0.5))))
        let g: f64 = 100.0
            * (k as f64
                + x_m
                    .iter()
                    .map(|&xi| {
                        (xi - 0.5).powi(2) - (20.0 * std::f64::consts::PI * (xi - 0.5)).cos()
                    })
                    .sum::<f64>());

        let mut f = vec![0.0; self.n_obj];

        // Standard DTLZ1 (1-indexed):
        // f1 = 0.5 * x1 * x2 * ... * x_{M-1} * (1+g)
        // fi = 0.5 * x1 * ... * x_{M-i} * (1 - x_{M-i+1}) * (1+g)  for 2 <= i < M
        // fM = 0.5 * (1 - x1) * (1+g)
        let m = self.n_obj;
        for i in 0..m {
            let mut prod = 1.0;
            for &xj in &x[..m - 1 - i] {
                prod *= xj;
            }
            if i == 0 {
                f[i] = 0.5 * prod * (1.0 + g);
            } else {
                f[i] = 0.5 * prod * (1.0 - x[m - 1 - i]) * (1.0 + g);
            }
        }

        f
    }
}

// ── DTLZ2: Sphere surface front ───────────────────────────────────

/// DTLZ2 benchmark function — unit sphere Pareto front.
///
/// g = sum((xi - 0.5)^2 for xi in X_K)
/// f_i use trigonometric product formulas producing points on a unit sphere.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DTLZ2 {
    n_vars: usize,
    n_obj: usize,
    bounds: Vec<(f64, f64)>,
}

impl DTLZ2 {
    pub fn new(n_vars: usize, n_obj: usize) -> Self {
        Self {
            n_vars,
            n_obj,
            bounds: bounds_01(n_vars),
        }
    }
}

impl BenchmarkFn for DTLZ2 {
    fn name(&self) -> &'static str {
        "DTLZ2"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0; self.n_obj]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "DTLZ2::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        self.evaluate_dtlz2_like(x)
    }
}

impl DTLZ2 {
    /// Shared evaluation for DTLZ2/3/4/5/6 which share the same f_i structure.
    /// `x_transformed` provides the position values (after any DTLZ4 alpha or DTLZ5/6 theta),
    /// `g` is the distance function value.
    fn evaluate_dtlz2_like_core(x_pos: &[f64], g: f64, m: usize) -> Vec<f64> {
        let mut f = vec![0.0; m];

        // Standard DTLZ2 (1-indexed):
        // f1 = (1+g) * cos(x1*pi/2) * cos(x2*pi/2) * ... * cos(x_{M-1}*pi/2)
        // f2 = (1+g) * cos(x1*pi/2) * cos(x2*pi/2) * ... * sin(x_{M-1}*pi/2)
        // ...
        // f_{M-1} = (1+g) * cos(x1*pi/2) * sin(x2*pi/2)
        // f_M = (1+g) * sin(x1*pi/2)

        // 0-indexed: x_pos[0]..x_pos[M-2] correspond to x1..x_{M-1}
        for i in 0..m {
            let mut prod = 1.0;
            // Product of cos(x_pos[j] * pi/2) for j in 0..(M-1-i)
            for &xj in &x_pos[..m - 1 - i] {
                prod *= xj.cos();
            }
            if i == 0 {
                f[i] = (1.0 + g) * prod;
            } else {
                f[i] = (1.0 + g) * prod * x_pos[m - 1 - i].sin();
            }
        }

        f
    }

    fn evaluate_dtlz2_like(&self, x: &[f64]) -> Vec<f64> {
        let _k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = sum((xi - 0.5)^2)
        let g: f64 = x_m.iter().map(|&xi| (xi - 0.5).powi(2)).sum();

        // Position variables: x_pos[j] = x[j] * pi/2
        let x_pos: Vec<f64> = x[0..self.n_obj - 1]
            .iter()
            .map(|&xj| xj * std::f64::consts::FRAC_PI_2)
            .collect();

        Self::evaluate_dtlz2_like_core(&x_pos, g, self.n_obj)
    }
}

// ── DTLZ3: Sphere surface, multimodal ─────────────────────────────

/// DTLZ3 benchmark function — unit sphere front with many local fronts.
///
/// Same f_i structure as DTLZ2, but with DTLZ1's Rastrigin-like g function
/// producing 3^k local Pareto-optimal fronts.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DTLZ3 {
    n_vars: usize,
    n_obj: usize,
    bounds: Vec<(f64, f64)>,
}

impl DTLZ3 {
    pub fn new(n_vars: usize, n_obj: usize) -> Self {
        Self {
            n_vars,
            n_obj,
            bounds: bounds_01(n_vars),
        }
    }
}

impl BenchmarkFn for DTLZ3 {
    fn name(&self) -> &'static str {
        "DTLZ3"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0; self.n_obj]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "DTLZ3::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // DTLZ1's g function
        let g: f64 = 100.0
            * (k as f64
                + x_m
                    .iter()
                    .map(|&xi| {
                        (xi - 0.5).powi(2) - (20.0 * std::f64::consts::PI * (xi - 0.5)).cos()
                    })
                    .sum::<f64>());

        let x_pos: Vec<f64> = x[0..self.n_obj - 1]
            .iter()
            .map(|&xj| xj * std::f64::consts::FRAC_PI_2)
            .collect();

        DTLZ2::evaluate_dtlz2_like_core(&x_pos, g, self.n_obj)
    }
}

// ── DTLZ4: Sphere surface, biased density ─────────────────────────

/// DTLZ4 benchmark function — unit sphere front with biased solution density.
///
/// Same f_i structure as DTLZ2, but position variables are raised to power
/// `alpha` (default 100) to bias solution density toward the f_M=0 edge.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DTLZ4 {
    n_vars: usize,
    n_obj: usize,
    alpha: f64,
    bounds: Vec<(f64, f64)>,
}

impl DTLZ4 {
    pub fn new(n_vars: usize, n_obj: usize) -> Self {
        Self {
            n_vars,
            n_obj,
            alpha: 100.0,
            bounds: bounds_01(n_vars),
        }
    }

    pub fn with_alpha(n_vars: usize, n_obj: usize, alpha: f64) -> Self {
        Self {
            n_vars,
            n_obj,
            alpha,
            bounds: bounds_01(n_vars),
        }
    }
}

impl BenchmarkFn for DTLZ4 {
    fn name(&self) -> &'static str {
        "DTLZ4"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0; self.n_obj]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "DTLZ4::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let _k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = sum((xi - 0.5)^2) — same as DTLZ2 (uses original x values)
        let g: f64 = x_m.iter().map(|&xi| (xi - 0.5).powi(2)).sum();

        // Position variables raised to alpha power: x[j]^alpha * pi/2
        let x_pos: Vec<f64> = x[0..self.n_obj - 1]
            .iter()
            .map(|&xj| xj.powf(self.alpha) * std::f64::consts::FRAC_PI_2)
            .collect();

        DTLZ2::evaluate_dtlz2_like_core(&x_pos, g, self.n_obj)
    }
}

// ── DTLZ5: Degenerate curve ───────────────────────────────────────

/// DTLZ5 benchmark function — degenerate curve (reduced dimensionality Pareto front).
///
/// Uses theta transformation on position variables then same f structure as DTLZ2.
/// The Pareto front is a curve regardless of M (number of objectives).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DTLZ5 {
    n_vars: usize,
    n_obj: usize,
    bounds: Vec<(f64, f64)>,
}

impl DTLZ5 {
    pub fn new(n_vars: usize, n_obj: usize) -> Self {
        Self {
            n_vars,
            n_obj,
            bounds: bounds_01(n_vars),
        }
    }
}

impl BenchmarkFn for DTLZ5 {
    fn name(&self) -> &'static str {
        "DTLZ5"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0; self.n_obj]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "DTLZ5::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let _k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = sum((xi - 0.5)^2)
        let g: f64 = x_m.iter().map(|&xi| (xi - 0.5).powi(2)).sum();

        // Theta transformation:
        // theta[0] = x[0] * pi/2
        // theta[t] = pi/(4*(1+g)) * (1 + 2*g*x[t]) for t = 1..M-2
        let m = self.n_obj;
        let mut theta = Vec::with_capacity(m - 1);
        for (i, &xi) in x[..m - 1].iter().enumerate() {
            let t = if i == 0 {
                xi * std::f64::consts::FRAC_PI_2
            } else {
                std::f64::consts::PI / (4.0 * (1.0 + g)) * (1.0 + 2.0 * g * xi)
            };
            theta.push(t);
        }

        DTLZ2::evaluate_dtlz2_like_core(&theta, g, self.n_obj)
    }
}

// ── DTLZ6: Degenerate curve, cubic g ──────────────────────────────

/// DTLZ6 benchmark function — degenerate curve with cubic distance function.
///
/// Same theta transformation as DTLZ5 but with g(x) = sqrt(sum(x_i^3)).
/// Produces a degenerate Pareto front regardless of M.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DTLZ6 {
    n_vars: usize,
    n_obj: usize,
    bounds: Vec<(f64, f64)>,
}

impl DTLZ6 {
    pub fn new(n_vars: usize, n_obj: usize) -> Self {
        Self {
            n_vars,
            n_obj,
            bounds: bounds_01(n_vars),
        }
    }
}

impl BenchmarkFn for DTLZ6 {
    fn name(&self) -> &'static str {
        "DTLZ6"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0; self.n_obj]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "DTLZ6::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let _k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = sqrt(sum(xi^3))
        let g: f64 = x_m.iter().map(|&xi| xi.powi(3)).sum::<f64>().sqrt();

        // Theta transformation (same as DTLZ5)
        let m = self.n_obj;
        let mut theta = Vec::with_capacity(m - 1);
        for (i, &xi) in x[..m - 1].iter().enumerate() {
            let t = if i == 0 {
                xi * std::f64::consts::FRAC_PI_2
            } else {
                std::f64::consts::PI / (4.0 * (1.0 + g)) * (1.0 + 2.0 * g * xi)
            };
            theta.push(t);
        }

        DTLZ2::evaluate_dtlz2_like_core(&theta, g, self.n_obj)
    }
}

// ── DTLZ7: Disconnected Pareto front ──────────────────────────────

/// DTLZ7 benchmark function — disconnected Pareto front.
///
/// f_i = x_i for i = 0..M-2
/// g = 1 + 9/k * sum(x_i for x_i in X_K)
/// h = M - sum(f_i/(1+g) * (1 + sin(3*pi*f_i/(1+g))) for i in 0..M-2)
/// f_{M-1} = (1+g) * h
///
/// Produces a disconnected Pareto front due to the sin term in h.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DTLZ7 {
    n_vars: usize,
    n_obj: usize,
    bounds: Vec<(f64, f64)>,
}

impl DTLZ7 {
    pub fn new(n_vars: usize, n_obj: usize) -> Self {
        Self {
            n_vars,
            n_obj,
            bounds: bounds_01(n_vars),
        }
    }
}

impl BenchmarkFn for DTLZ7 {
    fn name(&self) -> &'static str {
        "DTLZ7"
    }

    fn bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }

    fn optimum_value(&self) -> Vec<f64> {
        vec![0.0; self.n_obj]
    }

    fn evaluate(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(
            x.len(),
            self.n_vars,
            "DTLZ7::evaluate called with {} variables, expected {}",
            x.len(),
            self.n_vars
        );
        let k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = 1 + 9/k * sum(x_i)
        let g: f64 = 1.0 + 9.0 * x_m.iter().sum::<f64>() / k as f64;

        let m = self.n_obj;
        let mut f = vec![0.0; m];

        // f_i = x_i for i = 0..M-2
        f[..m - 1].copy_from_slice(&x[..m - 1]);

        // h = M - sum(f_i/(1+g) * (1 + sin(3*pi*f_i/(1+g))))
        let mut h_sum = 0.0;
        for &fi in &f[..m - 1] {
            let term = fi / (1.0 + g);
            h_sum += term * (1.0 + (3.0 * std::f64::consts::PI * term).sin());
        }
        let h = m as f64 - h_sum;

        f[m - 1] = (1.0 + g) * h;

        f
    }
}
