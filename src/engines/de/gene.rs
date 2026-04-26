//! `DeGene` trait — continuous-value extension for DE arithmetic.
//!
//! Differential Evolution requires treating gene values as `f64` numbers so
//! that mutation vectors can be computed via subtraction and scaling.  Any
//! gene type that can expose a `f64` value and create a new instance from a
//! `f64` may implement this trait.
//!
//! # Blanket implementation
//!
//! [`crate::genotypes::Range<f64>`] implements `DeGene` automatically, so
//! existing range chromosomes work with the DE engine out of the box.

use crate::traits::GeneT;
use crate::genotypes::Range;

/// Extension of [`GeneT`] that enables Differential Evolution arithmetic.
///
/// Implementors must be able to:
/// 1. expose their continuous value as `f64` via [`de_value`](DeGene::de_value).
/// 2. produce a copy of themselves with a new value via
///    [`with_de_value`](DeGene::with_de_value).
///
/// Out-of-bounds clamping after mutation is the caller's responsibility; this
/// trait makes no assumption about valid ranges.
pub trait DeGene: GeneT {
    /// Returns the gene's continuous value as `f64`.
    fn de_value(&self) -> f64;

    /// Returns a new gene with the same metadata but a different value.
    fn with_de_value(&self, value: f64) -> Self;
}

/// `Range<f64>` genes work with DE out of the box.
impl DeGene for Range<f64> {
    #[inline]
    fn de_value(&self) -> f64 {
        self.value
    }

    #[inline]
    fn with_de_value(&self, value: f64) -> Self {
        let mut g = self.clone();
        g.value = value;
        g
    }
}
