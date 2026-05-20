//! Built-in chromosome types.
//!
//! This module provides ready-to-use chromosome implementations:
//!
//! - [`Binary`] — a chromosome whose DNA is a vector of [`genotypes::Binary`](crate::genotypes::Binary) genes.
//! - [`Range`] — a chromosome whose DNA is a vector of [`genotypes::Range`](crate::genotypes::Range) genes.
//! - [`ListChromosome`] — a chromosome whose DNA is a vector of [`genotypes::List`](crate::genotypes::List) genes.
//! - [`ChromosomeLength`] — enum describing fixed or variable chromosome length.

pub mod binary;
pub mod length;
pub mod list;
mod range;

pub use binary::Binary;
pub use length::ChromosomeLength;
pub use list::ListChromosome;
pub use range::Range;
