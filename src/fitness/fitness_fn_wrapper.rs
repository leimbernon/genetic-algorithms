//! Thread-safe wrapper around a fitness function closure.
//!
//! [`FitnessFnWrapper`] stores an `Arc<dyn Fn(&[G]) -> f64>` so that the
//! same fitness function can be shared across chromosomes and threads
//! without requiring the user to manage `Arc` manually.

use crate::traits::GeneT;
use std::sync::Arc;

/// Type alias for the fitness function trait object.
type FitnessFnTrait<G> = dyn Fn(&[G]) -> f64 + Send + Sync;

/// A cloneable, thread-safe wrapper around a fitness evaluation closure.
///
/// Two wrappers are considered equal (`PartialEq`) only if they point to the
/// same `Arc` allocation (i.e., one was cloned from the other).
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::fitness::fitness_fn_wrapper::FitnessFnWrapper;
/// use genetic_algorithms::genotypes::Binary as BinaryGene;
///
/// let wrapper = FitnessFnWrapper::<BinaryGene>::new(|dna| dna.iter().filter(|g| g.value).count() as f64);
/// let dna = vec![BinaryGene { id: 0, value: true }, BinaryGene { id: 1, value: false }];
/// assert_eq!(wrapper.call(&dna), 1.0);
/// ```
pub struct FitnessFnWrapper<G: GeneT>(Arc<FitnessFnTrait<G>>);

impl<G: GeneT> Clone for FitnessFnWrapper<G> {
    fn clone(&self) -> Self {
        FitnessFnWrapper(Arc::clone(&self.0))
    }
}

impl<G: GeneT> Default for FitnessFnWrapper<G> {
    fn default() -> Self {
        FitnessFnWrapper(Arc::new(|_| 0.0))
    }
}

impl<G: GeneT> std::fmt::Debug for FitnessFnWrapper<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function>")
    }
}

impl<G: GeneT> PartialEq for FitnessFnWrapper<G> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<G: GeneT> FitnessFnWrapper<G> {
    /// Creates a new wrapper from any closure that maps a DNA slice to a fitness score.
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(&[G]) -> f64 + Send + Sync + 'static,
    {
        FitnessFnWrapper(Arc::new(func))
    }

    /// Evaluates the wrapped fitness function on the given DNA.
    #[inline]
    pub fn call(&self, dna: &[G]) -> f64 {
        (self.0)(dna)
    }
}
