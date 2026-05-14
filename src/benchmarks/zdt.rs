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
        assert_eq!(x.len(), self.n_vars,
            "ZDT1::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
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
        assert_eq!(x.len(), self.n_vars,
            "ZDT2::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
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
        assert_eq!(x.len(), self.n_vars,
            "ZDT3::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
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
        assert_eq!(x.len(), self.n_vars,
            "ZDT4::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
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
        assert_eq!(x.len(), self.n_vars,
            "ZDT5::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
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
        assert_eq!(x.len(), self.n_vars,
            "ZDT6::evaluate called with {} variables, expected {}",
            x.len(), self.n_vars);
        let f1 = 1.0 - (-4.0 * x[0]).exp() * (6.0 * std::f64::consts::PI * x[0]).sin().powi(6);
        let g = 1.0 + 9.0 * (x[1..].iter().sum::<f64>() / (self.n_vars - 1) as f64).powf(0.25);
        let f2 = g * (1.0 - (f1 / g).powi(2));
        vec![f1, f2]
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-12;

    // ── ZDT1 ────────────────────────────────────────────────────

    #[test]
    fn test_zdt1_optimum() {
        // At x = [0, 0, ..., 0]: f1 = 0, g = 1, f2 = 1 * (1 - 0) = 1
        let zdt1 = ZDT1::default();
        let result = zdt1.evaluate(&vec![0.0; 30]);
        assert!((result[0] - 0.0).abs() < EPSILON, "ZDT1 f1 expected 0, got {}", result[0]);
        assert!((result[1] - 1.0).abs() < EPSILON, "ZDT1 f2 expected 1, got {}", result[1]);
    }

    // ── ZDT2 ────────────────────────────────────────────────────

    #[test]
    fn test_zdt2_optimum() {
        let zdt2 = ZDT2::default();
        let result = zdt2.evaluate(&vec![0.0; 30]);
        assert!((result[0] - 0.0).abs() < EPSILON, "ZDT2 f1 expected 0, got {}", result[0]);
        assert!((result[1] - 1.0).abs() < EPSILON, "ZDT2 f2 expected 1, got {}", result[1]);
    }

    // ── ZDT3 ────────────────────────────────────────────────────

    #[test]
    fn test_zdt3_optimum() {
        let zdt3 = ZDT3::default();
        let result = zdt3.evaluate(&vec![0.0; 30]);
        // At x_0 = 0: sin(10*pi*0) = 0
        assert!((result[0] - 0.0).abs() < EPSILON, "ZDT3 f1 expected 0, got {}", result[0]);
        assert!((result[1] - 1.0).abs() < EPSILON, "ZDT3 f2 expected 1, got {}", result[1]);
    }

    // ── ZDT4 ────────────────────────────────────────────────────

    #[test]
    fn test_zdt4_optimum() {
        // At x = [0, 0, ..., 0]: g = 1 + 10*9 + sum(0 - 10*cos(0))
        // = 1 + 90 + 9*(-10) = 91 - 90 = 1. f1 = 0, f2 = 1*(1-0) = 1
        let zdt4 = ZDT4::default();
        let result = zdt4.evaluate(&vec![0.0; 10]);
        assert!((result[0] - 0.0).abs() < EPSILON, "ZDT4 f1 expected 0, got {}", result[0]);
        assert!((result[1] - 1.0).abs() < EPSILON, "ZDT4 f2 expected 1, got {}", result[1]);
    }

    // ── ZDT5 ────────────────────────────────────────────────────

    #[test]
    fn test_zdt5_optimum() {
        // At x = [0, ..., 0]: z_0=1, z_i=1, u=1, v=10, g=11, f1=2, f2=11/2=5.5
        let zdt5 = ZDT5::default();
        let result = zdt5.evaluate(&vec![0.0; 11]);
        assert!((result[0] - 2.0).abs() < EPSILON, "ZDT5 f1 expected 2, got {}", result[0]);
        assert!((result[1] - 5.5).abs() < EPSILON, "ZDT5 f2 expected 5.5, got {}", result[1]);
    }

    // ── ZDT6 ────────────────────────────────────────────────────

    #[test]
    fn test_zdt6_optimum() {
        // At x = [0, ..., 0]:
        // f1 = 1 - exp(0) * sin^6(0) = 1 - 1 * 0 = 1
        // g = 1 + 9 * (0/9)^0.25 = 1 + 0 = 1
        // f2 = 1 * (1 - (1/1)^2) = 0
        let zdt6 = ZDT6::default();
        let result = zdt6.evaluate(&vec![0.0; 10]);
        assert!((result[0] - 1.0).abs() < EPSILON, "ZDT6 f1 expected 1, got {}", result[0]);
        assert!((result[1] - 0.0).abs() < EPSILON, "ZDT6 f2 expected 0, got {}", result[1]);
    }

    // ── Dimension mismatch ──────────────────────────────────────

    #[test]
    #[should_panic(expected = "ZDT1::evaluate called with 2 variables, expected 3")]
    fn test_zdt_dimension_mismatch() {
        let zdt1 = ZDT1::new(3);
        zdt1.evaluate(&[0.0, 0.0]);
    }

    // ── Bounds ──────────────────────────────────────────────────

    #[test]
    fn test_zdt4_bounds_length() {
        let zdt4 = ZDT4::new(10);
        assert_eq!(zdt4.bounds().len(), 10);
    }

    #[test]
    fn test_zdt4_bounds_mixed() {
        let zdt4 = ZDT4::new(10);
        assert_eq!(zdt4.bounds()[0], (0.0, 1.0));
        assert_eq!(zdt4.bounds()[1], (-5.0, 5.0));
        assert_eq!(zdt4.bounds()[9], (-5.0, 5.0));
    }

    #[test]
    fn test_zdt_zdt5_default_dimension() {
        let zdt5 = ZDT5::default();
        assert_eq!(zdt5.n_vars, 11);
        assert_eq!(zdt5.bounds().len(), 11);
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn test_zdt1_serde_roundtrip() {
            let z = ZDT1::new(30);
            let json = serde_json::to_string(&z).unwrap();
            let z2: ZDT1 = serde_json::from_str(&json).unwrap();
            assert_eq!(z.n_vars, z2.n_vars);
        }

        #[test]
        fn test_zdt4_serde_roundtrip() {
            let z = ZDT4::new(10);
            let json = serde_json::to_string(&z).unwrap();
            let z2: ZDT4 = serde_json::from_str(&json).unwrap();
            assert_eq!(z.bounds()[0], (0.0, 1.0));
            assert_eq!(z2.bounds()[0], z.bounds()[0]);
        }

        #[test]
        fn test_zdt6_serde_roundtrip() {
            let z = ZDT6::new(10);
            let json = serde_json::to_string(&z).unwrap();
            let z2: ZDT6 = serde_json::from_str(&json).unwrap();
            assert_eq!(z.n_vars, z2.n_vars);
        }
    }
}
