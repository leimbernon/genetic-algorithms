//! List genotype (gene) for finite symbolic alphabets.
//!
//! Each [`List<T>`] gene carries an integer identifier, a finite ordered set of
//! alleles, and the current value (always `alleles[id]`). It is the building
//! block for [`crate::chromosomes::ListChromosome`] chromosomes used in
//! combinatorial and symbolic optimization problems.

use crate::error::GaError;
use crate::traits::GeneT;
use std::fmt;

/// A list gene drawn from a finite set of alleles.
///
/// The `id` field is the index into the `alleles` vector that determines the
/// current `value`. The invariant `value == alleles[id]` is maintained at all
/// times; use [`List::new`] to construct and [`GeneT::set_id`] to mutate.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::genotypes::List;
/// use genetic_algorithms::traits::GeneT;
///
/// let mut gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
/// assert_eq!(gene.id(), 0);
/// assert_eq!(gene.value(), 'a');
/// gene.set_id(2);
/// assert_eq!(gene.value(), 'c');
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct List<T> {
    pub id: i32,
    pub alleles: Vec<T>,
    pub value: T,
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.alleles == other.alleles && self.value == other.value
    }
}

impl<T: Default> Default for List<T> {
    fn default() -> Self {
        Self {
            id: 0,
            alleles: Vec::new(),
            value: T::default(),
        }
    }
}

impl<T: fmt::Debug> fmt::Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:?}", self.id, self.value)
    }
}

impl<T: Clone + Default + Sync + Send> List<T> {
    /// Creates a new `List` gene with the given identifier, allele set, and an
    /// initial value hint.
    ///
    /// The actual stored `value` is always derived from `alleles[id]`; the
    /// `_value` argument is accepted for API symmetry with `Range::new` but
    /// is otherwise ignored.
    ///
    /// # Errors
    ///
    /// Returns [`GaError::ValidationError`] when:
    /// - `alleles` is empty.
    /// - `id` is negative.
    /// - `id` is out of bounds (>= `alleles.len()`).
    pub fn new(id: i32, alleles: Vec<T>, _value: T) -> Result<Self, GaError> {
        if alleles.is_empty() {
            return Err(GaError::ValidationError(
                "Allele set must not be empty".to_string(),
            ));
        }
        if id < 0 {
            return Err(GaError::ValidationError(format!(
                "Gene id must be non-negative, got {}",
                id
            )));
        }
        if (id as usize) >= alleles.len() {
            return Err(GaError::ValidationError(format!(
                "Gene id {} is out of bounds for allele set of length {}",
                id,
                alleles.len()
            )));
        }
        let value = alleles[id as usize].clone();
        Ok(Self { id, alleles, value })
    }

    /// Returns a clone of the current value.
    pub fn value(&self) -> T {
        self.value.clone()
    }
}

impl<T: Clone + Default + Sync + Send> GeneT for List<T> {
    fn id(&self) -> i32 {
        self.id
    }

    /// Sets the gene id and updates the value to `alleles[id]`.
    ///
    /// If `id` is out of bounds or negative the call is silently ignored and
    /// the gene remains unchanged.
    fn set_id(&mut self, id: i32) -> &mut Self {
        if id >= 0 && (id as usize) < self.alleles.len() {
            self.id = id;
            self.value = self.alleles[id as usize].clone();
        } else {
            log::warn!(
                target: "ga_events",
                "List::set_id({}) out of bounds (alleles.len() = {}), ignoring",
                id,
                self.alleles.len()
            );
        }
        self
    }
}
