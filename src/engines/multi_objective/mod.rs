//! Shared multi-objective optimisation primitives.
//!
//! ## Description
//!
//! This module hosts the building blocks shared by all multi-objective engines
//! (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA): non-dominated sorting,
//! Pareto individual and front types, dominance predicates, quality indicators,
//! and the `ObjectiveFn<G>` type alias.
//!
//! Every multi-objective engine re-exports these symbols for backward
//! compatibility — existing user code continues to work via paths like
//! `genetic_algorithms::nsga2::pareto::ParetoIndividual`.
//!
//! ## Sub-modules
//!
//! - **`non_dominated_sort`** — Fast non-dominated sorting
//!   ([`Deb et al. 2002`](crate::nsga2::Nsga2Ga)),
//!   direction-aware sorting, and constrained-sort variant. Provides
//!   `assign_ranks` and `non_dominated_sort_with_directions`.
//!
//! - **`pareto`** — [`ParetoIndividual<U>`](pareto::ParetoIndividual) wrapper
//!   type (chromosome + objective vector + rank + crowding distance +
//!   constraint violation), [`ParetoFront<U>`](pareto::ParetoFront) container,
//!   and dominance predicates (`dominates_with_directions`).
//!
//! - **`indicators`** — Quality indicators for Pareto front evaluation:
//!   - [`hypervolume`](indicators::hypervolume) — Dominated hypervolume
//!     (S-metric, Lebesgue measure).
//!   - [`generational_distance`](indicators::generational_distance) — GD,
//!     convergence metric.
//!   - [`inverted_generational_distance`](indicators::inverted_generational_distance)
//!     — IGD, convergence + diversity metric.
//!   - [`spread`](indicators::spread) — Spread (Δ), diversity metric.
//!
//! ## Key Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`ObjectiveDirection`] | Minimize or Maximize per objective. |
//! | [`ObjectiveFn<G>`] | Closure signature `fn(&[G]) -> f64` for one objective. |
//! | [`ParetoIndividual<U>`](pareto::ParetoIndividual) | Chromosome + objective vector + rank + crowd + violation. |
//! | [`ParetoFront<U>`](pareto::ParetoFront) | Ordered collection of non-dominated individuals. |
//!
//! NSGA-II re-exports these symbols via `pub use crate::multi_objective::*`
//! for full backward compatibility — existing user code that uses paths like
//! `genetic_algorithms::nsga2::pareto::ParetoIndividual` continues to work.

pub mod non_dominated_sort;
pub mod pareto;
pub mod indicators;

/// Direction of optimization for a single objective.
///
/// This is the canonical definition. Both `nsga2::configuration` and
/// `nsga3::configuration` re-export this type for backward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectiveDirection {
    /// Minimize this objective (lower is better).
    Minimize,
    /// Maximize this objective (higher is better).
    Maximize,
}

/// Type alias for a single objective function shared across multi-objective engines.
pub type ObjectiveFn<G> = dyn Fn(&[G]) -> f64 + Send + Sync;
