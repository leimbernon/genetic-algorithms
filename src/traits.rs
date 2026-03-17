//! Core traits that define the genetic algorithm abstractions.
//!
//! This module re-exports all public traits from its sub-modules:
//!
//! - [`GeneT`] — the gene abstraction (identity, cloning).
//! - [`ChromosomeT`] — the chromosome abstraction (DNA, fitness, age).
//! - [`ConfigurationT`] and its sub-traits — builder-pattern configuration.
//! - Operator traits ([`SelectionOperator`], [`CrossoverOperator`],
//!   [`MutationOperator`], [`SurvivorOperator`]) — for custom operator
//!   implementations.
//! - Helper functions and type aliases ([`FitnessFn`], [`InitializationFn`]).

pub mod chromosome;
pub mod common;
pub mod configuration;
pub mod gene;
pub mod operators;

pub use chromosome::ChromosomeT;
pub use common::{initialize_chromosomes, initialize_chromosomes_par, FitnessFn, InitializationFn};
pub use configuration::{
    ConfigurationT, CrossoverConfig, ElitismConfig, ExtensionConfig, MutationConfig, NichingConfig,
    SelectionConfig, StoppingConfig,
};
pub use gene::GeneT;
pub use operators::{
    CrossoverOperator, ExtensionOperator, MutationOperator, SelectionOperator, SurvivorOperator,
};
