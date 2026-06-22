use genetic_algorithms::fitness::FitnessFnWrapper;
use genetic_algorithms::operations::mutation::ValueMutable;
use genetic_algorithms::traits::{
    ChromosomeT, GeneT, LinearChromosome, OperatorCompat, VectorFitness,
};
use std::borrow::Cow;

//Structures definition
#[derive(Debug, Copy, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chromosome {
    pub dna: Vec<Gene>,
    pub fitness: f64,
    pub age: usize,
    pub fitness_values: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<Gene>,
}

impl ChromosomeT for Chromosome {
    type Gene = Gene;

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

    fn calculate_fitness(&mut self) {
        let sum: f64 = self
            .dna
            .iter()
            .enumerate()
            .map(|(i, gene)| f64::from(gene.id() * i as i32))
            .sum();
        self.fitness = sum;
        self.fitness_values = vec![sum, -sum];
    }
}

impl LinearChromosome for Chromosome {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }

    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        self
    }

    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}

impl VectorFitness for Chromosome {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl ValueMutable for Chromosome {}

impl OperatorCompat for Chromosome {}

impl genetic_algorithms::traits::RealValuedMutation for Chromosome {}

/// Test fixture for lexicase selection and multi-objective engines.
/// Extends `Chromosome` with per-case / per-objective fitness scores via VectorFitness.
#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MultiCaseChromosome {
    pub dna: Vec<Gene>,
    pub fitness: f64,
    pub age: usize,
    pub fitness_values: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<Gene>,
}

impl ChromosomeT for MultiCaseChromosome {
    type Gene = Gene;

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

    fn calculate_fitness(&mut self) {
        // Populate fitness_values from gene ids (each gene contributes one case score).
        let scores: Vec<f64> = self.dna.iter().map(|g| f64::from(g.id())).collect();
        let mean = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
        self.fitness_values = scores;
        self.fitness = mean;
    }
}

impl LinearChromosome for MultiCaseChromosome {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }

    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        self
    }

    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}

impl VectorFitness for MultiCaseChromosome {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl ValueMutable for MultiCaseChromosome {}

impl OperatorCompat for MultiCaseChromosome {}

impl genetic_algorithms::traits::RealValuedMutation for MultiCaseChromosome {}
