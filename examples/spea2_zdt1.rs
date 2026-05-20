/*!
# SPEA2 Multi-Objective Optimization (ZDT1 Benchmark)

This example demonstrates multi-objective optimization using SPEA2 (Strength Pareto
Evolutionary Algorithm 2) on the ZDT1 benchmark problem. ZDT1 has two conflicting
minimization objectives defined over 30 continuous variables in [0, 1]. The
Pareto-optimal front is a convex curve where minimizing f1 forces f2 upward.

## Problem definition

- Variables: x = (x_1, ..., x_30) in [0, 1]
- f1(x) = x_1
- f2(x) = g(x) * (1 - sqrt(x_1 / g(x)))
- g(x) = 1 + (9 / 29) * sum(x_2, ..., x_30)

The Pareto-optimal front is: f2 = 1 - sqrt(f1), where g(x) = 1.

## Algorithm

SPEA2 (Zitzler, Laumanns & Thiele 2001) maintains a fixed-size external archive
of non-dominated solutions. Fitness is computed from raw strength (domination
count) plus density (k-nearest-neighbour distance), and the archive is truncated
using iterative nearest-neighbour removal when it exceeds capacity.

## Features demonstrated
- Multi-objective optimization (SPEA2)
- ZDT1 benchmark problem
- LogObserver as Spea2Observer — logs fitness and archive events

## Key configuration

- Population: 100 individuals
- Archive: 100 individuals (canonical SPEA2: archive_size == population_size)
- Generations: 250
- Chromosome: 30 continuous genes, each in [0, 1]
- Both objectives: Minimize

## Run

```sh
cargo run --example spea2_zdt1
```
*/

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::spea2::configuration::{Spea2Configuration, ObjectiveDirection};
use genetic_algorithms::spea2::Spea2Ga;
use genetic_algorithms::{LogObserver, Spea2Observer};
use std::sync::Arc;

fn main() {
    // --- Problem parameters ---
    const N_VARS: usize = 30;
    const POP_SIZE: usize = 100;
    const ARCHIVE_SIZE: usize = 100;  // canonical SPEA2: archive equals population
    const MAX_GENERATIONS: usize = 250;

    // --- SPEA2 configuration ---
    let spea2_config = Spea2Configuration::new()
        .with_num_objectives(2)
        .with_population_size(POP_SIZE)
        .with_archive_size(ARCHIVE_SIZE)
        .with_max_generations(MAX_GENERATIONS)
        .with_objective_directions(vec![
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
        ]);

    // --- Base GA configuration ---
    use genetic_algorithms::ChromosomeLength;
    use genetic_algorithms::traits::ConfigurationT;
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(N_VARS));

    // --- Allele definition: each of the 30 variables lives in [0.0, 1.0] ---
    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    // --- Objective functions (ZDT1) ---
    // f1 = x_1  (first variable)
    let obj_f1 = |dna: &[RangeGenotype<f64>]| -> f64 { dna[0].value };

    // f2 = g(x) * (1 - sqrt(x_1 / g(x)))
    //   where g(x) = 1 + (9 / (n-1)) * sum(x_2..x_n)
    let obj_f2 = |dna: &[RangeGenotype<f64>]| -> f64 {
        let n = dna.len();
        let g = 1.0 + (9.0 / (n - 1) as f64) * dna[1..].iter().map(|gene| gene.value).sum::<f64>();
        g * (1.0 - (dna[0].value / g).sqrt())
    };

    // --- Build the SPEA2 optimizer ---
    // LogObserver implements Spea2Observer — logs fitness and archive events
    let mut spea2 = Spea2Ga::<RangeChromosome<f64>>::new(spea2_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_objective_fns(vec![Box::new(obj_f1), Box::new(obj_f2)])
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn Spea2Observer<RangeChromosome<f64>> + Send + Sync>
        )
        .build()
        .expect("Failed to build SPEA2");

    println!("== SPEA2 ZDT1 Multi-Objective Optimization ==");
    println!(
        "Variables: {}, Population: {}, Archive: {}, Generations: {}",
        N_VARS, POP_SIZE, ARCHIVE_SIZE, MAX_GENERATIONS
    );
    println!("Objectives: f1 = x_1 (minimize), f2 = ZDT1 formula (minimize)");
    println!("Running SPEA2 (this may take a moment)...");
    println!("------------------------------------------------");

    // --- Run SPEA2 ---
    match spea2.run() {
        Ok(mut front) => {
            println!(
                "Completed. Pareto front: {} non-dominated solutions",
                front.len()
            );

            // Sort by f1 ascending to visualise the Pareto trade-off curve
            front
                .individuals
                .sort_by(|a, b| a.objectives[0].partial_cmp(&b.objectives[0]).unwrap());

            let n = front.len();
            let step = (n / 10).max(1);

            println!(
                "Pareto front (10 points sampled from {} non-dominated solutions):",
                n
            );
            for i in (0..n).step_by(step).take(10) {
                println!(
                    "  f1={:.4}, f2={:.4}",
                    front.individuals[i].objectives[0], front.individuals[i].objectives[1]
                );
            }
        }
        Err(e) => {
            eprintln!("SPEA2 failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
