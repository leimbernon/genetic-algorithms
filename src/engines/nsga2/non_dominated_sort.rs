//! Backward-compatible re-export of the shared non-dominated sorting utilities.
//!
//! The implementation moved to [`crate::multi_objective::non_dominated_sort`]
//! in v2.4.0; this module re-exports every public symbol so that
//! `genetic_algorithms::nsga2::non_dominated_sort::*` paths continue to work.

pub use crate::multi_objective::non_dominated_sort::*;
