//! Fitness — fitness function helpers and wrappers.
//!
//! Provides built-in fitness functions and the [`FitnessFnWrapper`] for
//! deferred fitness evaluation. Supports both single-objective (f64) and
//! multi-objective (`Vec<f64>`) fitness evaluation.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`count_true()`] | Simple fitness function counting `true` genes (OneMax) |
//! | [`FitnessFnWrapper`] | Deferred evaluation wrapper storing `Arc<dyn Fn>` |
//! | [`FitnessCache`] | Cached fitness evaluation for repeated lookups |
//!
//! # When to use
//! Use [`count_true()`] for binary OneMax-style problems. Use
//! [`FitnessFnWrapper`] when you need to store a fitness function alongside
//! a chromosome for deferred or batched evaluation.

pub mod cache;
pub mod count_true;
pub mod fitness_fn_wrapper;

pub use cache::FitnessCache;
pub use count_true::count_true;
pub use fitness_fn_wrapper::FitnessFnWrapper;
