//! Backward-compatible re-export of the shared Pareto types and dominance predicates.
//!
//! The implementation moved to [`crate::multi_objective::pareto`] in v2.4.0;
//! this module re-exports every public symbol so that
//! `genetic_algorithms::nsga2::pareto::*` paths continue to work.

pub use crate::multi_objective::pareto::*;
