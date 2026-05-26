//! Hill-climbing engine.
//!
//! Provides `HillClimbEngine<U>` with `Stochastic` and `SteepestAscent` modes,
//! observer wiring, and `Strategy<U>` implementation for runtime algorithm swapping.

pub mod configuration;
pub mod engine;

pub use configuration::{HillClimbConfiguration, HillClimbMode};
pub use engine::HillClimbEngine;
