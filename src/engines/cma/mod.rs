//! CMA-ES engine. Covariance Matrix Adaptation Evolution Strategy for real-valued black-box continuous optimization.

pub mod configuration;
pub mod engine;
pub mod restart;

pub use configuration::CmaConfiguration;
pub use engine::{CmaEngine, CmaResult};
pub use restart::{RestartEvent, RestartKind, RestartStrategy};
