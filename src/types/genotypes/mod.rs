//! Built-in gene (genotype) types.
//!
//! This module provides ready-to-use [`GeneT`](crate::traits::GeneT)
//! implementations:
//!
//! - [`Binary`] — a gene that holds a boolean value (`true`/`false`).
//! - [`Range`] — a gene that holds a numeric value within an interval.
//! - [`List`] — a gene that holds a value drawn from a finite set of alleles.

pub mod binary;
pub mod list;
pub mod range;

pub use binary::Binary;
pub use list::List;
pub use range::Range;
