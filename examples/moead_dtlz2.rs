/*!
# MOEA/D Decomposition-Based Multi-Objective Optimization (DTLZ2 3-objective Benchmark)

This example demonstrates decomposition-based multi-objective optimization
using MOEA/D (Zhang & Li 2007) on the DTLZ2 benchmark problem with three
objectives. The Pareto-optimal front lies on the unit sphere
f1² + f2² + f3² = 1 (when g(x) = 0).

## Problem definition

- Variables: x = (x_1, ..., x_12) in [0, 1] (M=3, k=10 — standard DTLZ2 setup)
- f_1 = cos(x_1 · π/2) · cos(x_2 · π/2) · (1 + g(x))
- f_2 = cos(x_1 · π/2) · sin(x_2 · π/2) · (1 + g(x))
- f_3 = sin(x_1 · π/2) · (1 + g(x))
- g(x) = Σ_{i=2}^{n-1} (x_i − 0.5)² (zero-indexed positions 2..n)

The Pareto-optimal front is f1² + f2² + f3² = 1 with x_3,…,x_12 = 0.5.

## GA mode

MOEA/D with Tchebycheff scalarization. Each weight vector defines one
sub-problem; offspring compete only within a T=20 nearest-neighbourhood.
Weight vectors are auto-generated via the Das-Dennis simplex lattice
(p=12 → C(14,2)=91 weight vectors).

Note: C(p+M-1, M-1) for M=3 is (p+2)(p+1)/2; p=12 gives 91 (population size
matches the weight-vector count, the standard MOEA/D convention).

## Features demonstrated
- Decomposition-based multi-objective optimization (MOEA/D)
- Tchebycheff scalarization
- DTLZ2 benchmark
- Auto-generated Das-Dennis weight vectors
- LogObserver attached as MoeaDObserver
*/

use std::borrow::Cow;
use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;

use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::moead::configuration::{
    MoeaDConfiguration, ObjectiveDirection, ScalarizationFn,
};
use genetic_algorithms::moead::MoeaDGa;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome, VectorFitness};
use genetic_algorithms::{LogObserver, MoeaDObserver};

const N_VARS: usize = 12;
const POP_SIZE: usize = 91; // C(14, 2) = 91 weight vectors with p=12, M=3
const MAX_GENERATIONS: usize = 300;
const DAS_DENNIS_P: usize = 12;
const NEIGHBORHOOD_SIZE: usize = 20;        // Zhang & Li 2007 baseline
const MAX_NEIGHBOR_REPLACEMENTS: usize = 2; // Zhang & Li 2007 baseline

// Custom chromosome that encodes DTLZ2 objectives directly in calculate_fitness().
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
    // --- MOEA/D configuration ---
    let moead_config = MoeaDConfiguration::new()
        .with_num_objectives(3)
        .with_population_size(POP_SIZE)
        .with_max_generations(MAX_GENERATIONS)
        .with_objective_directions(vec![
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
        ])
        .with_weight_vectors_auto(DAS_DENNIS_P)
        .with_scalarization(ScalarizationFn::Tchebycheff)
        .with_neighborhood_size(NEIGHBORHOOD_SIZE)
        .with_max_neighbor_replacements(MAX_NEIGHBOR_REPLACEMENTS);

    // --- Base GA configuration ---
    use genetic_algorithms::ChromosomeLength;
    use genetic_algorithms::traits::ConfigurationT;
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(N_VARS));

    // --- Allele definition: each of the 12 variables lives in [0.0, 1.0] ---
    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    // --- Build the MOEA/D optimizer ---
    let mut moead = MoeaDGa::<Dtlz2Chromosome>::new(moead_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn MoeaDObserver<Dtlz2Chromosome> + Send + Sync>,
        )
        .build()
        .expect("Failed to build MOEA/D");

    println!("== MOEA/D DTLZ2 Decomposition-Based Multi-Objective Optimization ==");
    println!(
        "Variables: {}, Population: {}, Generations: {}, Weight vectors (Das-Dennis p={}): {}",
        N_VARS,
        POP_SIZE,
        MAX_GENERATIONS,
        DAS_DENNIS_P,
        // Compute C(p+M-1, M-1) for M=3 -> C(p+2, 2) = (p+2)(p+1)/2
        (DAS_DENNIS_P + 2) * (DAS_DENNIS_P + 1) / 2
    );
    println!(
        "Scalarization: Tchebycheff, T (neighbourhood): {}, max replacements: {}",
        NEIGHBORHOOD_SIZE, MAX_NEIGHBOR_REPLACEMENTS
    );

    match moead.run() {
        Ok(mut front) => {
            println!(
                "\nPareto front: {} non-dominated solutions",
                front.individuals.len()
            );
            // Sort individuals by f_1 ascending for readable output.
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
        }
        Err(e) => {
            eprintln!("MOEA/D failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
