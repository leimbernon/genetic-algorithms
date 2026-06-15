/*!
# NSGA-III Many-Objective Optimization (DTLZ2 3-objective Benchmark)

This example demonstrates many-objective optimization using NSGA-III on the
DTLZ2 benchmark problem with three objectives. The Pareto-optimal front lies
on the unit sphere f1² + f2² + f3² = 1 (when g(x) = 0).

## Problem definition

- Variables: x = (x_1, ..., x_12) in [0, 1] (M=3, k=10 — standard DTLZ2 setup)
- f_1 = cos(x_1 · π/2) · cos(x_2 · π/2) · (1 + g(x))
- f_2 = cos(x_1 · π/2) · sin(x_2 · π/2) · (1 + g(x))
- f_3 = sin(x_1 · π/2) · (1 + g(x))
- g(x) = Σ_{i=2}^{n-1} (x_i − 0.5)² (zero-indexed positions 2..n)

The Pareto-optimal front is f1² + f2² + f3² = 1 with x_3,…,x_12 = 0.5.

## GA mode

NSGA-III with non-dominated sorting and reference-point niche association.
Reference points are generated automatically via the Das-Dennis simplex lattice
(p=12 → C(14,2)=91 reference points, ~population size).

## Features demonstrated
- Many-objective optimization (NSGA-III)
- DTLZ2 benchmark
- Auto-generated Das-Dennis reference points
- LogObserver attached as Nsga3Observer
*/

use std::borrow::Cow;
use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;

use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};
use genetic_algorithms::nsga3::Nsga3Ga;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome, VectorFitness};
use genetic_algorithms::{LogObserver, Nsga3Observer};

const N_VARS: usize = 12;
const POP_SIZE: usize = 100;
const MAX_GENERATIONS: usize = 200;
const DAS_DENNIS_P: usize = 12;

// DTLZ2 chromosome: 3 objectives on the unit sphere surface
#[derive(Debug, Clone, Default)]
struct Dtlz2Chromosome {
    dna: Vec<RangeGenotype<f64>>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for Dtlz2Chromosome {
    type Gene = RangeGenotype<f64>;
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) -> &mut Self { self.fitness = v; self }
    fn set_age(&mut self, _: usize) -> &mut Self { self }
    fn age(&self) -> usize { 0 }
    fn calculate_fitness(&mut self) {
        if self.dna.len() < 3 {
            self.fitness_values = vec![0.0, 0.0, 0.0];
            self.fitness = 0.0;
            return;
        }
        let g: f64 = self.dna[2..].iter().map(|gene| (gene.value - 0.5).powi(2)).sum();
        let f1 = (self.dna[0].value * FRAC_PI_2).cos() * (self.dna[1].value * FRAC_PI_2).cos() * (1.0 + g);
        let f2 = (self.dna[0].value * FRAC_PI_2).cos() * (self.dna[1].value * FRAC_PI_2).sin() * (1.0 + g);
        let f3 = (self.dna[0].value * FRAC_PI_2).sin() * (1.0 + g);
        self.fitness_values = vec![f1, f2, f3];
        self.fitness = f1 + f2 + f3;
    }
}

impl LinearChromosome for Dtlz2Chromosome {
    fn dna(&self) -> &[Self::Gene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { &mut self.dna }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned(); self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static { self }
}

impl VectorFitness for Dtlz2Chromosome {
    fn fitness_values(&self) -> &[f64] { &self.fitness_values }
    fn set_fitness_values(&mut self, values: Vec<f64>) { self.fitness_values = values; }
}

impl genetic_algorithms::operations::mutation::ValueMutable for Dtlz2Chromosome {}
impl genetic_algorithms::traits::OperatorCompat for Dtlz2Chromosome {}

fn main() {
    let _ = env_logger::try_init();
    let nsga3_config = Nsga3Configuration::new()
        .with_num_objectives(3)
        .with_population_size(POP_SIZE)
        .with_max_generations(MAX_GENERATIONS)
        .with_objective_directions(vec![
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
        ])
        .with_reference_points_auto(DAS_DENNIS_P);

    use genetic_algorithms::ChromosomeLength;
    use genetic_algorithms::traits::ConfigurationT;
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(N_VARS));

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut nsga3 = Nsga3Ga::<Dtlz2Chromosome>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn Nsga3Observer<Dtlz2Chromosome> + Send + Sync>
        )
        .build()
        .expect("Failed to build NSGA-III");

    println!("== NSGA-III DTLZ2 Many-Objective Optimization ==");
    println!(
        "Variables: {}, Population: {}, Generations: {}, Reference points (Das-Dennis p={}): {}",
        N_VARS,
        POP_SIZE,
        MAX_GENERATIONS,
        DAS_DENNIS_P,
        (DAS_DENNIS_P + 2) * (DAS_DENNIS_P + 1) / 2
    );

    match nsga3.run() {
        Ok(mut front) => {
            println!(
                "\nPareto front: {} non-dominated solutions",
                front.individuals.len()
            );
            front.individuals.sort_by(|a, b| {
                a.objectives[0]
                    .partial_cmp(&b.objectives[0])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let sample = front.individuals.iter().take(10);
            println!("\nFirst 10 individuals (sorted by f1):");
            println!("    f_1        f_2        f_3        ||f||²");
            for ind in sample {
                let f1 = ind.objectives[0];
                let f2 = ind.objectives[1];
                let f3 = ind.objectives[2];
                let norm_sq = f1 * f1 + f2 * f2 + f3 * f3;
                println!("  {:>8.4}   {:>8.4}   {:>8.4}   {:>8.4}", f1, f2, f3, norm_sq);
            }

            #[cfg(feature = "visualization")]
            if std::env::args().any(|a| a == "--plot") {
                // requires --features visualization
                let points: Vec<(f64, f64, f64)> = front.individuals.iter()
                    .map(|ind| (ind.objectives[0], ind.objectives[1], ind.objectives[2]))
                    .collect();
                std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
                genetic_algorithms::visualization::plot_pareto_front_3d(
                    &points,
                    "docs/images/nsga3_dtlz2.png",
                )
                .expect("plot failed");
                println!("Pareto front plot saved to docs/images/nsga3_dtlz2.png");
            }
        }
        Err(e) => {
            eprintln!("NSGA-III failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
