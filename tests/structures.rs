use genetic_algorithms::traits::{GeneT, ChromosomeT};
use std::{fmt::Debug, sync::Arc};
use std::fmt;

//Structures definition
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Gene{
    pub id: i32,
}
impl GeneT for Gene{
    fn get_id(&self) -> i32{
        self.id
    }
    fn set_id(&mut self, id: i32)->&mut Self {
        self.id = id;
        self
    }
}

pub struct Chromosome{
    pub dna: Vec<Gene>,
    pub fitness: f64,
    pub age: i32,
    pub fitness_fn: Arc<dyn Fn(&[Gene]) -> f64 + Send + Sync>,
}

impl Default for Chromosome {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            fitness: 0.0,
            age: 0,
            fitness_fn: Arc::new(|_| 0.0),
        }
    }
}

impl Debug for Chromosome{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Binary")
            .field("dna", &self.dna)
            .field("fitness", &self.fitness)
            .field("age", &self.age)
            // Custom message for the function since functions cannot be printed
            .field("fitness_fn", &"<function>")
            .finish()
    }
}

impl Clone for Chromosome{
    fn clone(&self) -> Self {
        Self {
            dna: self.dna.clone(),
            fitness: self.fitness,
            age: self.age,
            // Clone the Arc, which increments the reference count
            fitness_fn: Arc::clone(&self.fitness_fn),
        }
    }
}

impl PartialEq for Chromosome {
    fn eq(&self, other: &Self) -> bool {
        self.dna == other.dna
            && self.fitness == other.fitness
            && self.age == other.age
    }
}

impl ChromosomeT for Chromosome{
    type Gene = Gene;
    fn get_dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn get_fitness(&self) -> f64 {
        self.fitness
    }
    fn set_fitness(&mut self, fitness: f64)->&mut Self {
        self.fitness = fitness;
        self
    }
    fn set_age(&mut self, age:i32)->&mut Self{
        self.age = age;
        self
    }
    fn get_age(&self) -> i32 {
        self.age
    }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = Arc::new(fitness_fn);
        self
    }
    fn calculate_fitness(&mut self) {
        
        self.fitness = 0.0;

        for (i, gene) in self.dna.iter().enumerate() {
            let fitness = f64::from(gene.get_id()*i as i32);
            self.fitness += fitness;
        }
    }
    fn set_dna(&mut self, dna: &[Self::Gene])->&mut Self{
        self.dna = dna.to_vec();
        self
    }
}