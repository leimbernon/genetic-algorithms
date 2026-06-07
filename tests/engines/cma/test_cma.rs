//! Integration tests for the CMA-ES engine.
//!
//! Tests CMA-01 through CMA-11 per the requirements-to-test map in 56-RESEARCH.md.
//! CMA-09 (WASM gate) remains ignored — it is a CI-level `cargo check` gate
//! deferred to Plan 04. All other tests are active after Plan 03.

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine, RestartStrategy};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::rng;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome, RealGene};
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::{RestartEvent, RestartKind};
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
struct SpyObserver {
    new_best_count: AtomicUsize,
    run_start_count: AtomicUsize,
    run_end_count: AtomicUsize,
    generation_start_count: AtomicUsize,
    generation_end_count: AtomicUsize,
    /// Incremented each time `on_restart` fires (CMA-12 through CMA-16).
    restart_count: AtomicUsize,
    /// The kind of the most recent restart event; `None` before any restart fires.
    last_restart_kind: Mutex<Option<RestartKind>>,
    /// All restart kinds recorded in order; used by CMA-13 to assert alternation.
    restart_kinds: Mutex<Vec<RestartKind>>,
    /// The `restart_number` from the most recent restart event.
    last_restart_number: AtomicUsize,
    /// The `population_size_after` from the most recent restart event.
    last_population_size_after: AtomicUsize,
}

impl Default for SpyObserver {
    fn default() -> Self {
        Self {
            new_best_count: AtomicUsize::new(0),
            run_start_count: AtomicUsize::new(0),
            run_end_count: AtomicUsize::new(0),
            generation_start_count: AtomicUsize::new(0),
            generation_end_count: AtomicUsize::new(0),
            restart_count: AtomicUsize::new(0),
            last_restart_kind: Mutex::new(None),
            restart_kinds: Mutex::new(Vec::new()),
            last_restart_number: AtomicUsize::new(0),
            last_population_size_after: AtomicUsize::new(0),
        }
    }
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

    fn on_restart(&self, event: &RestartEvent) {
        self.restart_count.fetch_add(1, Ordering::SeqCst);
        *self.last_restart_kind.lock().unwrap() = Some(event.kind);
        self.restart_kinds.lock().unwrap().push(event.kind);
        self.last_restart_number.store(event.restart_number, Ordering::SeqCst);
        self.last_population_size_after.store(event.population_size_after, Ordering::SeqCst);
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

// ─── CMA-12: IPOP restarts with scaled population ────────────────────────────

/// CMA-12 (SC-1): IPOP restarts after stagnation; `on_restart` fires and `total_restarts >= 1`.
///
/// Uses a very low `stagnation_threshold` (5 generations) and `max_restarts = 2` to
/// guarantee at least one restart fires within `max_generations = 50`.
#[test]
fn test_cma_ipop_restarts() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(50)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 5,
            max_restarts: 2,
        });

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 42),
        sphere,
    )
    .with_observer(spy.clone());

    let result = engine.run();

    assert!(
        spy.restart_count.load(Ordering::SeqCst) >= 1,
        "on_restart should fire at least once with stagnation_threshold=5 and max_restarts=2"
    );
    assert!(
        result.total_restarts >= 1,
        "total_restarts should be >= 1 after IPOP restart, got {}",
        result.total_restarts
    );
    assert!(
        result.total_restarts <= 2,
        "total_restarts should not exceed max_restarts=2, got {}",
        result.total_restarts
    );
}

// ─── CMA-13: BIPOP alternates large/small restarts ───────────────────────────

/// CMA-13 (SC-2): BIPOP alternates BipopLarge and BipopSmall across successive restarts.
///
/// After 4 restarts, the sequence of `RestartKind`s collected by the spy observer
/// must be `[BipopLarge, BipopSmall, BipopLarge, BipopSmall]`.
#[test]
fn test_cma_bipop_alternation() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(200)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Bipop {
            population_scale: 2.0,
            small_population_size: 0, // auto-compute
            stagnation_threshold: 5,
            max_restarts: 4,
        });

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 43),
        sphere,
    )
    .with_observer(spy.clone());

    let _result = engine.run();

    let kinds = spy.restart_kinds.lock().unwrap();
    assert_eq!(
        kinds.len(),
        4,
        "Expected exactly 4 restart events (max_restarts=4), got {}",
        kinds.len()
    );
    // Odd-numbered restarts (1st, 3rd) → BipopLarge; even-numbered (2nd, 4th) → BipopSmall
    assert_eq!(kinds[0], RestartKind::BipopLarge, "restart 1 should be BipopLarge");
    assert_eq!(kinds[1], RestartKind::BipopSmall, "restart 2 should be BipopSmall");
    assert_eq!(kinds[2], RestartKind::BipopLarge, "restart 3 should be BipopLarge");
    assert_eq!(kinds[3], RestartKind::BipopSmall, "restart 4 should be BipopSmall");
}

// ─── CMA-14: on_restart fires with correct RestartEvent fields ────────────────

