//! `RealGene` trait — continuous-value arithmetic for DE, Scatter, and CMA-ES engines.
//!
//! Differential Evolution, Scatter Search, and CMA-ES all require treating gene
//! values as `f64` numbers so that mutation vectors and covariance updates can be
//! computed via subtraction and scaling.  Any gene type that can expose a `f64`
//! value and create a new instance from a `f64` may implement this trait.
//!
//! # Blanket implementations
//!
//! [`crate::genotypes::Range<f64>`] and [`crate::genotypes::MultiRangeGenotype<f64>`]
//! implement `RealGene` automatically, so existing range chromosomes work with
//! all real-valued engines out of the box.

use crate::genotypes::{MultiRangeGenotype, Range};
use crate::traits::GeneT;

/// Extension of [`GeneT`] that enables real-valued arithmetic.
///
/// Any gene type that can expose a `f64` value and create a new instance from a
/// `f64` may implement this trait. Used by [`DeEngine`](crate::de::DeEngine),
/// [`ScatterEngine`](crate::scatter::ScatterEngine), and
/// [`CmaEngine`](crate::cma::CmaEngine).
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::genotypes::Range;
/// use genetic_algorithms::traits::{GeneT, RealGene};
///
/// let mut gene: Range<f64> = Default::default();
/// gene.set_value(3.14);
/// assert!((gene.real_value() - 3.14).abs() < 1e-10);
/// let shifted = gene.with_real_value(2.71);
/// assert!((shifted.real_value() - 2.71).abs() < 1e-10);
/// ```
pub trait RealGene: GeneT {
    /// Returns the gene's continuous value as `f64`.
    fn real_value(&self) -> f64;

    /// Returns a new gene with the same metadata but a different value.
    fn with_real_value(&self, value: f64) -> Self;

    /// Returns the `(lo, hi)` bounds for this gene if available.
    ///
    /// Used by the PSO engine for velocity initialization and boundary enforcement.
    /// The default implementation returns `None`; gene types backed by an explicit
    /// range table should override this method to expose those bounds.
    fn bounds(&self) -> Option<(f64, f64)> {
        None
    }
}

/// `Range<f64>` genes work with real-valued engines out of the box.
impl RealGene for Range<f64> {
    #[inline]
    fn real_value(&self) -> f64 {
        self.value
    }

    #[inline]
    fn with_real_value(&self, value: f64) -> Self {
        let mut g = self.clone();
        g.value = value;
        g
    }

    /// Returns the first `(lo, hi)` range entry for this gene.
    ///
    /// For genes constructed with multiple range entries, only the first entry is used.
    /// The caller is responsible for constructing such genes with a representative
    /// range in position 0.
    #[inline]
    fn bounds(&self) -> Option<(f64, f64)> {
        self.ranges.first().copied()
    }
}

/// `MultiRangeGenotype<f64>` genes work with real-valued engines out of the box.
impl RealGene for MultiRangeGenotype<f64> {
    #[inline]
    fn real_value(&self) -> f64 {
        self.value
    }

    #[inline]
    fn with_real_value(&self, value: f64) -> Self {
        let mut g = self.clone();
        g.value = value;
        g
    }

    #[inline]
    fn bounds(&self) -> Option<(f64, f64)> {
        Some((self.lo, self.hi))
    }
}
