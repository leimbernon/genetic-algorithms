/*!
# IPOP-CMA-ES: Rastrigin Minimization with Restart Strategies

IPOP-CMA-ES on the 10-dimensional Rastrigin function — a classic multimodal
benchmark with many local optima. IPOP restarts the CMA-ES run with an
increasingly larger population when stagnation is detected, helping escape
local minima that a fixed-population run would get stuck in.

Run with:
```sh
cargo run --example ipop_rastrigin
```
*/

use std::borrow::Cow;
use std::f64::consts::PI;
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine, RestartStrategy};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
use genetic_algorithms::LogObserver;
use rand::Rng;

const DIMENSIONS: usize = 10;
const SEARCH_LO: f64 = -5.12;
const SEARCH_HI: f64 = 5.12;

/// Rastrigin function: f(x) = 10·n + Σᵢ (xᵢ² − 10·cos(2πxᵢ))
///
/// Global minimum: f(0,…,0) = 0.0
fn rastrigin(dna: &[RangeGene<f64>]) -> f64 {
    let n = dna.len() as f64;
    10.0 * n
        + dna
            .iter()
            .map(|g| {
                let x = g.real_value();
                x * x - 10.0 * (2.0 * PI * x).cos()
            })
            .sum::<f64>()
}

/// Build an initial population of `n` chromosomes, each with `DIMENSIONS`
/// genes sampled uniformly from `[SEARCH_LO, SEARCH_HI]`.
fn init_population(n: usize) -> Vec<RangeChromosome<f64>> {
    rng::set_seed(Some(42));
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
    let config = CmaConfiguration::default_for_dim(DIMENSIONS)
        .with_sigma0(0.5)
        .with_max_generations(200)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 50,
            max_restarts: 3,
        });

    let mut engine = CmaEngine::new(config, init_population, rastrigin)
        .with_observer(Arc::new(LogObserver));

    let result = engine.run();

    println!("Total restarts: {}", result.total_restarts);
    println!("Generations:    {}", result.generations);
    println!("Best fitness:   {:.6}", result.best_fitness);
    assert!(
        result.best_fitness.is_finite(),
        "Expected finite best fitness"
    );
}
