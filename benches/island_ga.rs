use criterion::{
    criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration,
};

use genetic_algorithms::chromosomes::ChromosomeLength;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::fitness::FitnessFnWrapper;
use genetic_algorithms::island::configuration::IslandConfiguration;
use genetic_algorithms::island::topology::MigrationTopology;
use genetic_algorithms::island::IslandGa;
use genetic_algorithms::operations::mutation::ValueMutable;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, GeneT, LinearChromosome, MutationConfig,
    OperatorCompat, SelectionConfig, StoppingConfig,
};
use rand::Rng;
use std::borrow::Cow;

use genetic_algorithms::configuration::GaConfiguration;

// ---------------------------------------------------------------------------
// Chromosome / Gene types
// ---------------------------------------------------------------------------

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
struct SimpleChromosome {
    dna: Vec<Gene>,
    fitness: f64,
    age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    fitness_fn: FitnessFnWrapper<Gene>,
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
        for (i, gene) in self.dna.iter().enumerate() {
            self.fitness += f64::from(gene.id() * i as i32);
        }
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
impl ValueMutable for SimpleChromosome {}
impl OperatorCompat for SimpleChromosome {}

// ---------------------------------------------------------------------------
// Setup helper
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn build_island_ga(
    num_islands: usize,
    population_per_island: usize,
    gene_length: usize,
    max_generations: usize,
    migration_interval: usize,
) -> IslandGa<SimpleChromosome> {
    let alleles: Vec<Gene> = (0..gene_length as i32).map(|i| Gene { id: i }).collect();
    let alleles_for_init = alleles.clone();

    let island_config = IslandConfiguration::new()
        .with_num_islands(num_islands)
        .with_migration_interval(migration_interval)
        .with_migration_count(1)
        .with_topology(MigrationTopology::Ring);

    let ga_config = GaConfiguration::new()
        .with_population_size(population_per_island)
        .with_chromosome_length(ChromosomeLength::Fixed(gene_length))
        .with_max_generations(max_generations)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization);

    IslandGa::<SimpleChromosome>::new(island_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |genes_per_chrom, _alleles| {
            let mut rng = rand::rng();
            (0..genes_per_chrom)
                .map(|_| alleles_for_init[rng.random_range(0..alleles_for_init.len())])
                .collect()
        })
        .with_fitness_fn(|dna: &[Gene]| dna.iter().map(|g| g.id as f64).sum())
        .build()
        .expect("IslandGa configuration should be valid")
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

#[cfg(not(tarpaulin_include))]
fn benchmark_island_ga_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("island_ga_run");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let configs: Vec<(usize, usize, usize, usize, usize)> = vec![
        // (num_islands, pop_per_island, gene_length, max_generations, migration_interval)
        (2, 20, 6, 10, 5),
        (3, 20, 6, 10, 5),
        (4, 20, 6, 10, 5),
        (3, 50, 6, 10, 5),
        (3, 20, 6, 20, 5),
    ];

    for &(islands, pop, genes, gens, mig) in &configs {
        group.bench_with_input(
            BenchmarkId::new(
                "IslandGa::run",
                format!(
                    "islands_{}_pop_{}_genes_{}_gen_{}_mig_{}",
                    islands, pop, genes, gens, mig
                ),
            ),
            &(islands, pop, genes, gens, mig),
            |b, &(ni, pp, gl, mg, mi)| {
                b.iter_batched(
                    || build_island_ga(ni, pp, gl, mg, mi),
                    |mut ga| {
                        let _ = ga.run();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = island_benchmarks;
    config = Criterion::default();
    targets = benchmark_island_ga_run
}

criterion_main!(island_benchmarks);
