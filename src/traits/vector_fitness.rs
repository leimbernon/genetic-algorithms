//! Vector fitness trait for multi-case selection and multi-objective optimisation.
//!
//! [`VectorFitness`] is an opt-in supertrait of [`ChromosomeT`] that lets a chromosome
//! carry a `Vec<f64>` of per-case or per-objective scores alongside its scalar fitness.
//!
//! It is used by two independent subsystems:
//!
//! - **Lexicase selection** — each case score measures performance on one test case; the
//!   selection operator iterates cases in a random order, retaining only individuals that
//!   match or beat the best on that case.
//! - **Multi-objective engines** — NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA
//!   store one objective value per slot and read them for dominance ranking,
//!   decomposition, and indicator-based comparison.
//!
//! # Usage
//!
//! Implement this trait alongside [`ChromosomeT`].  Call `set_fitness_values` inside your
//! `calculate_fitness()` implementation to populate the vector before any operator reads it.
//!
//! # No default impl
//!
//! [`VectorFitness`] intentionally provides **no default implementation** for either
//! method.  The reason is a lifetime mismatch: [`ChromosomeT::fitness`] returns `f64` by
//! value — there is no `&f64` inside the chromosome to borrow, so a blanket default for
//! `fitness_values` cannot be derived from the scalar fitness field.  Each implementor
//! must store a dedicated `fitness_values: Vec<f64>` field and provide both methods
//! explicitly.

use crate::traits::ChromosomeT;

/// Opt-in trait enabling [`Selection::Lexicase`](crate::operations::Selection), epsilon-lexicase,
/// and multi-objective engines (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA).
///
/// Implement alongside [`ChromosomeT`]. Call [`set_fitness_values`](VectorFitness::set_fitness_values)
/// inside your `calculate_fitness()` implementation.
pub trait VectorFitness: ChromosomeT {
    /// Returns the per-case or per-objective fitness values set during `calculate_fitness`.
    fn fitness_values(&self) -> &[f64];

    /// Sets the per-case or per-objective fitness values. Called inside `calculate_fitness`.
    fn set_fitness_values(&mut self, values: Vec<f64>);
}
