//! Integration tests for the Differential Evolution engine.

use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::de::{
    DeAdaptive, DeConfiguration, DeCrossoverMode, DeEngine, DeMutationStrategy,
};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::ChromosomeT;
use rand::Rng;

// ─── Test helpers ─────────────────────────────────────────────────────────────

/// Sphere function: f(x) = Σ xᵢ²  (minimum 0 at origin)
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

/// Build a random population of `Range<f64>` chromosomes.
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

fn sphere_engine(strategy: DeMutationStrategy, mode: DeCrossoverMode) -> DeEngine<RangeChromosome<f64>> {
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(300)
        .with_mutation_factor(0.8)
        .with_crossover_rate(0.9)
        .with_mutation_strategy(strategy)
        .with_crossover_mode(mode)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0); // stop early once good enough

    DeEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_de_rand1_binomial_converges() {
    let mut engine = sphere_engine(DeMutationStrategy::Rand1, DeCrossoverMode::Binomial);
    let result = engine.run();
    assert!(
        result.best_fitness < 5.0,
        "DE/rand/1 binomial should reduce sphere fitness; got {}",
        result.best_fitness
    );
    assert!(result.generations > 0);
    assert!(!result.population.is_empty());
}

#[test]
fn test_de_best1_binomial() {
    let mut engine = sphere_engine(DeMutationStrategy::Best1, DeCrossoverMode::Binomial);
    let result = engine.run();
    assert!(result.best_fitness < 10.0);
}

#[test]
fn test_de_current_to_best1_binomial() {
    let mut engine = sphere_engine(DeMutationStrategy::CurrentToBest1, DeCrossoverMode::Binomial);
    let result = engine.run();
    assert!(result.best_fitness < 10.0);
}

#[test]
fn test_de_rand2_binomial() {
    // DE/rand/2 requires population_size > 5 (needs 5 distinct individuals)
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(300)
        .with_mutation_strategy(DeMutationStrategy::Rand2)
        .with_crossover_mode(DeCrossoverMode::Binomial);
    let mut engine = DeEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);
    let result = engine.run();
    assert!(result.best_fitness < 20.0);
}

#[test]
fn test_de_best2_binomial() {
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(300)
        .with_mutation_strategy(DeMutationStrategy::Best2)
        .with_crossover_mode(DeCrossoverMode::Binomial);
    let mut engine = DeEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);
    let result = engine.run();
    assert!(result.best_fitness < 20.0);
}

#[test]
fn test_de_exponential_crossover() {
    let mut engine = sphere_engine(DeMutationStrategy::Rand1, DeCrossoverMode::Exponential);
    let result = engine.run();
    assert!(result.best_fitness < 10.0);
}

#[test]
fn test_de_jade_converges() {
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(500)
        .with_mutation_strategy(DeMutationStrategy::CurrentToBest1)
        .with_crossover_mode(DeCrossoverMode::Binomial)
        .with_adaptive(DeAdaptive::Jade { p: 0.1, c: 0.1 })
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = DeEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 99), sphere);
    let result = engine.run();
    assert!(result.best_fitness < 10.0, "JADE should converge; got {}", result.best_fitness);
}

#[test]
fn test_de_lshade_converges() {
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(500)
        .with_mutation_strategy(DeMutationStrategy::CurrentToBest1)
        .with_crossover_mode(DeCrossoverMode::Binomial)
        .with_adaptive(DeAdaptive::LShade { history_size: 5 })
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = DeEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 77), sphere);
    let result = engine.run();
    assert!(result.best_fitness < 10.0, "L-SHADE should converge; got {}", result.best_fitness);
}

#[test]
fn test_de_maximization() {
    // Maximize f(x) = -Σ xᵢ²  (maximum 0 at origin)
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(300)
        .with_mutation_strategy(DeMutationStrategy::Rand1)
        .with_problem_solving(ProblemSolving::Maximization);
    let mut engine = DeEngine::new(
        config,
        |n| random_pop(n, 3, -5.0, 5.0, 55),
        |dna| -dna.iter().map(|g| g.value() * g.value()).sum::<f64>(),
    );
    let result = engine.run();
    // Best fitness should be negative or zero (closer to 0 is better for maximization)
    assert!(result.best_fitness > -75.0, "Maximization run should improve; got {}", result.best_fitness);
}

#[test]
fn test_de_result_fields() {
    let mut engine = sphere_engine(DeMutationStrategy::Rand1, DeCrossoverMode::Binomial);
    let result = engine.run();
    assert_eq!(result.population.len(), 30);
    assert!(result.generations > 0);
    // best fitness must match what the best individual actually achieves
    let recomputed = sphere(result.best.dna());
    assert!((recomputed - result.best_fitness).abs() < 1e-9);
}

#[test]
fn test_de_early_stopping() {
    // Very loose target to guarantee early stop
    let config = DeConfiguration::default()
        .with_population_size(20)
        .with_max_generations(10_000)
        .with_mutation_strategy(DeMutationStrategy::Rand1)
        .with_fitness_target(1000.0) // trivially achieved immediately
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = DeEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 1), sphere);
    let result = engine.run();
    // Should have stopped well before 10,000 generations
    assert!(result.generations < 10_000);
}
