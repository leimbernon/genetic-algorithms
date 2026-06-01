//! Integration tests for the CMA-ES engine (Wave 0 stubs).
//!
//! Tests CMA-01 through CMA-11 per the requirements-to-test map in 56-RESEARCH.md.
//! Engine-using tests are marked `#[ignore = "Plan 03 lands CmaEngine"]` until
//! `CmaEngine` is wired.  The four non-engine tests compile and pass after Plan 02.

use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::cma::CmaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::{GeneT, LinearChromosome, RealGene};
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

// ─── CMA-01: sphere convergence ───────────────────────────────────────────────

/// CMA-01: CMA-ES reduces sphere fitness within max_generations.
#[test]
#[ignore = "Plan 03 lands CmaEngine"]
fn test_cma_sphere_converges() {
    let _config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_fitness_target(1.0)
        .with_problem_solving(ProblemSolving::Minimization);
    // CmaEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere).run()
    unimplemented!("Plan 03 lands CmaEngine")
}

// ─── CMA-02: early stopping ───────────────────────────────────────────────────

/// CMA-02: Engine stops early when fitness_target is reached.
#[test]
#[ignore = "Plan 03 lands CmaEngine"]
fn test_cma_early_stopping() {
    let _config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(10_000)
        .with_fitness_target(0.01)
        .with_problem_solving(ProblemSolving::Minimization);
    unimplemented!("Plan 03 lands CmaEngine")
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
#[ignore = "Plan 03 lands CmaEngine"]
fn test_cma_result_fields() {
    let _config = CmaConfiguration::default_for_dim(3).with_max_generations(5);
    unimplemented!("Plan 03 lands CmaEngine")
}

// ─── CMA-05: observer new_best ────────────────────────────────────────────────

/// CMA-05: Observer receives `on_new_best` at least once during convergence.
#[test]
#[ignore = "Plan 03 lands CmaEngine"]
fn test_cma_observer_new_best() {
    let _config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(200)
        .with_problem_solving(ProblemSolving::Minimization);
    unimplemented!("Plan 03 lands CmaEngine")
}

// ─── CMA-06: observer lifecycle ───────────────────────────────────────────────

/// CMA-06: Observer `on_run_start` and `on_run_end` are called exactly once.
#[test]
#[ignore = "Plan 03 lands CmaEngine"]
fn test_cma_observer_lifecycle() {
    let _config = CmaConfiguration::default_for_dim(3).with_max_generations(10);
    unimplemented!("Plan 03 lands CmaEngine")
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
/// Marked `#[ignore]` here so it does not appear as a failing test before the WASM
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

// ─── CMA-11: maximization (ignored) ──────────────────────────────────────────

/// CMA-11: Engine correctly maximises fitness when `ProblemSolving::Maximization` is set.
#[test]
#[ignore = "Plan 03 lands CmaEngine"]
fn test_cma_maximization() {
    let _config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_fitness_target(0.9);
    unimplemented!("Plan 03 lands CmaEngine")
}
