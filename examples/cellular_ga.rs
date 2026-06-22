/*!
# Cellular GA: Sphere Minimization

Cellular GA on the 4-dimensional Sphere function. Individuals live on a
toroidal 2D grid and interact only with their local neighborhood, which
promotes spatial diversity and slows premature convergence.

Two topologies are available — switch `Neighborhood::Moore` to
`Neighborhood::VonNeumann` to compare convergence rates.

Run with:
```sh
cargo run --example cellular_ga
```
*/

use std::borrow::Cow;

use genetic_algorithms::cellular::{
    CellularConfiguration, CellularEngine, Neighborhood, UpdateMode,
};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation, Selection};
use genetic_algorithms::rng;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
use rand::Rng;

const DIMENSIONS: usize = 4;
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

    // 10×10 grid = 100 individuals
    let config = CellularConfiguration::default()
        .with_grid(10, 10)
        .with_neighborhood(Neighborhood::Moore)   // 8-cell neighborhood
        .with_update_mode(UpdateMode::Asynchronous)
        .with_selection(Selection::Tournament)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.2) }))
        .with_max_generations(300)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1e-4);

    let mut engine = CellularEngine::<RangeChromosome<f64>>::new(
        config,
        init_population,
        sphere,
    ).expect("CellularEngine::new should succeed");

    println!("== Cellular GA (Moore, Async): {DIMENSIONS}D Sphere ==");
    println!("grid=10×10, max_generations=300, target=1e-4");
    println!("-----------------------------------------------------");

    let result = engine.run();

    println!("Generations: {}", result.generations);
    println!("Best fitness: {:.8}", result.best_fitness);
    let dna_str: Vec<String> = result.best.dna().iter()
        .map(|g| format!("{:.5}", g.real_value()))
        .collect();
    println!("Best DNA:    [{}]", dna_str.join(", "));
    assert!(result.best_fitness.is_finite(), "best_fitness must be finite");
}
