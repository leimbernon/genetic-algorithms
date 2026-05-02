//! Integration tests for the Scatter Search engine.

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::rng;
use genetic_algorithms::scatter::{ScatterConfiguration, ScatterEngine};
use genetic_algorithms::stats::GenerationStats;
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

// ─── SpyObserver for observer tests ──────────────────────────────────────────

#[derive(Default)]
struct SpyData {
    run_start: AtomicUsize,
    generation_start: AtomicUsize,
    new_best: AtomicUsize,
    generation_end: AtomicUsize,
    run_end: AtomicUsize,
    run_end_cause: Mutex<Option<TerminationCause>>,
    run_end_stats_len: AtomicUsize,
}

struct SpyObserver {
    data: Arc<SpyData>,
}

impl GaObserver<RangeChromosome<f64>> for SpyObserver {
    fn on_run_start(&self) {
        self.data.run_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_start(&self, _g: usize) {
        self.data.generation_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_new_best(&self, _g: usize, _best: RangeChromosome<f64>) {
        self.data.new_best.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.data.generation_end.fetch_add(1, Ordering::Relaxed);
    }
    fn on_run_end(&self, cause: TerminationCause, all_stats: &[GenerationStats]) {
        self.data.run_end.fetch_add(1, Ordering::Relaxed);
        *self.data.run_end_cause.lock().unwrap() = Some(cause);
        self.data.run_end_stats_len.store(all_stats.len(), Ordering::Relaxed);
    }
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

#[test]
fn test_scatter_observer_fires_5_hooks() {
    let max_iters = 10usize;
    let data = Arc::new(SpyData::default());
    let spy = Arc::new(SpyObserver { data: Arc::clone(&data) });

    let config = ScatterConfiguration::default()
        .with_population_size(20)
        .with_reference_set_size(4)
        .with_max_iterations(max_iters)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 3, -5.0, 5.0, 42), sphere)
        .with_observer(spy);
    engine.run();

    assert_eq!(data.run_start.load(Ordering::Relaxed), 1);
    assert_eq!(data.generation_start.load(Ordering::Relaxed), max_iters);
    assert_eq!(data.generation_end.load(Ordering::Relaxed), max_iters);
    assert_eq!(data.run_end.load(Ordering::Relaxed), 1);
    assert_eq!(
        *data.run_end_cause.lock().unwrap(),
        Some(TerminationCause::GenerationLimitReached)
    );
    assert_eq!(data.run_end_stats_len.load(Ordering::Relaxed), max_iters);
    assert!(data.new_best.load(Ordering::Relaxed) >= 1);
}

#[test]
fn test_scatter_no_observer_no_panic() {
    let config = ScatterConfiguration::default()
        .with_population_size(10)
        .with_reference_set_size(4)
        .with_max_iterations(5)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = ScatterEngine::new(config, |n| random_pop(n, 3, -5.0, 5.0, 1), sphere);
    engine.run(); // no observer attached — must not panic
}
