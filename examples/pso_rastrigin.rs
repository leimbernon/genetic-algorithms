/*!
# PSO: 10D Rastrigin Minimization

Particle Swarm Optimization on the 10-dimensional Rastrigin function — a classic
multimodal benchmark with many local optima and a global minimum of 0 at the origin.

Demonstrates PSO with:
- `LinearDecay` inertia (0.9 → 0.4, Shi & Eberhart 1998)
- `Global` (gbest) topology for fast convergence
- `LogObserver` wired from day one (D-09 observer contract)
- Fitness-target early-stop at 1e-3

Run with:
```sh
cargo run --release --example pso_rastrigin
```
*/

use std::borrow::Cow;
use std::f64::consts::PI;
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};
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
    rng::set_seed(Some(99));
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
    let config = PsoConfiguration {
        population_size: 200,
        max_generations: 1000,
        problem_solving: ProblemSolving::Minimization,
        fitness_target: Some(1e-3),
        inertia: PsoInertia::LinearDecay {
            w_start: 0.9,
            w_end: 0.4,
        },
        c1: 2.0,
        c2: 2.0,
        topology: PsoTopology::Global,
    };

    let mut engine = PsoEngine::new(config, init_population, rastrigin)
        .with_observer(Arc::new(LogObserver));

    println!("== PSO: {DIMENSIONS}D Rastrigin Minimization ==");
    println!("particles=200, max_generations=1000, target=1e-3");
    println!("inertia=LinearDecay(0.9→0.4), c1=2.0, c2=2.0, topology=Global");
    println!("--------------------------------------------------");

    let result = engine.run();

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
