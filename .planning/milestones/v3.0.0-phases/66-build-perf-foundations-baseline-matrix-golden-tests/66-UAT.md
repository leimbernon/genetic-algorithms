---
status: complete
phase: 66-build-perf-foundations-baseline-matrix-golden-tests
source: [66-01-SUMMARY.md, 66-02-SUMMARY.md, 66-03-SUMMARY.md]
started: 2026-06-17T00:00:00Z
updated: 2026-06-17T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. build_perf.sh exists and is executable
expected: `bench/build_perf.sh` is present and executable (`ls -la bench/build_perf.sh` shows `-rwxr-xr-x` permissions).
result: pass

### 2. Baseline JSON has all 6 metrics
expected: `.planning/baselines/v3.0.0-baseline.json` exists and contains exactly 6 fields: `dev_build_s`, `wasm_check_s`, `test_suite_s`, `dep_count`, `public_api_hash`, `captured_at`. Running `cat .planning/baselines/v3.0.0-baseline.json` shows valid JSON with numeric timing values and `dep_count=97`.
result: pass

### 3. Examples accept --seed arg
expected: Running `cargo run --example rastrigin --release -- --seed 42` twice produces identical output both times (`Finished. Best fitness: 0.000153`). The seed is wired deterministically.
result: pass

### 4. Golden tests pass
expected: Running `cargo test golden` executes 4 tests (`golden_rastrigin`, `golden_nsga2_zdt1`, `golden_cma_es_rastrigin`, `golden_pso_rastrigin`) and all 4 pass with `test result: ok. 4 passed; 0 failed`.
result: pass

### 5. Feature matrix YAML is correct
expected: `.github/workflows/feature-matrix.yml` exists. It has 8 matrix entries (default, serde, visualization, benchmarks, observer-tracing, observer-metrics, all-features, wasm32). It triggers on push to `main` and `milestone/**` only — `grep pull_request .github/workflows/feature-matrix.yml` returns nothing.
result: pass

### 6. build-perf-gate CI workflow is correct
expected: `.github/workflows/build-perf-gate.yml` exists, triggers on `pull_request`, and does NOT reference sccache (`grep -i sccache .github/workflows/build-perf-gate.yml` returns nothing). It reads `.planning/baselines/v3.0.0-baseline.json` and enforces 2%/0% regression budgets.
result: pass

## Summary

total: 6
passed: 6
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none yet]
