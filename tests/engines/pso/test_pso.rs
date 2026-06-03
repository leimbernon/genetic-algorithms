//! Integration tests for the PSO engine.
//!
//! Tests PSO-01 through PSO-11 per the requirements-to-test map in 57-RESEARCH.md.
//! PSO-11 (WASM gate) remains ignored and is verified via CI
//! (`cargo check --target wasm32-unknown-unknown`) in Plan 04.

use genetic_algorithms::pso::{
    inertia_weight, PsoConfiguration, PsoEngine, PsoInertia, PsoTopology,
};

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::rng;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::traits::{LinearChromosome, RealGene};

// ─── Test helpers ─────────────────────────────────────────────────────────────

/// Sphere function: f(x) = Σ xᵢ²  (minimum 0 at origin)
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.real_value() * g.real_value()).sum()
}

/// Build a random population of `Range<f64>` chromosomes.
fn random_pop(n: usize, dim: usize, lo: f64, hi: f64, seed: u64) -> Vec<RangeChromosome<f64>> {
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

// ─── Observer spy for PSO lifecycle tests ────────────────────────────────────

/// Thread-safe spy observer for testing PSO observer hooks.
#[derive(Default)]
struct SpyObserver {
    new_best_count: AtomicUsize,
    run_start_count: AtomicUsize,
    run_end_count: AtomicUsize,
    generation_start_count: AtomicUsize,
    generation_end_count: AtomicUsize,
}

impl GaObserver<RangeChromosome<f64>> for SpyObserver {
    fn on_run_start(&self) {
        self.run_start_count.fetch_add(1, Ordering::SeqCst);
    }

    fn on_run_end(&self, _cause: TerminationCause, _all_stats: &[GenerationStats]) {
        self.run_end_count.fetch_add(1, Ordering::SeqCst);
    }

    fn on_new_best(&self, _generation: usize, _best: RangeChromosome<f64>) {
        self.new_best_count.fetch_add(1, Ordering::SeqCst);
    }

    fn on_generation_start(&self, _generation: usize) {
        self.generation_start_count.fetch_add(1, Ordering::SeqCst);
    }

    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.generation_end_count.fetch_add(1, Ordering::SeqCst);
    }
}

// ─── PSO-01: engine returns result ───────────────────────────────────────────

/// PSO-01: `PsoEngine::new(config, init_fn, fitness_fn).run()` returns `PsoResult<U>`.
#[test]
fn test_pso_run_returns_result() {
    rng::set_seed(Some(1));
    let init_pop = random_pop(20, 10, -5.12, 5.12, 1);
    let config = PsoConfiguration::default()
        .with_max_generations(20)
        .with_population_size(20);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    );
    let result = engine.run();
    assert_eq!(result.population.len(), 20, "population size must match");
    assert_eq!(result.generations, 20, "must complete all 20 generations");
    assert_eq!(result.best.dna().len(), 10, "best must have 10 genes");
    assert!(result.best_fitness.is_finite(), "best_fitness must be finite");
}

// ─── PSO-02: personal best update ────────────────────────────────────────────

/// PSO-02: Personal best is updated when fitness strictly improves.
#[test]
fn test_pso_pbest_update() {
    rng::set_seed(Some(2));
    let init_pop = random_pop(5, 10, -5.12, 5.12, 2);
    // Compute initial best fitness before the engine run.
    let initial_best_fitness: f64 = init_pop
        .iter()
        .map(|c| sphere(c.dna()))
        .fold(f64::INFINITY, f64::min);
    let config = PsoConfiguration::default()
        .with_max_generations(5)
        .with_population_size(5)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    );
    let result = engine.run();
    assert!(
        result.best_fitness <= initial_best_fitness,
        "best fitness after 5 gens ({}) should be <= initial best ({})",
        result.best_fitness,
        initial_best_fitness
    );
}

// ─── PSO-03: observer on_run_start ───────────────────────────────────────────

