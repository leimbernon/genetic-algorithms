use criterion::{
    criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration,
};

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::fitness::FitnessFnWrapper;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::genotypes::Range as RangeGene;
use rand::Rng;
use std::borrow::Cow;

use genetic_algorithms::operations::mutation::bit_flip::bit_flip;
use genetic_algorithms::operations::mutation::creep::creep_mutation;
use genetic_algorithms::operations::mutation::gaussian::gaussian_mutation;
use genetic_algorithms::operations::mutation::inversion::inversion;
use genetic_algorithms::operations::mutation::scramble::scramble;
use genetic_algorithms::operations::mutation::swap::swap;
use genetic_algorithms::operations::mutation::value::value_mutation;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};

// ---------------------------------------------------------------------------
// Generic chromosome for swap/inversion/scramble (gene-id based)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Binary chromosome for bit_flip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
struct BinaryChromosome {
    dna: Vec<BinaryGene>,
    fitness: f64,
    age: usize,
    fitness_fn: FitnessFnWrapper<BinaryGene>,
}
impl ChromosomeT for BinaryChromosome {
    type Gene = BinaryGene;
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
impl LinearChromosome for BinaryChromosome {
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

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn setup_chromosome(gene_length: usize) -> SimpleChromosome {
    let mut rng = rand::rng();
    SimpleChromosome {
        fitness: rng.random_range(0.0..1.0),
        dna: (0..gene_length)
            .map(|_| Gene {
                id: rng.random_range(0..255),
            })
            .collect(),
        age: rng.random_range(0..100),
        fitness_fn: FitnessFnWrapper::default(),
    }
}

#[cfg(not(tarpaulin_include))]
fn setup_binary_chromosome(gene_length: usize) -> BinaryChromosome {
    let mut rng = rand::rng();
    BinaryChromosome {
        fitness: rng.random_range(0.0..1.0),
        dna: (0..gene_length)
            .map(|i| {
                let mut g = <BinaryGene as Default>::default();
                g.set_id(i as i32);
                g.value = rng.random_range(0..=1) == 1;
                g
            })
            .collect(),
        age: 0,
        fitness_fn: FitnessFnWrapper::default(),
    }
}

#[cfg(not(tarpaulin_include))]
fn setup_range_chromosome(gene_length: usize) -> RangeChromosome<f64> {
    let mut rng = rand::rng();
    let alleles: Vec<(f64, f64)> = (0..gene_length).map(|_| (0.0, 100.0)).collect();
    let mut chromosome = RangeChromosome::<f64>::new();
    chromosome.dna = (0..gene_length)
        .map(|i| RangeGene::new(i as i32, alleles.clone(), rng.random_range(0.0..100.0)))
        .collect();
    chromosome
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn benchmark_mutation_methods(c: &mut Criterion) {
    let gene_lengths = vec![10, 100, 1000];

    let mut group = c.benchmark_group("mutation_methods");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &gene_length in &gene_lengths {
        let chromosome = setup_chromosome(gene_length);

        // Swap mutation — setup clone outside the timed loop
        group.bench_with_input(
            BenchmarkId::new("swap", format!("genes_{}", gene_length)),
            &chromosome,
            |b, chromosome| {
                b.iter_batched(
                    || chromosome.clone(),
                    |mut c| swap(&mut c),
                    BatchSize::SmallInput,
                );
            },
        );

        // Inversion mutation
        group.bench_with_input(
            BenchmarkId::new("inversion", format!("genes_{}", gene_length)),
            &chromosome,
            |b, chromosome| {
                b.iter_batched(
                    || chromosome.clone(),
                    |mut c| inversion(&mut c),
                    BatchSize::SmallInput,
                );
            },
        );

        // Scramble mutation
        group.bench_with_input(
            BenchmarkId::new("scramble", format!("genes_{}", gene_length)),
            &chromosome,
            |b, chromosome| {
                b.iter_batched(
                    || chromosome.clone(),
                    |mut c| scramble(&mut c),
                    BatchSize::SmallInput,
                );
            },
        );

        // Bit-flip mutation (binary chromosome)
        let binary_chromosome = setup_binary_chromosome(gene_length);
        group.bench_with_input(
            BenchmarkId::new("bit_flip", format!("genes_{}", gene_length)),
            &binary_chromosome,
            |b, chromosome| {
                b.iter_batched(
                    || chromosome.clone(),
                    |mut c| bit_flip(&mut c),
                    BatchSize::SmallInput,
                );
            },
        );

        // Value mutation (range chromosome)
        let range_chromosome = setup_range_chromosome(gene_length);
        group.bench_with_input(
            BenchmarkId::new("value", format!("genes_{}", gene_length)),
            &range_chromosome,
            |b, chromosome: &RangeChromosome<f64>| {
                b.iter_batched(
                    || chromosome.clone(),
                    |mut c| value_mutation(&mut c),
                    BatchSize::SmallInput,
                );
            },
        );

        // Creep mutation (range chromosome)
        group.bench_with_input(
            BenchmarkId::new("creep", format!("genes_{}", gene_length)),
            &range_chromosome,
            |b, chromosome: &RangeChromosome<f64>| {
                b.iter_batched(
                    || chromosome.clone(),
                    |mut c| creep_mutation(&mut c, 1.0),
                    BatchSize::SmallInput,
                );
            },
        );

        // Gaussian mutation (range chromosome)
        group.bench_with_input(
            BenchmarkId::new("gaussian", format!("genes_{}", gene_length)),
            &range_chromosome,
            |b, chromosome: &RangeChromosome<f64>| {
                b.iter_batched(
                    || chromosome.clone(),
                    |mut c| gaussian_mutation(&mut c, 1.0),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = mutation_benchmarks;
    config = Criterion::default();
    targets = benchmark_mutation_methods
}

criterion_main!(mutation_benchmarks);
