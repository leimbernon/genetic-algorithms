/*!
# Differential Evolution: Rastrigin Minimization

L-SHADE DE on the 5-dimensional Rastrigin function — a classic multimodal
benchmark. L-SHADE adapts `F` and `CR` from a rolling success history, making
it robust without manual tuning.

Run with:
```sh
cargo run --example de_rastrigin
```
*/

use std::borrow::Cow;
use std::f64::consts::PI;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::de::{DeAdaptive, DeConfiguration, DeMutationStrategy, DeEngine};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
use rand::Rng;

const DIMENSIONS: usize = 5;
const SEARCH_LO: f64 = -5.12;
const SEARCH_HI: f64 = 5.12;

fn rastrigin(dna: &[RangeGene<f64>]) -> f64 {
    let n = dna.len() as f64;
    10.0 * n
        + dna.iter().map(|g| {
            let x = g.real_value();
            x * x - 10.0 * (2.0 * PI * x).cos()
        }).sum::<f64>()
}

fn init_population(n: usize) -> Vec<RangeChromosome<f64>> {
    let mut r = rng::make_rng();
    (0..n).map(|_| {
        let dna: Vec<RangeGene<f64>> = (0..DIMENSIONS).map(|j| {
            let v = r.random::<f64>() * (SEARCH_HI - SEARCH_LO) + SEARCH_LO;
            RangeGene::new(j as i32, vec![(SEARCH_LO, SEARCH_HI)], v)
        }).collect();
        let mut c = <RangeChromosome<f64> as Default>::default();
        c.set_dna(Cow::Owned(dna));
        c
    }).collect()
}

fn main() {
    let _ = env_logger::try_init();
    rng::set_seed(Some(42));

    let config = DeConfiguration::default()
        .with_population_size(50)
        .with_max_generations(500)
        .with_mutation_strategy(DeMutationStrategy::Rand1)
        .with_adaptive(DeAdaptive::LShade { history_size: 5 })
        .with_fitness_target(1e-3);

    let mut engine: DeEngine<RangeChromosome<f64>> =
        DeEngine::new(config, init_population, rastrigin);

    println!("== DE/rand/1 + L-SHADE: {DIMENSIONS}D Rastrigin ==");
    println!("population=50, max_generations=500, target=1e-3");
    println!("--------------------------------------------------");

    let result = engine.run();

    println!("Generations: {}", result.generations);
    println!("Best fitness: {:.6}", result.best_fitness);
    let dna_str: Vec<String> = result.best.dna().iter()
        .map(|g| format!("{:.4}", g.real_value()))
        .collect();
    println!("Best DNA:    [{}]", dna_str.join(", "));
    assert!(result.best_fitness.is_finite(), "best_fitness must be finite");
}
