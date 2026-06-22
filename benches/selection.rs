use genetic_algorithms::fitness::FitnessFnWrapper;
use rand::Rng;
use std::borrow::Cow;

use genetic_algorithms::operations::selection::fitness_proportionate::roulette_wheel_selection;
use genetic_algorithms::operations::selection::fitness_proportionate::stochastic_universal_sampling;
use genetic_algorithms::operations::selection::random::random;
use genetic_algorithms::operations::selection::rank::rank_selection;
use genetic_algorithms::operations::selection::tournament::tournament;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};

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

// Setup function to create a population with configurable size and gene length
#[cfg(not(tarpaulin_include))]
fn setup_population(population_size: usize, gene_length: usize) -> Vec<SimpleChromosome> {
    let mut rng = rand::rng();
    (0..population_size)
        .map(|_| SimpleChromosome {
            fitness: rng.random_range(0.0..=1.0),
            dna: (0..gene_length)
                .map(|_| Gene {
                    id: rng.random_range(0..255),
                })
                .collect(),
            age: rng.random_range(0..=100),
            fitness_fn: FitnessFnWrapper::default(),
        })
        .collect()
}

mod selection_methods {
    use super::*;

    /// args = (population_size, gene_length)
    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [
        (10usize, 10usize), (10, 100), (10, 1000),
        (100, 10), (100, 100), (100, 1000),
        (1000, 10), (1000, 100), (1000, 1000),
    ])]
    fn random_selection(bencher: divan::Bencher, (population_size, gene_length): (usize, usize)) {
        let chromosomes = setup_population(population_size, gene_length);
        bencher.bench(|| {
            let _ = random(&chromosomes, 2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [
        (10usize, 10usize), (10, 100), (10, 1000),
        (100, 10), (100, 100), (100, 1000),
        (1000, 10), (1000, 100), (1000, 1000),
    ])]
    fn roulette_wheel(bencher: divan::Bencher, (population_size, gene_length): (usize, usize)) {
        let chromosomes = setup_population(population_size, gene_length);
        let couples = population_size / 2;
        bencher.bench(|| {
            let _ = roulette_wheel_selection(&chromosomes, couples, 2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [
        (10usize, 10usize), (10, 100), (10, 1000),
        (100, 10), (100, 100), (100, 1000),
        (1000, 10), (1000, 100), (1000, 1000),
    ])]
    fn stochastic_universal_sampling(
        bencher: divan::Bencher,
        (population_size, gene_length): (usize, usize),
    ) {
        let chromosomes = setup_population(population_size, gene_length);
        let couples = population_size / 2;
        bencher.bench(|| {
            let _ = super::stochastic_universal_sampling(&chromosomes, couples, 2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [
        (10usize, 10usize), (10, 100), (10, 1000),
        (100, 10), (100, 100), (100, 1000),
        (1000, 10), (1000, 100), (1000, 1000),
    ])]
    fn rank_selection(bencher: divan::Bencher, (population_size, gene_length): (usize, usize)) {
        let chromosomes = setup_population(population_size, gene_length);
        let couples = population_size / 2;
        bencher.bench(|| {
            let _ = super::rank_selection(&chromosomes, couples, 2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [
        (10usize, 10usize), (10, 100), (10, 1000),
        (100, 10), (100, 100), (100, 1000),
        (1000, 10), (1000, 100), (1000, 1000),
    ])]
    fn tournament(bencher: divan::Bencher, (population_size, gene_length): (usize, usize)) {
        let chromosomes = setup_population(population_size, gene_length);
        let couples = population_size / 2;
        bencher.bench(|| {
            let _ = super::tournament(&chromosomes, couples, 1, 2);
        });
    }
}

fn main() {
    divan::main();
}
