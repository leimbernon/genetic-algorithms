use genetic_algorithms::fitness::FitnessFnWrapper;
use genetic_algorithms::operations::mutation::ValueMutable;
use genetic_algorithms::traits::{ChromosomeT, GeneT};
use std::borrow::Cow;

//Structures definition
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Gene {
    pub id: i32,
}
impl GeneT for Gene {
    fn id(&self) -> i32 {
        self.id
    }
    fn set_id(&mut self, id: i32) -> &mut Self {
        self.id = id;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chromosome {
    pub dna: Vec<Gene>,
    pub fitness: f64,
    pub age: usize,
    pub fitness_fn: FitnessFnWrapper<Gene>,
}

impl ChromosomeT for Chromosome {
    type Gene = Gene;
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }
    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }
    fn age(&self) -> usize {
        self.age
    }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
    fn calculate_fitness(&mut self) {
        self.fitness = 0.0;

        for (i, gene) in self.dna.iter().enumerate() {
            let fitness = f64::from(gene.id() * i as i32);
            self.fitness += fitness;
        }
    }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        self
    }
}
impl ValueMutable for Chromosome {}
