---
phase: 60
plan: 01
subsystem: fitness
tags: [rust, ga, fitness-cache, batch-evaluator, traits, stats]
dependency_graph:
  requires: []
  provides:
    - BatchFitnessEvaluator<U> trait (src/fitness/batch.rs)
    - wrap_with_cache tuple return (src/fitness/cache.rs)
    - GenerationStats.cache_hits / cache_misses fields (src/stats.rs)
    - fitness_cache field on Ga<U> (src/engines/ga.rs)
  affects:
    - src/fitness/batch.rs (new)
    - src/fitness.rs (pub mod batch + re-export)
    - src/fitness/cache.rs (wrap_with_cache signature change)
    - src/stats.rs (two new optional fields)
    - src/lib.rs (BatchFitnessEvaluator re-export)
    - src/engines/ga.rs (call site fix + fitness_cache field)
    - tests/test_stats.rs (cache_stats_default_none active test + ignored stub)
    - tests/fitness/test_cache.rs (wrap_with_cache_returns_handle active test)
    - tests/engines/test_ga.rs (8 ignored Wave 0 stubs)
    - tests/engines/cma/test_cma.rs (5 ignored Wave 0 stubs)
tech_stack:
  added: []
  patterns:
    - pub trait + Send + Sync (BatchFitnessEvaluator, modelled after GaObserver)
    - Arc<Mutex<FitnessCache>> shared handle returned from wrap_with_cache
    - Option<T> + serde(default) for backward-compatible field extension on GenerationStats
key_files:
  created:
    - src/fitness/batch.rs
  modified:
    - src/fitness.rs
    - src/fitness/cache.rs
    - src/stats.rs
    - src/lib.rs
    - src/engines/ga.rs
    - tests/test_stats.rs
    - tests/fitness/test_cache.rs
    - tests/engines/test_ga.rs
    - tests/engines/cma/test_cma.rs
decisions:
  - wrap_with_cache returns tuple (Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>) so callers can observe hit/miss stats without needing a separate registration step
  - fitness_cache field on Ga<U> reserved with None default in Wave 1 to avoid second struct churn in Wave 2
  - call site fix moved into Task 1 (deviation Rule 3) to satisfy Task 1 build verification; Task 2 stores the handle properly
metrics:
  duration_minutes: 20
  completed_date: "2026-06-07"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 10
---

# Phase 60 Plan 01: Phase 60 Foundation — BatchFitnessEvaluator, wrap_with_cache refactor, GenerationStats extension

Established the Phase 60 foundation: `BatchFitnessEvaluator<U>` trait introduced, `wrap_with_cache` refactored to return a `(fn, handle)` tuple, `GenerationStats` extended with optional cache delta fields, and 15 Wave 0 test stubs scaffolded for Nyquist gate compliance.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | BatchFitnessEvaluator trait, wrap_with_cache refactor, GenerationStats extension | 55807c4 | src/fitness/batch.rs (new), src/fitness/cache.rs, src/stats.rs, src/lib.rs, src/engines/ga.rs |
| 2 | Ga struct fitness_cache field + Wave 0 test stubs | 868312c | src/engines/ga.rs, tests/test_stats.rs, tests/fitness/test_cache.rs, tests/engines/test_ga.rs, tests/engines/cma/test_cma.rs |

## Decisions Made

1. `wrap_with_cache` returns `(Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)` tuple — callers get the cache handle without a separate registration API; the pattern matches the existing `Arc<dyn GaObserver<U>>` approach.

2. `fitness_cache: Option<Arc<Mutex<FitnessCache>>>` field reserved on `Ga<U>` in Wave 1 with `None` default — avoids a second struct field addition and Default-impl churn in Wave 2 when the field is wired to stats delta reporting.

3. Call-site fix in `Ga::build()` moved into Task 1 commit (not Task 2) — Rule 3 deviation, required to satisfy Task 1 build verification. Task 2 upgraded `_cache_handle` to proper storage in `self.fitness_cache`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed wrap_with_cache call site in Ga::build() during Task 1**
- **Found during:** Task 1 build verification
- **Issue:** Task 1 changes the `wrap_with_cache` return type to a tuple, which immediately broke the existing call site in `Ga::build()`. Task 1's acceptance criteria requires `cargo build` to succeed, but the plan said to fix the call site in Task 2.
- **Fix:** Applied the call-site fix (destructuring tuple with `_cache_handle`) in Task 1 to unblock the build. Task 2 then upgraded `_cache_handle` to `self.fitness_cache = Some(cache_handle)`.
- **Files modified:** src/engines/ga.rs
- **Commit:** 55807c4 (call site fix), 868312c (storage wired)

## Active Tests Added

| Test | File | Status |
|------|------|--------|
| `cache_stats_default_none` | tests/test_stats.rs | Active (passes) |
| `wrap_with_cache_returns_handle` | tests/fitness/test_cache.rs | Active (passes) |

## Wave 0 Ignored Stubs (Nyquist gate)

15 ignored stubs across 3 test files covering all Phase 60 success criteria:

- `tests/engines/test_ga.rs` — 8 stubs in `mod batch_evaluator_tests`
- `tests/engines/cma/test_cma.rs` — 5 stubs in `mod batch_and_cache_tests`
- `tests/test_stats.rs` — 1 ignored stub (`cache_stats_serde_compat_old_checkpoint`)
- `tests/fitness/test_cache.rs` — 0 additional ignored (active test instead)

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. `Arc<Mutex<FitnessCache>>` is WASM-safe (no `Instant`, no `par_iter`). `BatchFitnessEvaluator` is in-process user code with the same trust model as `GaObserver`. No new threat flags.

## Known Stubs

None — all new fields have correct `None` defaults and the trait is fully defined (no placeholder implementations).

## Self-Check: PASSED

Files exist:
- src/fitness/batch.rs: FOUND
- src/fitness.rs (updated): FOUND
- src/fitness/cache.rs (updated): FOUND
- src/stats.rs (updated): FOUND
- src/lib.rs (updated): FOUND
- src/engines/ga.rs (updated): FOUND

Commits exist:
- 55807c4: FOUND (feat(60-01): add BatchFitnessEvaluator trait...)
- 868312c: FOUND (feat(60-01): add fitness_cache field to Ga struct...)
