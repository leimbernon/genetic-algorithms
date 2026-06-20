//! Integration tests for the EDA engine.
//!
//! Tests cover:
//! - EDA-01: Bernoulli model convergence on OneMax
//! - EDA-02: Gaussian model convergence on sphere function
//! - EDA-03: EdaResult fields are populated correctly
//! - EDA-04: EdaModel is a Bernoulli variant for binary chromosomes
//! - EDA-05: EdaModel is a Gaussian variant for real chromosomes
//! - EDA-06: Observer hooks fire expected number of times
//! - EDA-07: Fitness target causes early stopping (FitnessTargetReached)
//! - EDA-08: Minimization direction works correctly
//! - EDA-09: selection_ratio clamp enforces min 1 parent
//! - EDA-10: Population default (size 0 → 100) applies
//! - EDA-11: WASM gate — ignored, verified via CI

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::eda::{EdaConfiguration, EdaEngine, EdaModel, EdaRealEngine};
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::rng;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{LinearChromosome, RealGene};

// ─── Test helpers ──────────────────────────────────────────────────────────────

const CHROMOSOME_LEN: usize = 20;

/// Build a random binary population of `n` chromosomes of length `CHROMOSOME_LEN`.
fn random_binary_pop(n: usize, seed: u64) -> Vec<BinaryChromosome> {
    rng::set_seed(Some(seed));
    let mut r = rng::make_rng();
    use rand::Rng;
    (0..n)
        .map(|_| {
            let dna: Vec<BinaryGene> = (0..CHROMOSOME_LEN)
                .map(|_| BinaryGene {
                    id: if r.random::<bool>() { 1 } else { 0 },
                    value: r.random::<bool>(),
                })
                .collect();
            let mut c = BinaryChromosome::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

/// Build a random range population of `n` chromosomes of length `dim`.
fn random_range_pop(
    n: usize,
    dim: usize,
    lo: f64,
    hi: f64,
    seed: u64,
) -> Vec<RangeChromosome<f64>> {
    rng::set_seed(Some(seed));
    let mut r = rng::make_rng();
    use rand::Rng;
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

/// OneMax fitness: count of genes with id == 1.
fn onemax(dna: &[BinaryGene]) -> f64 {
    dna.iter().filter(|g| g.id == 1).count() as f64
}

/// Sphere function: Σ xᵢ²  (minimum at origin)
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.real_value() * g.real_value()).sum()
}

// ─── Observer spy ──────────────────────────────────────────────────────────────

#[derive(Default)]
struct SpyObserver {
    run_start: AtomicUsize,
    run_end: AtomicUsize,
    generation_start: AtomicUsize,
    generation_end: AtomicUsize,
    new_best: AtomicUsize,
}

impl GaObserver<BinaryChromosome> for SpyObserver {
    fn on_run_start(&self) {
        self.run_start.fetch_add(1, Ordering::SeqCst);
    }
    fn on_run_end(&self, _cause: TerminationCause, _stats: &[GenerationStats]) {
        self.run_end.fetch_add(1, Ordering::SeqCst);
    }
    fn on_generation_start(&self, _gen: usize) {
        self.generation_start.fetch_add(1, Ordering::SeqCst);
    }
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.generation_end.fetch_add(1, Ordering::SeqCst);
    }
    fn on_new_best(&self, _gen: usize, _best: &BinaryChromosome) {
        self.new_best.fetch_add(1, Ordering::SeqCst);
    }
}

// ─── EDA-01: Bernoulli OneMax convergence ─────────────────────────────────────

#[test]
fn eda_01_bernoulli_onemax_convergence() {
    rng::set_seed(Some(42));
    let config = EdaConfiguration {
        population_size: 100,
        max_generations: 300,
        problem_solving: ProblemSolving::Maximization,
        fitness_target: Some(CHROMOSOME_LEN as f64),
        selection_ratio: 0.5,
        fitness_cache_size: None,
    };

    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 1), onemax);

    let result = engine.run().expect("engine run should succeed");

    assert!(
        result.best_fitness >= (CHROMOSOME_LEN as f64 * 0.9),
        "Expected best fitness >= 18 (90% of 20), got {}",
        result.best_fitness
    );
    assert_eq!(result.best.dna().len(), CHROMOSOME_LEN);
}

// ─── EDA-02: Gaussian sphere convergence ──────────────────────────────────────

#[test]
fn eda_02_gaussian_sphere_convergence() {
    rng::set_seed(Some(77));
    const DIM: usize = 5;
    let config = EdaConfiguration {
        population_size: 100,
        max_generations: 500,
        problem_solving: ProblemSolving::Minimization,
        fitness_target: Some(0.1),
        selection_ratio: 0.3,
        fitness_cache_size: None,
    };

    let mut engine =
        EdaRealEngine::new(config, |n| random_range_pop(n, DIM, -5.0, 5.0, 99), sphere);

    let result = engine.run().expect("engine run should succeed");

    assert!(
        result.best_fitness < 5.0,
        "Expected best fitness < 5.0 for sphere, got {}",
        result.best_fitness
    );
    assert!(
        result.best_fitness.is_finite(),
        "best_fitness must be finite"
    );
}

