//! Chromosome length type.
//!
//! Defines [`ChromosomeLength`], a standalone public enum that describes whether
//! a chromosome has a fixed number of genes or a variable-length range. Used by
//! [`LimitConfiguration`](crate::configuration::LimitConfiguration) and exposed
//! as a first-class public type from the crate root.

/// Specifies how many genes a chromosome contains.
///
/// - [`Fixed(n)`](ChromosomeLength::Fixed) — every chromosome always has exactly `n` genes.
/// - [`Variable { min, max }`](ChromosomeLength::Variable) — chromosome length may vary between
///   `min` and `max` genes (inclusive). Used by variable-length representations introduced in
///   later phases.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::ChromosomeLength;
///
/// let fixed = ChromosomeLength::Fixed(10);
/// let variable = ChromosomeLength::Variable { min: 5, max: 20 };
///
/// assert_eq!(fixed, ChromosomeLength::Fixed(10));
/// assert_eq!(variable, ChromosomeLength::Variable { min: 5, max: 20 });
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChromosomeLength {
    /// Every chromosome contains exactly this many genes.
    Fixed(usize),
    /// Chromosome length varies between `min` and `max` genes (inclusive).
    Variable { min: usize, max: usize },
}

impl Default for ChromosomeLength {
    /// Returns [`ChromosomeLength::Fixed(0)`](ChromosomeLength::Fixed), making
    /// `ChromosomeLength` embeddable in `Default`-deriving configuration structs.
    fn default() -> Self {
        ChromosomeLength::Fixed(0)
    }
}
