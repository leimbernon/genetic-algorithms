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
//! - [`GpCrossover`] — tree subtree crossover operator (Wave 1)
//! - [`GpMutation`] — tree mutation operators: SubtreeMutation, PointMutation,
//!   HoistMutation (Wave 1)
//! - [`ramped_half_and_half`] — population initializer (Wave 2)
//! - [`GpGa`] / [`GpResult`] — full GP engine (Wave 2)

pub mod chromosome;
pub mod configuration;
pub mod crossover;
pub mod engine;
pub mod init;
pub mod mutation;
pub mod node;
pub mod primitives;

pub use chromosome::{GpChromosome, GpGene, TreeChromosome};
pub use configuration::GpConfiguration;
pub use crossover::GpCrossover;
pub use engine::{GpGa, GpResult};
pub use init::ramped_half_and_half;
pub use mutation::GpMutation;
pub use node::{GpNode, Node};
pub use primitives::{BoolNode, MathNode};
