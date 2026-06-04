---
phase: 56-cma-es-engine
plan: "03"
subsystem: engines/cma
tags: [cma-es, engine, observer, jacobi, box-muller, wasm-gated]
dependency_graph:
  requires: [56-01, 56-02]
  provides:
    - CmaEngine<U> at crate::cma::CmaEngine
    - CmaResult<U> at crate::cma::CmaResult
  affects:
    - src/engines/cma/mod.rs (engine module added)
    - tests/engines/cma/test_cma.rs (all 10 active tests passing)
tech_stack:
  added: []
  patterns:
    - CMA-ES (Hansen arXiv:1604.00772) run loop
    - Jacobi eigendecomposition (classical rotation, ≤50 sweeps)
    - Box-Muller transform for standard-normal sampling
    - GaObserver hook wiring (5 hooks: on_run_start, on_generation_start, on_generation_end, on_new_best, on_run_end)
    - SpyObserver (AtomicUsize counters) for lifecycle/new_best testing
key_files:
  created:
    - src/engines/cma/engine.rs
  modified:
    - src/engines/cma/mod.rs
    - tests/engines/cma/test_cma.rs
decisions:
  - "CmaState and CmaEngine implemented in single engine.rs file — both share private helpers and the file is the natural unit of cohesion"
  - "No rand_distr dependency — Box-Muller inline avoids new package install"
  - "Jacobi eigendecomposition chosen over LAPACK binding — pure Rust, ≤50 sweeps sufficient for n≤100"
  - "on_run_start fires once before generation 0; on_new_best fires for initial best at gen 0 (inclusive); all observer hooks verified by SpyObserver"
  - "Threat T-56-03-02 mitigated: new_mean NaN/Inf guard breaks loop early with warning log"
  - "CMA-09 WASM test stays #[ignore] — Plan 04 owns the cargo check --target wasm32-unknown-unknown gate"
metrics:
  duration: "~30 minutes"
  completed: "2026-06-01"
  tasks_completed: 3
  files_changed: 3
---

# Phase 56 Plan 03: CmaEngine Run Loop + Observer Wiring — Summary

CMA-ES engine implementing Hansen's arXiv:1604.00772 reference algorithm with a Jacobi eigendecomposition, Box-Muller sampling, and complete GaObserver lifecycle hook wiring. All 10 active CMA tests pass (1 WASM gate deferred to Plan 04).

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | CmaState + Jacobi eigendecompose + Box-Muller helpers + matvec; `pub mod engine` in mod.rs | 34cde97 |
| 2 | CmaEngine struct, new(), with_observer(), run() loop, all observer hooks | 34cde97 |
| 3 | Un-ignore 6 CMA tests; add SpyObserver; fix clippy warnings; full test suite passes | 68ee039 |

(Tasks 1 and 2 share a single commit because they produce the same artifact — `src/engines/cma/engine.rs`.)

## Verification Results

- `cargo test --test test_engines engines::cma`: 10 passed, 0 failed, 1 ignored
- `cargo test --test test_engines engines::de`: 11 passed, 0 failed (regression)
- `cargo test --test test_engines engines::scatter`: 7 passed, 0 failed (regression)
- `cargo test --features serde`: 39 passed, 0 failed
- `cargo clippy --all-targets -- -D warnings`: no issues
- `grep -c "#\\[ignore" tests/engines/cma/test_cma.rs`: 1 (only CMA-09 WASM gate)

## Acceptance Criteria Verification

| Criterion | Result |
|-----------|--------|
| `src/engines/cma/engine.rs` contains `struct CmaState` | PASS |
| `src/engines/cma/engine.rs` contains `fn jacobi_eigendecompose` | PASS |
| `src/engines/cma/engine.rs` contains `fn standard_normal` (Box-Muller) | PASS |
| `grep -c "rand_distr" engine.rs` == 0 | PASS |
| `grep -c "par_iter" engine.rs` == 0 | PASS |
| `grep -c "#[cfg(test)]" engine.rs` == 0 | PASS |
| File contains `Hansen arXiv:1604.00772` comment | PASS |
| File contains canonical t_eigen formula `(n as f64).powf(1.5) * 10.0 / lambda as f64` | PASS |
| `pub struct CmaEngine` present | PASS |
| `pub struct CmaResult` present | PASS |
| All 5 observer hook sites present (count ≥ 5) | PASS (8 call sites) |
| `mod.rs` has `pub mod engine;` and `pub use engine::{CmaEngine, CmaResult};` | PASS |
| Sphere convergence test passes (best_fitness < 5.0 in 500 gens) | PASS |
| Observer lifecycle test passes | PASS |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ChromosomeT trait not in scope for test_cma_result_fields**
- **Found during:** Task 3 — test file compilation
- **Issue:** `result.best.fitness()` failed to compile: `ChromosomeT` trait not imported
- **Fix:** Added `use genetic_algorithms::traits::ChromosomeT;` to test imports
- **Files modified:** `tests/engines/cma/test_cma.rs`
- **Commit:** 68ee039

**2. [Rule 1 - Bug] Clippy: loop variable `j` used to index `new_mean`**
- **Found during:** Task 3 clippy gate
- **Issue:** `for j in 0..n { new_mean[j] += ... }` — clippy::needless_range_loop
- **Fix:** Replaced with iterator-based `for (j, nm) in new_mean.iter_mut().enumerate()`
- **Files modified:** `src/engines/cma/engine.rs`
- **Commit:** 68ee039

**3. [Rule 1 - Bug] Clippy: clamp-like pattern not using `.clamp()`**
- **Found during:** Task 3 clippy gate
- **Issue:** `sigma.max(1e-20).min(1e20)` — clippy::manual_clamp
- **Fix:** Replaced with `sigma.clamp(1e-20, 1e20)`
- **Files modified:** `src/engines/cma/engine.rs`
- **Commit:** 68ee039

## Known Stubs

None. The engine is fully implemented. The only `#[ignore]` test (CMA-09) is an intentional CI-level gate for Plan 04's WASM compilation check — not a stub in the behavioral sense.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes.

Threat mitigations implemented:
- **T-56-03-01:** Empty population guard — panics with a clear message (per plan spec: "rather than panicking" was relaxed to a descriptive panic since we cannot construct `CmaResult<U>` without a `best: U`)
- **T-56-03-02:** NaN/Inf mean guard — logs warning via `log::warn!(target: "cma_events", ...)` and breaks loop early
- **T-56-03-04:** Jacobi `d_vec` clamped to `≥ 1e-10 * max(d_vec)`; symmetry enforced after each C update; sigma clamped via `.clamp(1e-20, 1e20)`
- **T-56-03-05:** No `Instant::now()` used in the core loop at all (optional hooks skipped)

## Self-Check

- [x] `src/engines/cma/engine.rs` exists and contains `pub struct CmaEngine`, `pub struct CmaResult`, `struct CmaState`, `fn jacobi_eigendecompose`, `fn standard_normal`
- [x] `src/engines/cma/mod.rs` contains `pub mod engine;` and `pub use engine::{CmaEngine, CmaResult};`
- [x] `tests/engines/cma/test_cma.rs` has exactly 1 `#[ignore` attribute (CMA-09)
- [x] Commits 34cde97 and 68ee039 exist
- [x] All 10 active CMA tests pass; 1 WASM test deferred

## Self-Check: PASSED
