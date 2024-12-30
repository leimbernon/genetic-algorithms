use std::fmt;
use std::sync::Arc;
use crate::traits::ChromosomeT;
use crate::genotypes::Binary as BinaryGenotype;

pub struct Binary{
    pub dna: Vec<BinaryGenotype>,
    pub fitness: f64,
    pub age: i32,
    pub fitness_fn: Arc<dyn Fn(&[BinaryGenotype]) -> f64 + Send + Sync>,
}

// Implement Default for Binary
impl Default for Binary {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            fitness: 0.0,
            age: 0,
            fitness_fn: Arc::new(|_| 0.0),
        }
    }
}

// Implement Clone for Binary
impl Clone for Binary {
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

// Implement Debug for Binary
impl fmt::Debug for Binary {
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

// Implement PartialEq for Binary
impl PartialEq for Binary {
    fn eq(&self, other: &Self) -> bool {
        self.dna == other.dna
            && self.fitness == other.fitness
            && self.age == other.age
    }
}

impl ChromosomeT for Binary{
    type Gene = BinaryGenotype;

    fn get_dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn set_dna(&mut self, dna: &[Self::Gene]) -> &mut Self {
        self.dna = dna.to_vec();
        self
    }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[BinaryGenotype]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = Arc::new(fitness_fn);
        self
    }
    fn calculate_fitness(&mut self) {
        self.fitness = (self.fitness_fn)(&self.dna);
    }
    fn get_fitness(&self) -> f64 {
        self.fitness
    }
    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }
    fn set_age(&mut self, age: i32) -> &mut Self {
        self.age = age;
        self
    }
    fn get_age(&self) -> i32 {
        self.age
    }
}
impl Binary {

    pub fn new() -> Self {
        Self {
            dna: Vec::new(),
            fitness: 0.0,
            age: 0,
            fitness_fn: Arc::new(|_| 0.0),
        }
    }

    pub fn phenotype(&self) -> String {
        self.dna
            .iter()
            .map(|gene| if gene.value { '1' } else { '0' })
            .collect()
    }

    pub fn dna_from_string(&mut self, s: &str) {
        let mut dna = Vec::with_capacity(s.len());

        for (index, char) in s.chars().enumerate() {
            match char {
                '1' => dna.push(BinaryGenotype { id: index as i32, value: true }),
                '0' => dna.push(BinaryGenotype { id: index as i32, value: false }),
                _ => {
                    panic!("Invalid character '{}' at position {}; only '1' and '0' are allowed",
                           char, index);
                }
            }
        }

        self.dna = dna;
    }
}