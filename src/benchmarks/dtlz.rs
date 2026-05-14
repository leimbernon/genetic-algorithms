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
        assert_eq!(x.len(), self.n_vars,
            "DTLZ1::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
        let k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..]; // distance variables

        // g(x) = 100 * (k + sum((xi - 0.5)^2 - cos(20*pi*(xi - 0.5))))
        let g: f64 = 100.0 * (k as f64 + x_m.iter().map(|&xi| {
            (xi - 0.5).powi(2) - (20.0 * std::f64::consts::PI * (xi - 0.5)).cos()
        }).sum::<f64>());

        let mut f = vec![0.0; self.n_obj];

        // Standard DTLZ1 (1-indexed):
        // f1 = 0.5 * x1 * x2 * ... * x_{M-1} * (1+g)
        // fi = 0.5 * x1 * ... * x_{M-i} * (1 - x_{M-i+1}) * (1+g)  for 2 <= i < M
        // fM = 0.5 * (1 - x1) * (1+g)
        let m = self.n_obj;
        for i in 0..m {
            let mut prod = 1.0;
            for j in 0..(m - 1 - i) {
                prod *= x[j];
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
        assert_eq!(x.len(), self.n_vars,
            "DTLZ2::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
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
            for j in 0..(m - 1 - i) {
                prod *= x_pos[j].cos();
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
        assert_eq!(x.len(), self.n_vars,
            "DTLZ3::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
        let k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // DTLZ1's g function
        let g: f64 = 100.0 * (k as f64 + x_m.iter().map(|&xi| {
            (xi - 0.5).powi(2) - (20.0 * std::f64::consts::PI * (xi - 0.5)).cos()
        }).sum::<f64>());

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
        assert_eq!(x.len(), self.n_vars,
            "DTLZ4::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
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
        assert_eq!(x.len(), self.n_vars,
            "DTLZ5::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
        let _k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = sum((xi - 0.5)^2)
        let g: f64 = x_m.iter().map(|&xi| (xi - 0.5).powi(2)).sum();

        // Theta transformation:
        // theta[0] = x[0] * pi/2
        // theta[t] = pi/(4*(1+g)) * (1 + 2*g*x[t]) for t = 1..M-2
        let m = self.n_obj;
        let mut theta = Vec::with_capacity(m - 1);
        for i in 0..(m - 1) {
            let t = if i == 0 {
                x[i] * std::f64::consts::FRAC_PI_2
            } else {
                std::f64::consts::PI / (4.0 * (1.0 + g)) * (1.0 + 2.0 * g * x[i])
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
        assert_eq!(x.len(), self.n_vars,
            "DTLZ6::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
        let _k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = sqrt(sum(xi^3))
        let g: f64 = x_m.iter().map(|&xi| xi.powi(3)).sum::<f64>().sqrt();

        // Theta transformation (same as DTLZ5)
        let m = self.n_obj;
        let mut theta = Vec::with_capacity(m - 1);
        for i in 0..(m - 1) {
            let t = if i == 0 {
                x[i] * std::f64::consts::FRAC_PI_2
            } else {
                std::f64::consts::PI / (4.0 * (1.0 + g)) * (1.0 + 2.0 * g * x[i])
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
        assert_eq!(x.len(), self.n_vars,
            "DTLZ7::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
        let k = self.n_vars - self.n_obj + 1;
        let x_m = &x[self.n_obj - 1..];

        // g = 1 + 9/k * sum(x_i)
        let g: f64 = 1.0 + 9.0 * x_m.iter().sum::<f64>() / k as f64;

        let m = self.n_obj;
        let mut f = vec![0.0; m];

        // f_i = x_i for i = 0..M-2
        for i in 0..(m - 1) {
            f[i] = x[i];
        }

        // h = M - sum(f_i/(1+g) * (1 + sin(3*pi*f_i/(1+g))))
        let mut h_sum = 0.0;
        for i in 0..(m - 1) {
            let term = f[i] / (1.0 + g);
            h_sum += term * (1.0 + (3.0 * std::f64::consts::PI * term).sin());
        }
        let h = m as f64 - h_sum;

        f[m - 1] = (1.0 + g) * h;

        f
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-12;

    // ── DTLZ1 ────────────────────────────────────────────────────

    #[test]
    fn test_dtlz1_form() {
        // At x = [0]*7 with M=3: g is non-negative, output length = 3
        let dtlz1 = DTLZ1::new(7, 3);
        let result = dtlz1.evaluate(&vec![0.0; 7]);
        assert_eq!(result.len(), 3);
        // g should be positive (125.0) since 0 != 0.5
        assert!(result[2] > 0.0, "DTLZ1 f3 should be > 0");
    }

    #[test]
    fn test_dtlz1_uniform_optimum() {
        // At x = [0.5]*7 with M=3: g = 0 (each term = 0 - cos(0) = -1,
        // so sum = 5 * (0 - 1) = -5, plus k=5: g = 100*(5-5) = 0)
        // f1 = 0.5 * 0.5 * 0.5 = 0.125
        // f2 = 0.5 * 0.5 * (1-0.5) = 0.125
        // f3 = 0.5 * (1-0.5) = 0.25
        let dtlz1 = DTLZ1::new(7, 3);
        let result = dtlz1.evaluate(&vec![0.5; 7]);
        assert!((result[0] - 0.125).abs() < EPSILON, "DTLZ1 f1 expected 0.125, got {}", result[0]);
        assert!((result[1] - 0.125).abs() < EPSILON, "DTLZ1 f2 expected 0.125, got {}", result[1]);
        assert!((result[2] - 0.25).abs() < EPSILON, "DTLZ1 f3 expected 0.25, got {}", result[2]);
    }

    // ── DTLZ2 ────────────────────────────────────────────────────

    #[test]
    fn test_dtlz2_sphere_surface() {
        // At x = [0.5]*12, M=3: all distance vars at 0.5 -> g=0.
        // f1 = cos(pi/4)^2 = 0.5
        // f2 = cos(pi/4)*sin(pi/4) = 0.5
        // f3 = sin(pi/4) = sqrt(2)/2 ≈ 0.7071
        // Sum of squares: 0.25 + 0.25 + 0.5 = 1.0
        let dtlz2 = DTLZ2::new(12, 3);
        let result = dtlz2.evaluate(&vec![0.5; 12]);
        let sum_sq: f64 = result.iter().map(|&fi| fi * fi).sum();
        assert!((sum_sq - 1.0).abs() < 1e-10,
            "DTLZ2 sum of squares expected 1.0, got {}", sum_sq);
    }

    // ── DTLZ3 ────────────────────────────────────────────────────

    #[test]
    fn test_dtlz3_uniform_optimum() {
        // At x = [0.5]*12, M=3: distance vars at 0.5 -> DTLZ1 g = 0
        // Same result as DTLZ2 uniform point
        let dtlz3 = DTLZ3::new(12, 3);
        let result = dtlz3.evaluate(&vec![0.5; 12]);
        let sum_sq: f64 = result.iter().map(|&fi| fi * fi).sum();
        assert!((sum_sq - 1.0).abs() < 1e-10,
            "DTLZ3 sum of squares expected 1.0, got {}", sum_sq);
    }

    // ── DTLZ4 ────────────────────────────────────────────────────

    #[test]
    fn test_dtlz4_default_alpha() {
        let dtlz4 = DTLZ4::new(12, 3);
        let result = dtlz4.evaluate(&vec![0.5; 12]);
        assert_eq!(result.len(), 3);
        // With alpha=100, x[0]^100 = (0.5)^100 ≈ 7.89e-31
        // cos(x[0]^100 * pi/2) ≈ cos(0) = 1
        // sin(x[0]^100 * pi/2) ≈ sin(0) = 0
        // So f3 ≈ 0, meaning the solution is on the f1-f2 plane
        assert!(result[2].abs() < 1e-10,
            "DTLZ4 f3 should be near 0 with alpha=100, got {}", result[2]);
    }

    #[test]
    fn test_dtlz4_alpha_one() {
        // With alpha=1, DTLZ4 should equal DTLZ2
        let dtlz4 = DTLZ4::with_alpha(12, 3, 1.0);
        let dtlz2 = DTLZ2::new(12, 3);
        let result4 = dtlz4.evaluate(&vec![0.3; 12]);
        let result2 = dtlz2.evaluate(&vec![0.3; 12]);
        for i in 0..3 {
            assert!((result4[i] - result2[i]).abs() < EPSILON,
                "DTLZ4(alpha=1) and DTLZ2 differ at f{}: {} vs {}", i, result4[i], result2[i]);
        }
    }

    // ── DTLZ5 ────────────────────────────────────────────────────

    #[test]
    fn test_dtlz5_uniform_optimum() {
        // At x = [0.5]*12, M=3: g=0, theta[0]=pi/4, theta[1]=pi/4/(4*(1))
        // theta[1] = pi/16
        // f1 = (1+0) * cos(pi/4) * cos(pi/16) = 0.7071 * 0.9808 ≈ 0.6935
        // f2 = (1+0) * cos(pi/4) * sin(pi/16) = 0.7071 * 0.1951 ≈ 0.1380
        // f3 = (1+0) * sin(pi/4) = 0.7071
        let dtlz5 = DTLZ5::new(12, 3);
        let result = dtlz5.evaluate(&vec![0.5; 12]);
        assert_eq!(result.len(), 3);
        // Just verify the expected output length and structure
        assert!(result[0] > 0.0, "DTLZ5 f1 should be positive");
        assert!(result[2] > result[1], "DTLZ5 f3 should be > f2 with uniform input");
    }

    // ── DTLZ6 ────────────────────────────────────────────────────

    #[test]
    fn test_dtlz6_zero_g() {
        // At x = [0.5]*12, M=3: all distance vars at 0.5 -> x_i^3 = 0.125
        // g = sqrt(10 * 0.125) = sqrt(1.25) ≈ 1.118
        let dtlz6 = DTLZ6::new(12, 3);
        let result = dtlz6.evaluate(&vec![0.5; 12]);
        assert_eq!(result.len(), 3);
        // With non-zero g, theta values change
        assert!(result.iter().all(|&v| v >= 0.0), "DTLZ6 all values should be non-negative");
    }

    // ── DTLZ7 ────────────────────────────────────────────────────

    #[test]
    fn test_dtlz7_return_length() {
        let dtlz7 = DTLZ7::new(22, 3);
        let result = dtlz7.evaluate(&vec![0.5; 22]);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_dtlz7_first_two_linear() {
        // f[0] = x[0], f[1] = x[1]
        let dtlz7 = DTLZ7::new(22, 3);
        let x = vec![0.2, 0.7, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
                     0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
                     0.5, 0.5];
        let result = dtlz7.evaluate(&x);
        assert!((result[0] - 0.2).abs() < EPSILON, "DTLZ7 f1 expected 0.2, got {}", result[0]);
        assert!((result[1] - 0.7).abs() < EPSILON, "DTLZ7 f2 expected 0.7, got {}", result[1]);
    }

    // ── Dimension validation ─────────────────────────────────────

    #[test]
    #[should_panic(expected = "DTLZ2::evaluate called with 11 variables, expected 12")]
    fn test_dtlz_dimension_mismatch() {
        let dtlz2 = DTLZ2::new(12, 3);
        dtlz2.evaluate(&[0.0; 11]);
    }

    #[test]
    fn test_dtlz_bounds_length() {
        let dtlz2 = DTLZ2::new(12, 3);
        assert_eq!(dtlz2.bounds().len(), 12);
    }

    #[test]
    fn test_all_dtlz_bound_01() {
        for name in &["DTLZ1", "DTLZ2", "DTLZ3", "DTLZ4", "DTLZ5", "DTLZ6", "DTLZ7"] {
            let d: Box<dyn BenchmarkFn> = match *name {
                "DTLZ1" => Box::new(DTLZ1::new(10, 3)),
                "DTLZ2" => Box::new(DTLZ2::new(10, 3)),
                "DTLZ3" => Box::new(DTLZ3::new(10, 3)),
                "DTLZ4" => Box::new(DTLZ4::new(10, 3)),
                "DTLZ5" => Box::new(DTLZ5::new(10, 3)),
                "DTLZ6" => Box::new(DTLZ6::new(10, 3)),
                "DTLZ7" => Box::new(DTLZ7::new(10, 3)),
                _ => unreachable!(),
            };
            for (j, &(low, high)) in d.bounds().iter().enumerate() {
                assert!((low - 0.0).abs() < EPSILON,
                    "{} bound[{}].0 expected 0, got {}", name, j, low);
                assert!((high - 1.0).abs() < EPSILON,
                    "{} bound[{}].1 expected 1, got {}", name, j, high);
            }
        }
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn test_dtlz1_serde_roundtrip() {
            let d = DTLZ1::new(7, 3);
            let json = serde_json::to_string(&d).unwrap();
            let d2: DTLZ1 = serde_json::from_str(&json).unwrap();
            assert_eq!(d.n_vars, d2.n_vars);
            assert_eq!(d.n_obj, d2.n_obj);
        }

        #[test]
        fn test_dtlz2_serde_roundtrip() {
            let d = DTLZ2::new(12, 3);
            let json = serde_json::to_string(&d).unwrap();
            let d2: DTLZ2 = serde_json::from_str(&json).unwrap();
            assert_eq!(d.n_vars, d2.n_vars);
            assert_eq!(d.n_obj, d2.n_obj);
        }

        #[test]
        fn test_dtlz4_serde_roundtrip() {
            let d = DTLZ4::new(10, 3);
            let json = serde_json::to_string(&d).unwrap();
            let d2: DTLZ4 = serde_json::from_str(&json).unwrap();
            assert_eq!(d.alpha, d2.alpha);
        }

        #[test]
        fn test_dtlz7_serde_roundtrip() {
            let d = DTLZ7::new(22, 3);
            let json = serde_json::to_string(&d).unwrap();
            let d2: DTLZ7 = serde_json::from_str(&json).unwrap();
            assert_eq!(d.n_vars, d2.n_vars);
            assert_eq!(d.n_obj, d2.n_obj);
        }
    }
}
