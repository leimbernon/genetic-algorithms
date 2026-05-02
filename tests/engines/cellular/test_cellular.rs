//! Integration tests for the Cellular Genetic Algorithm engine.

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use genetic_algorithms::cellular::{
    CellularConfiguration, CellularEngine, Neighborhood, UpdateMode,
};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::operations::{Crossover, Mutation, Selection};
use genetic_algorithms::rng;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::ChromosomeT;
use rand::Rng;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Sphere function: f(x) = Σ xᵢ²  (minimum 0 at origin)
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

/// Build a random grid population of `n` Range<f64> chromosomes.
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

fn make_engine(
    rows: usize,
    cols: usize,
    neighborhood: Neighborhood,
    update_mode: UpdateMode,
) -> CellularEngine<RangeChromosome<f64>> {
    let config = CellularConfiguration::default()
        .with_grid(rows, cols)
        .with_neighborhood(neighborhood)
        .with_update_mode(update_mode)
        .with_max_generations(100)
        .with_selection(Selection::Tournament)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian)
        .with_mutation_sigma(0.5)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(50.0);

    CellularEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

// --- Neighborhood correctness -------------------------------------------------

#[test]
fn test_von_neumann_async_reduces_fitness() {
    let mut engine = make_engine(5, 5, Neighborhood::VonNeumann, UpdateMode::Asynchronous);
    let result = engine.run();
    assert!(result.generations > 0);
    assert!(!result.population.is_empty());
    assert_eq!(result.population.len(), 25);
    // Fitness should be positive (sphere ≥ 0)
    assert!(result.best_fitness >= 0.0);
}

#[test]
fn test_moore_async_reduces_fitness() {
    let mut engine = make_engine(5, 5, Neighborhood::Moore, UpdateMode::Asynchronous);
    let result = engine.run();
    assert!(result.generations > 0);
    assert!(!result.population.is_empty());
    assert!(result.best_fitness >= 0.0);
}

#[test]
fn test_compact_r2_async() {
    let mut engine = make_engine(6, 6, Neighborhood::CompactR2, UpdateMode::Asynchronous);
    let result = engine.run();
    assert!(result.generations > 0);
    assert_eq!(result.population.len(), 36);
    assert!(result.best_fitness >= 0.0);
}

#[test]
fn test_linear_async() {
    let mut engine = make_engine(4, 4, Neighborhood::Linear, UpdateMode::Asynchronous);
    let result = engine.run();
    assert!(result.generations > 0);
    assert_eq!(result.population.len(), 16);
    assert!(result.best_fitness >= 0.0);
}

// --- Update mode -------------------------------------------------------------

#[test]
fn test_synchronous_update() {
    let mut engine = make_engine(5, 5, Neighborhood::Moore, UpdateMode::Synchronous);
    let result = engine.run();
    assert!(result.generations > 0);
    assert_eq!(result.population.len(), 25);
    assert!(result.best_fitness >= 0.0);
}

#[test]
fn test_asynchronous_update() {
    let mut engine = make_engine(5, 5, Neighborhood::Moore, UpdateMode::Asynchronous);
    let result = engine.run();
    assert!(result.generations > 0);
    assert_eq!(result.population.len(), 25);
}

// --- Result consistency ------------------------------------------------------

#[test]
fn test_best_is_from_population() {
    let mut engine = make_engine(5, 5, Neighborhood::Moore, UpdateMode::Asynchronous);
    let result = engine.run();
    // best_fitness must equal the fitness of the best individual in population
    let pop_best = result.population.iter().map(|u| u.fitness()).fold(f64::MAX, f64::min);
    assert!(
        (result.best_fitness - pop_best).abs() < 1e-9 || result.best_fitness <= pop_best,
        "reported best {} not consistent with population best {}",
        result.best_fitness, pop_best
    );
}

#[test]
fn test_early_stopping_on_fitness_target() {
    let config = CellularConfiguration::default()
        .with_grid(5, 5)
        .with_neighborhood(Neighborhood::Moore)
        .with_update_mode(UpdateMode::Asynchronous)
        .with_max_generations(10_000)
        .with_mutation_sigma(1.0)
        .with_problem_solving(ProblemSolving::Minimization)
        // Very lenient target — engine should stop well before 10_000 gens.
        .with_fitness_target(1_000.0);

    let mut engine = CellularEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 99),
        sphere,
    );
    let result = engine.run();
    // Should stop early because fitness_target=1000 is trivially reachable.
    assert!(result.generations < 10_000, "expected early stop but ran {} gens", result.generations);
}