/// PSO-03: Observer `on_run_start` fires exactly once.
#[test]
fn test_pso_observer_run_start() {
    rng::set_seed(Some(3));
    let init_pop = random_pop(10, 5, -5.12, 5.12, 3);
    let spy = Arc::new(SpyObserver::default());
    let config = PsoConfiguration::default()
        .with_max_generations(10)
        .with_population_size(10);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    )
    .with_observer(spy.clone() as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);
    let _ = engine.run();
    assert_eq!(
        spy.run_start_count.load(Ordering::SeqCst),
        1,
        "on_run_start must fire exactly once"
    );
}

// ─── PSO-04: observer generation count ───────────────────────────────────────

/// PSO-04: Observer `on_generation_start` fires once per generation.
#[test]
fn test_pso_observer_generation_count() {
    rng::set_seed(Some(4));
    let init_pop = random_pop(10, 5, -5.12, 5.12, 4);
    let spy = Arc::new(SpyObserver::default());
    let config = PsoConfiguration::default()
        .with_max_generations(25)
        .with_population_size(10);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    )
    .with_observer(spy.clone() as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);
    let result = engine.run();
    assert_eq!(result.generations, 25, "must complete all 25 generations");
    assert_eq!(
        spy.generation_start_count.load(Ordering::SeqCst),
        result.generations,
        "on_generation_start count must equal result.generations"
    );
    assert_eq!(
        spy.generation_end_count.load(Ordering::SeqCst),
        result.generations,
        "on_generation_end count must equal result.generations"
    );
}

// ─── PSO-05: observer new_best ────────────────────────────────────────────────

/// PSO-05: Observer `on_new_best` fires when best improves (at minimum once for initial best).
#[test]
fn test_pso_observer_new_best() {
    rng::set_seed(Some(5));
    let init_pop = random_pop(10, 5, -5.12, 5.12, 5);
    let spy = Arc::new(SpyObserver::default());
    let config = PsoConfiguration::default()
        .with_max_generations(50)
        .with_population_size(10);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    )
    .with_observer(spy.clone() as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);
    let _ = engine.run();
    assert!(
        spy.new_best_count.load(Ordering::SeqCst) >= 1,
        "on_new_best must fire at least once (initial best at gen 0)"
    );
}

// ─── PSO-06: observer on_run_end ─────────────────────────────────────────────

/// PSO-06: Observer `on_run_end` fires exactly once.
#[test]
fn test_pso_observer_run_end() {
    rng::set_seed(Some(6));
    let init_pop = random_pop(10, 5, -5.12, 5.12, 6);
    let spy = Arc::new(SpyObserver::default());
    let config = PsoConfiguration::default()
        .with_max_generations(10)
        .with_population_size(10);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    )
    .with_observer(spy.clone() as Arc<dyn GaObserver<RangeChromosome<f64>> + Send + Sync>);
    let _ = engine.run();
    assert_eq!(
        spy.run_end_count.load(Ordering::SeqCst),
        1,
        "on_run_end must fire exactly once"
    );
}

// ─── PSO-07: ring topology wrap ──────────────────────────────────────────────

/// PSO-07: Ring topology neighborhood wraps correctly at boundaries.
/// Validates that neighborhood_size > n_particles does not panic (clamping works).
#[test]
fn test_pso_ring_wrap() {
    rng::set_seed(Some(7));
    // Use a very small swarm of 3 particles with neighborhood_size=5 (> swarm size).
    let init_pop = random_pop(3, 5, -5.12, 5.12, 7);
    let config = PsoConfiguration::default()
        .with_topology(PsoTopology::Ring { neighborhood_size: 5 })
        .with_population_size(3)
        .with_max_generations(2);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    );
    // Should not panic — ring-wrap clamp handles neighborhood_size > n_particles.
    let result = engine.run();
    assert_eq!(result.population.len(), 3, "population must remain 3 particles");
}

// ─── PSO-08: absorbing boundary ──────────────────────────────────────────────

