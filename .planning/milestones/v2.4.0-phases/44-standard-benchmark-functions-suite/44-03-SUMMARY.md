---
phase: 44-standard-benchmark-functions-suite
plan: 03
wave: 3
executor_model: Claude Code (Sonnet 4.6)
duration_minutes: 22
tasks_total: 3
tasks_completed: 3
files_modified:
  - Cargo.toml
  - benches/de.rs
  - src/benchmarks/single_objective.rs
  - src/benchmarks/zdt.rs
  - src/benchmarks/dtlz.rs
  - examples/sms_emoa_zdt1.rs
  - examples/ibea_zdt1.rs
  - .planning/REQUIREMENTS.md
commit_hashes:
  - 6e06c02: feat(44-03): add serde derives to benchmark structs and migrate benches/de.rs
  - 17a0e45: feat(44-03): migrate sms_emoa_zdt1 and ibea_zdt1 examples to shared ZDT1
  - 8d79ed1: docs(44-03): mark BEN-01 complete with traceability to three plans
requirements:
  - BEN-01
---

# Phase 44 Plan 03: Serde Derives, Existing File Migration, and Verification Gate

**One-liner:** Added conditional serde derives to all 16 benchmark structs with 10 round-trip tests, migrated benchmarks/de.rs (D-10) and examples/sms_emoa_zdt1/ibea_zdt1 (D-12) to use shared library benchmarks, updated Cargo.toml bench entries, and closed BEN-01 with full traceability.

## Completed Tasks

### Task 1: Serde derives on benchmark structs and benches/de.rs migration

- Added `#[derive(Clone, Debug)]` and `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` to all 16 benchmark structs across three files (3 in single_objective.rs, 6 in zdt.rs, 7 in dtlz.rs)
- Added 10 serde round-trip tests gated behind `#[cfg(feature = "serde")]`: 3 for single-objective (Sphere, Rastrigin, Ackley), 3 for ZDT (ZDT1, ZDT4, ZDT6), 4 for DTLZ (DTLZ1, DTLZ2, DTLZ4, DTLZ7)
- Migrated `benches/de.rs` to import `Sphere` from `genetic_algorithms::benchmarks` and use `Sphere::evaluate()` instead of the locally-defined `sphere()` function (D-10 compliance)
- Added `required-features = ["benchmarks"]` to the `[[bench]]` `de` entry in `Cargo.toml`
- Verification: `cargo check --bench de --features benchmarks` compiles cleanly

### Task 2: Migrate examples using inline benchmark functions to shared library

- Migrated `examples/sms_emoa_zdt1.rs` to import `ZDT1` from `genetic_algorithms::benchmarks` and use `ZDT1::evaluate()` for both objective functions (D-12 compliance)
- Migrated `examples/ibea_zdt1.rs` with the same pattern
- Cloned `ZDT1` into each closure to satisfy `'static` lifetime requirement of `Box<dyn Fn(...) + Send + Sync + 'static>`
- Updated doc comment run commands to `cargo run --example <name> --features benchmarks`
- Verified both examples run successfully with correct Pareto front output

### Task 3: Phase verification gate and REQUIREMENTS.md traceability

- `cargo test --lib --features benchmarks`: 79 passed
- `cargo test --lib --features benchmarks,serde`: 89 passed (including 10 serde round-trip tests)
- `cargo clippy --features benchmarks,serde --lib`: 0 errors (pre-existing warnings only, no new warnings from benchmark code)
- `cargo check --target wasm32-unknown-unknown --features benchmarks`: Pre-existing failure from getrandom 0.3.1 (documented in 44-01-SUMMARY.md)
- `cargo doc --no-deps --features benchmarks`: Generated successfully
- Updated REQUIREMENTS.md: marked BEN-01 as complete (`- [x]`) with traceability reference `44-01-PLAN.md, 44-02-PLAN.md, 44-03-PLAN.md`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Serde round-trip tests were not specified in plan but needed for verification**

