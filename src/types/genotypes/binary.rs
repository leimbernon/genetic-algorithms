//! Binary genotype (gene) for binary-encoded chromosomes.
//!
//! Each [`Binary`] gene carries an integer identifier and a boolean value.
//! It is the building block for [`crate::chromosomes::Binary`] chromosomes
//! used in problems with binary / bit-string representations.

/// A binary gene with an identifier and a boolean value.
///
/// This struct implements the `GeneT` trait, allowing it to be used in genetic
/// algorithms. The `id` field uniquely identifies the gene, while the `value` field
/// represents its binary state.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::Binary;
/// use genetic_algorithms::traits::GeneT;
///
/// let mut gene = <Binary as Default>::default();
/// gene.set_id(1);
/// gene.set_value(true);
/// assert_eq!(gene.id(), 1);
/// assert_eq!(gene.value(), true);
/// ```
///
/// The binary gene can be used in mutation and crossover operations to
/// evolve populations in a genetic algorithm.
use crate::traits::GeneT;
use std::fmt;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A binary gene type with an identifier and a boolean value.
///
/// Represents a single binary allele. Each gene has a unique `id` for
/// positional tracking and a `value` that determines the allele state.
/// Commonly used for feature selection, knapsack, and onemax problems.
pub struct Binary {
    /// Unique identifier for this gene, used for positional tracking and comparison.
    pub id: i32,
    /// The boolean state of the gene (`true` = 1, `false` = 0).
    pub value: bool,
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.id, if self.value { '1' } else { '0' })
    }
}

impl GeneT for Binary {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

impl Binary {
    /// Creates a new `Binary` gene with the given identifier and value.
    ///
    /// # Arguments
    ///
    /// * `id` - An integer representing the unique identifier.
    /// * `value` - A boolean representing the binary state.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self`.
    pub fn new(&mut self, id: i32, value: bool) -> &mut Self {
        self.id = id;
        self.value = value;
        self
    }

    /// Returns the binary value of the gene.
    pub fn value(&self) -> bool {
        self.value
    }

    /// Sets the binary value of the gene.
    ///
    /// # Arguments
    ///
    /// * `value` - A boolean representing the new binary state.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self`.
    pub fn set_value(&mut self, value: bool) -> &mut Self {
        self.value = value;
        self
    }
}
