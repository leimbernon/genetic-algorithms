//! Age-Layered Population Structure (ALPS) engine.
//!
//! Provides an ALPS implementation with 3 age schemes, cross-layer mating, and
//! periodic injection of fresh individuals into the youngest layer.

pub mod configuration;
pub mod engine;

pub use configuration::{AlpsAgeScheme, AlpsConfiguration};
pub use engine::{AlpsEngine, AlpsResult};
