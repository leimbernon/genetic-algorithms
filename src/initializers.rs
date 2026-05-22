//! Population initialization functions.
//!
//! These functions create the initial DNA for each chromosome before the
//! first generation. The GA accepts any function with the signature
//! `Fn(usize, Option<&[Gene]>, Option<bool>) -> Vec<Gene>`, but this module
//! provides convenient defaults:
//!
//! - **binary** — random `true`/`false` genes.
//! - **range** — random numeric genes within allele bounds.
//! - **generic** — random selection from a provided alleles list, with or
//!   without repetition.
//! - **list** — random allele-index genes from a finite allele set.

pub mod binary_initializer;
pub mod generic_initializer;
pub mod list_initializer;
pub mod range_initializer;
pub mod unique_initializer;

pub use binary_initializer::*;
pub use generic_initializer::*;
pub use list_initializer::*;
pub use range_initializer::*;
pub use unique_initializer::*;
