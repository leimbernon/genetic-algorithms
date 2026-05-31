//! RealValued marker trait for compile-time enforcement on multi-parent crossover.
//!
//! Implementors of this trait gain access to `factory_multi_parent` dispatch for
//! `Crossover::Undx`, `Crossover::Spx`, and `Crossover::Pcx`.
//!
//! Binary and permutation chromosomes must **not** implement `RealValued` — doing so
//! would bypass the compile-time guard that restricts multi-parent crossover to
//! real-valued search spaces.

use crate::traits::LinearChromosome;

/// Marker trait for real-valued (continuous or integer-range) chromosomes.
///
/// Implementing `RealValued` signals that a chromosome's genes occupy a
/// continuous or bounded numeric space, making it eligible for multi-parent
/// crossover operators:
///
/// - [`Crossover::Undx`](crate::operations::Crossover::Undx) — Unimodal Normal Distribution Crossover
/// - [`Crossover::Spx`](crate::operations::Crossover::Spx) — Simplex Crossover
/// - [`Crossover::Pcx`](crate::operations::Crossover::Pcx) — Parent-Centric Crossover
///
/// # Who should implement this trait?
///
/// Implement `RealValued` on any chromosome whose genes carry numeric values
/// that support arithmetic operations (e.g., addition, subtraction, scalar
/// multiplication). The built-in [`Range<T>`](crate::chromosomes::Range) and
/// [`MultiRangeChromosome<T>`](crate::chromosomes::MultiRangeChromosome) types
/// implement this trait.
///
/// Binary chromosomes (`Binary`) and permutation chromosomes (`Unique<T>`)
/// must **not** implement `RealValued`.
///
/// # Example
///
/// ```rust,no_run
/// use genetic_algorithms::traits::{LinearChromosome, RealValued};
/// # use genetic_algorithms::traits::{ChromosomeT, GeneT};
/// # use std::borrow::Cow;
///
/// // Custom real-valued chromosome implementing RealValued
/// # #[derive(Clone, Default)]
/// # struct MyGene;
/// # impl GeneT for MyGene {
/// #     fn id(&self) -> i32 { 0 }
/// #     fn set_id(&mut self, _: i32) -> &mut Self { self }
/// # }
/// # #[derive(Clone, Default)]
/// # struct MyChromosome;
/// # impl ChromosomeT for MyChromosome {
/// #     type Gene = MyGene;
/// #     fn fitness(&self) -> f64 { 0.0 }
/// #     fn set_fitness(&mut self, _: f64) -> &mut Self { self }
/// #     fn age(&self) -> usize { 0 }
/// #     fn set_age(&mut self, _: usize) -> &mut Self { self }
/// #     fn calculate_fitness(&mut self) {}
/// # }
/// # impl LinearChromosome for MyChromosome {
/// #     fn dna(&self) -> &[MyGene] { &[] }
/// #     fn dna_mut(&mut self) -> &mut [MyGene] { &mut [] }
/// #     fn set_dna<'a>(&mut self, _: Cow<'a, [MyGene]>) -> &mut Self { self }
/// #     fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self where F: Fn(&[MyGene]) -> f64 + Send + Sync + 'static { self }
/// # }
/// impl RealValued for MyChromosome {}
/// ```
pub trait RealValued: LinearChromosome {}
