---
phase: 66-build-perf-foundations-baseline-matrix-golden-tests
plan: "03"
subsystem: build-perf
tags: [golden-tests, ci, determinism, regression-gate]
dependency_graph:
  requires:
    - bench/build_perf.sh (from 66-01)
    - .planning/baselines/v3.0.0-baseline.json (from 66-01)
    - examples/*.rs with --seed args (from 66-01)
  provides:
    - tests/golden/rastrigin.txt
    - tests/golden/nsga2_zdt1.txt
    - tests/golden/cma_es_rastrigin.txt
    - tests/golden/pso_rastrigin.txt
    - tests/golden_tests.rs
    - .github/workflows/build-perf-gate.yml
  affects:
    - examples/rastrigin.rs
tech_stack:
  added: []
  patterns:
    - include_str!() for compile-time golden file inclusion in integration tests
    - rayon::ThreadPoolBuilder::build_global() to enforce single-thread mode for seeded runs
    - inline Python3 regression check in GitHub Actions workflow
key_files:
  created:
    - tests/golden/rastrigin.txt
    - tests/golden/nsga2_zdt1.txt
    - tests/golden/cma_es_rastrigin.txt
    - tests/golden/pso_rastrigin.txt
    - tests/golden_tests.rs
    - .github/workflows/build-perf-gate.yml
  modified:
    - examples/rastrigin.rs
decisions:
  - "Used with_rng_seed() on the GA builder (not just rng::set_seed() in main) because the GA engine's run() method calls rng::set_seed(config.rng_seed) internally, which resets to None and overrides any set_seed() call made before build()"
  - "Fixed rayon to 1 thread when --seed is provided (rayon::ThreadPoolBuilder::build_global(1)) because the counter-based make_rng() is called by rayon worker threads in non-deterministic order across runs, making the seeded output non-reproducible even with a fixed seed"
  - "CMA-ES, PSO, and NSGA2 were already deterministic with --seed; only the main rastrigin GA example used rayon-parallel crossover/mutation that caused non-determinism"
  - "build-perf-gate excludes sccache per D-04 (cold build measurement must not use artifact caches)"
  - "Used include_str!() in golden_tests.rs so a missing .txt file is a compile error, not a runtime panic"
metrics:
  duration: "~25m"
  completed_date: "2026-06-14"
  tasks_completed: 3
  files_changed: 7
---

# Phase 66 Plan 03: Golden Tests and Build-Perf Gate Summary

**One-liner:** Seed-42 golden output regression tests (4 examples) + `build-perf-gate` CI job enforcing 2%/0% timing/count regression budgets on every PR, with a root-cause fix for non-deterministic rastrigin output under rayon parallelism.

## What Was Built

### Task 1: --seed wiring verification + determinism fix

Plan 66-01 already wired `--seed <N>` args into all four examples. Task 1 confirmed this and uncovered a critical determinism bug in the rastrigin example:

- **Root cause 1:** `Ga::run()` calls `rng::set_seed(self.configuration.rng_seed)` internally, where `rng_seed` defaults to `None`. This overrode the `set_seed(Some(42))` call in `main()`, reverting to random seeding.
- **Root cause 2:** Even with the seed applied correctly, rayon worker threads call `make_rng()` in non-deterministic order, producing different counter values (and thus different RNG streams) on each run.
- **Fix:** Use `.with_rng_seed(seed)` on the GA builder so the engine re-applies the seed at `run()` time. Also call `rayon::ThreadPoolBuilder::new().num_threads(1).build_global()` when `--seed` is present to enforce deterministic counter ordering.
- **Verification:** Three consecutive `cargo run --example rastrigin --release -- --seed 42` invocations produce identical output after the fix (`Finished. Best fitness: 0.000153` all three times).

CMA-ES, PSO, and NSGA2 examples were already deterministic (no changes needed).

### Task 2: Golden .txt files and golden_tests.rs

Four golden files captured via `--seed 42`:

| File | Content |
|------|---------|
| `tests/golden/rastrigin.txt` | `Finished. Best fitness: 0.000153` |
| `tests/golden/nsga2_zdt1.txt` | `Completed. Pareto front: 100 non-dominated solutions` |
| `tests/golden/cma_es_rastrigin.txt` | `Best fitness: 4.974795` |
| `tests/golden/pso_rastrigin.txt` | `Best fitness: 1.989918` |

`tests/golden_tests.rs` contains four `#[test]` functions (`golden_rastrigin`, `golden_nsga2_zdt1`, `golden_cma_es_rastrigin`, `golden_pso_rastrigin`). Each invokes `cargo run --example <name> --release -- --seed 42`, extracts the relevant output line, and asserts equality against the `include_str!("golden/<name>.txt")` expected value.

### Task 3: build-perf-gate CI workflow

`.github/workflows/build-perf-gate.yml` runs on `pull_request` to `main` and `milestone/**` branches. The job:

1. Checks out with `actions/checkout@v4`
2. Installs stable toolchain + wasm32 target via `dtolnay/rust-toolchain@stable`
3. Caches cargo deps via `Swatinem/rust-cache@v2` (no sccache per D-04)
4. Runs `bench/build_perf.sh` to measure current metrics
5. Compares against `.planning/baselines/v3.0.0-baseline.json` with inline Python3:
   - Timing fields (`dev_build_s`, `wasm_check_s`, `test_suite_s`): fail if regression > 2%
   - `dep_count`: fail if value changes (0% tolerance)
   - `public_api_hash`: fail if changed (skip if either side is `"unavailable"`)
6. Prints a metric diff summary table

## Task Commits

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Fix rastrigin --seed determinism | 6fad76f | examples/rastrigin.rs |
| 2 | Capture golden files + write golden_tests.rs | dae4434 | tests/golden/*.txt, tests/golden_tests.rs |
| 3 | Create build-perf-gate CI workflow | 741d6bc | .github/workflows/build-perf-gate.yml |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Non-deterministic rastrigin output with --seed**

- **Found during:** Task 1 (attempting to capture golden output)
- **Issue:** `cargo run --example rastrigin --release -- --seed 42` produced a different fitness value on each invocation. Root cause: two interacting bugs:
  (a) `Ga::run()` calls `rng::set_seed(config.rng_seed)` = `set_seed(None)` which overrode the `set_seed(Some(42))` in `main()`.
  (b) Rayon worker threads call `make_rng()` in non-deterministic order, making the counter-derived seeds differ across runs.
- **Fix:** Added `with_rng_seed(seed)` to the GA builder (so the engine applies the seed correctly at run time) AND `rayon::ThreadPoolBuilder::new().num_threads(1).build_global()` (to serialize counter increments).
- **Files modified:** `examples/rastrigin.rs`
- **Commit:** 6fad76f

### Context Note

Task 1 in the plan was largely satisfied by Plan 66-01 (which already wired `--seed` args into all four examples). The work done here was discovering and fixing the determinism bug that was blocking golden capture.

## Known Stubs

None. All four golden values are real measurements from seeded, deterministic runs.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced. The CI workflow reads only version-controlled files within the checkout.

## Self-Check

### Created Files Exist

- [x] `tests/golden/rastrigin.txt` — exists with content `Finished. Best fitness: 0.000153`
- [x] `tests/golden/nsga2_zdt1.txt` — exists with content `Completed. Pareto front: 100 non-dominated solutions`
- [x] `tests/golden/cma_es_rastrigin.txt` — exists with content `Best fitness: 4.974795`
- [x] `tests/golden/pso_rastrigin.txt` — exists with content `Best fitness: 1.989918`
- [x] `tests/golden_tests.rs` — exists, 4 test functions
- [x] `.github/workflows/build-perf-gate.yml` — exists, valid YAML, no sccache

### Commits Exist

- [x] 6fad76f — fix(66-03): make rastrigin --seed run deterministic
- [x] dae4434 — feat(66-03): add golden output files and golden_tests.rs
- [x] 741d6bc — feat(66-03): add build-perf-gate CI workflow

## Self-Check: PASSED
