//! Integration tests for the Scatter Search engine.

use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::scatter::{ScatterConfiguration, ScatterEngine};
use genetic_algorithms::traits::LinearChromosome;
use rand::Rng;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

fn random_pop(n: usize, dim: usize, lo: f64, hi: f64, seed: u64) -> Vec<RangeChromosome<f64>> {
    rng::set_seed(Some(seed));
    let mut r = rng::make_rng();
    (0..n)
        .map(|_| {
            let dna: Vec<RangeGene<f64>> = (0..dim)
                .map(|j| {
                    let v = r.random::<f64>() * (hi - lo) + lo;
                    RangeGene::new(j as i32, vec![(lo, hi)], v)
                })
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_scatter_basic_convergence() {
    let config = ScatterConfiguration::default()
        .with_population_size(30)
        .with_reference_set_size(6)
        .with_max_iterations(50)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);
    let result = engine.run();

    assert!(!result.reference_set.is_empty());
    assert!(result.iterations > 0);
    // Sphere starts with max fitness ~125 (5² × 5 dims); should improve
    assert!(result.best_fitness < 125.0);
}

#[test]
fn test_scatter_reference_set_maintained() {
    let config = ScatterConfiguration::default()
        .with_population_size(40)
        .with_reference_set_size(8)
        .with_max_iterations(20);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 3, -5.0, 5.0, 7), sphere);
    let result = engine.run();

    assert_eq!(result.reference_set.len(), 8);
}

#[test]
fn test_scatter_with_local_search() {
    let config = ScatterConfiguration::default()
        .with_population_size(30)
        .with_reference_set_size(6)
        .with_max_iterations(30)
        .with_local_search(true)
        .with_local_search_steps(10)
        .with_local_search_step_size(0.5)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 13), sphere);
    let result = engine.run();

    // Local search enabled — should achieve good results
    assert!(!result.reference_set.is_empty());
    assert!(result.best_fitness < 125.0);
}

#[test]
fn test_scatter_without_local_search() {
    let config = ScatterConfiguration::default()
        .with_population_size(30)
        .with_reference_set_size(6)
        .with_max_iterations(30)
        .with_local_search(false);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 21), sphere);
    let result = engine.run();

    assert!(!result.reference_set.is_empty());
}

#[test]
fn test_scatter_maximization() {
    let config = ScatterConfiguration::default()
        .with_population_size(30)
        .with_reference_set_size(6)
        .with_max_iterations(30)
        .with_problem_solving(ProblemSolving::Maximization);

    // Maximize -sphere (maximum = 0)
    let mut engine = ScatterEngine::new(
        config,
        |n| random_pop(n, 3, -5.0, 5.0, 99),
        |dna| -sphere(dna),
    );
    let result = engine.run();

    assert!(result.best_fitness > -75.0);
}

#[test]
fn test_scatter_early_stopping() {
    let config = ScatterConfiguration::default()
        .with_population_size(20)
        .with_reference_set_size(4)
        .with_max_iterations(10_000)
        .with_fitness_target(1_000.0) // trivially reachable immediately
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 3), sphere);
    let result = engine.run();

    assert!(result.iterations < 10_000);
}

#[test]
fn test_scatter_result_fields() {
    let config = ScatterConfiguration::default()
        .with_population_size(20)
        .with_reference_set_size(4)
        .with_max_iterations(10);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 3, -5.0, 5.0, 5), sphere);
    let result = engine.run();

    // best_fitness must match recomputed sphere on best individual
    let recomputed = sphere(result.best.dna());
    assert!((recomputed - result.best_fitness).abs() < 1e-9);
    assert!(result.iterations > 0);
}

/// Convergence regression test: Scatter must reach sphere minimum < 1.0
/// on 5 dimensions within 500 iterations. Prevents silent regressions in search dynamics.
#[test]
fn test_scatter_convergence() {
    let config = ScatterConfiguration::default()
        .with_population_size(30)
        .with_reference_set_size(6)
        .with_max_iterations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0);

    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);
    let result = engine.run();

    assert!(
        result.best_fitness < 1.0,
        "Scatter should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