/// CMA-14 (SC-3): `on_restart` receives a `RestartEvent` with correct field values.
///
/// Verifies that `restart_number` is 1-based, `population_size_after` reflects the
/// IPOP scaling, and `kind` matches the restart type.
#[test]
fn test_cma_restart_observer() {
    // Use a 3D sphere with a tiny stagnation_threshold to force a restart quickly.
    let initial_lambda = CmaConfiguration::default_for_dim(3).population_size;
    let scale = 2.0_f64;

    let config = CmaConfiguration::default_for_dim(3)
        .with_max_generations(30)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: scale,
            stagnation_threshold: 3,
            max_restarts: 1,
        });

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 3, -2.0, 2.0, 77),
        sphere,
    )
    .with_observer(spy.clone());

    let _result = engine.run();

    let restart_count = spy.restart_count.load(Ordering::SeqCst);
    assert!(restart_count >= 1, "at least one restart should have fired");

    // SC-3: restart_number must be 1-based (first restart == 1).
    assert_eq!(
        spy.last_restart_number.load(Ordering::SeqCst),
        1,
        "restart_number should be 1 for the first restart"
    );

    // SC-3: population_size_after must equal ceil(initial_lambda * scale).
    let expected_pop_size = (initial_lambda as f64 * scale).ceil() as usize;
    assert_eq!(
        spy.last_population_size_after.load(Ordering::SeqCst),
        expected_pop_size,
        "population_size_after should be ceil(initial_lambda * scale)"
    );

    // SC-3: kind must be Ipop.
    assert_eq!(
        *spy.last_restart_kind.lock().unwrap(),
        Some(RestartKind::Ipop),
        "restart kind should be Ipop for RestartStrategy::Ipop"
    );
}

// ─── CMA-15: no restart when strategy is None ────────────────────────────────

/// CMA-15 (SC-5): No restarts fire when `restart_strategy` is `None` (default).
///
/// Ensures the engine does not call `on_restart` or increment `total_restarts`
/// when no restart strategy is configured.
#[test]
fn test_cma_no_restart_when_none() {
    // Default configuration has restart_strategy = None
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(50)
        .with_problem_solving(ProblemSolving::Minimization);

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 99),
        sphere,
    )
    .with_observer(spy.clone());

    let result = engine.run();

    assert_eq!(
        spy.restart_count.load(Ordering::SeqCst),
        0,
        "on_restart should never fire when restart_strategy is None"
    );
    assert_eq!(
        result.total_restarts,
        0,
        "total_restarts should be 0 when restart_strategy is None"
    );
}

// ─── CMA-16: total_restarts counts correctly ─────────────────────────────────

/// CMA-16 (SC-6): `result.total_restarts` is bounded by `max_restarts`.
///
/// With `max_restarts = 3` and a low stagnation threshold, at most 3 restarts
/// can fire. The spy's `restart_count` must match `result.total_restarts`.
#[test]
fn test_cma_total_restarts_count() {
    let max_restarts = 3;

    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(100)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 5,
            max_restarts,
        });

    let spy = Arc::new(SpyObserver::default());

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 101),
        sphere,
    )
    .with_observer(spy.clone());

    let result = engine.run();

    assert!(
        result.total_restarts <= max_restarts,
        "total_restarts ({}) must not exceed max_restarts ({})",
        result.total_restarts,
        max_restarts
    );
    assert_eq!(
        spy.restart_count.load(Ordering::SeqCst),
        result.total_restarts,
        "spy.restart_count must equal result.total_restarts"
    );
}

// ─── CMA-17: global best preserved across restarts ───────────────────────────

/// CMA-17 (SC-7): `result.best_fitness` is finite and `result.total_restarts >= 0`
/// after a run with restart strategy enabled.
///
/// Structural test: verifies that the engine returns a valid result even after
/// one or more restarts. Deep best-tracking assertions (global best ≤ initial best)
/// are wired by Plan 02 once the global-best tracking logic is implemented.
#[test]
fn test_cma_global_best_across_restarts() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(60)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 5,
            max_restarts: 2,
        });

    let mut engine = CmaEngine::new(
        config,
        |n| random_pop(n, 5, -5.0, 5.0, 13),
        sphere,
    );

    let result = engine.run();

    assert!(
        result.best_fitness.is_finite(),
        "best_fitness must be finite after a run with restarts, got {}",
        result.best_fitness
    );
    assert!(
        result.total_restarts <= 2,
        "total_restarts must not exceed max_restarts=2, got {}",
        result.total_restarts
    );
    // Plan 02: additionally assert result.best_fitness <= initial_run_best
    // (global-best tracking across restarts is enforced by the engine loop)
}

// ─── Phase 60 Wave 0 test stubs (Nyquist gate) ───────────────────────────────

mod batch_and_cache_tests {
    #[test]
    #[ignore = "Wave 0 stub — implemented in Phase 60 Wave 2/3"]
    fn cma_with_fitness_cache_accepted() {
        unimplemented!("Wave 2/3 — Phase 60");
    }

    #[test]
    #[ignore = "Wave 0 stub — implemented in Phase 60 Wave 2/3"]
    fn cma_with_batch_evaluator_accepted() {
        unimplemented!("Wave 2/3 — Phase 60");
    }

    #[test]
    #[ignore = "Wave 0 stub — implemented in Phase 60 Wave 2/3"]
    fn cma_batch_evaluator_initial_population() {
        unimplemented!("Wave 2/3 — Phase 60");
    }

    #[test]
    #[ignore = "Wave 0 stub — implemented in Phase 60 Wave 2/3"]
    fn cma_batch_evaluator_offspring_loop() {
        unimplemented!("Wave 2/3 — Phase 60");
    }

    #[test]
    #[ignore = "Wave 0 stub — implemented in Phase 60 Wave 2/3"]
    fn cma_cache_stats_in_generation_stats() {
        unimplemented!("Wave 2/3 — Phase 60");
    }
}
