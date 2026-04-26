//! Differential Evolution engine.
//!
//! Provides a complete DE implementation with 5 mutation strategies, 2
//! crossover modes, and JADE / L-SHADE adaptive parameter control.

pub mod configuration;
pub mod crossover;
pub mod engine;
pub mod gene;
pub mod mutation;

pub use configuration::{DeAdaptive, DeConfiguration, DeCrossoverMode, DeMutationStrategy};
pub use engine::{DeEngine, DeResult};
pub use gene::DeGene;
