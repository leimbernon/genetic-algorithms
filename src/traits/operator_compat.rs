//! Operator compatibility trait for build-time operator validation.
//!
//! `OperatorCompat` is an opt-in trait that lets a chromosome type declare
//! which crossover and mutation operators are valid for it. `Ga::build()`
//! checks these restrictions before the first generation runs, returning
//! `GaError::ConfigurationError` immediately if an incompatible operator is
//! configured.
//!
//! # Semantics
//!
//! - Both methods return `None` by default (no restriction — any operator is
//!   allowed). Override them to restrict the valid set.
//! - Return `Some(&[Crossover::Pmx, ...])` / `Some(&[Mutation::Swap, ...])` to
//!   declare the allowed operators. A static slice keeps this zero-allocation.
//! - `Ga::build()` compares the configured operator against the valid set and
//!   fails fast with `GaError::ConfigurationError` if there is a mismatch.
//!
//! # Example
//!
//! ```rust,ignore
//! use genetic_algorithms::operations::{Crossover, Mutation};
//! use genetic_algorithms::traits::OperatorCompat;
//!
//! impl OperatorCompat for MyChromosome {
//!     fn valid_crossovers() -> Option<&'static [Crossover]> {
//!         Some(&[Crossover::Pmx, Crossover::Order])
//!     }
//!     fn valid_mutations() -> Option<&'static [Mutation]> {
//!         Some(&[Mutation::Swap, Mutation::Inversion])
//!     }
//! }
//! ```
//!
//! # Design note — no blanket impl
//!
//! This trait intentionally has NO blanket implementation over all `LinearChromosome`
//! types. Such a blanket impl would prevent concrete restriction impls for permutation
//! chromosome types like `UniqueChromosome<T>` on Rust stable (no specialization).
//! Instead, each chromosome type that needs the `OperatorCompat` bound satisfied must
//! add an explicit impl — either empty `{}` (inherits `None` defaults) or with
//! a restricted valid set.

use crate::operations::{Crossover, Mutation};

/// Opt-in operator-compatibility trait for build-time validation.
///
/// Implement this trait on a chromosome type to declare which crossover and
/// mutation operators are valid for it. `Ga::build()` checks these restrictions
/// before any generation runs.
///
/// Both methods default to `None` (no restriction). Override them to restrict
/// the valid operator set to a `Some(&'static [...])`  slice.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::{Crossover, Mutation};
/// use genetic_algorithms::traits::OperatorCompat;
/// use genetic_algorithms::chromosomes::UniqueChromosome;
///
/// // Permutation chromosomes restrict to permutation-preserving operators:
/// // (UniqueChromosome already does this — shown here for illustration)
/// struct MyPermutationChromosome;
/// impl OperatorCompat for MyPermutationChromosome {
///     fn valid_crossovers() -> Option<&'static [Crossover]> {
///         Some(&[Crossover::Pmx, Crossover::Order, Crossover::EdgeRecombination])
///     }
///     fn valid_mutations() -> Option<&'static [Mutation]> {
///         Some(&[Mutation::Swap, Mutation::Inversion, Mutation::Scramble])
///     }
/// }
/// ```
pub trait OperatorCompat {
    /// Returns the set of valid crossover operators for this chromosome type.
    ///
    /// - `None` — no restriction; any crossover is accepted (default).
    /// - `Some(&[...])` — only the listed crossovers are valid; any other
    ///   crossover configured in `Ga::build()` returns `GaError::ConfigurationError`.
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        None
    }

    /// Returns the set of valid mutation operators for this chromosome type.
    ///
    /// - `None` — no restriction; any mutation is accepted (default).
    /// - `Some(&[...])` — only the listed mutations are valid; any other
    ///   mutation configured in `Ga::build()` returns `GaError::ConfigurationError`.
    fn valid_mutations() -> Option<&'static [Mutation]> {
        None
    }
}
