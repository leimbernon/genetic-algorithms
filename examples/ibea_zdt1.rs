/*!
# IBEA Multi-Objective Optimization (ZDT1 Benchmark)

This example demonstrates multi-objective optimization using IBEA (Indicator-Based
Evolutionary Algorithm) on the ZDT1 benchmark problem. ZDT1 has two conflicting
minimization objectives defined over 30 continuous variables in [0, 1]. The
Pareto-optimal front is a convex curve where minimizing f1 forces f2 upward.

IBEA (Zitzler & Kunzli 2004) uses a pairwise indicator function (additive epsilon,
I_eps+) to assign fitness. Fitness F(x) = sum_{y != x} -exp(-I_eps+(y, x) / c)
where c is an adaptive scaling factor. Environmental selection iteratively removes
the individual with lowest fitness, recalculating fitnesses after each removal.

## Features demonstrated
- Multi-objective optimization (IBEA)
- ZDT1 benchmark problem
- LogObserver as IbeaObserver -- logs indicator fitness and selection events

## Key configuration

- Population: 100 individuals
- Generations: 250
- Chromosome: 30 continuous genes, each in [0, 1]
- Both objectives: Minimize

## Run

```sh
cargo run --example ibea_zdt1 --features benchmarks
```
*/

use genetic_algorithms::benchmarks::BenchmarkFn;
use genetic_algorithms::benchmarks::ZDT1;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::ibea::configuration::{IbeaConfiguration, ObjectiveDirection};
use genetic_algorithms::ibea::IbeaGa;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::{IbeaObserver, LogObserver};
use std::sync::Arc;

fn main() {
    const N_VARS: usize = 30;
    const POP_SIZE: usize = 100;
    const MAX_GENERATIONS: usize = 250;

    let ibea_config = IbeaConfiguration::new()
        .with_num_objectives(2)
        .with_population_size(POP_SIZE)
        .with_max_generations(MAX_GENERATIONS)
        .with_objective_directions(vec![
            ObjectiveDirection::Minimize,
            ObjectiveDirection::Minimize,
        ]);

    use genetic_algorithms::ChromosomeLength;
    use genetic_algorithms::traits::ConfigurationT;
    let ga_config = GaConfiguration::default()
        .with_chromosome_length(ChromosomeLength::Fixed(N_VARS));

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

    let mut ibea = IbeaGa::<RangeChromosome<f64>>::new(ibea_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_objective_fns(vec![Box::new(obj_f1), Box::new(obj_f2)])
        .with_observer(
            Arc::new(LogObserver) as Arc<dyn IbeaObserver<RangeChromosome<f64>> + Send + Sync>
        )
        .build()
        .expect("Failed to build IBEA");

    println!("== IBEA ZDT1 Multi-Objective Optimization ==");
    println!(
        "Variables: {}, Population: {}, Generations: {}",
        N_VARS, POP_SIZE, MAX_GENERATIONS
    );
    println!("Objectives: f1 = x_1 (minimize), f2 = ZDT1 formula (minimize)");
    println!("Indicator: additive epsilon (I_eps+) with adaptive scaling");
    println!("Running IBEA (this may take a moment)...");
    println!("------------------------------------------------");

    match ibea.run() {
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
            eprintln!("IBEA failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
