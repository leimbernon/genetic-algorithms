/*!
# ALPS: Age-Layered Sphere Minimization

ALPS on the 5-dimensional Sphere function. The population is divided into
age layers; younger layers turn over rapidly while older layers accumulate
high-quality individuals. Fresh individuals are periodically injected into
layer 0 to maintain diversity.

The Fibonacci age scheme is used here — layers have age limits proportional
to Fibonacci numbers, giving lower layers very short lifetimes and upper
layers very long ones.

Run with:
```sh
cargo run --example alps
```
*/

use std::borrow::Cow;

use genetic_algorithms::alps::{AlpsAgeScheme, AlpsConfiguration, AlpsEngine};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation};
use genetic_algorithms::rng;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
use rand::Rng;

const DIMENSIONS: usize = 5;
const SEARCH_LO: f64 = -5.0;
const SEARCH_HI: f64 = 5.0;

/// Sphere function: f(x) = Σ xᵢ² — global minimum at origin (f=0).
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.real_value().powi(2)).sum()
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

    let config = AlpsConfiguration::default()
        .with_n_layers(5)
        .with_layer_size(20)
        .with_age_scheme(AlpsAgeScheme::Fibonacci)
        .with_age_gap(5)
        .with_injection_interval(10)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.15) }))
        .with_max_generations(400)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1e-4);

    let max_ages = config.max_ages();

    let mut engine = AlpsEngine::<RangeChromosome<f64>>::new(
        config,
        init_population,
        sphere,
    ).expect("AlpsEngine::new should succeed");

    println!("== ALPS (Fibonacci): {DIMENSIONS}D Sphere ==");
    println!("layers=5, layer_size=20, age_gap=5, inject_every=10");
    println!("Age limits per layer: {:?}", max_ages);
    println!("---------------------------------------------------");

    let result = engine.run();

    println!("Generations: {}", result.generations);
    println!("Layers populated: {}", result.layers.len());
    for (i, layer) in result.layers.iter().enumerate() {
        println!("  Layer {i}: {} individuals", layer.len());
    }
    println!("Best fitness: {:.8}", result.best_fitness);
    let dna_str: Vec<String> = result.best.dna().iter()
        .map(|g| format!("{:.5}", g.real_value()))
        .collect();
    println!("Best DNA:    [{}]", dna_str.join(", "));
    assert!(result.best_fitness.is_finite(), "best_fitness must be finite");
}
