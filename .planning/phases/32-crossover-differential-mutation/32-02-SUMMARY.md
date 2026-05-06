---
phase: 32-crossover-differential-mutation
plan: "02"
subsystem: mutation-operators
tags: [mutation, differential-evolution, range-chromosome, de-style]
dependency_graph:
  requires: ["32-01"]
  provides: ["differential_mutation free function", "Mutation::Differential enum variant", "MutationConfiguration::differential_f field", "MutationConfig::with_differential_f trait method"]
  affects: ["src/operations.rs", "src/operations/mutation.rs", "src/operations/mutation/differential.rs", "src/configuration.rs", "src/traits/configuration.rs"]
tech_stack:
  added: []
  patterns: ["enum variant + free function (not MutationOperator trait)", "macro-based type dispatch (try_type!)", "GaussianConvertible for f64 arithmetic on integer genes"]
key_files:
  created:
    - src/operations/mutation/differential.rs
    - tests/operations/test_mutation_differential.rs
  modified:
    - src/operations.rs
    - src/operations/mutation.rs
    - src/configuration.rs
    - src/traits/configuration.rs
    - tests/observe/test_serde.rs
    - tests/test_operations.rs
decisions:
  - "with_differential_f uses default body `where Self: Sized` in trait to compile standalone before Plan 03 adds proper Ga<U> impl"
  - "differential_mutation mutates all genes per call (DE-style), unlike gaussian which mutates one gene"
  - "clamping uses ranges[0] per gene (first declared range) matching gaussian.rs approach"
metrics:
  duration_seconds: 319
  completed_date: "2026-05-06T07:41:59Z"
  tasks_completed: 2
  files_changed: 8
---

# Phase 32 Plan 02: Differential Mutation Operator Summary

DE-style `differential_mutation` free function for `Range<T>` chromosomes: `mutant[i] = x_r1[i] + F * (x_r2[i] - x_r3[i])` clamped to gene ranges, with `Mutation::Differential` enum variant, `MutationConfiguration::differential_f` config field, and `MutationConfig::with_differential_f` trait builder method.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Enum variant, config field, trait builder, safety-net arms | 74f5c2c | src/operations.rs, src/operations/mutation.rs, src/configuration.rs, src/traits/configuration.rs, src/operations/mutation/differential.rs |
| 2 | differential_mutation implementation and 6 tests | 6416f55 | tests/operations/test_mutation_differential.rs, tests/test_operations.rs |

## Verification

- `cargo test` — 736 passed (all suites)
- `cargo test --features serde` — 766 passed (all suites)
- `cargo clippy --all-targets -- -D warnings` — clean

## Decisions Made

1. **Default trait body for `with_differential_f`**: Added `where Self: Sized` default body so the standalone plan compiles without requiring Plan 03 to add the `Ga<U>` impl. Plan 03 will add proper implementations to `GaConfiguration` and `Ga<U>` that set `mutation_configuration.differential_f`.

2. **All-gene mutation**: `differential_mutation` mutates all genes per invocation (DE-style), unlike `gaussian_mutation` which mutates a single randomly chosen gene.

3. **Range access**: Uses `gene.ranges[0]` (first declared range) per gene for clamping bounds. This mirrors `gaussian.rs` behavior.

4. **Type dispatch via `try_type!` macro**: Supports `f64`, `f32`, `i32`, `i64` — same as gaussian and polynomial operators.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] serde test struct literal missing new field**
- **Found during:** cargo test --features serde
- **Issue:** `tests/observe/test_serde.rs` constructs `MutationConfiguration` with all fields explicit; adding `differential_f` to the struct made it fail to compile.
- **Fix:** Added `differential_f: None` to the struct literal in the serde test.
- **Files modified:** tests/observe/test_serde.rs
- **Commit:** 723a8ef

## Known Stubs

None — `differential_mutation` is fully implemented. The `with_differential_f` trait method has a default body that is a no-op (returns `self` unchanged) until Plan 03 wires up the proper `Ga<U>` implementation.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All threats identified in the plan's threat model are mitigated:
- T-32-05 (DoS/panic with small pop): D-03 guard returns error before index sampling
- T-32-06 (wrong output on Binary): D-02 type check falls through to explicit error
- T-32-07 (i32/i64 overflow): all arithmetic in f64 via GaussianConvertible
- T-32-08 (out-of-range gene): mandatory `clamp(lo_f, hi_f)` before writing back
- T-32-09 (misuse via factory): safety-net arms in `mutate()` and `factory_non_value()`

## Self-Check: PASSED

- `src/operations/mutation/differential.rs` — EXISTS
- `src/configuration.rs` contains `differential_f: Option<f64>` — CONFIRMED
- `src/traits/configuration.rs` contains `fn with_differential_f` — CONFIRMED
- `tests/operations/test_mutation_differential.rs` has 6 test functions — CONFIRMED
- Commits 74f5c2c, 6416f55, 723a8ef — EXIST in git log
