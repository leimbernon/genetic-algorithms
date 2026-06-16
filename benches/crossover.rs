use genetic_algorithms::fitness::FitnessFnWrapper;
use rand::seq::SliceRandom;
use rand::Rng;
use std::borrow::Cow;

use genetic_algorithms::operations::crossover::cycle;
use genetic_algorithms::operations::crossover::multipoint;
use genetic_algorithms::operations::crossover::order;
use genetic_algorithms::operations::crossover::single_point;
use genetic_algorithms::operations::crossover::uniform;
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

/// Creates a pair of chromosomes that share the same set of gene IDs (permutations),
/// which is required for cycle and order crossover.
#[cfg(not(tarpaulin_include))]
fn setup_permutation_pair(gene_length: usize) -> (SimpleChromosome, SimpleChromosome) {
    let mut rng = rand::rng();

    let base_genes: Vec<Gene> = (0..gene_length as i32).map(|i| Gene { id: i }).collect();

    let mut dna1 = base_genes.clone();
    let mut dna2 = base_genes;
    dna1.shuffle(&mut rng);
    dna2.shuffle(&mut rng);

    let mut make = |dna: Vec<Gene>| SimpleChromosome {
        fitness: rng.random_range(0.0..1.0),
        dna,
        age: rng.random_range(0..100),
        fitness_fn: FitnessFnWrapper::default(),
    };

    (make(dna1), make(dna2))
}

/// Creates a pair of chromosomes with random (non-permutation) gene IDs,
/// suitable for multipoint, single-point, and uniform crossover.
#[cfg(not(tarpaulin_include))]
fn setup_random_pair(gene_length: usize) -> (SimpleChromosome, SimpleChromosome) {
    let mut rng = rand::rng();

    let mut make = || SimpleChromosome {
        fitness: rng.random_range(0.0..1.0),
        dna: (0..gene_length)
            .map(|_| Gene {
                id: rng.random_range(0..255),
            })
            .collect(),
        age: rng.random_range(0..100),
        fitness_fn: FitnessFnWrapper::default(),
    };

    (make(), make())
}

mod crossover_methods {
    use super::*;

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn cycle(bencher: divan::Bencher, gene_length: usize) {
        let (perm_p1, perm_p2) = setup_permutation_pair(gene_length);
        bencher.bench(|| {
            let _ = super::cycle(&perm_p1, &perm_p2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn order(bencher: divan::Bencher, gene_length: usize) {
        let (perm_p1, perm_p2) = setup_permutation_pair(gene_length);
        bencher.bench(|| {
            let _ = super::order(&perm_p1, &perm_p2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn single_point(bencher: divan::Bencher, gene_length: usize) {
        let (rand_p1, rand_p2) = setup_random_pair(gene_length);
        bencher.bench(|| {
            let _ = super::single_point(&rand_p1, &rand_p2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn multipoint_1(bencher: divan::Bencher, gene_length: usize) {
        let (rand_p1, rand_p2) = setup_random_pair(gene_length);
        bencher.bench(|| {
            let _ = super::multipoint(&rand_p1, &rand_p2, 1);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn multipoint_2(bencher: divan::Bencher, gene_length: usize) {
        let (rand_p1, rand_p2) = setup_random_pair(gene_length);
        bencher.bench(|| {
            let _ = super::multipoint(&rand_p1, &rand_p2, 2);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn multipoint_3(bencher: divan::Bencher, gene_length: usize) {
        let (rand_p1, rand_p2) = setup_random_pair(gene_length);
        bencher.bench(|| {
            let _ = super::multipoint(&rand_p1, &rand_p2, 3);
        });
    }

    #[cfg(not(tarpaulin_include))]
    #[divan::bench(args = [10usize, 100, 1000])]
    fn uniform(bencher: divan::Bencher, gene_length: usize) {
        let (rand_p1, rand_p2) = setup_random_pair(gene_length);
        bencher.bench(|| {
            let _ = super::uniform(&rand_p1, &rand_p2);
        });
    }
}

fn main() {
    divan::main();
}
