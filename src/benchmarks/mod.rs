//! Standard benchmark functions for evaluating optimization algorithms.
//!
//! This module provides implementations of well-known single-objective,
//! bi-objective (ZDT), and many-objective (DTLZ) benchmark functions behind
//! the `benchmarks` feature flag.
//!
//! All functions implement the [`BenchmarkFn`] trait which provides a unified
//! `evaluate(&[f64]) -> Vec<f64>` interface along with metadata.

pub mod single_objective;

pub use single_objective::{Ackley, Rastrigin, Sphere};

/// Shared trait for all benchmark functions.
///
/// Provides a unified interface for evaluating benchmark functions and
/// accessing their metadata (name, bounds, known optimum).
pub trait BenchmarkFn {
    /// Human-readable name of the benchmark function.
    fn name(&self) -> &'static str;

    /// Per-dimension bounds as `(lower, upper)` pairs. Length equals the
    /// number of decision variables expected by `evaluate()`.
    fn bounds(&self) -> &[(f64, f64)];

    /// Known optimum objective vector. For single-objective this is
    /// `vec![f_min]`; for multi-objective it is `vec![f1_min, f2_min, ...]`.
    fn optimum_value(&self) -> Vec<f64>;

    /// Evaluate the benchmark function at point `x`.
    ///
    /// Returns a `Vec<f64>` containing one or more objective values.
    /// Single-objective functions return `vec![f]`; multi-objective functions
    /// return `vec![f1, f2, ..., fM]`.
    ///
    /// # Panics
    ///
    /// Panics if `x.len()` does not match the number of decision variables
    /// expected by the function (as returned by `bounds().len()`).
    fn evaluate(&self, x: &[f64]) -> Vec<f64>;
}
