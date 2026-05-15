//! Integration tests for the ALPS (Age-Layered Population Structure) engine.

use std::borrow::Cow;

use genetic_algorithms::alps::{AlpsAgeScheme, AlpsConfiguration, AlpsEngine};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::{Crossover, Mutation};
use genetic_algorithms::rng;
use genetic_algorithms::traits::ChromosomeT;
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

fn make_engine(scheme: AlpsAgeScheme) -> AlpsEngine<RangeChromosome<f64>> {
    let config = AlpsConfiguration::default()
        .with_n_layers(4)
        .with_layer_size(15)
        .with_age_scheme(scheme)
        .with_age_gap(5)
        .with_injection_interval(10)
        .with_max_generations(100)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian)
        .with_mutation_sigma(0.5)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(50.0);

    AlpsEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

// --- Age schemes -------------------------------------------------------------

#[test]
fn test_linear_age_scheme_runs() {
    let mut engine = make_engine(AlpsAgeScheme::Linear);
    let result = engine.run();
    assert!(result.generations > 0);
    assert_eq!(result.layers.len(), 4);
    assert!(result.best_fitness >= 0.0);
}

#[test]
fn test_fibonacci_age_scheme_runs() {
    let mut engine = make_engine(AlpsAgeScheme::Fibonacci);
    let result = engine.run();
    assert!(result.generations > 0);
    assert_eq!(result.layers.len(), 4);
    assert!(result.best_fitness >= 0.0);
}

#[test]
fn test_polynomial_age_scheme_runs() {
    let mut engine = make_engine(AlpsAgeScheme::Polynomial);
    let result = engine.run();
    assert!(result.generations > 0);
    assert_eq!(result.layers.len(), 4);
    assert!(result.best_fitness >= 0.0);
}

// --- Age scheme threshold correctness ----------------------------------------

#[test]
fn test_linear_age_thresholds() {
    let config = AlpsConfiguration::default()
        .with_n_layers(4)
        .with_age_scheme(AlpsAgeScheme::Linear)
        .with_age_gap(5);
    let ages = config.max_ages();
    assert_eq!(ages, vec![5, 10, 15, 20]);
}

#[test]
fn test_fibonacci_age_thresholds() {
    let config = AlpsConfiguration::default()
        .with_n_layers(5)
        .with_age_scheme(AlpsAgeScheme::Fibonacci)
        .with_age_gap(5);
    let ages = config.max_ages();
    // Fibonacci: fib(2..7) = 1,2,3,5,8 — multiplied by age_gap=5
    assert_eq!(ages, vec![5, 10, 15, 25, 40]);
}

#[test]
fn test_polynomial_age_thresholds() {
    let config = AlpsConfiguration::default()
        .with_n_layers(4)
        .with_age_scheme(AlpsAgeScheme::Polynomial)
        .with_age_gap(3);
    let ages = config.max_ages();
    // (i+1)^2 * 3 for i=0..3 → 3, 12, 27, 48
    assert_eq!(ages, vec![3, 12, 27, 48]);
}

// --- Cross-layer mating -------------------------------------------------------

#[test]
fn test_cross_layer_mating_produces_result() {
    // Run long enough that some individuals should be promoted to older layers.
    let config = AlpsConfiguration::default()
        .with_n_layers(3)
        .with_layer_size(10)
        .with_age_scheme(AlpsAgeScheme::Linear)
        .with_age_gap(2)   // short age limits to force promotions quickly
        .with_injection_interval(0) // disable injection to isolate cross-layer behavior
        .with_max_generations(50)
        .with_mutation_sigma(0.5);

    let mut engine = AlpsEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    );
    let result = engine.run();
    assert!(result.generations > 0);
    // Engine should have run without panicking; all layers returned.
    assert_eq!(result.layers.len(), 3);
}

// --- Injection ----------------------------------------------------------------

#[test]
fn test_injection_enabled_runs() {
    let config = AlpsConfiguration::default()
        .with_n_layers(3)
        .with_layer_size(10)
        .with_age_scheme(AlpsAgeScheme::Fibonacci)
        .with_age_gap(3)
        .with_injection_interval(5)
        .with_max_generations(30)
        .with_mutation_sigma(0.5);

    let mut engine = AlpsEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 7),
        sphere,
    );
    let result = engine.run();
    // With injection_interval=5 and 30 generations, layer 0 is reseeded ~5 times.
    assert!(result.generations > 0);
    assert_eq!(result.layers.len(), 3);
}

#[test]
fn test_injection_disabled_runs() {
    let config = AlpsConfiguration::default()
        .with_n_layers(3)
        .with_layer_size(10)
        .with_injection_interval(0) // disabled
        .with_max_generations(30)
        .with_mutation_sigma(0.5);

    let mut engine = AlpsEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 13),
        sphere,
    );
    let result = engine.run();
    assert!(result.generations > 0);
}

// --- Early stopping -----------------------------------------------------------

#[test]
fn test_early_stopping() {
    let config = AlpsConfiguration::default()
        .with_n_layers(3)
        .with_layer_size(10)
        .with_max_generations(100_000)
        .with_mutation_sigma(1.0)
        .with_fitness_target(1_000.0) // trivially reachable
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = AlpsEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 99),
        sphere,
    );
    let result = engine.run();
    assert!(result.generations < 100_000, "expected early stop but ran {} gens", result.generations);
}

// --- Result consistency -------------------------------------------------------

#[test]
fn test_best_fitness_consistent() {
    let mut engine = make_engine(AlpsAgeScheme::Fibonacci);
    let result = engine.run();

    // best_fitness must be <= (or == for maximization) any individual in all layers.
    let all_fitnesses: Vec<f64> = result.layers.iter().flat_map(|l| l.iter().map(|u| u.fitness())).collect();
    let pop_best = all_fitnesses.iter().cloned().fold(f64::MAX, f64::min);

    // best_fitness may be better than current population because injection
    // can replace it — but it should never be worse than the reported best.
    assert!(
        result.best_fitness <= pop_best + 1e-9,
        "reported best {} is worse than population best {}",
        result.best_fitness, pop_best
    );
}
