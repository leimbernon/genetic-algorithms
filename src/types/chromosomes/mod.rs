//! Built-in chromosome types.
//!
//! This module provides ready-to-use chromosome implementations:
//!
//! - [`Binary`] — a chromosome whose DNA is a vector of [`genotypes::Binary`](crate::genotypes::Binary) genes.
//! - [`Range`] — a chromosome whose DNA is a vector of [`genotypes::Range`](crate::genotypes::Range) genes.
//! - [`ListChromosome`] — a chromosome whose DNA is a vector of [`genotypes::List`](crate::genotypes::List) genes.
//! - [`UniqueChromosome`] — a chromosome whose DNA is a permutation of a shared alphabet.
//! - [`MultiRangeChromosome`] — a chromosome with per-gene independent `(lo, hi)` bounds and mutation rates.
//! - [`MultiUniqueChromosome`] — a chromosome with multiple permutation groups.
//! - [`ChromosomeLength`] — enum describing fixed or variable chromosome length. Used by
//!   `Mutation::Insertion` / `Mutation::Deletion` to control length-changing operators.

pub mod binary;
pub mod length;
pub mod list;
pub mod multi_range;
pub mod multi_unique;
mod range;
pub mod unique;

pub use binary::Binary;
pub use length::ChromosomeLength;
pub use list::ListChromosome;
pub use multi_range::MultiRangeChromosome;
pub use multi_unique::MultiUniqueChromosome;
pub use range::Range;
pub use unique::UniqueChromosome;
