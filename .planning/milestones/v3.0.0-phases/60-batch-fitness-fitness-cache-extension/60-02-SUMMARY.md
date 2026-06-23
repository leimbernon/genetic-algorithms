---
phase: 60-batch-fitness-fitness-cache-extension
plan: "02"
subsystem: fitness/ga
tags: [rust, ga, fitness-cache, batch-evaluator, batch-cache-partition]

dependency_graph:
  requires:
    - BatchFitnessEvaluator<U> trait (plan 60-01)
    - fitness_cache field on Ga<U> (plan 60-01)
    - Wave 0 test stubs in tests/engines/test_ga.rs (plan 60-01)
  provides:
    - batch_evaluator field + with_batch_evaluator() builder on Ga<U> / GaConfiguration
    - GaError::ConfigurationError when both fitness_fn and batch_evaluator set (D-03)
    - batch_evaluate() free function — batch path for offspring + initial population (D-02)
    - D-06 batch+cache partition algorithm (lock-release-before-batch pattern)
    - D-07 GenerationStats.cache_hits / cache_misses delta values in Ga
    - 8 active batch_evaluator_tests in tests/engines/test_ga.rs
  affects:
    - src/engines/ga.rs
    - tests/engines/test_ga.rs

tech_stack:
  added: []
  patterns:
    - "batch_evaluate free function (not method) to avoid Ga borrow conflict between &self.batch_evaluator and &mut pop"
    - "D-06 lock-release-before-batch: acquire lock → partition hits/misses → release lock → call evaluate_batch → reacquire → put misses"
    - "D-07 saturating_sub snapshot: snapshot cache counters before generation body; delta = after - before"

key_files:
  modified:
    - src/engines/ga.rs
    - tests/engines/test_ga.rs

key-decisions:
  - "batch_evaluate is a free function, not a Ga method — Rust borrow checker cannot split &self.batch_evaluator from &mut pop when both are Ga fields; free function with explicit args resolves the conflict"
  - "GaError::ConfigurationError returned at Ga::build() when both fitness_fn (Some) and batch_evaluator are set — early error is clearer than silent precedence"
  - "D-06 lock released before evaluate_batch call — holding the mutex across an arbitrarily-slow user callback would serialize all threads; Pitfall 2 from RESEARCH.md"
  - "Cache bootstrap for batch-only-with-cache wired at run() start (not build()) — cache handle must outlive individual generation calls; run() is the natural lifetime anchor"

requirements-completed:
  - D-02
  - D-03
  - D-06
  - D-07

duration: 30min
completed: 2026-06-08
---

# Phase 60 Plan 02: Ga Batch Evaluator + Cache Integration

**`batch_evaluator` field + `with_batch_evaluator()` builder wired into `Ga<U>`; mutual exclusivity enforced at `build()`; D-06 batch+cache partition algorithm implemented; D-07 cache delta stats populated per generation; all 8 `batch_evaluator_tests` active and passing**

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | batch_evaluator field, builder, mutual-exclusivity, batch_evaluate helper | 541f072 | src/engines/ga.rs |
| 2 | Wire batch path into Ga::run() + cache delta stats | 4553123 | src/engines/ga.rs, tests/engines/test_ga.rs |

## Decisions Made

1. **`batch_evaluate` as free function** — splitting `&self.batch_evaluator` from `&mut pop` inside a `Ga` method hit the borrow checker. A free function that takes both as explicit arguments sidesteps the conflict cleanly.

2. **Mutual exclusivity at `build()`** — returns `GaError::ConfigurationError` when both `fitness_fn` (non-None) and `batch_evaluator` are set. Failing early at build time gives a clear error rather than a silent override at run time.

3. **D-06 lock released before `evaluate_batch`** — the cache mutex is acquired, hits/misses are partitioned, the lock is dropped, `evaluate_batch` is called with the miss chromosomes, then the lock is reacquired to store results. Holding the lock across user code (potentially slow GPU calls) would serialize the engine.

4. **D-07 `saturating_sub` snapshot** — cache counters are snaphotted before each generation body; per-generation deltas are computed as `after.saturating_sub(before)`. `saturating_sub` prevents wrapping if counters reset between snapshots.

## Files Modified

- `src/engines/ga.rs` — `batch_evaluator: Option<Arc<dyn BatchFitnessEvaluator<U>>>` field added; `with_batch_evaluator()` builder; `Ga::build()` mutual-exclusivity check; `batch_evaluate()` free function; `run()` wired with batch initial-pop eval, per-generation batch+cache path, cache bootstrap, delta stats
- `tests/engines/test_ga.rs` — all 8 `batch_evaluator_tests` activated: `ga_batch_evaluator_replaces_calculate_fitness`, `ga_batch_evaluator_initial_population_evaluated`, `ga_batch_evaluator_mutual_exclusivity_error`, `ga_batch_and_cache_only_misses_sent_to_batch`, `ga_cache_hits_stats_populated`, `ga_batch_evaluator_wrong_length_panics`, `ga_batch_evaluator_zero_population_noop`, `ga_cache_only_no_batch_still_works`

## Self-Check: PASSED

- `cargo test --test engines/test_ga batch_evaluator_tests`: 8 passed, 0 failed
- `cargo test`: no regressions
- `cargo clippy -- -D warnings`: 0 warnings
- `cargo check --target wasm32-unknown-unknown`: 0 errors
