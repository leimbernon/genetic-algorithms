//! Multi-range genotype (gene) for chromosomes with per-gene independent bounds and mutation rates.
//!
//! Each [`MultiRangeGenotype<T>`] gene carries its own `(lo, hi)` bounds and
//! `mutation_rate`, enabling heterogeneous real-valued optimization where each
//! gene occupies a different search space. This is the building block for
//! [`crate::chromosomes::MultiRangeChromosome`].
//!
//! Unlike [`crate::genotypes::Range`], bounds and mutation rates live directly
//! on each gene as flat fields (no `Arc` indirection) — decision D-08.

use crate::traits::GeneT;
use std::fmt;

/// A multi-range gene with per-gene independent bounds and mutation rate.
///
/// Each gene carries its own `(lo, hi)` bounds and `mutation_rate`, enabling
/// heterogeneous real-valued search spaces where each dimension has different
/// bounds and step sizes.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::MultiRangeGenotype;
/// use genetic_algorithms::traits::GeneT;
///
/// let mut gene = MultiRangeGenotype::new(0, -5.0_f64, 5.0, 0.0, 0.1);
/// assert_eq!(gene.id(), 0);
/// assert_eq!(gene.lo, -5.0);
/// assert_eq!(gene.hi, 5.0);
/// assert_eq!(gene.mutation_rate, 0.1);
/// gene.set_id(3);
/// assert_eq!(gene.id(), 3);
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
pub struct MultiRangeGenotype<T> {
    /// Integer identifier for this gene position.
    pub id: i32,
    /// Lower bound (inclusive) for this gene's value.
    pub lo: T,
    /// Upper bound (exclusive) for this gene's value.
    pub hi: T,
    /// Current value of this gene.
    pub value: T,
    /// Per-gene mutation rate used by Gaussian mutation (replaces global sigma).
    pub mutation_rate: f64,
}

impl<T: fmt::Display> fmt::Display for MultiRangeGenotype<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.id, self.value)
    }
}

impl<T: Default> Default for MultiRangeGenotype<T> {
    fn default() -> Self {
        Self {
            id: 0,
            lo: Default::default(),
            hi: Default::default(),
            value: Default::default(),
            mutation_rate: 0.0,
        }
    }
}

impl<T: Sync + Send + Copy + Default> GeneT for MultiRangeGenotype<T> {
    fn id(&self) -> i32 {
        self.id
    }

    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

impl<T: Copy + Default> MultiRangeGenotype<T> {
    /// Creates a new `MultiRangeGenotype` with explicit per-gene bounds and mutation rate.
    ///
    /// # Arguments
    ///
    /// * `id` - The gene's integer identifier.
    /// * `lo` - Lower bound for this gene's value (inclusive).
    /// * `hi` - Upper bound for this gene's value (exclusive). Must satisfy `lo < hi`.
    /// * `value` - Initial value for this gene (should be in `[lo, hi)`).
    /// * `mutation_rate` - Per-gene mutation rate used by Gaussian mutation.
    pub fn new(id: i32, lo: T, hi: T, value: T, mutation_rate: f64) -> Self {
        Self {
            id,
            lo,
            hi,
            value,
            mutation_rate,
        }
    }

    /// Returns the current value of this gene (Copy semantics).
    pub fn value(&self) -> T {
        self.value
    }

    /// Sets the value of this gene and returns a mutable reference to `self`.
    pub fn set_value(&mut self, value: T) -> &mut Self {
        self.value = value;
        self
    }
}
