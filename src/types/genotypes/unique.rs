//! Unique genotype (gene) for permutation-encoded chromosomes.
//!
//! Each [`UniqueGenotype<T>`] gene carries an integer identifier and a value
//! of type `T` drawn from a shared alphabet. It is the building block for
//! [`crate::chromosomes::UniqueChromosome`] chromosomes used in combinatorial
//! optimization problems where the chromosome must be a permutation of the
//! alphabet (no duplicate genes, all elements present).
//!
//! The alphabet lives on the chromosome — not per gene — keeping each gene
//! lightweight.

use crate::traits::GeneT;
use std::fmt;

/// A permutation gene with an identifier and a value.
///
/// This struct implements the [`GeneT`] trait, allowing it to be used in
/// genetic algorithms. The `id` field uniquely identifies the gene within the
/// chromosome, while the `value` field holds the gene's current value from
/// the chromosome's shared alphabet.
///
/// **No per-gene alphabet**: the alphabet lives on [`UniqueChromosome<T>`](crate::chromosomes::UniqueChromosome),
/// not on individual genes. This keeps genes lightweight and makes cloning O(1).
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::UniqueGenotype;
/// use genetic_algorithms::traits::GeneT;
///
/// let mut gene = UniqueGenotype::new(0, 42i32);
/// gene.set_id(1);
/// assert_eq!(gene.id(), 1);
/// assert_eq!(gene.value(), 42);
/// ```
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct UniqueGenotype<T> {
    pub id: i32,
    pub value: T,
}

impl<T: Default> Default for UniqueGenotype<T> {
    fn default() -> Self {
        Self {
            id: 0,
            value: Default::default(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for UniqueGenotype<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.id, self.value)
    }
}

impl<T: Clone + Default + Sync + Send> GeneT for UniqueGenotype<T> {
    fn id(&self) -> i32 {
        self.id
    }

    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

impl<T: Clone + Default> UniqueGenotype<T> {
    /// Creates a new `UniqueGenotype` with the given identifier and value.
    ///
    /// # Arguments
    ///
    /// * `id` - An integer representing the unique identifier for this gene.
    /// * `value` - The gene's value, drawn from the chromosome's alphabet.
    ///
    /// # Returns
    ///
    /// A new `UniqueGenotype<T>`.
    pub fn new(id: i32, value: T) -> Self {
        Self { id, value }
    }

    /// Returns a clone of the gene's value.
    pub fn value(&self) -> T {
        self.value.clone()
    }

    /// Sets the gene's value.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value to assign.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for method chaining.
    pub fn set_value(&mut self, value: T) -> &mut Self {
        self.value = value;
        self
    }
}
