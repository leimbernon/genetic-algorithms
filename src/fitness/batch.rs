//! Batch fitness evaluation trait for evaluating entire populations at once.
//!
//! [`BatchFitnessEvaluator`] is an alternative to the standard per-chromosome
//! fitness function. It allows the engine to pass a full slice of chromosomes
//! to a single evaluation call, which is useful for:
//!
//! - **Vectorised computation** (e.g. GPU shaders, SIMD, matrix math)
//! - **External simulators** that amortise setup cost over a batch
//! - **Multi-threaded external calls** where the user manages parallelism
//!
//! # Contract
//!
//! Implementors **must** return a `Vec<f64>` whose length equals
//! `chromosomes.len()`, with fitness values in the same positional order as
//! the input slice. Violating this contract will cause a panic in the engine
//! (enforced by `debug_assert_eq!` in Wave 2/3 caller).

use crate::traits::ChromosomeT;

/// Trait for evaluating an entire population of chromosomes in a single call.
///
/// # Required implementation
///
/// ```text
/// fn evaluate_batch(&self, chromosomes: &[U]) -> Vec<f64>
/// ```
///
/// **Positional contract:** `result[i]` must be the fitness of `chromosomes[i]`.
/// The returned `Vec` must have the same length as the input slice.
///
/// # Thread safety
///
/// Implementors must be `Send + Sync` so that the evaluator can be stored in
/// an `Arc<dyn BatchFitnessEvaluator<U>>` and shared across engine threads.
///
/// # Example
///
/// ```rust,no_run
/// // no_run: stub implementation — not a runnable example
/// use genetic_algorithms::BatchFitnessEvaluator;
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// struct MyBatchEval;
///
/// impl BatchFitnessEvaluator<RangeChromosome<f64>> for MyBatchEval {
///     fn evaluate_batch(&self, chromosomes: &[RangeChromosome<f64>]) -> Vec<f64> {
///         // chromosomes.iter().map(|c| c.fitness).collect()
///         todo!()
///     }
/// }
/// ```
pub trait BatchFitnessEvaluator<U: ChromosomeT>: Send + Sync {
    /// Evaluates the fitness of every chromosome in `chromosomes`.
    ///
    /// Returns a `Vec<f64>` of the same length as `chromosomes`, where element
    /// `i` is the fitness of `chromosomes[i]`.
    ///
    /// # Panics (caller-side)
    ///
    /// The engine asserts `result.len() == chromosomes.len()` with
    /// `debug_assert_eq!`. In debug builds a length mismatch panics
    /// immediately; in release builds it leads to silent data corruption.
    /// Always return a correctly sized Vec.
    fn evaluate_batch(&self, chromosomes: &[U]) -> Vec<f64>;
}
