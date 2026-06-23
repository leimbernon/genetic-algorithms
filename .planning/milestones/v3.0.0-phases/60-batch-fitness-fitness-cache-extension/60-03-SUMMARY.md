---
phase: 60
plan: 03
subsystem: cma + phase-gate
tags: [rust, cma-es, fitness-cache, batch-evaluator, phase-gate]
dependency_graph:
  requires:
    - BatchFitnessEvaluator<U> trait (plan 60-01)
    - Ga batch+cache wiring (plan 60-02)
  provides:
    - CmaEngine batch+cache parity with Ga (D-03, D-04, D-05, D-06, D-07)
    - Phase 60 all CI gates green (SC #4)
  affects:
    - src/engines/cma/configuration.rs (fitness_cache_size field + with_fitness_cache builder)
    - src/engines/cma/engine.rs (batch_evaluator/fitness_cache fields, builder, helper, both eval sites, delta stats)
    - tests/engines/cma/test_cma.rs (5 batch_and_cache_tests activated)
    - src/engines/ga.rs (clippy fix: if-let replaces unwrap-after-is_some)
tech_stack:
  added: []
  patterns:
    - Structural replication of Ga::batch_evaluate_pop into CmaEngine (no shared utility — bounded footprint)
    - D-06 lock-release-before-batch pattern (Pitfall 2 / T-60-05)
    - Cache handle bootstrapped at run() start (mirrors Ga::build() wiring)
key_files:
  modified:
    - src/engines/cma/configuration.rs
    - src/engines/cma/engine.rs
    - tests/engines/cma/test_cma.rs
    - src/engines/ga.rs
decisions:
  - CmaConfiguration carries fitness_cache_size (not CmaEngine) to follow the existing config/engine split — all tunable parameters live on the config struct
  - CmaEngine.fitness_fn is non-Optional; D-03 mutual exclusivity is enforced at run() start by last-writer-wins: when batch_evaluator is set, scalar fitness_fn calls are skipped entirely (no ConfigurationError — Ga's stricter check relies on Optional fitness_fn, which CMA doesn't have)
  - batch_evaluate_pop structurally replicated on CmaEngine rather than extracted to a shared free function — keeps Phase 60 footprint bounded; extraction can happen in a dedicated refactor phase
  - cache handle bootstrapped once per run() call (not per restart) — the handle persists across restarts so cumulative hit/miss counters remain monotone
metrics:
  duration_minutes: 35
  completed_date: "2026-06-08"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 4
---

# Phase 60 Plan 03: CMA batch+cache wiring + Phase 60 verification gate

Applied Phase 60 contracts to `CmaEngine`, bringing it to full feature parity with `Ga` for batch fitness evaluation and fitness caching. Ran the complete Phase 60 CI gate — all 7 commands pass with zero warnings.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | CmaConfiguration/CmaEngine fields, builder, helper, both eval sites, delta stats, 5 tests | (this commit) | src/engines/cma/configuration.rs, src/engines/cma/engine.rs, tests/engines/cma/test_cma.rs, src/engines/ga.rs |
| 2 | Phase 60 verification gate + SUMMARY | (this commit) | .planning/phases/60-batch-fitness-fitness-cache-extension/60-03-SUMMARY.md |

## Decisions Implemented (D-01 through D-08)

| Decision | Status | Notes |
|----------|--------|-------|
| D-01: `BatchFitnessEvaluator<U>` trait — `fn evaluate_batch(&self, &[U]) -> Vec<f64>` | Delivered (plan 60-01) | Trait signature locked; `&[U]` forces miss-chromosome clone in batch+cache path |
| D-02: Batch replaces `calculate_fitness` entirely for Ga offspring + initial pop | Delivered (plan 60-02) | Verified by `ga_batch_evaluator_replaces_calculate_fitness` and `ga_batch_evaluator_initial_population_evaluated` |
| D-03: `with_batch_evaluator` builder on Ga (ConfigurationError if both set) + CmaEngine (last-writer-wins) | Delivered | Ga: strict mutual-exclusivity via `GaError::ConfigurationError`. CMA: scalar path skipped when batch set — discretion choice documented below |
| D-04: CMA initial pop and offspring batch-evaluated (collect-then-batch) | Delivered | Both sites patched: init loop (`batch_evaluate_pop` call) + offspring loop (`collect → batch_evaluate_pop` post-loop) |
| D-05: `with_fitness_cache(size)` on CmaConfiguration | Delivered | `fitness_cache_size: Option<usize>` on config; cache bootstrapped at `run()` start |
| D-06: Batch+cache partition — only cache misses sent to `evaluate_batch` | Delivered (Ga plan 60-02; CMA this plan) | `batch_evaluate_pop` Case C: acquire→partition→release→batch→reacquire→put |
| D-07: `GenerationStats.cache_hits` / `cache_misses` delta values | Delivered (Ga plan 60-02; CMA this plan) | `saturating_sub` snapshot pattern before generation loop body; `None` when no cache |
| D-08: `wrap_with_cache` returns `(fn, handle)` tuple | Delivered (plan 60-01) | Used in Ga build() and CmaEngine run() scalar path |

## Discretion Choices

**D-03 CMA mutual-exclusivity:** Ga returns `GaError::ConfigurationError` when both `fitness_fn` (Optional) and `batch_evaluator` are set. CMA's `fitness_fn` is non-Optional (required at construction). Rather than making it Optional (breaking change) or returning an error when batch is configured alongside an always-present `fitness_fn`, CMA uses last-writer-wins semantics: when `batch_evaluator.is_some()`, all `(self.fitness_fn)(...)` calls are skipped in `run()`. This is consistent with D-03's intent (batch replaces scalar) without a breaking API change.

**Structural replication of `batch_evaluate_pop`:** The helper was replicated on `CmaEngine` rather than extracted to a shared free function. `Ga` uses a free function (`batch_evaluate`) to work around Rust's borrow checker (struct owns both the evaluator and the mutable population slice). `CmaEngine`'s population is a local variable inside `run()`, so the borrow issue doesn't arise — but extracting now would require a generic free function with complex bounds. Deferred to a future refactor phase.

## Assumption Verification

| Assumption | Status | Evidence |
|------------|--------|---------|
| A1: Passing `None` fitness_fn to parent_crossover suppresses per-child eval | Verified | `ga_batch_evaluator_replaces_calculate_fitness` — all chromosomes have batch-assigned 42.0 |
| A2: CmaEngine fields accessible in run() without borrow issues | Verified | `self.batch_evaluator.is_some()` / `self.fitness_cache` readable in run() without conflict |
| A3: Miss-chromosome clone is correct cost given D-01 locked `&[U]` signature | Accepted | Clone is O(chromosome_size); avoids unsafe aliasing; matches the Ga path |

## Threat Dispositions

| Threat | Disposition | Mitigation |
|--------|-------------|-----------|
| T-60-01: evaluate_batch returns wrong-length Vec | Mitigated | `debug_assert_eq!` in both `Ga::batch_evaluate` and `CmaEngine::batch_evaluate_pop` |
| T-60-02: Cache mutex poisoning | Accepted | `.expect("fitness cache lock poisoned")` surfaces immediately; matches existing pattern |
| T-60-04: Caller sets both fitness_fn and batch_evaluator | Mitigated | Ga: `GaError::ConfigurationError`; CMA: last-writer-wins (batch silently overrides scalar) |
| T-60-05: Lock held across expensive evaluate_batch call | Mitigated | Helper explicitly drops lock before `evaluate_batch`; re-acquires only for cache puts |

## Deferred Scope (out of Phase 60)

- `PsoEngine`, `EdaEngine`, `AlpsEngine`, `ScatterSearchEngine`, `CellularGaEngine`, `DeEngine` — batch+cache parity deferred to dedicated phases or a batch-wiring sweep
- Async `BatchFitnessEvaluator` (tokio-backed GPU evaluation) — deferred; requires async trait design and feature gate
- Per-observer cache hooks (notify observer of hit/miss events) — deferred to Observability milestone
- `batch_evaluate_pop` extraction to a shared utility — deferred to Performance/Refactor milestone

## Test Counts (Phase 60 active tests)

| Module | Active Tests |
|--------|-------------|
| `tests/engines/test_ga.rs::batch_evaluator_tests` | 8 |
| `tests/engines/cma/test_cma.rs::batch_and_cache_tests` | 5 |
| `tests/test_stats.rs` (cache_stats_default_none) | 1 |
| `tests/fitness/test_cache.rs` (wrap_with_cache_returns_handle) | 1 |
| **Total Phase 60 active tests** | **15** |

## CI Gate Results

| Command | Result |
|---------|--------|
| `cargo build` | PASS |
| `cargo build --features serde` | PASS |
| `cargo test` | PASS — 1207 passed, 45 ignored |
| `cargo test --features serde` | PASS — 1247 passed, 46 ignored |
| `cargo clippy --all-targets -- -D warnings` | PASS — zero warnings |
| `cargo doc --no-deps` | PASS — zero warnings |
| `cargo check --target wasm32-unknown-unknown` | PASS |

No remaining `#[ignore = "Wave 0 stub"]` entries in Phase 60 test modules — all 15 tests active.
