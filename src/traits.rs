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
//! - [`ChromosomeT`] — the minimal chromosome evaluation contract (fitness, age).
//! - [`LinearChromosome`] — the flat-slice chromosome contract (DNA, set_gene, reset).
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
//! | [`ChromosomeT`] | Minimal evaluation contract: fitness, age, calculate_fitness |
//! | [`LinearChromosome`] | Flat-slice contract: dna, set_dna, set_fitness_fn, set_gene, reset |
//! | [`ConfigurationT`] | Fluent builder trait for configuring engines |
//! | [`SelectionOperator`] | Trait for custom selection implementations |
//! | [`CrossoverOperator`] | Trait for custom crossover implementations |
//! | [`MutationOperator`] | Trait for custom mutation implementations |
//! | [`SurvivorOperator`] | Trait for custom survivor selection implementations |
//! | [`Strategy`] | Common interface for runtime algorithm swapping via `Box<dyn Strategy<U>>` |
//!
//! # When to use
//! Implement these traits when creating custom chromosome types, gene types,
//! or operator strategies. See the [`chromosomes`](crate::chromosomes), [`genotypes`](crate::genotypes), and
//! [`operations`](crate::operations) modules for built-in implementations.

pub mod chromosome;
pub mod common;
pub mod configuration;
pub mod gene;
pub mod group_aware;
pub mod linear_chromosome;
pub mod vector_fitness;
pub mod operator_compat;
pub mod operators;
pub mod real_valued;
pub mod self_adaptive;
pub mod strategy;
pub use strategy::Strategy;

pub use chromosome::ChromosomeT;
pub use common::{initialize_chromosomes, initialize_chromosomes_par, FitnessFn, InitializationFn};
pub use linear_chromosome::LinearChromosome;
pub use vector_fitness::VectorFitness;
pub use real_valued::RealValued;
pub use self_adaptive::SelfAdaptive;
pub use configuration::{
    ConfigurationT, CrossoverConfig, ElitismConfig, ExtensionConfig, LocalSearchConfig,
    MutationConfig, NichingConfig, SelectionConfig, StoppingConfig, SurvivorConfig,
};
pub use gene::GeneT;
pub use group_aware::GroupAware;
pub use operator_compat::OperatorCompat;
pub use operators::{
    CrossoverOperator, ExtensionOperator, LocalSearchOperator, MutationOperator, SelectionOperator,
    SurvivorOperator,
};
