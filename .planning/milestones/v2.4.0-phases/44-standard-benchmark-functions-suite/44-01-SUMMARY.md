---
phase: 44-standard-benchmark-functions-suite
plan: 01
wave: 1
executor_model: Claude Code (Sonnet 4.6)
duration_minutes: 18
tasks_total: 2
tasks_completed: 2
files_modified:
  - Cargo.toml
  - src/lib.rs
files_created:
  - src/benchmarks/mod.rs
  - src/benchmarks/single_objective.rs
commit_hashes:
  - 579b3e1: feat(44-01): add BenchmarkFn trait and single-objective functions behind benchmarks feature flag
requirements:
  - BEN-01
---

# Phase 44 Plan 01: Benchmark Module Foundation

**One-liner:** Created `src/benchmarks/` module behind a new `benchmarks` feature flag with the `BenchmarkFn` trait defining `name()`, `bounds()`, `optimum_value()`, and `evaluate(&[f64]) -> Vec<f64>` and three single-objective implementations (Sphere, Rastrigin, Ackley) with 7 unit tests verifying known optima and dimension validation.

## Completed Tasks

### Task 1: Feature flag, lib.rs entry, trait definition, and single-objective functions

- Added `benchmarks = []` to `Cargo.toml` `[features]` section
- Added `#[cfg(feature = "benchmarks")] pub mod benchmarks;` to `src/lib.rs`
- Created `src/benchmarks/mod.rs` with:
  - `BenchmarkFn` trait (name, bounds, optimum_value, evaluate)
  - `pub use` re-exports of Sphere, Rastrigin, Ackley
- Created `src/benchmarks/single_objective.rs` with:
  - `Sphere` struct (n=30 default, -5.12..5.12 domain, optimum at origin)
  - `Rastrigin` struct (n=30 default, -5.12..5.12 domain, optimum at origin)
  - `Ackley` struct (n=30 default, -32..32 domain, optimum at origin)
  - 7 unit tests: optimum values, known non-zero values, dimension mismatch panic, bounds length
- Verified: `cargo check --features benchmarks` compiles cleanly
- Verified: `cargo test --features benchmarks -- single_objective` passes all 7 tests
- Verified: No clippy warnings from new files

### Task 2: WASM compilation check

- Verified: The `getrandom` dependency issue (from `rand` crate, not benchmark code) causes `cargo check --target wasm32-unknown-unknown` to fail even without `--features benchmarks` -- this is a pre-existing dependency issue unrelated to benchmark code.
- Verified: The benchmark code itself contains zero WASM-incompatible patterns (no `std::time::Instant`, no `rayon`, no threads).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing runtime correctness] bounds() storage uses runtime Vec instead of compile-time const slice**

- **Found during:** Task 1
- **Issue:** The plan specifies `bounds()` returns `&[(LOW, HIGH); self.n]` (a const-length replicated array), but `self.n` is a runtime value -- a `[T; N]` where `N` is a compile-time const cannot be created from a runtime `usize`. This is a fundamental Rust constraint.
- **Fix:** Stored per-variable bounds as `Vec<(f64, f64)>` in each struct, populated during `new()` and dereferenced by `bounds()`. This satisfies the `bounds().len() == self.n` test exactly as the plan intended.
- **Files modified:** `src/benchmarks/single_objective.rs`
- **Commit:** 579b3e1

### Pre-existing Issues (not caused by this plan)

**WASM compilation failure:** The `getrandom 0.3.1` crate, a transitive dependency of `rand`, does not support `wasm32-unknown-unknown` without additional configuration flags. This affects the entire project, not just the benchmark module. The CI workflow (`wasm-check.yml`) runs `cargo check --target wasm32-unknown-unknown --lib` with default features -- this command also fails identically on this branch without benchmarks. The benchmark code itself is pure math and WASM-compatible. This issue is outside the scope of this plan.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check --features benchmarks` | Passed (no errors, no warnings on new files) |
| `cargo test --features benchmarks -- single_objective` | 7 passed |
| `cargo clippy --features benchmarks` | No warnings on new benchmark files (10 pre-existing warnings in other modules) |
| `cargo check --target wasm32-unknown-unknown --features benchmarks` | Pre-existing failure (getrandom 0.3.1, not benchmark code) |
| No WASM-incompatible patterns in benchmark code | Verified (no Instant, no rayon, no std::time) |

## Key Decisions

- Each benchmark struct stores bounds as `Vec<(f64, f64)>` to support runtime-variable dimensions, rather than relying on compile-time const array replication (see deviation above).
- Default dimension is 30 for all functions, matching standard literature conventions (ZDT conventions).
- Tests include dimension mismatch panic verification to enforce the documented panic contract.

## Known Stubs

None. All three benchmark functions are fully implemented with verified optima.

## Threat Flags

None. The threat model accepted T-44-01 (DoS via large evaluate input) as a design trade-off; no new attack surface introduced.
