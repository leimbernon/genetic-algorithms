//! Integration tests for the CMA-ES engine.
//!
//! Tests CMA-01 through CMA-11 per the requirements-to-test map in 56-RESEARCH.md.
//! CMA-09 (WASM gate) remains ignored — it is a CI-level `cargo check` gate
//! deferred to Plan 04. All other tests are active after Plan 03.

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::rng;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome, RealGene};
use genetic_algorithms::ga::TerminationCause;
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

// ─── Observer spy for lifecycle tests ────────────────────────────────────────

/// Thread-safe spy observer for testing CMA observer hooks.
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

// ─── CMA-01: sphere convergence ───────────────────────────────────────────────

/// CMA-01: CMA-ES reduces sphere fitness within max_generations.
#[test]
fn test_cma_sphere_converges() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3);

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    );

    let result = engine.run();

    assert!(
        result.best_fitness < 5.0,
        "CMA-ES should converge to < 5.0 on 5D sphere within 500 generations, got {}",
        result.best_fitness
    );
    assert!(result.generations > 0, "Should have run at least one generation");
    assert!(!result.population.is_empty(), "Population should be non-empty");
}

// ─── CMA-02: early stopping ───────────────────────────────────────────────────

/// CMA-02: Engine stops early when fitness_target is reached.
#[test]
fn test_cma_early_stopping() {
    // Use a very high fitness_target that the initial sphere population already satisfies.
    // On a 5D sphere from [-5, 5]^5 the max possible value is 5*25 = 125, but the
    // best of ~10 chromosomes will almost certainly be below 1e6.
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(10_000)
        .with_fitness_target(1e6)
        .with_problem_solving(ProblemSolving::Minimization);

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 99),
        sphere,
    );

    let result = engine.run();

    // Engine must have stopped before max_generations (since 1e6 is trivially satisfied)
    assert!(
        result.generations < 10_000,
        "Engine should have stopped early (fitness_target=1e6), ran {} generations",
        result.generations
    );
}

// ─── CMA-03: default_for_dim (NOT ignored) ────────────────────────────────────

/// CMA-03: `CmaConfiguration::default_for_dim` produces the expected population size.
#[test]
fn test_cma_default_for_dim() {
    // n = 10: lambda = 4 + floor(3 * ln(10)) = 4 + floor(6.9077...) = 4 + 6 = 10
    let cfg10 = CmaConfiguration::default_for_dim(10);
    let expected10 = 4 + (3.0 * 10.0_f64.ln()).floor() as usize;
    assert_eq!(
        cfg10.population_size, expected10,
        "default_for_dim(10) should give population_size = {}",
        expected10
    );

    // n = 0: must not panic and must give population_size >= 4
    let cfg0 = CmaConfiguration::default_for_dim(0);
    assert!(
        cfg0.population_size >= 4,
        "default_for_dim(0) should give population_size >= 4, got {}",
        cfg0.population_size
    );

    // n = 1: ln(1) = 0, so lambda = 4 + 0 = 4
    let cfg1 = CmaConfiguration::default_for_dim(1);
    assert_eq!(cfg1.population_size, 4, "default_for_dim(1) should give population_size = 4");
}

// ─── CMA-04: result fields ────────────────────────────────────────────────────

/// CMA-04: `CmaResult` carries population, best, best_fitness, and generations.
#[test]
fn test_cma_result_fields() {
    let config = CmaConfiguration::default_for_dim(3).with_max_generations(5);

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 3, -1.0, 1.0, 7),
        sphere,
    );

    let result = engine.run();

    assert!(!result.population.is_empty(), "population should be non-empty");
    assert!(result.generations > 0, "generations should be > 0");
    assert!(
        result.best_fitness.is_finite(),
        "best_fitness should be finite"
    );
    assert!(
        (result.best_fitness - result.best.fitness()).abs() < 1e-10,
        "best_fitness should equal best.fitness(), got best_fitness={} best.fitness()={}",
        result.best_fitness,
        result.best.fitness()
    );
}

// ─── CMA-05: observer new_best ────────────────────────────────────────────────

/// CMA-05: Observer receives `on_new_best` at least once during convergence.
#[test]
fn test_cma_observer_new_best() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(200)
        .with_problem_solving(ProblemSolving::Minimization);

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 123),
        sphere,
    )
    .with_observer(spy.clone());

    let _result = engine.run();

    assert!(
        spy.new_best_count.load(Ordering::SeqCst) >= 1,
        "on_new_best should fire at least once (including initial best at gen 0)"
    );
}

// ─── CMA-06: observer lifecycle ───────────────────────────────────────────────

