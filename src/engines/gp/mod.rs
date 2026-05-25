//! Genetic Programming (GP) subsystem.
//!
//! This module provides the core types and traits for tree-based Genetic
//! Programming:
//!
//! - [`GpNode`] — implement this on your primitive set enum
//! - [`Node<N>`] — recursive expression tree stored inside [`GpChromosome<N>`]
//! - [`GpChromosome<N>`] — library-provided concrete tree chromosome
//! - [`TreeChromosome`] — supertrait of `ChromosomeT` for tree chromosomes
//! - [`GpConfiguration`] — engine parameter shell (extended in Wave 2)
//! - [`MathNode`] / [`BoolNode`] — built-in primitive sets
//!
//! The `GpGa` engine, GP operators (crossover, mutation), and initializers are
//! added in Waves 1–3.

pub mod chromosome;
pub mod configuration;
pub mod node;
pub mod primitives;

pub use chromosome::{GpChromosome, GpGene, TreeChromosome};
pub use configuration::GpConfiguration;
pub use node::{GpNode, Node};
pub use primitives::{BoolNode, MathNode};
