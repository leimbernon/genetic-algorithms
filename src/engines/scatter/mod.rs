//! Scatter Search engine.
//!
//! Maintains a small reference set of high-quality and diverse solutions;
//! generates new candidates by linear combination and optional local search.

pub mod configuration;
pub mod engine;

pub use configuration::ScatterConfiguration;
pub use engine::{ScatterEngine, ScatterResult};
