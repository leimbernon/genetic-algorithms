//! Built-in gene (genotype) types.
//!
//! This module provides ready-to-use [`GeneT`](crate::traits::GeneT)
//! implementations:
//!
//! - [`Binary`] — a gene that holds a boolean value (`true`/`false`).
//! - [`Range`] — a gene that holds a numeric value within an interval.
//! - [`List`] — a gene that holds a value drawn from a finite set of alleles.
//! - [`UniqueGenotype`] — a gene that holds a value from a shared alphabet; used for permutation chromosomes.
//! - [`MultiRangeGenotype`] — a gene with per-gene independent `(lo, hi)` bounds and mutation rate.

pub mod binary;
pub mod list;
pub mod multi_range;
pub mod range;
pub mod unique;

pub use binary::Binary;
pub use list::List;
pub use multi_range::MultiRangeGenotype;
pub use range::Range;
pub use unique::UniqueGenotype;
