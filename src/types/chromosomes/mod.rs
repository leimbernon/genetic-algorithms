//! Built-in chromosome types.
//!
//! This module provides ready-to-use chromosome implementations:
//!
//! - [`Binary`] — a chromosome whose DNA is a vector of [`genotypes::Binary`](crate::genotypes::Binary) genes.
//! - [`Range`] — a chromosome whose DNA is a vector of [`genotypes::Range`](crate::genotypes::Range) genes.
//! - [`ListChromosome`] — a chromosome whose DNA is a vector of [`genotypes::List`](crate::genotypes::List) genes.
//! - [`ChromosomeLength`] — configuration enum for fixed vs variable-length chromosomes.

pub mod binary;
pub mod list;
mod range;

pub use binary::Binary;
pub use list::ListChromosome;
pub use range::Range;

/// Specifies whether a chromosome's length is fixed or variable.
///
/// Used with `Mutation::Insertion` and `Mutation::Deletion` to control
/// length-changing operators. Variable-length chromosomes allow the GA to
/// evolve both the content and structure of solutions.
///
/// # Variants
///
/// - `Fixed(usize)` — chromosome length is constant; `Insertion` and `Deletion`
///   return `GaError::MutationError` when this variant is active.
/// - `Variable { min, max }` — chromosome length may change between `min` and
///   `max` (inclusive). `Insertion` grows by one gene (clamped to `max`);
///   `Deletion` shrinks by one gene (clamped to `min`).
///
/// # Example
///
/// ```rust
/// use genetic_algorithms::chromosomes::ChromosomeLength;
///
/// let fixed = ChromosomeLength::Fixed(10);
/// let variable = ChromosomeLength::Variable { min: 2, max: 20 };
///
/// assert!(matches!(fixed, ChromosomeLength::Fixed(10)));
/// assert!(matches!(variable, ChromosomeLength::Variable { min: 2, max: 20 }));
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChromosomeLength {
    /// Chromosome length is fixed at the given value.
    /// `Mutation::Insertion` and `Mutation::Deletion` return an error for this variant.
    Fixed(usize),
    /// Chromosome length may vary between `min` and `max` (inclusive).
    Variable {
        /// Minimum allowed chromosome length (inclusive lower bound for Deletion).
        min: usize,
        /// Maximum allowed chromosome length (inclusive upper bound for Insertion).
        max: usize,
    },
}
