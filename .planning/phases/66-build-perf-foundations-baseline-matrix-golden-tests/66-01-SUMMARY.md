---
phase: 66-build-perf-foundations-baseline-matrix-golden-tests
plan: "01"
subsystem: build-perf
tags: [baseline, measurement, harness, golden-tests]
dependency_graph:
  requires: []
  provides:
    - bench/build_perf.sh
    - .planning/baselines/v3.0.0-baseline.json
    - target/build-perf/*-seed42.txt (ephemeral, for Plan 66-03)
  affects:
    - examples/rastrigin.rs
    - examples/nsga2_zdt1.rs
    - examples/cma_es_rastrigin.rs
    - examples/pso_rastrigin.rs
    - .gitignore
tech_stack:
  added: []
  patterns:
    - bash harness script with set -euo pipefail for reliable measurement
    - --seed <N> CLI arg pattern for reproducible example runs
key_files:
  created:
    - bench/build_perf.sh
    - .planning/baselines/v3.0.0-baseline.json
    - .planning/baselines/  (new directory)
  modified:
    - .gitignore
    - examples/rastrigin.rs
    - examples/nsga2_zdt1.rs
    - examples/cma_es_rastrigin.rs
    - examples/pso_rastrigin.rs
decisions:
  - "Added --seed <N> CLI arg to the four reference examples so build_perf.sh can capture deterministic golden outputs"
  - "Used git add -f for .planning/baselines/v3.0.0-baseline.json because .planning/ is gitignored (all existing .planning/ files were committed before the gitignore rule was added); new files in that directory require force-add"
  - "target/build-perf/ is implicitly covered by the existing /target gitignore rule; added explicit target/build-perf/ entry for clarity"
  - "public_api_hash set to 'unavailable' because cargo-public-api is not installed in the local environment; non-blocking per plan spec"
  - "cma_es_rastrigin.rs had hardcoded rng::set_seed(Some(42)) inside init_population(); moved it to main() so --seed arg controls it properly"
metrics:
  duration: "6m 43s"
  completed_date: "2026-06-14"
  tasks_completed: 2
  files_changed: 7
---

# Phase 66 Plan 01: Baseline Harness Summary

**One-liner:** Build-performance measurement harness (`bench/build_perf.sh`) with committed v3.0.0 baseline: dev_build_s=3.658, wasm_check_s=5.321, test_suite_s=55.790, dep_count=97, public_api_hash=unavailable.

## What Was Built

`bench/build_perf.sh` is a reproducible bash script that measures six canonical build-performance metrics for the `genetic_algorithms` crate:

1. **dev_build_s** — wall-clock for `cargo clean && cargo build` (default features)
2. **wasm_check_s** — wall-clock for `cargo clean && cargo check --target wasm32-unknown-unknown --lib`
3. **test_suite_s** — wall-clock for `cargo clean && cargo test --quiet`
4. **dep_count** — unique transitive dep count via `cargo tree`
5. **public_api_hash** — SHA-256 of `cargo public-api` output (or `"unavailable"` if not installed)
6. **captured_at** — ISO date of measurement

The script writes `target/build-perf/results.json` (gitignored, ephemeral) and, when invoked with `--commit`, copies it to `.planning/baselines/v3.0.0-baseline.json` (tracked, canonical).

The script also runs four reference examples with `--seed 42` and captures stdout to `target/build-perf/<name>-seed42.txt` for Plan 66-03 golden tests.

## v3.0.0 Baseline Values (2026-06-14)

| Metric | Value |
|--------|-------|
| dev_build_s | 3.658 s |
| wasm_check_s | 5.321 s |
| test_suite_s | 55.790 s |
| dep_count | 97 |
| public_api_hash | unavailable |
| captured_at | 2026-06-14 |

## Task Commits

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Author bench/build_perf.sh | 293322e | bench/build_perf.sh, .gitignore, examples/*.rs |
| 2 | Validate baseline JSON and commit | ca3c150 | .planning/baselines/v3.0.0-baseline.json |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Added --seed CLI arg to four reference examples**
- **Found during:** Task 1
- **Issue:** The plan specified `cargo run --example <name> --release -- --seed 42` but none of the four examples (`rastrigin`, `nsga2_zdt1`, `cma_es_rastrigin`, `pso_rastrigin`) accepted a `--seed` CLI argument. Without this, the golden captures would be non-deterministic across runs.
- **Fix:** Added `--seed <N>` argument parsing in each example's `main()`. `rastrigin` and `nsga2_zdt1` call `rng::set_seed(Some(N))`. `pso_rastrigin` and `cma_es_rastrigin` already used hardcoded seeds; updated them to accept CLI override with the original value as fallback.
- **Files modified:** `examples/rastrigin.rs`, `examples/nsga2_zdt1.rs`, `examples/cma_es_rastrigin.rs`, `examples/pso_rastrigin.rs`
- **Commit:** 293322e

**2. [Rule 3 - Blocking Issue] Fixed cargo clean deleting target/build-perf/ output directory**
- **Found during:** Task 1 (first run)
- **Issue:** The script ran `mkdir -p target/build-perf/` at startup, but the three sequential `cargo clean` invocations deleted the entire `target/` tree. When the script tried to write `results.json`, the directory no longer existed.
- **Fix:** Moved the `mkdir -p "$OUT_DIR"` call to just before writing `results.json` (after all `cargo clean` invocations).
- **Files modified:** `bench/build_perf.sh`
- **Commit:** 293322e

**3. [Rule 1 - Bug] Fixed cma_es_rastrigin hardcoded seed in init_population**
- **Found during:** Task 1
- **Issue:** `cma_es_rastrigin.rs` called `rng::set_seed(Some(42))` inside `init_population()`, which would override the CLI seed set in `main()` on every call to the initialization function.
- **Fix:** Removed the hardcoded `rng::set_seed(Some(42))` from `init_population()`; moved seed initialization to `main()` with `--seed` arg support (defaulting to 42).
- **Files modified:** `examples/cma_es_rastrigin.rs`
- **Commit:** 293322e

### Gitignore Note

`.planning/baselines/v3.0.0-baseline.json` required `git add -f` because `.planning/` is listed in `.gitignore`. This is consistent with how all existing `.planning/` files are handled — they were committed before the ignore rule existed. The `target/build-perf/` entry added to `.gitignore` is explicit documentation, though `/target` already covers it.

## Known Stubs

None. All six metric values are real measurements from the local machine (not placeholders). `public_api_hash="unavailable"` is not a stub — it is the correct value when `cargo-public-api` is not installed, per plan spec.

## Threat Flags

None. No new network endpoints, auth paths, or schema changes introduced. The script only invokes `cargo` subcommands.

## Self-Check

### Created Files Exist
- [x] `bench/build_perf.sh` — exists, executable
- [x] `.planning/baselines/v3.0.0-baseline.json` — exists, valid JSON

### Commits Exist
- [x] 293322e — feat(66-01): author bench/build_perf.sh measurement harness
- [x] ca3c150 — feat(66-01): commit v3.0.0 baseline JSON with six canonical metrics

## Self-Check: PASSED
