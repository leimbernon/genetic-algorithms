//! CMA-ES engine. Covariance Matrix Adaptation Evolution Strategy for real-valued black-box continuous optimization.

pub mod configuration;
pub mod engine;

pub use configuration::CmaConfiguration;
pub use engine::{CmaEngine, CmaResult};
