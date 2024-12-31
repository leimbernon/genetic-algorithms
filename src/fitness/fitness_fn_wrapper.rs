use std::sync::Arc;
use crate::traits::GeneT;

pub struct FitnessFnWrapper<G: GeneT>(Arc<dyn Fn(&[G]) -> f64 + Send + Sync>);

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
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(&[G]) -> f64 + Send + Sync + 'static,
    {
        FitnessFnWrapper(Arc::new(func))
    }

    pub fn call(&self, dna: &[G]) -> f64 {
        (self.0)(dna)
    }
}