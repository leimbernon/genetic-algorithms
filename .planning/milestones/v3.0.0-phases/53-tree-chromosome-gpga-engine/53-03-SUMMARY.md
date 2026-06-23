---
phase: 53-tree-chromosome-gpga-engine
plan: "03"
subsystem: engines/gp
tags:
  - genetic-programming
  - gp-engine
  - ramped-half-and-half
  - observer-hooks
  - bloat-control
  - wasm-safe

dependency_graph:
  requires:
    - phase: 53-01
      provides: GpNode, Node<N>, GpChromosome<N>, GpConfiguration shell
    - phase: 53-02
      provides: GpCrossover::SubtreeCrossover, GpMutation variants, grow_tree/check_limits
  provides:
    - gp::ramped_half_and_half() — standard GP population initializer
    - gp::GpGa<N> — full GP engine with run() returning GpResult<N>
    - gp::GpResult<N> — population, best, best_fitness, generations
    - GpConfiguration extended with selection, survivor, mutations, crossover, is_maximization, max_stagnation, fitness_target
  affects:
    - tests/gp.rs (3 new Wave-2 tests activated)

tech_stack:
  added: []
  patterns:
    - "Type alias pattern (GpFitnessFn<N>, GpInitFn<N>) to satisfy clippy::type_complexity on Arc<dyn Fn> fields"
    - "init_fn closure owns its own RNG (make_rng()) — avoids dyn RngCore trait object complexity"
    - "WASM-gated par_iter_mut(): #[cfg(not(target_arch = wasm32))] guard on evaluate_population"
    - "Bloat retry loop: up to 3 crossover attempts then fallback to better parent copy (T-53-08)"
    - "selection::factory and survivor::factory reused unchanged from existing infrastructure"

key_files:
  created:
    - src/engines/gp/init.rs
    - src/engines/gp/engine.rs
  modified:
    - src/engines/gp/configuration.rs
    - src/engines/gp/mod.rs
    - tests/gp.rs

decisions:
  - "init_fn takes (usize, usize) not (usize, usize, &mut dyn RngCore) — avoids the Sized/dyn issue when passing grow_tree (which requires impl Rng sized) via trait object; each init_fn closure calls make_rng() internally"
  - "GpGa<N: GpNode + ...> not GpGa<U: TreeChromosome> — engine works exclusively with GpChromosome<N>; users who want a custom TreeChromosome wrapper can supply a custom init_fn that returns Vec<GpChromosome<N>>"
  - "GpFitnessFn<N> and GpInitFn<N> type aliases extracted to satisfy clippy::type_complexity (same pattern as TreeFitnessFn<N> in Wave 0)"
  - "observer field typed as Arc<dyn GaObserver<GpChromosome<N>> + Send + Sync> — same pattern as Ga<U> and DeEngine<U>"

metrics:
  duration: "10m 24s"
  completed: "2026-05-25"
  tasks_completed: 2
  files_modified: 5
---

# Phase 53 Plan 03: GP Engine (Wave 2) Summary

**GpGa::run() fully operational — ramped_half_and_half initializer, observer hooks, bloat retry, avg_node_count stats wired**

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Ramped half-and-half init + GpConfiguration extension | 8ec3489 | src/engines/gp/init.rs, src/engines/gp/configuration.rs, src/engines/gp/mod.rs |
| 2 | GpGa engine loop with observer hooks, bloat retry, avg_node_count | d1c94b3 | src/engines/gp/engine.rs, tests/gp.rs |

## Verification Results

- `cargo check` — zero errors
- `cargo check --target wasm32-unknown-unknown` — zero errors (par_iter_mut gated)
- `cargo test --test gp` — 15 passed, 0 failed, 0 ignored
- `cargo clippy -- -D warnings` — no warnings
- `cargo test` — no regressions (full suite passes)
- `cargo test --features serde` — 331 passed, 0 failed

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] dyn RngCore not Sized — init_fn closure signature redesigned**
- **Found during:** Task 2 (first `cargo check` after engine.rs creation)
- **Issue:** The plan described `init_fn: impl Fn(usize, usize, &mut dyn rand::RngCore) -> Vec<U>` but `dyn RngCore` is `!Sized`, and `ramped_half_and_half` takes `&mut impl Rng` (which requires `Sized`). Passing a `&mut dyn RngCore` into `ramped_half_and_half` fails because `impl Rng + ?Sized` is not object-safe for recursive calls inside `full_tree` and `grow_tree`.
- **Fix:** Changed `init_fn` signature to `Fn(usize, usize) -> Vec<GpChromosome<N>>`. Each closure creates its own RNG via `make_rng()` internally. This is cleaner: the engine's `rng` (used for crossover/mutation) is independent from the init-phase RNG.
- **Files modified:** `src/engines/gp/engine.rs`
- **Commit:** d1c94b3

**2. [Rule 1 - Bug] clippy::type_complexity on Arc<dyn Fn> fields**
- **Found during:** Task 2 (`cargo clippy -- -D warnings`)
- **Issue:** `Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>` triggers `clippy::type_complexity`.
- **Fix:** Added `type GpFitnessFn<N>` and `type GpInitFn<N>` type aliases. Same pattern applied in Wave 0 (`type TreeFitnessFn<N>`).
- **Files modified:** `src/engines/gp/engine.rs`
- **Commit:** d1c94b3 (same commit — fixed before committing)

## WASM Compatibility

`par_iter_mut()` is gated behind `#[cfg(not(target_arch = "wasm32"))]` in `GpGa::evaluate_population()`. The WASM fallback uses `iter_mut()` with identical closure body. `cargo check --target wasm32-unknown-unknown` passes.

## Known Stubs

None. All Wave 2 functionality is fully wired. The `test_serde_deep_tree` test remains `#[ignore]` as it is designated for Wave 3 (Plan 53-04).

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. All operations are in-memory tree manipulations. Threat model items T-53-07 through T-53-SC are addressed:
- T-53-07: panicking fitness_fn propagates as a Rust panic — documented behavior, same contract as all other engines
- T-53-08: hard cap of 3 crossover retries, fallback to better parent copy — maintains population size invariant
- T-53-09: `par_iter_mut()` gated with `#[cfg(not(target_arch = "wasm32"))]` in `evaluate_population`
- T-53-10: `stagnation_count` compared against `max_stagnation` (user-bounded `Option<usize>`) — no wrapping arithmetic
- T-53-SC: No new Cargo dependencies added in Wave 2

## Self-Check: PASSED

Files exist:
- FOUND: src/engines/gp/init.rs
- FOUND: src/engines/gp/engine.rs
- FOUND: src/engines/gp/configuration.rs (modified)
- FOUND: src/engines/gp/mod.rs (modified)
- FOUND: tests/gp.rs (modified)

Commits exist:
- FOUND: 8ec3489 (Task 1 — init.rs + configuration.rs)
- FOUND: d1c94b3 (Task 2 — engine.rs + tests)

Tests:
- cargo test --test gp: 15 passed, 0 ignored

## Next Phase Readiness

- Wave 3 (Plan 53-04) can import `GpGa` and call `run()` to get `GpResult<N>`
- `test_serde_deep_tree` is still `#[ignore]` — Wave 3 will implement serde support for deep trees
- `GpGa::with_observer()` is wired and tested — Wave 3 can use observer hooks for logging/tracing