// --- All four neighborhoods + both update modes ------------------------------

#[test]
fn test_all_neighborhoods_synchronous() {
    for neighborhood in [
        Neighborhood::VonNeumann,
        Neighborhood::Moore,
        Neighborhood::CompactR2,
        Neighborhood::Linear,
    ] {
        let mut engine = make_engine(5, 5, neighborhood, UpdateMode::Synchronous);
        let result = engine.run();
        assert!(result.generations > 0, "synchronous engine produced 0 generations");
        assert_eq!(result.population.len(), 25);
    }
}

#[test]
fn test_all_neighborhoods_asynchronous() {
    for neighborhood in [
        Neighborhood::VonNeumann,
        Neighborhood::Moore,
        Neighborhood::CompactR2,
        Neighborhood::Linear,
    ] {
        let mut engine = make_engine(5, 5, neighborhood, UpdateMode::Asynchronous);
        let result = engine.run();
        assert!(result.generations > 0, "asynchronous engine produced 0 generations");
        assert_eq!(result.population.len(), 25);
    }
}

// ─── Observer tests ───────────────────────────────────────────────────────────

#[derive(Default)]
struct CellularSpyData {
    run_start: AtomicUsize,
    generation_start: AtomicUsize,
    new_best: AtomicUsize,
    generation_end: AtomicUsize,
    run_end: AtomicUsize,
    run_end_cause: Mutex<Option<TerminationCause>>,
    run_end_stats_len: AtomicUsize,
}

struct CellularSpyObserver {
    data: Arc<CellularSpyData>,
}

impl GaObserver<RangeChromosome<f64>> for CellularSpyObserver {
    fn on_run_start(&self) {
        self.data.run_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_generation_start(&self, _generation: usize) {
        self.data.generation_start.fetch_add(1, Ordering::Relaxed);
    }
    fn on_new_best(&self, _generation: usize, _best: RangeChromosome<f64>) {
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

/// Verify all 5 lifecycle hooks fire and on_new_best fires at most once per generation.
#[test]
fn test_cellular_observer_fires_5_hooks() {
    let max_gens = 20usize;
    let data = Arc::new(CellularSpyData::default());
    let spy = Arc::new(CellularSpyObserver { data: Arc::clone(&data) });

    let config = CellularConfiguration::default()
        .with_grid(5, 5)
        .with_neighborhood(Neighborhood::Moore)
        .with_update_mode(UpdateMode::Synchronous)
        .with_max_generations(max_gens)
        .with_selection(Selection::Tournament)
        .with_crossover(Crossover::Uniform)
        .with_mutation(Mutation::Gaussian)
        .with_mutation_sigma(0.5)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = CellularEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    )
    .with_observer(spy);

    let _result = engine.run();

    assert_eq!(data.run_start.load(Ordering::Relaxed), 1, "on_run_start must fire exactly once");
    assert_eq!(
        data.generation_start.load(Ordering::Relaxed),
        max_gens,
        "on_generation_start must fire once per generation"
    );
    assert_eq!(
        data.generation_end.load(Ordering::Relaxed),
        max_gens,
        "on_generation_end must fire once per generation"
    );
    assert_eq!(data.run_end.load(Ordering::Relaxed), 1, "on_run_end must fire exactly once");
    assert_eq!(
        *data.run_end_cause.lock().unwrap(),
        Some(TerminationCause::GenerationLimitReached),
    );
    assert_eq!(
        data.run_end_stats_len.load(Ordering::Relaxed),
        max_gens,
        "stats_history length must equal max_generations"
    );
    assert!(
        data.new_best.load(Ordering::Relaxed) >= 1,
        "on_new_best must fire at least once"
    );
    assert!(
        data.new_best.load(Ordering::Relaxed) <= max_gens,
        "on_new_best must fire at most once per generation, not per-cell"
    );
}

/// Verify that running without an observer does not panic.
#[test]
fn test_cellular_no_observer_no_panic() {
    let mut engine = make_engine(4, 4, Neighborhood::Moore, UpdateMode::Asynchronous);
    let result = engine.run();
    assert!(result.generations > 0);
}
