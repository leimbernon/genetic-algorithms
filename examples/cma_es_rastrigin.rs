/*!
# CMA-ES: Rastrigin Minimization

CMA-ES on the 5-dimensional Rastrigin function — a classic multimodal benchmark
with many local optima. CMA-ES typically escapes the local minima where a plain
GA struggles, because it adapts the full covariance matrix of the search
distribution.

Run with:
```sh
cargo run --example cma_es_rastrigin
```
*/

use std::f64::consts::PI;
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::{LinearChromosome, RealGene};
use genetic_algorithms::LogObserver;
use rand::Rng;
use std::borrow::Cow;

const DIMENSIONS: usize = 5;
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
    // Seed must be set by the caller (main) before invoking init_population.
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
    // Parse optional --seed <N> argument for reproducible runs (used by build_perf.sh golden capture).
    // Falls back to seed 42 when not specified (original behaviour).
    let args: Vec<String> = std::env::args().collect();
    let seed = args
        .iter()
        .position(|a| a == "--seed")
        .and_then(|pos| args.get(pos + 1))
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(42);
    rng::set_seed(Some(seed));

    let config = CmaConfiguration::default_for_dim(DIMENSIONS)
        .with_sigma0(0.5)
        .with_max_generations(300)
        .with_fitness_target(1e-3)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine =
        CmaEngine::new(config, init_population, rastrigin).with_observer(Arc::new(LogObserver));

    println!("== CMA-ES: {DIMENSIONS}D Rastrigin Minimization ==");
    println!("sigma0=0.5, max_generations=300, target=1e-3");
    println!("--------------------------------------------------");

    let result = engine.run().expect("engine run should succeed");

    println!("Generations: {}", result.generations);
    println!("Best fitness: {:.6}", result.best_fitness);
    let dna_str: Vec<String> = result
        .best
        .dna()
        .iter()
        .map(|g| format!("{:.4}", g.real_value()))
        .collect();
    println!("Best DNA:    [{}]", dna_str.join(", "));
    assert!(
        result.best_fitness.is_finite(),
        "best_fitness must be finite"
    );
}
