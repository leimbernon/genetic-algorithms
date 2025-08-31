use crate::traits::GeneT;
use std::borrow::Cow;

pub trait ChromosomeT: Clone + Default + Send + Sync + 'static{

    type Gene: GeneT;

    fn new() -> Self{
        Default::default()
    }
    fn default(mut self) -> Self{
        self.set_fitness(0.0);
        self.set_age(0);
        self.set_dna(Cow::Borrowed(&[]));
        self
    }
    fn new_gene() -> Self::Gene{
        Self::Gene::new()
    }
    fn get_dna(&self) -> &[Self::Gene];

    /// Sets the DNA using Cow to avoid unnecessary copies.
    /// - Borrowed: stores a cloned Vec.
    /// - Owned: moves the Vec into the chromosome.
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self;

    fn set_gene(&mut self, gene_index: usize, gene: Self::Gene)->&mut Self{
        let mut dna_temp = self.get_dna().to_vec();
        dna_temp[gene_index] = gene;
        // Move the vector to avoid an extra clone
        self.set_dna(Cow::Owned(dna_temp));
        self
    }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static;
    fn calculate_fitness(&mut self);
    fn get_fitness(&self) -> f64;
    fn set_fitness(&mut self, fitness: f64)->&mut Self;
    fn set_age(&mut self, age: i32)->&mut Self;
    fn get_age(&self) -> i32;

    fn get_fitness_distance(&self, fitness_target: &f64) -> f64 {
        (fitness_target - self.get_fitness()).abs()
    }
}