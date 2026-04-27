//! Cellular Genetic Algorithm engine.
//!
//! Provides a cGA implementation where individuals are placed on a 2D toroidal
//! grid and evolve through local neighbourhood interactions.

pub mod configuration;
pub mod engine;

pub use configuration::{CellularConfiguration, Neighborhood, UpdateMode};
pub use engine::{CellularEngine, CellularResult};
