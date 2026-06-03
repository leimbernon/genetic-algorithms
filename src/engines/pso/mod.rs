//! PSO engine. Particle Swarm Optimization for real-valued continuous optimization.

pub mod configuration;
pub mod engine;

pub use configuration::{inertia_weight, PsoConfiguration, PsoInertia, PsoTopology};
pub use engine::{PsoEngine, PsoResult};
