//! Multi-case fitness trait for lexicase selection.

use crate::traits::ChromosomeT;

/// Opt-in trait enabling `Selection::Lexicase` and `Selection::EpsilonLexicase`.
///
/// Implement alongside [`ChromosomeT`]. Call `set_case_fitness` inside your
/// `calculate_fitness()` implementation.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::MultiCaseFitness;
/// use genetic_algorithms::chromosomes::Binary;
///
/// // The Binary chromosome already implements VectorFitness but not MultiCaseFitness.
/// // This example shows how to implement for a custom chromosome:
/// use genetic_algorithms::traits::{ChromosomeT, GeneT};
/// use genetic_algorithms::genotypes::Binary as BinaryGene;
/// use genetic_algorithms::chromosomes::Binary as BinaryChrom;
///
/// // Built-in chromosomes do not implement MultiCaseFitness.
/// // Implement it on a custom type that holds per-case scores:
/// struct MyCaseChromosome { cases: Vec<f64> }
/// ```
pub trait MultiCaseFitness: ChromosomeT {
    /// Returns the per-case fitness scores set during `calculate_fitness`.
    fn case_fitness(&self) -> &[f64];

    /// Sets the per-case fitness scores. Called inside `calculate_fitness`.
    fn set_case_fitness(&mut self, scores: Vec<f64>);
}
