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

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga3::configuration::{Nsga3Configuration, ObjectiveDirection};
use genetic_algorithms::nsga3::Nsga3Ga;
use genetic_algorithms::{LogObserver, Nsga3Observer};
use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;

const N_VARS: usize = 12;
const POP_SIZE: usize = 100;
const MAX_GENERATIONS: usize = 200;
const DAS_DENNIS_P: usize = 12; // C(14, 2) = 91 reference points

fn main() {
    // --- NSGA-III configuration ---
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

    // --- Base GA configuration ---
    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = N_VARS;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    // --- Allele definition: each of the 12 variables lives in [0.0, 1.0] ---
    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    // --- Objective functions (DTLZ2, M=3) ---
    let g_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
        dna[2..].iter().map(|gene| (gene.value - 0.5).powi(2)).sum()
    };

    let f1 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let g = g_fn(dna);
        (dna[0].value * FRAC_PI_2).cos() * (dna[1].value * FRAC_PI_2).cos() * (1.0 + g)
    };
    let f2 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let g = g_fn(dna);
        (dna[0].value * FRAC_PI_2).cos() * (dna[1].value * FRAC_PI_2).sin() * (1.0 + g)
    };
    let f3 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let g = g_fn(dna);
        (dna[0].value * FRAC_PI_2).sin() * (1.0 + g)
    };

    // --- Build the NSGA-III optimizer ---
    let mut nsga3 = Nsga3Ga::<RangeChromosome<f64>>::new(nsga3_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(vec![Box::new(f1), Box::new(f2), Box::new(f3)])
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn Nsga3Observer<RangeChromosome<f64>> + Send + Sync>
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
        // Compute C(p+M-1, M-1) for M=3 -> C(p+2, 2) = (p+2)(p+1)/2
        (DAS_DENNIS_P + 2) * (DAS_DENNIS_P + 1) / 2
    );

    match nsga3.run() {
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
            eprintln!("NSGA-III failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