- **Found during:** Task 1
- **Issue:** The plan verification and done criteria reference "serde round-trip tests pass" but no serde round-trip tests existed. The `cargo test --features benchmarks,serde` step would verify derives compile but wouldn't confirm serialization/deserialization round-trip correctness.
- **Fix:** Added 10 serde round-trip tests across all three benchmark files, gated behind `#[cfg(feature = "serde")]`. Tests verify JSON serialization round-trips correctly reconstructing fields for each struct.
- **Files modified:** `src/benchmarks/single_objective.rs`, `src/benchmarks/zdt.rs`, `src/benchmarks/dtlz.rs`
- **Commit:** 6e06c02

**2. [Rule 1 - Bug] Closures capturing ZDT1 via reference required `'static` lifetime**

- **Found during:** Task 2
- **Issue:** The initial closure pattern `{ let z = &zdt1; move |...| z.evaluate(...) }` failed because `Box<dyn Fn(...)>` defaults to `'static` lifetime, but `zdt1` is a local variable that doesn't live long enough. Compiler error: "does not live long enough."
- **Fix:** Changed to cloning `ZDT1` into each `move` closure (ZDT1 now implements `Clone` from Task 1). Each closure owns its copy of ZDT1, satisfying the `'static` bound.
- **Files modified:** `examples/sms_emoa_zdt1.rs`, `examples/ibea_zdt1.rs`
- **Commit:** 17a0e45

### Pre-existing Issues (not caused by this plan)

- `cargo test --features benchmarks,serde` (full integration test suite) fails with `test_serde.rs:119:18: missing field` errors from `GaConfiguration` — this is a pre-existing issue from Phase 43 AOS changes where `GaConfiguration` struct gained new fields (`aos_reward_window`, `aos_strategy`, `crossover_portfolio`, `aos_fitness_share`) but `tests/observe/test_serde.rs` wasn't updated. Library tests pass (`cargo test --lib --features benchmarks,serde`: 89 passed).
- WASM compilation failure: pre-existing from getrandom 0.3.1, documented in 44-01-SUMMARY.md and 44-02-SUMMARY.md.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check --features benchmarks,serde` | Passed (0 errors) |
| `cargo test --lib --features benchmarks` | 79 passed |
| `cargo test --lib --features benchmarks,serde` | 89 passed |
| `cargo clippy --features benchmarks,serde --lib` | 0 errors (pre-existing warnings only) |
| `cargo check --target wasm32-unknown-unknown --features benchmarks` | Pre-existing failure (getrandom, not benchmark code) |
| `cargo doc --no-deps --features benchmarks` | Generated successfully |
| `cargo check --bench de --features benchmarks` | Passed |
| `cargo run --example sms_emoa_zdt1 --features benchmarks` | Runs successfully |
| `cargo run --example ibea_zdt1 --features benchmarks` | Runs successfully |

## Key Decisions

- Used `move` closures that own cloned instances of benchmark structs to satisfy `'static` lifetime requirements of boxed trait objects, rather than using `Arc` or `Box::leak`. Cloning ZDT1 (which is just a `usize` + `Vec<(f64,f64)>`) is negligible overhead.
- Added serde round-trip tests for representative structs from each module: 3 for single-objective (Sphere, Rastrigin, Ackley), 3 for ZDT (ZDT1, ZDT4, ZDT6), 4 for DTLZ (DTLZ1, DTLZ2, DTLZ4, DTLZ7). Covers all struct categories including edge cases (ZDT4 with mixed bounds, DTLZ4 with alpha field).

## Known Stubs

None. All 16 benchmark structs have verified serde derives, benches/de.rs uses shared Sphere, and two ZDT examples use shared ZDT1.

## Threat Flags

None. The threat model accepted T-44-03 (serde deserialization) as an accept disposition. Serde derives follow the established codebase pattern (`#[cfg_attr(feature = "serde", derive(...))]`) used throughout the library.

## Self-Check

```
FOUND: Cargo.toml
FOUND: benches/de.rs
FOUND: src/benchmarks/single_objective.rs
FOUND: src/benchmarks/zdt.rs
FOUND: src/benchmarks/dtlz.rs
FOUND: examples/sms_emoa_zdt1.rs
FOUND: examples/ibea_zdt1.rs
FOUND: 6e06c02
FOUND: 17a0e45
FOUND: 8d79ed1
```

## Self-Check: PASSED
