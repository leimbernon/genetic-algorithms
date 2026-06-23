/*!
# Scatter Search: Sphere Minimization

Scatter Search on the 5-dimensional Sphere function. Scatter Search maintains
a small reference set of high-quality and diverse solutions, generating
candidates by linear combination. Local search is enabled here to show the
combined effect on solution quality.

Run with:
```sh
cargo run --example scatter_search
```
*/

use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::scatter::{ScatterConfiguration, ScatterEngine};
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
    (0..n)
        .map(|_| {
            let dna: Vec<RangeGene<f64>> = (0..DIMENSIONS)
                .map(|j| {
                    let v = r.random::<f64>() * (SEARCH_HI - SEARCH_LO) + SEARCH_LO;
                    RangeGene::new(j as i32, vec![(SEARCH_LO, SEARCH_HI)], v)
                })
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

fn main() {
    let _ = env_logger::try_init();
    rng::set_seed(Some(42));

    let config = ScatterConfiguration::default()
        .with_population_size(80)
        .with_reference_set_size(12)
        .with_max_iterations(150)
        .with_local_search(true)
        .with_local_search_steps(30)
        .with_local_search_step_size(0.05)
        .with_fitness_target(1e-4);

    let mut engine: ScatterEngine<RangeChromosome<f64>> =
        ScatterEngine::new(config, init_population, sphere);

    println!("== Scatter Search: {DIMENSIONS}D Sphere ==");
    println!("pop=80, ref_set=12, iterations=150, local_search=ON");
    println!("------------------------------------------------");

    let result = engine.run();

    println!("Iterations: {}", result.iterations);
    println!("Reference set size: {}", result.reference_set.len());
    println!("Best fitness: {:.8}", result.best_fitness);
    let dna_str: Vec<String> = result
        .best
        .dna()
        .iter()
        .map(|g| format!("{:.5}", g.real_value()))
        .collect();
    println!("Best DNA:    [{}]", dna_str.join(", "));
    assert!(
        result.best_fitness.is_finite(),
        "best_fitness must be finite"
    );
}
