//! End-to-end integration tests for Phase 51: multi-parent crossover (UNDX/SPX/PCX)
//! and self-adaptive Gaussian mutation wired into `Ga<RangeChromosome<f64>>.run()`.
//!
//! These tests exercise the full execution loop (selection → crossover → mutation →
//! survivor → fitness) and confirm that:
//! - UNDX/SPX/PCX complete all generations without CrossoverError
//! - SelfAdaptiveGaussian produces measurable sigma drift from the lazy-init value
//! - Strategy parameters survive a serde round-trip (serde-gated test)

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{
    Crossover, GaussianParams, Mutation, Selection, SelfAdaptiveGaussianParams, Survivor,
};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::ChromosomeLength;

// ─── Shared helpers ───────────────────────────────────────────────────────────

/// Sphere fitness for minimization: returns `-sum(x_i^2)` so that larger values
/// correspond to chromosomes closer to the origin. The engine treats higher fitness
/// as better (maximization), so we negate the sum to minimize the sphere.
fn sphere_fitness(dna: &[RangeGenotype<f64>]) -> f64 {
    -dna.iter().map(|g| g.value * g.value).sum::<f64>()
}

/// Builds a `Ga<RangeChromosome<f64>>` configured for the sphere function with the
/// given crossover method. Population: 20, chromosomes: 5 genes in [-5.0, 5.0],
/// 30 generations, crossover probability 0.9, mutation probability 0.1.
fn build_ga_with_crossover(method: Crossover) -> Ga<RangeChromosome<f64>> {
    let alleles = vec![RangeGenotype::new(0, vec![(-5.0_f64, 5.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();
    Ga::new()
        .with_population_size(20)
        .with_chromosome_length(ChromosomeLength::Fixed(5))
        .with_initialization_fn(move |n, _| range_random_initialization(n, Some(&alleles_clone)))
        .with_fitness_fn(sphere_fitness)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(method)
        .with_crossover_probability_max(0.9)
        .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: None }))
        .with_mutation_probability_max(0.1)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(30)
        .with_alleles(alleles)
        .build()
        .expect("Failed to build Ga")
}

// ─── Multi-parent crossover tests ────────────────────────────────────────────

#[test]
fn end_to_end_undx_runs_without_panic() {
    let mut ga = build_ga_with_crossover(Crossover::Undx { num_parents: 4 });
    let result = ga.run();
    assert!(
        result.is_ok(),
        "Ga.run() with UNDX returned error: {:?}",
        result.err()
    );
    let population = result.unwrap();
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness from UNDX run is NaN or Inf: {}",
        best.fitness()
    );
}

#[test]
fn end_to_end_spx_runs_without_panic() {
    let mut ga = build_ga_with_crossover(Crossover::Spx { num_parents: 4 });
    let result = ga.run();
    assert!(
        result.is_ok(),
        "Ga.run() with SPX returned error: {:?}",
        result.err()
    );
    let population = result.unwrap();
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness from SPX run is NaN or Inf: {}",
        best.fitness()
    );
}

#[test]
fn end_to_end_pcx_runs_without_panic() {
    let mut ga = build_ga_with_crossover(Crossover::Pcx { num_parents: 4 });
    let result = ga.run();
    assert!(
        result.is_ok(),
        "Ga.run() with PCX returned error: {:?}",
        result.err()
    );
    let population = result.unwrap();
    let best = &population.best_chromosome;
    assert!(
        best.fitness().is_finite(),
        "Best fitness from PCX run is NaN or Inf: {}",
        best.fitness()
    );
}

// ─── SelfAdaptiveGaussian test ────────────────────────────────────────────────

#[test]
fn end_to_end_self_adaptive_gaussian_sigmas_evolve() {
    let alleles = vec![RangeGenotype::new(0, vec![(-5.0_f64, 5.0_f64)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
        .with_population_size(20)
        .with_chromosome_length(ChromosomeLength::Fixed(5))
        .with_initialization_fn(move |n, _| range_random_initialization(n, Some(&alleles_clone)))
        .with_fitness_fn(sphere_fitness)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_crossover_probability_max(0.9)
        .with_mutation_method(Mutation::SelfAdaptiveGaussian(SelfAdaptiveGaussianParams {
            tau: None,
            tau_prime: None,
            sigma_min: None,
            sigma_max: None,
        }))
        .with_mutation_probability_max(0.9) // high probability to ensure mutations fire
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(50)
        .with_alleles(alleles)
        .build()
        .expect("Failed to build Ga for SelfAdaptiveGaussian test");

    let result = ga.run();
    assert!(
        result.is_ok(),
        "Ga.run() with SelfAdaptiveGaussian returned error: {:?}",
        result.err()
    );

    let population = result.unwrap();

    // After 50 generations with high mutation probability (0.9), at least one individual
    // in the population should have sigma values that have drifted from 1.0.
    // We check the entire population because the best chromosome may have remained
    // a parent that was never selected as an offspring (and thus never mutated in place).
    let any_sigma_drifted = population
        .chromosomes
        .iter()
        .any(|c| c.strategy_params.iter().any(|&s| (s - 1.0).abs() > 1e-9));
    assert!(
        any_sigma_drifted,
        "No sigma drift detected in any chromosome after 50 generations. \
         All strategy_params vectors are at the lazy-init value of 1.0."
    );

    // All sigmas in all population chromosomes must respect the sigma_min floor (1e-5)
    let all_above_min = population
        .chromosomes
        .iter()
        .all(|c| c.strategy_params.iter().all(|&s| s >= 1e-5));
    assert!(
        all_above_min,
        "At least one sigma is below sigma_min 1e-5 in the final population."
    );
}

// ─── Serde round-trip test (serde feature gate) ───────────────────────────────

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip_strategy_params_preserved() {
    use genetic_algorithms::traits::{LinearChromosome, SelfAdaptive};
    use std::borrow::Cow;

    let mut chromosome = RangeChromosome::<f64>::new();
    let dna = vec![
        RangeGenotype::new(0, vec![(-5.0, 5.0)], 1.0),
        RangeGenotype::new(1, vec![(-5.0, 5.0)], -2.0),
        RangeGenotype::new(2, vec![(-5.0, 5.0)], 0.5),
    ];
    chromosome.set_dna(Cow::Owned(dna));
    // Set a valid fitness (NaN serializes as `null` in JSON which serde_json cannot
    // deserialize back as f64 — the existing serde test sets fitness to a concrete value)
    chromosome.set_fitness(0.5);
    // Explicitly set strategy params
    chromosome.set_strategy_params(vec![0.2, 0.5, 0.7]);

    let json = serde_json::to_string(&chromosome).expect("Serialization failed");
    let restored: RangeChromosome<f64> =
        serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(
        restored.strategy_params(),
        &[0.2, 0.5, 0.7],
        "Strategy params should survive serde round-trip"
    );
}
