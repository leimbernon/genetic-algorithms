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
cargo run --example ibea_zdt1
```
*/

use std::borrow::Cow;
use std::sync::Arc;

use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::ibea::configuration::{IbeaConfiguration, ObjectiveDirection};
use genetic_algorithms::ibea::IbeaGa;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::traits::{ChromosomeT, LinearChromosome, VectorFitness};
use genetic_algorithms::{IbeaObserver, LogObserver};

const N_VARS: usize = 30;

// ZDT1 chromosome: f1 = x_0, f2 = g * (1 - sqrt(x_0/g)), g = 1 + 9/(n-1) * sum(x_1..x_n)
#[derive(Debug, Clone, Default)]
struct Zdt1Chromosome {
    dna: Vec<RangeGenotype<f64>>,
    fitness: f64,
    fitness_values: Vec<f64>,
}

impl ChromosomeT for Zdt1Chromosome {
    type Gene = RangeGenotype<f64>;
    fn fitness(&self) -> f64 {
        self.fitness
    }
    fn set_fitness(&mut self, v: f64) -> &mut Self {
        self.fitness = v;
        self
    }
    fn set_age(&mut self, _: usize) -> &mut Self {
        self
    }
    fn age(&self) -> usize {
        0
    }
    fn calculate_fitness(&mut self) {
        if self.dna.is_empty() {
            self.fitness_values = vec![0.0, 0.0];
            return;
        }
        let n = self.dna.len();
        let x0 = self.dna[0].value;
        let g = 1.0 + (9.0 / (n - 1) as f64) * self.dna[1..].iter().map(|g| g.value).sum::<f64>();
        let f1 = x0;
        let f2 = g * (1.0 - (x0 / g).sqrt());
        self.fitness_values = vec![f1, f2];
        self.fitness = f1;
    }
}

impl LinearChromosome for Zdt1Chromosome {
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }
    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = dna.into_owned();
        self
    }
    fn set_fitness_fn<F>(&mut self, _: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        self
    }
}

impl VectorFitness for Zdt1Chromosome {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }
    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl genetic_algorithms::operations::mutation::ValueMutable for Zdt1Chromosome {}
impl genetic_algorithms::traits::OperatorCompat for Zdt1Chromosome {}
impl genetic_algorithms::traits::RealValuedMutation for Zdt1Chromosome {}

fn main() {
    let _ = env_logger::try_init();
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

    use genetic_algorithms::traits::ConfigurationT;
    use genetic_algorithms::ChromosomeLength;
    let ga_config =
        GaConfiguration::default().with_chromosome_length(ChromosomeLength::Fixed(N_VARS));

    let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut ibea = IbeaGa::<Zdt1Chromosome>::new(ibea_config, ga_config)
        .with_alleles(alleles)
        .with_initialization_fn(move |n, _| range_random_initialization(n, Some(&alleles_clone)))
        .with_observer(Arc::new(LogObserver) as Arc<dyn IbeaObserver<Zdt1Chromosome> + Send + Sync>)
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

            front.individuals.sort_by(|a, b| {
                a.objectives[0]
                    .partial_cmp(&b.objectives[0])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

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

            #[cfg(feature = "visualization")]
            if std::env::args().any(|a| a == "--plot") {
                // requires --features "visualization,benchmarks"
                let points: Vec<(f64, f64)> = front
                    .individuals
                    .iter()
                    .map(|ind| (ind.objectives[0], ind.objectives[1]))
                    .collect();
                std::fs::create_dir_all("docs/images").expect("failed to create docs/images");
                genetic_algorithms::visualization::plot_pareto_front_2d(
                    &points,
                    "docs/images/ibea_zdt1.png",
                )
                .expect("plot failed");
                println!("Pareto front plot saved to docs/images/ibea_zdt1.png");
            }
        }
        Err(e) => {
            eprintln!("IBEA failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