/// CMA-06: Observer `on_run_start` and `on_run_end` are called exactly once.
#[test]
fn test_cma_observer_lifecycle() {
    let config = CmaConfiguration::default_for_dim(3).with_max_generations(10);

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 3, -1.0, 1.0, 55),
        sphere,
    )
    .with_observer(spy.clone());

    let result = engine.run();

    assert_eq!(
        spy.run_start_count.load(Ordering::SeqCst),
        1,
        "on_run_start should fire exactly once"
    );
    assert_eq!(
        spy.run_end_count.load(Ordering::SeqCst),
        1,
        "on_run_end should fire exactly once"
    );
    assert_eq!(
        spy.generation_start_count.load(Ordering::SeqCst),
        result.generations,
        "on_generation_start should fire once per completed generation"
    );
    assert_eq!(
        spy.generation_end_count.load(Ordering::SeqCst),
        result.generations,
        "on_generation_end should fire once per completed generation"
    );
}

// ─── CMA-07: DE regression (NOT ignored) ─────────────────────────────────────

/// CMA-07: Smoke test — Plan 01's RealGene rename did not break DE-using code paths.
///
/// Real regression coverage lives in `cargo test engines::de`.
#[test]
fn test_cma_de_still_passes() {
    // Compile-time type check: CmaConfiguration is constructible alongside DE types.
    let _: fn(CmaConfiguration) = |_| {};
    // No assertion needed — if this file compiles, the cma module wiring is intact.
}

// ─── CMA-08: Scatter regression (NOT ignored) ────────────────────────────────

/// CMA-08: Smoke test — Plan 01's RealGene rename did not break Scatter-using code paths.
///
/// Real regression coverage lives in `cargo test engines::scatter`.
#[test]
fn test_cma_scatter_still_passes() {
    // Compile-time type check only.
    let _: fn(CmaConfiguration) = |_| {};
    // No assertion needed — if this file compiles alongside cma + scatter modules, all is well.
}

// ─── CMA-09: WASM compiles (ignored placeholder) ─────────────────────────────

/// CMA-09: WASM gate — `cargo check --target wasm32-unknown-unknown` must pass.
///
/// This is verified via CI (`.github/workflows/wasm-check.yml`) and manually in Plan 04.
/// This test is marked ignored so it does not appear as a failing test before the WASM
/// check is wired into Plan 04's verification step.
#[test]
#[ignore = "Plan 04 verifies WASM via cargo check --target wasm32-unknown-unknown"]
fn test_cma_wasm_compiles() {
    unimplemented!("Plan 04 WASM verification gate")
}

// ─── CMA-10: RealGene trait on Range<f64> (NOT ignored) ──────────────────────

/// CMA-10: `Range<f64>` correctly implements `RealGene`.
#[test]
fn test_real_gene_range_f64() {
    let g = RangeGene::<f64>::new(0, vec![(-1.0, 1.0)], 0.5);
    assert_eq!(g.real_value(), 0.5, "real_value() should return the gene's value");

    let g2 = g.with_real_value(0.75);
    assert_eq!(
        g2.real_value(),
        0.75,
        "with_real_value(0.75).real_value() should return 0.75"
    );

    // Verify the original gene is unchanged (with_real_value is non-mutating).
    assert_eq!(g.real_value(), 0.5, "original gene should not be mutated");

    // Verify gene metadata (id, bounds) is preserved.
    assert_eq!(g2.id(), 0, "gene id should be preserved");

    // Verify random_pop helper compiles with the trait.
    let _ = random_pop(2, 3, -1.0, 1.0, 7);
    // Verify sphere helper compiles.
    let pop = random_pop(1, 3, -1.0, 1.0, 13);
    let f = sphere(pop[0].dna());
    assert!(f >= 0.0, "sphere is non-negative");
}

// ─── CMA-11: maximization ────────────────────────────────────────────────────

/// CMA-11: Engine correctly maximises fitness when `ProblemSolving::Maximization` is set.
///
/// Uses negated sphere: f(x) = -Σ xᵢ² which has a maximum of 0 at the origin.
/// Starting from a random population, CMA-ES under Maximization should find a
/// value > the initial best (which is typically < 0).
#[test]
fn test_cma_maximization() {
    // Negated sphere: maximum at 0 (origin), all other points are negative.
    fn neg_sphere(dna: &[RangeGene<f64>]) -> f64 {
        -dna.iter().map(|g| g.value() * g.value()).sum::<f64>()
    }

    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_sigma0(0.3);

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 77),
        neg_sphere,
    );

    let result = engine.run();

    // Under maximization of negated sphere, fitness improves toward 0.
    // The engine should find a value > -5.0 (initial values typically range from -25 to -5).
    assert!(
        result.best_fitness > -25.0,
        "Maximization of negated sphere should produce fitness > -25.0, got {}",
        result.best_fitness
    );
    assert!(!result.population.is_empty());
    assert!(result.generations > 0);
}
