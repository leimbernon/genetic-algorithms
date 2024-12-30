use crate::fitness::FitnessFnWrapper;
use crate::traits::ChromosomeT;
use crate::genotypes::Binary as BinaryGenotype;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Binary{
    pub dna: Vec<BinaryGenotype>,
    pub fitness: f64,
    pub age: i32,
    pub fitness_fn: FitnessFnWrapper<BinaryGenotype>,
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
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
    fn calculate_fitness(&mut self) {
        self.fitness_fn.call(&self.dna);
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
            fitness_fn: FitnessFnWrapper::default(),
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