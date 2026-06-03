//! Integration tests for the PSO engine.
//!
//! Tests PSO-01 through PSO-11 per the requirements-to-test map in 57-RESEARCH.md.
//! All tests are `#[ignore]`-gated in Wave 1 — Plans 02 and 03 un-ignore them as
//! the engine implementation lands. PSO-11 (WASM gate) remains ignored and is verified
//! via CI (`cargo check --target wasm32-unknown-unknown`) in Plan 04.

// TODO(plan-02): re-add `use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};`
// once src/engines/pso lands.

use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
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
#[allow(dead_code)]
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.real_value() * g.real_value()).sum()
}

/// Build a random population of `Range<f64>` chromosomes.
#[allow(dead_code)]
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

// ─── Observer spy for PSO lifecycle tests ────────────────────────────────────

/// Thread-safe spy observer for testing PSO observer hooks.
#[derive(Default)]
#[allow(dead_code)]
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
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_run_returns_result() {
    unimplemented!("Plan 03: PsoEngine::run() returns PsoResult<U>");
}

// ─── PSO-02: personal best update ────────────────────────────────────────────

/// PSO-02: Personal best is updated when fitness strictly improves.
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_pbest_update() {
    unimplemented!("Plan 03: personal best updated when fitness improves");
}

// ─── PSO-03: observer on_run_start ───────────────────────────────────────────

/// PSO-03: Observer `on_run_start` fires exactly once.
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_observer_run_start() {
    unimplemented!("Plan 03: on_run_start fires exactly once");
}

// ─── PSO-04: observer generation count ───────────────────────────────────────

/// PSO-04: Observer `on_generation_start` fires once per generation.
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_observer_generation_count() {
    unimplemented!("Plan 03: on_generation_start fires once per generation");
}

// ─── PSO-05: observer new_best ────────────────────────────────────────────────

/// PSO-05: Observer `on_new_best` fires when best improves.
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_observer_new_best() {
    unimplemented!("Plan 03: on_new_best fires when best improves");
}

// ─── PSO-06: observer on_run_end ─────────────────────────────────────────────

/// PSO-06: Observer `on_run_end` fires exactly once.
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_observer_run_end() {
    unimplemented!("Plan 03: on_run_end fires exactly once");
}

// ─── PSO-07: ring topology wrap ──────────────────────────────────────────────

/// PSO-07: Ring topology neighborhood wraps correctly at boundaries.
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_ring_wrap() {
    unimplemented!("Plan 03: ring topology neighborhood wraps at boundaries");
}

// ─── PSO-08: absorbing boundary ──────────────────────────────────────────────

/// PSO-08: Absorbing boundary zeroes velocity at gene bounds.
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_absorbing_boundary() {
    unimplemented!("Plan 03: absorbing boundary zeroes velocity at bounds");
}

// ─── PSO-09: linear decay inertia ────────────────────────────────────────────

/// PSO-09: LinearDecay inertia produces w_start at gen 0 and w_end at max_generations.
#[test]
#[ignore = "Plan 02 will implement PsoInertia; un-ignore once available"]
fn test_pso_linear_decay() {
    unimplemented!("Plan 02: LinearDecay inertia produces w_start at gen 0 and w_end at max");
}

// ─── PSO-10: sphere convergence ──────────────────────────────────────────────

/// PSO-10: Sphere function is minimized (convergence smoke test).
#[test]
#[ignore = "Plan 03 will implement engine; un-ignore once available"]
fn test_pso_sphere_converges() {
    unimplemented!("Plan 03: PSO minimizes sphere function within max_generations");
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
