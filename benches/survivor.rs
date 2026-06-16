use genetic_algorithms::fitness::FitnessFnWrapper;
use rand::Rng;
use std::borrow::Cow;

use genetic_algorithms::operations::survivor::age::age_based;
use genetic_algorithms::operations::survivor::fitness::fitness_based;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};

#[derive(Debug, Clone, Default, PartialEq)]
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
struct SimpleChromosome {
    dna: Vec<Gene>,
    pub fitness: f64,
    pub age: usize,
    pub fitness_fn: FitnessFnWrapper<Gene>,
}
impl ChromosomeT for SimpleChromosome {
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
        self.fitness = 0.0;
    }
}
impl LinearChromosome for SimpleChromosome {
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
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}

const POPULATION_SIZE: usize = 1000;

#[cfg(not(tarpaulin_include))]
fn setup_population(population_size: usize, gene_length: usize) -> Vec<SimpleChromosome> {
    let mut rng = rand::rng();
    (0..population_size)
        .map(|_| SimpleChromosome {
            fitness: rng.random_range(0.0..1.0),
            dna: (0..gene_length)
                .map(|_| Gene {
                    id: rng.random_range(0..255),
                })
                .collect(),
            age: rng.random_range(0..100),
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect()
}

mod survivor_methods {
    use super::*;

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn age_survivor(bencher: divan::Bencher, gene_length: usize) {
        let chromosomes = setup_population(POPULATION_SIZE, gene_length);
        bencher
            .with_inputs(|| chromosomes.clone())
            .bench_values(|mut chromosomes| age_based(&mut chromosomes, POPULATION_SIZE));
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn fitness_survivor(bencher: divan::Bencher, gene_length: usize) {
        let chromosomes = setup_population(POPULATION_SIZE, gene_length);
        bencher
            .with_inputs(|| chromosomes.clone())
            .bench_values(|mut chromosomes| {
                let limit_configuration =
                    genetic_algorithms::configuration::LimitConfiguration::default();
                fitness_based(&mut chromosomes, POPULATION_SIZE, limit_configuration);
            });
    }
}

fn main() {
    divan::main();
}