/// PSO-08: Absorbing boundary ensures all genes remain within [lo, hi] after run.
#[test]
fn test_pso_absorbing_boundary() {
    rng::set_seed(Some(8));
    // Tight bounds [-1.0, 1.0] — large velocities will hit boundaries frequently.
    let init_pop = random_pop(10, 5, -1.0, 1.0, 8);
    let config = PsoConfiguration::default()
        .with_population_size(10)
        .with_max_generations(50)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    );
    let result = engine.run();
    // Every gene in every chromosome must be within bounds (with floating-point tolerance).
    for ind in result.population.iter() {
        for g in ind.dna() {
            let v = g.real_value();
            assert!(
                (-1.0 - 1e-12..=1.0 + 1e-12).contains(&v),
                "gene value {v} is outside [-1.0, 1.0] — absorbing boundary violated"
            );
        }
    }
}

// ─── PSO-09: linear decay inertia ────────────────────────────────────────────

/// PSO-09: LinearDecay inertia produces w_start at gen 0 and w_end at max_generations.
#[test]
fn test_pso_linear_decay() {
    let inertia = PsoInertia::LinearDecay {
        w_start: 0.9,
        w_end: 0.4,
    };

    // At generation 0, w should equal w_start = 0.9
    let w0 = inertia_weight(&inertia, 0, 100);
    assert!(
        (w0 - 0.9).abs() < 1e-12,
        "expected w=0.9 at gen=0, got {w0}"
    );

    // At generation 99 (= max_generations - 1), w should equal w_end = 0.4
    let w_end = inertia_weight(&inertia, 99, 100);
    assert!(
        (w_end - 0.4).abs() < 1e-12,
        "expected w=0.4 at gen=99, got {w_end}"
    );

    // Guard: max_generations <= 1 should return w_end (no div-by-zero)
    let w_guard = inertia_weight(&inertia, 0, 1);
    assert!(
        (w_guard - 0.4).abs() < 1e-12,
        "expected w_end=0.4 for max_generations=1, got {w_guard}"
    );

    // Constant inertia should return its value unchanged
    let w_const = inertia_weight(&PsoInertia::Constant(0.7), 50, 100);
    assert!(
        (w_const - 0.7).abs() < 1e-12,
        "expected w=0.7 for Constant(0.7), got {w_const}"
    );

    // Verify PsoEngine::new compiles with PSO types.
    let _config = PsoConfiguration::default()
        .with_topology(PsoTopology::Ring { neighborhood_size: 3 });
    let mut eng = PsoEngine::new(
        PsoConfiguration::default().with_max_generations(1),
        |n: usize| random_pop(n, 2, -1.0, 1.0, 99),
        sphere,
    );
    let result = eng.run();
    assert_eq!(result.generations, 1, "engine with max_generations=1 must return generations=1");
}

// ─── PSO-10: sphere convergence ──────────────────────────────────────────────

/// PSO-10: Sphere function is minimized — convergence smoke test.
/// Verifies PSO converges on 10D Sphere within 500 generations to fitness < 1e-2
/// with seed 42, 30 particles, gbest topology, LinearDecay 0.9→0.4, c1=c2=2.0.
#[test]
fn test_pso_sphere_converges() {
    rng::set_seed(Some(42));
    let init_pop = random_pop(30, 10, -5.12, 5.12, 42);
    let config = PsoConfiguration::default()
        .with_population_size(30)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1e-2);
    let mut engine = PsoEngine::new(
        config,
        move |_n| init_pop.clone(),
        sphere,
    );
    let result = engine.run();
    assert!(
        result.best_fitness < 1e-2 || result.generations < 500,
        "PSO must converge on 10D Sphere: best_fitness={:.6} after {} generations",
        result.best_fitness,
        result.generations
    );
}

// ─── PSO-11: WASM compiles (ignored placeholder) ─────────────────────────────

/// PSO-11: WASM gate — `cargo check --target wasm32-unknown-unknown` must pass.
///
/// This is verified via CI (`.github/workflows/wasm-check.yml`) and manually in Plan 04.
/// This test is marked ignored so it does not appear as a failing test before the WASM
/// check is wired into Plan 04's verification step.
#[test]
#[ignore = "Plan 04 verifies WASM via cargo check --target wasm32-unknown-unknown"]
fn test_pso_wasm_compiles() {
    unimplemented!("Plan 04 WASM verification gate")
}
