//! Shared multi-objective optimization primitives.
//!
//! This module hosts the building blocks shared by the NSGA-II, NSGA-III,
//! and future MOEA engines: non-dominated sorting, Pareto individual/front
//! types, dominance predicates, and the `ObjectiveFn<G>` type alias.
//!
//! NSGA-II re-exports these symbols via `pub use crate::multi_objective::*`
//! for full backward compatibility — existing user code that uses paths like
//! `genetic_algorithms::nsga2::pareto::ParetoIndividual` continues to work.

pub mod non_dominated_sort;
pub mod pareto;

/// Type alias for a single objective function shared across multi-objective engines.
pub type ObjectiveFn<G> = dyn Fn(&[G]) -> f64 + Send + Sync;
