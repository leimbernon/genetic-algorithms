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
    /// `_value` argument is accepted for API symmetry with [`Range::new`] but
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── List::new ────────────────────────────────────────────────────────────

    #[test]
    fn list_gene_new_valid_id_zero() {
        let gene = List::new(0, vec!['a', 'b', 'c'], 'z').unwrap();
        assert_eq!(gene.id(), 0);
        assert_eq!(gene.value(), 'a'); // value derived from alleles[0], not passed 'z'
        assert_eq!(gene.alleles, vec!['a', 'b', 'c']);
    }

    #[test]
    fn list_gene_new_valid_id_nonzero() {
        let gene = List::new(2, vec!['a', 'b', 'c'], 'a').unwrap();
        assert_eq!(gene.id(), 2);
        assert_eq!(gene.value(), 'c'); // alleles[2]
    }

    #[test]
    fn list_gene_new_id_out_of_bounds() {
        let result = List::new(3, vec!['a', 'b', 'c'], 'a');
        assert!(matches!(result, Err(GaError::ValidationError(_))));
    }

    #[test]
    fn list_gene_new_negative_id() {
        let result = List::new(-1, vec!['a', 'b', 'c'], 'a');
        assert!(matches!(result, Err(GaError::ValidationError(_))));
    }

    #[test]
    fn list_gene_new_empty_alleles() {
        let result = List::new(0, vec![], 'a');
        match result {
            Err(GaError::ValidationError(msg)) => {
                assert!(msg.contains("empty"), "message was: {}", msg)
            }
            _ => panic!("expected ValidationError for empty alleles"),
        }
    }

    // ── GeneT::id ────────────────────────────────────────────────────────────

    #[test]
    fn list_gene_id_returns_stored_id() {
        let gene = List::new(1, vec!['x', 'y', 'z'], 'x').unwrap();
        assert_eq!(gene.id(), 1);
    }

    // ── GeneT::set_id ────────────────────────────────────────────────────────

    #[test]
    fn list_gene_set_id_updates_value() {
        let mut gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
        gene.set_id(1);
        assert_eq!(gene.id(), 1);
        assert_eq!(gene.value(), 'b');
    }

    #[test]
    fn list_gene_set_id_out_of_bounds_ignored() {
        let mut gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
        gene.set_id(99); // out of bounds — should be silently ignored
        assert_eq!(gene.id(), 0);
        assert_eq!(gene.value(), 'a');
    }

    // ── Default ──────────────────────────────────────────────────────────────

    #[test]
    fn list_gene_default() {
        let gene: List<char> = Default::default();
        assert_eq!(gene.id, 0);
        assert!(gene.alleles.is_empty());
        assert_eq!(gene.value, char::default());
    }

    // ── Clone ────────────────────────────────────────────────────────────────

    #[test]
    fn list_gene_clone_is_independent() {
        let gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
        let mut clone = gene.clone();
        clone.alleles.push('d');
        assert_eq!(gene.alleles.len(), 3); // original unchanged
    }

    // ── serde (feature-gated) ────────────────────────────────────────────────

    #[cfg(feature = "serde")]
    #[test]
    fn list_gene_serde_roundtrip() {
        let gene = List::new(1, vec!['a', 'b', 'c'], 'a').unwrap();
        let json = serde_json::to_string(&gene).expect("serialize");
        let restored: List<char> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.id, 1);
        assert_eq!(restored.alleles, vec!['a', 'b', 'c']);
        assert_eq!(restored.value, 'b');
    }
}
