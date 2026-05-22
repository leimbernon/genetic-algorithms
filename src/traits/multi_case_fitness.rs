//! Multi-case fitness trait for lexicase selection.

use crate::traits::ChromosomeT;

/// Opt-in trait enabling [`Selection::Lexicase`] and [`Selection::EpsilonLexicase`].
///
/// Implement alongside [`ChromosomeT`]. Call `set_case_fitness` inside your
/// `calculate_fitness()` implementation.
pub trait MultiCaseFitness: ChromosomeT {
    /// Returns the per-case fitness scores set during `calculate_fitness`.
    fn case_fitness(&self) -> &[f64];

    /// Sets the per-case fitness scores. Called inside `calculate_fitness`.
    fn set_case_fitness(&mut self, scores: Vec<f64>);
}