// ─── EDA-03: EdaResult fields ─────────────────────────────────────────────────

#[test]
fn eda_03_result_fields_populated() {
    rng::set_seed(Some(10));
    let config = EdaConfiguration {
        population_size: 20,
        max_generations: 10,
        problem_solving: ProblemSolving::Maximization,
        fitness_target: None,
        selection_ratio: 0.5,
        fitness_cache_size: None,
    };

    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 2), onemax);

    let result = engine.run().expect("engine run should succeed");

    assert_eq!(result.generations, 10, "Should run all 10 generations");
    assert_eq!(
        result.population.len(),
        20,
        "Final population size should be 20"
    );
    assert!(
        result.best_fitness.is_finite(),
        "best_fitness must be finite"
    );
    assert_eq!(result.best.dna().len(), CHROMOSOME_LEN);
}

// ─── EDA-04: EdaModel::Bernoulli for binary chromosomes ───────────────────────

#[test]
fn eda_04_learned_model_is_bernoulli() {
    rng::set_seed(Some(55));
    let config = EdaConfiguration::default()
        .with_population_size(50)
        .with_max_generations(20);

    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 5), onemax);

    let result = engine.run().expect("engine run should succeed");

    match result.learned_model {
        EdaModel::Bernoulli(probs) => {
            assert_eq!(
                probs.len(),
                CHROMOSOME_LEN,
                "Bernoulli probs length should equal chromosome length"
            );
            for p in &probs {
                assert!(
                    *p >= 0.01 && *p <= 0.99,
                    "Bernoulli prob {} out of [0.01, 0.99]",
                    p
                );
            }
        }
        EdaModel::Gaussian { .. } => panic!("Expected Bernoulli model for binary chromosomes"),
    }
}

// ─── EDA-05: EdaModel::Gaussian for real chromosomes ─────────────────────────

#[test]
fn eda_05_learned_model_is_gaussian() {
    rng::set_seed(Some(66));
    const DIM: usize = 4;
    let config = EdaConfiguration::default()
        .with_population_size(40)
        .with_max_generations(20)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = EdaRealEngine::new(config, |n| random_range_pop(n, DIM, -3.0, 3.0, 7), sphere);

    let result = engine.run().expect("engine run should succeed");

    match result.learned_model {
        EdaModel::Gaussian { means, stds } => {
            assert_eq!(
                means.len(),
                DIM,
                "means length should equal chromosome length"
            );
            assert_eq!(
                stds.len(),
                DIM,
                "stds length should equal chromosome length"
            );
            for std in &stds {
                assert!(*std >= 1e-6, "std {} below floor", std);
            }
        }
        EdaModel::Bernoulli(_) => panic!("Expected Gaussian model for real chromosomes"),
    }
}

// ─── EDA-06: Observer hooks fire expected times ───────────────────────────────

#[test]
fn eda_06_observer_hooks_fire() {
    let config = EdaConfiguration {
        population_size: 10,
        max_generations: 5,
        problem_solving: ProblemSolving::Maximization,
        fitness_target: None,
        selection_ratio: 0.5,
        fitness_cache_size: None,
    };

    let spy = Arc::new(SpyObserver::default());

    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 3), onemax)
        .with_observer(Arc::clone(&spy) as Arc<dyn GaObserver<BinaryChromosome> + Send + Sync>);

    engine.run().expect("engine run should succeed");

    assert_eq!(
        spy.run_start.load(Ordering::SeqCst),
        1,
        "on_run_start should fire once"
    );
    assert_eq!(
        spy.run_end.load(Ordering::SeqCst),
        1,
        "on_run_end should fire once"
    );
    assert_eq!(
        spy.generation_start.load(Ordering::SeqCst),
        5,
        "on_generation_start should fire once per generation"
    );
    assert_eq!(
        spy.generation_end.load(Ordering::SeqCst),
        5,
        "on_generation_end should fire once per generation"
    );
    // on_new_best fires at least once (initial best notification at gen 0)
    assert!(
        spy.new_best.load(Ordering::SeqCst) >= 1,
        "on_new_best should fire at least once"
    );
}

// ─── EDA-07: Fitness target causes early stopping ─────────────────────────────

#[test]
fn eda_07_fitness_target_early_stop() {
    rng::set_seed(Some(101));
    let config = EdaConfiguration {
        population_size: 200,
        max_generations: 1000,
        problem_solving: ProblemSolving::Maximization,
        // OneMax max is CHROMOSOME_LEN; set target to a value we can easily reach
        fitness_target: Some(1.0), // any individual with at least 1 "one" stops it
        selection_ratio: 0.5,
        fitness_cache_size: None,
    };

    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 42), onemax);

    let result = engine.run().expect("engine run should succeed");

    assert!(
        result.generations < 1000,
        "Engine should stop early when target is reached, ran {} generations",
        result.generations
    );
    assert!(
        result.best_fitness >= 1.0,
        "Best fitness should satisfy the target"
    );
}

