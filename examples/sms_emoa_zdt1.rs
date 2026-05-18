/*!
# SMS-EMOA Multi-Objective Optimization (ZDT1 Benchmark)

This example demonstrates multi-objective optimization using SMS-EMOA (S-Metric
Selection Evolutionary Multi-Objective Algorithm) on the ZDT1 benchmark problem.
ZDT1 has two conflicting minimization objectives defined over 30 continuous
variables in [0, 1]. The Pareto-optimal front is a convex curve where minimizing
f1 forces f2 upward.

SMS-EMOA (Beume, Naujoks & Emmerich 2007) is a steady-state (mu+1) MOEA that uses
hypervolume contribution to select which individual is removed each generation.
At each step, one offspring is created, and the individual with the smallest
contribution to the hypervolume of the worst non-dominated front is removed.

## Features demonstrated
- Multi-objective optimization (SMS-EMOA)
- ZDT1 benchmark problem
- LogObserver as SmsEmoaObserver -- logs HV contribution and removal events

## Key configuration

- Population: 100 individuals
- Generations: 250
- Chromosome: 30 continuous genes, each in [0, 1]
- Both objectives: Minimize

## Run

```sh
cargo run --example sms_emoa_zdt1 --features benchmarks
```
*/

use genetic_algorithms::benchmarks::BenchmarkFn;
use genetic_algorithms::benchmarks::ZDT1;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::sms_emoa::configuration::{ObjectiveDirection, SmsEmoaConfiguration};
use genetic_algorithms::sms_emoa::SmsEmoaGa;
use genetic_algorithms::{LogObserver, SmsEmoaObserver};
use std::sync::Arc;

fn main() {
    const N_VARS: usize = 30;
    const POP_SIZE: usize = 100;
    const MAX_GENERATIONS: usize = 250;

    let sms_config = SmsEmoaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(POP_SIZE)
        .with_max_generations(MAX_GENERATIONS)
        .with_objective_directions(vec![
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
        ]);

    let mut ga_config = GaConfiguration::default();
    ga_config.limit_configuration.genes_per_chromosome = N_VARS;
    ga_config.limit_configuration.alleles_can_be_repeated = true;

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let zdt1 = ZDT1::new(N_VARS);
    let zdt1_clone = zdt1.clone();
    let obj_f1 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let x: Vec<f64> = dna.iter().map(|g| g.value).collect();
        zdt1.evaluate(&x)[0]
    };
    let obj_f2 = move |dna: &[RangeGenotype<f64>]| -> f64 {
        let x: Vec<f64> = dna.iter().map(|g| g.value).collect();
        zdt1_clone.evaluate(&x)[1]
    };

    let mut sms = SmsEmoaGa::<RangeChromosome<f64>>::new(sms_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _, _| {
            range_random_initialization(n, Some(&alleles_clone), Some(true))
        })
        .with_objective_fns(vec![Box::new(obj_f1), Box::new(obj_f2)])
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn SmsEmoaObserver<RangeChromosome<f64>> + Send + Sync>
        )
        .build()
        .expect("Failed to build SMS-EMOA");

    println!("== SMS-EMOA ZDT1 Multi-Objective Optimization ==");
    println!(
        "Variables: {}, Population: {}, Generations: {}",
        N_VARS, POP_SIZE, MAX_GENERATIONS
    );
    println!("Objectives: f1 = x_1 (minimize), f2 = ZDT1 formula (minimize)");
    println!("Steady-state (mu+1): hypervolume contribution removal");
    println!("Running SMS-EMOA (this may take a moment)...");
    println!("------------------------------------------------");

    match sms.run() {
        Ok(mut front) => {
            println!(
                "Completed. Pareto front: {} non-dominated solutions",
                front.len()
            );

            front.individuals.sort_by(|a, b| a.objectives[0].partial_cmp(&b.objectives[0]).unwrap());

            let n = front.len();
            let step = (n / 10).max(1);

            println!("Pareto front (10 points sampled from {} non-dominated solutions):", n);
            for i in (0..n).step_by(step).take(10) {
                println!(
                    "  f1={:.4}, f2={:.4}",
                    front.individuals[i].objectives[0],
                    front.individuals[i].objectives[1]
                );
            }
        }
        Err(e) => {
            eprintln!("SMS-EMOA failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
