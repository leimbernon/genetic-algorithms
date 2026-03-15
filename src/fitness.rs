//! Fitness-function helpers and wrappers.
//!
//! - [`count_true()`] — a simple fitness function that counts the number of `true`
//!   genes in a binary chromosome (useful for OneMax-style problems).
//! - [`FitnessFnWrapper`] — a wrapper that stores an `Arc<dyn Fn>` fitness
//!   function alongside a chromosome, enabling deferred evaluation.

pub mod count_true;
pub mod fitness_fn_wrapper;

pub use count_true::count_true;
pub use fitness_fn_wrapper::FitnessFnWrapper;
