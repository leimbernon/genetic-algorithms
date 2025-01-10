use log::{debug, trace};
use crate::configuration::ProblemSolving;
use crate::traits::GeneT;

pub trait ChromosomeT: Clone + Default{

    type Gene: GeneT;

    fn new() -> Self{
        Default::default()
    }
    fn default(mut self) -> Self{
        self.set_fitness(0.0);
        self.set_age(0);
        self.set_dna(Vec::new().as_slice());
        self
    }
    fn new_gene() -> Self::Gene{
        Self::Gene::new()
    }
    fn get_dna(&self) -> &[Self::Gene];
    fn set_dna(&mut self, dna: &[Self::Gene])->&mut Self;
    fn set_gene(&mut self, gene_index: usize, gene: Self::Gene)->&mut Self{
        let mut dna_temp = self.get_dna().to_vec();
        dna_temp[gene_index] = gene;
        self.set_dna(dna_temp.as_slice());
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

    fn get_best_chromosome(&self, external_chromosome: &Self, problem_solving: ProblemSolving) -> Self
    {
        debug!(target="chromosome_events", method="get_best_chromosome"; "Started the best chromosome method");
        trace!(target="chromosome_events", method="get_best_chromosome"; "Self chromosome fitness: {} - Chromosome 2 fitness: {}", self.get_fitness(), external_chromosome.get_fitness());

        let is_self_better = match problem_solving {
            ProblemSolving::Maximization => self.get_fitness() >= external_chromosome.get_fitness(),
            ProblemSolving::Minimization => self.get_fitness() < external_chromosome.get_fitness(),
            _ => self.get_fitness() >= external_chromosome.get_fitness(),
        };

        let best_chromosome = if is_self_better {
            self.clone()
        } else {
            external_chromosome.clone()
        };

        debug!(target="chromosome_events", method="get_best_chromosome"; "Best chromosome method finished");

        best_chromosome
    }
}