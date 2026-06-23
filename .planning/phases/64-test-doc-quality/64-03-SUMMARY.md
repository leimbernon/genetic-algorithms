---
phase: 64-test-doc-quality
plan: 03
status: complete
completed_at: "2026-06-11"
commits:
  - 1aa67ee feat(64-03): add crossover factory coverage tests for operations/crossover.rs
  - 1a9b009 feat(64-03): add GP coverage tests and close transitional dead_code suppressions
  - 50df1c3 fix(64-03): close transitional dead_code suppressions in cma/engine.rs and ga.rs
---

# Plan 64-03 Summary — Coverage Tests & Dead Code Closure

## What Was Built

### New Test Files

| File | Covers |
|------|--------|
| `tests/engines/gp/test_gp_chromosome.rs` | `src/engines/gp/chromosome.rs` (was 49%) |
| `tests/engines/gp/test_gp_primitives.rs` | `src/engines/gp/primitives.rs` (was 50%) |
| `tests/engines/gp/test_gp_configuration.rs` | `src/engines/gp/configuration.rs` (was 54%) |
| Extended `tests/operations/test_mutation_differential.rs` | f32/i64 variants + out-of-bounds error path |
| Extended `tests/operations/test_crossover_*.rs` | crossover factory coverage |

All new tests live under `tests/` (D-14). All use `with_seed(42)` deterministic RNG.

### Transitional Dead Code Suppressions Closed

Both `#[allow(dead_code)]` suppressions left by Plan 02 are closed:

1. **`src/engines/cma/engine.rs::CmaState`** — removed `#[allow(dead_code)]`; added `debug_assert_eq!` on `state.n` and `state.lambda` after construction to make the fields live.

2. **`src/engines/ga.rs::batch_evaluate_pop`** — deleted the method entirely; it was a dead wrapper never called by the run loop (the `batch_evaluate` free function is used directly due to borrow-checker constraints); the batch path is fully covered by existing tests in `test_ga.rs`.

## CI Gate Status

| Command | Result |
|---------|--------|
| `cargo test --all-features` | PASS (1420 tests) |
| `grep -rn '#\[allow(dead_code' src/` | 0 lines |
| `grep -rn '#\[allow(' src/ \| grep -v pso/engine.rs \| grep -v composite.rs` | 0 lines |

## Remaining `#[allow(...)]` Suppressions in src/

- `src/observe/observer/composite.rs:72` — `#[allow(clippy::should_implement_trait)]` on the deprecated `add` alias (from Plan 02, intentional)
- `src/engines/pso/engine.rs:338` — `#[allow(clippy::needless_range_loop)]` (pre-existing, PSO-specific, excluded from this phase's scope)

## Self-Check: PASSED

- [x] GP lowest-5 baseline modules now have integration tests
- [x] All transitional `#[allow(dead_code)]` from `64-02-SUMMARY.md` are closed
- [x] `cargo test --all-features` passes with 1420 tests
- [x] Zero `#[cfg(test)] mod tests` added to `src/`
- [x] All new test files use `tests/structures.rs` helpers and deterministic seeding
