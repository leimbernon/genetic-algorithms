//! Traits — core abstraction contracts for the genetic algorithm framework.
//!
//! Defines the trait interfaces that all chromosomes, genes, operators, and
//! configurations must implement. Every public type in the library derives its
//! behavior from one or more of these trait contracts, enabling full genericity
//! over chromosome and gene types.
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
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`GeneT`] | Minimal gene trait: identity, default, comparison |
//! | [`ChromosomeT`] | Core chromosome trait: DNA access, fitness, age, mutation |
//! | [`ConfigurationT`] | Fluent builder trait for configuring engines |
//! | [`SelectionOperator`] | Trait for custom selection implementations |
//! | [`CrossoverOperator`] | Trait for custom crossover implementations |
//! | [`MutationOperator`] | Trait for custom mutation implementations |
//! | [`SurvivorOperator`] | Trait for custom survivor selection implementations |
//!
//! # When to use
//! Implement these traits when creating custom chromosome types, gene types,
//! or operator strategies. See the [`chromosomes`](crate::chromosomes), [`genotypes`](crate::genotypes), and
//! [`operations`](crate::operations) modules for built-in implementations.

pub mod chromosome;
pub mod common;
pub mod configuration;
pub mod gene;
pub mod operators;

pub use chromosome::ChromosomeT;
pub use common::{initialize_chromosomes, initialize_chromosomes_par, FitnessFn, InitializationFn};
pub use configuration::{
    ConfigurationT, CrossoverConfig, ElitismConfig, ExtensionConfig, LocalSearchConfig,
    MutationConfig, NichingConfig, SelectionConfig, StoppingConfig, SurvivorConfig,
};
pub use gene::GeneT;
pub use operators::{
    CrossoverOperator, ExtensionOperator, LocalSearchOperator, MutationOperator,
    SelectionOperator, SurvivorOperator,
};