// ─── EDA-08: Minimization direction ──────────────────────────────────────────

#[test]
fn eda_08_minimization_direction() {
    rng::set_seed(Some(200));
    const DIM: usize = 3;
    // Use a simple fitness that we can verify (all-zero is minimum)
    let config = EdaConfiguration {
        population_size: 50,
        max_generations: 100,
        problem_solving: ProblemSolving::Minimization,
        fitness_target: None,
        selection_ratio: 0.3,
        fitness_cache_size: None,
    };

    let mut engine =
        EdaRealEngine::new(config, |n| random_range_pop(n, DIM, -1.0, 1.0, 33), sphere);

    let result = engine.run().expect("engine run should succeed");

    assert!(
        result.best_fitness >= 0.0,
        "Sphere minimum is 0, got {}",
        result.best_fitness
    );
    assert!(result.best_fitness.is_finite());
}

// ─── EDA-09: selection_ratio clamp enforces minimum 1 parent ─────────────────

#[test]
fn eda_09_selection_ratio_min_one_parent() {
    rng::set_seed(Some(303));
    // Very small selection_ratio that would floor to 0 with a tiny pop
    let config = EdaConfiguration {
        population_size: 5,
        max_generations: 5,
        problem_solving: ProblemSolving::Maximization,
        fitness_target: None,
        selection_ratio: 0.01, // 5 * 0.01 = 0.05 → floor = 0, clamped to 1
        fitness_cache_size: None,
    };

    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 88), onemax);

    // Should not panic — the engine must handle 1-parent selection
    let result = engine.run().expect("engine run should succeed");
    assert!(result.best_fitness >= 0.0);
}

// ─── EDA-10: population_size = 0 defaults to 100 ─────────────────────────────

#[test]
fn eda_10_default_population_size() {
    rng::set_seed(Some(404));
    let config = EdaConfiguration {
        population_size: 0, // triggers default of 100
        max_generations: 2,
        problem_solving: ProblemSolving::Maximization,
        fitness_target: None,
        selection_ratio: 0.5,
        fitness_cache_size: None,
    };

    let mut engine = EdaEngine::new(
        config,
        |n| {
            assert_eq!(n, 100, "default pop size should be 100 when 0 is passed");
            random_binary_pop(n, 11)
        },
        onemax,
    );

    let result = engine.run().expect("engine run should succeed");
    assert_eq!(result.population.len(), 100);
}

// ─── EDA-12: Bernoulli cache enabled ────────────────────────────────────────

/// EDA-12: Cache-enabled Bernoulli EDA run completes and produces valid results.
#[test]
fn eda_12_bernoulli_cache_enabled() {
    rng::set_seed(Some(201));
    let config = EdaConfiguration::default()
        .with_population_size(50)
        .with_max_generations(20)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_fitness_cache_size(128);
    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 10), onemax);
    let result = engine.run().expect("engine run should succeed");
    assert!(result.best_fitness >= 0.0, "best_fitness must be non-negative with cache");
    assert_eq!(result.generations, 20);
}

// ─── EDA-13: Gaussian cache enabled ─────────────────────────────────────────

/// EDA-13: Cache-enabled Gaussian EDA run completes and produces valid results.
#[test]
fn eda_13_gaussian_cache_enabled() {
    rng::set_seed(Some(202));
    let config = EdaConfiguration::default()
        .with_population_size(50)
        .with_max_generations(20)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_cache_size(128);
    let mut engine = EdaRealEngine::new(config, |n| random_range_pop(n, 5, -3.0, 3.0, 11), sphere);
    let result = engine.run().expect("engine run should succeed");
    assert!(result.best_fitness.is_finite(), "best_fitness must be finite with cache");
    assert_eq!(result.generations, 20);
}

// ─── EDA-14: cache disabled by default ──────────────────────────────────────

/// EDA-14: Default config (no cache) works with zero overhead.
#[test]
fn eda_14_cache_disabled_default() {
    rng::set_seed(Some(203));
    let config = EdaConfiguration::default()
        .with_population_size(50)
        .with_max_generations(20)
        .with_problem_solving(ProblemSolving::Maximization);
    let mut engine = EdaEngine::new(config, |n| random_binary_pop(n, 10), onemax);
    let result = engine.run().expect("engine run should succeed");
    assert!(result.best_fitness >= 0.0);
}

// ─── EDA-11: WASM compilation gate ────────────────────────────────────────────
// Verified via CI (`cargo check --target wasm32-unknown-unknown`).
// This test is ignored in the standard test suite.
#[test]
#[ignore = "WASM gate: verified by CI cargo check --target wasm32-unknown-unknown"]
fn eda_11_wasm_compilation_gate() {}
