---
phase: 62
plan: 01
subsystem: fitness
tags: [surrogate, trait, stats, wave-0, tests]
dependency_graph:
  requires: []
  provides:
    - SurrogateModel<U> trait at crate root
    - GenerationStats.true_fitness_calls field
    - Wave 0 integration tests (test_surrogate.rs)
  affects:
    - src/fitness/surrogate.rs (new)
    - src/fitness.rs (module + re-export)
    - src/lib.rs (crate-root re-export)
    - src/stats.rs (new field)
    - tests/test_surrogate.rs (new)
    - src/engines/hill_climb/engine.rs (Rule 1 fix)
    - src/engines/permutate/engine.rs (Rule 1 fix)
tech_stack:
  added: []
  patterns:
    - Mirrors BatchFitnessEvaluator module layout exactly (same file structure, same serde gating)
    - Option<u64> cluster pattern in GenerationStats for optional stat fields
key_files:
  created:
    - src/fitness/surrogate.rs
    - tests/test_surrogate.rs
  modified:
    - src/fitness.rs
    - src/lib.rs
    - src/stats.rs
    - src/engines/hill_climb/engine.rs
    - src/engines/permutate/engine.rs
decisions:
  - "Tie-breaking among equal predicted scores is unstable sort order — explicitly documented in surrogate.rs to resolve RESEARCH.md Open Question #1"
  - "NaN predictions substitute to NEG_INFINITY before sort — locked in test SC-1g"
  - "Engine-dependent tests (SC-1b, SC-1c, SC-1e, SC-1f, SC-2a, SC-2b, SC-3) deferred to Plan 02"
metrics:
  duration_seconds: 834
  completed_date: "2026-06-09"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 7
---

# Phase 62 Plan 01: SurrogateModel Trait and Wave 0 Tests Summary

**One-liner:** SurrogateModel<U: ChromosomeT>: Send + Sync trait defined with prescreening contract (D-01/D-02/D-04/D-08), GenerationStats.true_fitness_calls field wired with serde(default), and four green Wave 0 tests at tests/test_surrogate.rs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Define SurrogateModel trait and wire module | bb6b921 | src/fitness/surrogate.rs, src/fitness.rs, src/lib.rs |
| 2 | Add true_fitness_calls field to GenerationStats | 71c2aa1 | src/stats.rs, src/engines/hill_climb/engine.rs, src/engines/permutate/engine.rs |
| 3 | Create tests/test_surrogate.rs with Wave 0 tests | 7ec1be0 | tests/test_surrogate.rs |

## Verification Results

- `cargo build --lib` — clean (0 errors, 0 relevant warnings)
- `cargo build --lib --features serde` — clean
- `cargo clippy --lib -- -D warnings` — clean
- `cargo doc --no-deps --lib` — zero warnings
- `cargo check --target wasm32-unknown-unknown` — clean (no Instant, no par_iter in new files)
- `cargo test --test test_surrogate` — 3 passed (SC-1a, SC-1d, SC-1g)
- `cargo test --test test_surrogate --features serde` — 4 passed (+ SC-2c)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing true_fitness_calls in hill_climb and permutate engine constructors**
- **Found during:** Task 2 verification (cargo build failed with E0063)
- **Issue:** `src/engines/hill_climb/engine.rs:128` and `src/engines/permutate/engine.rs:107` construct `GenerationStats` directly (not via `from_fitness_values`). Adding the new field to the struct requires updating all construction sites.
- **Fix:** Added `true_fitness_calls: None` to both direct constructors.
- **Files modified:** src/engines/hill_climb/engine.rs, src/engines/permutate/engine.rs
- **Commit:** 71c2aa1

**2. [Rule 1 - Bug] NanSurrogate used Cell<usize> which does not satisfy Sync**
- **Found during:** Task 3 first test run (E0277: Cell<usize> is not Sync)
- **Issue:** SurrogateModel trait requires Send + Sync on implementors. Cell<usize> is !Sync.
- **Fix:** Replaced Cell<usize> with AtomicUsize in NanSurrogate test struct.
- **Files modified:** tests/test_surrogate.rs
- **Commit:** 7ec1be0

**3. [Rule 1 - Bug] Comment in test file matched #[ignore] acceptance grep**
- **Found during:** Task 3 acceptance assertion check
- **Issue:** The comment `// NO #[ignore] attributes in this file.` caused `grep -c "#\[ignore"` to return 1 instead of 0.
- **Fix:** Changed comment to `// Zero ignore attributes in this file.`
- **Files modified:** tests/test_surrogate.rs
- **Commit:** 7ec1be0

## Known Stubs

None. All four Wave 0 tests run green. No stubs or placeholders exist in the created files.

## Threat Flags

None. The new `SurrogateModel` trait is a pure Rust trait — no network endpoints, no file I/O, no auth paths, no schema changes at trust boundaries.

## Self-Check: PASSED

- src/fitness/surrogate.rs: FOUND
- src/stats.rs (true_fitness_calls): FOUND
- tests/test_surrogate.rs: FOUND
- Commit bb6b921: FOUND (Task 1)
- Commit 71c2aa1: FOUND (Task 2)
- Commit 7ec1be0: FOUND (Task 3)
