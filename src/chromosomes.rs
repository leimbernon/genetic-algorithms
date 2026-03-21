//! Built-in chromosome types.
//!
//! This module provides ready-to-use chromosome implementations:
//!
//! - [`Binary`] — a chromosome whose DNA is a vector of [`genotypes::Binary`](crate::genotypes::Binary) genes.
//! - [`Range`] — a chromosome whose DNA is a vector of [`genotypes::Range`](crate::genotypes::Range) genes.
//! - [`ListChromosome`] — a chromosome whose DNA is a vector of [`genotypes::List`](crate::genotypes::List) genes.

pub mod binary;
mod range;
pub mod list;

pub use binary::Binary;
pub use range::Range;
pub use list::ListChromosome;
