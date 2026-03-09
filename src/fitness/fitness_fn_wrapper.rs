use crate::traits::GeneT;
use std::sync::Arc;

/// Type alias for the fitness function trait object.
type FitnessFnTrait<G> = dyn Fn(&[G]) -> f64 + Send + Sync;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genotypes::Binary as BinaryGene;
    use crate::traits::GeneT;

    fn make_gene(id: i32) -> BinaryGene {
        let mut g = <BinaryGene as Default>::default();
        g.set_id(id);
        g
    }

    #[test]
    fn new_and_call() {
        let wrapper = FitnessFnWrapper::new(|genes: &[BinaryGene]| genes.len() as f64 * 2.0);
        let dna = vec![make_gene(1), make_gene(2), make_gene(3)];
        assert_eq!(wrapper.call(&dna), 6.0);
    }

    #[test]
    fn default_returns_zero() {
        let wrapper = FitnessFnWrapper::<BinaryGene>::default();
        let dna = vec![make_gene(1), make_gene(2)];
        assert_eq!(wrapper.call(&dna), 0.0);
    }

    #[test]
    fn default_returns_zero_empty_dna() {
        let wrapper = FitnessFnWrapper::<BinaryGene>::default();
        assert_eq!(wrapper.call(&[]), 0.0);
    }

    #[test]
    fn clone_shares_same_arc() {
        let original = FitnessFnWrapper::new(|genes: &[BinaryGene]| genes.len() as f64);
        let cloned = original.clone();
        // PartialEq uses Arc::ptr_eq, so clones should be equal
        assert_eq!(original, cloned);
    }

    #[test]
    fn clone_produces_same_results() {
        let original = FitnessFnWrapper::new(|genes: &[BinaryGene]| {
            genes.iter().map(|g| g.id() as f64).sum::<f64>()
        });
        let cloned = original.clone();
        let dna = vec![make_gene(10), make_gene(20)];
        assert_eq!(original.call(&dna), cloned.call(&dna));
    }

    #[test]
    fn partial_eq_same_arc_is_true() {
        let a = FitnessFnWrapper::new(|_: &[BinaryGene]| 1.0);
        let b = a.clone();
        assert!(a == b);
    }

    #[test]
    fn partial_eq_different_arcs_is_false() {
        let a = FitnessFnWrapper::new(|_: &[BinaryGene]| 1.0);
        let b = FitnessFnWrapper::new(|_: &[BinaryGene]| 1.0);
        // Different Arcs, even if closures do the same thing
        assert!(a != b);
    }

    #[test]
    fn debug_format() {
        let wrapper = FitnessFnWrapper::new(|_: &[BinaryGene]| 0.0);
        let debug = format!("{:?}", wrapper);
        assert_eq!(debug, "<function>");
    }
}
