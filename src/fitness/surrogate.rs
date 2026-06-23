//! Surrogate-assisted fitness prescreening for offspring evaluation.
//!
//! A [`SurrogateModel`] is a cheap approximation of the true fitness function.
//! Before offspring are passed to the (expensive) true evaluator, the engine
//! uses the surrogate to predict fitness and retains only the most promising
//! fraction. This reduces the number of true fitness evaluations per generation
//! without changing the population size or selection pressure.
//!
//! # Prescreening contract (D-04, D-08)
//!
//! 1. The surrogate runs **before** [`FitnessCache`] and [`BatchFitnessEvaluator`]
//!    in the engine's evaluation pipeline. Only offspring that survive
//!    prescreening are passed to the true evaluator.
//! 2. Rejected offspring are **dropped permanently**. They are never evaluated
//!    with the true fitness function and never inserted into the population.
//! 3. The surviving offspring fraction is controlled by
//!    `GaConfiguration::surrogate_fraction` (Plan 02). The engine always retains
//!    at least `max(1, floor(n * fraction))` offspring.
//!
//! # Tie-breaking
//!
//! When multiple offspring have the same predicted score, the engine sorts them
//! using an **unstable sort**. The relative order of equal-scored offspring is
//! not guaranteed. Users must not rely on tie-breaking order for determinism
//! across platforms or compiler versions.
//!
//! [`FitnessCache`]: crate::fitness::FitnessCache
//! [`BatchFitnessEvaluator`]: crate::fitness::BatchFitnessEvaluator

use crate::traits::ChromosomeT;

/// Surrogate model for cheap offspring prescreening before true fitness evaluation.
///
/// Implementors provide a single `predict` method that returns an approximate
/// fitness score for a chromosome. The engine uses these scores to rank offspring
/// and discard the least-promising fraction before calling the (expensive) true
/// fitness function.
///
/// # Required implementation
///
/// ```text
/// fn predict(&self, chromosome: &U) -> f64
/// ```
///
/// The returned value is a **relative score** used only for ordering: higher
/// values are treated as better (maximization-consistent). The engine normalises
/// nothing — if your true fitness is minimization-based, return the negated
/// approximation so that better chromosomes still score higher.
///
/// # Thread safety
///
/// Implementors must be `Send + Sync` so that the model can be stored in an
/// `Arc<dyn SurrogateModel<U>>` and accessed across rayon threads.
///
/// # Tie-breaking
///
/// The prescreening sort is unstable. Equal-scored offspring are retained or
/// rejected in an unspecified order. Do not rely on tie-breaking for
/// deterministic behaviour across platforms or compiler versions.
///
/// # Example
///
/// ```rust,no_run
/// // no_run: stub implementation — not a runnable example
/// use genetic_algorithms::SurrogateModel;
/// use genetic_algorithms::chromosomes::Range as RangeChromosome;
///
/// struct LinearSurrogate;
///
/// impl SurrogateModel<RangeChromosome<f64>> for LinearSurrogate {
///     fn predict(&self, _chromosome: &RangeChromosome<f64>) -> f64 {
///         // Cheap approximation: sum of gene values
///         // chromosome.dna().iter().map(|g| g.value()).sum()
///         todo!()
///     }
/// }
/// ```
pub trait SurrogateModel<U: ChromosomeT>: Send + Sync {
    /// Returns a predicted fitness score for `chromosome`.
    ///
    /// Higher scores are treated as better (maximization-consistent). The score
    /// is used only for ranking offspring during prescreening — it is not stored
    /// on the chromosome or visible to user callbacks.
    ///
    /// If this method returns `f64::NAN`, the engine treats it as the worst
    /// possible score (`f64::NEG_INFINITY` after substitution) so that NaN
    /// offspring are prescreened out first.
    fn predict(&self, chromosome: &U) -> f64;
}
