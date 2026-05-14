---
phase: 44-standard-benchmark-functions-suite
plan: 02
wave: 2
executor_model: Claude Code (Sonnet 4.6)
duration_minutes: 25
tasks_total: 3
tasks_completed: 3
files_modified:
  - src/benchmarks/mod.rs
files_created:
  - src/benchmarks/zdt.rs
  - src/benchmarks/dtlz.rs
commit_hashes:
  - a6ca76c: feat(44-02): implement ZDT1-ZDT6 benchmark functions
  - cc32f2d: feat(44-02): implement DTLZ1-DTLZ7 benchmark functions
requirements:
  - BEN-01
---

# Phase 44 Plan 02: ZDT and DTLZ Benchmark Functions

**One-liner:** Implemented ZDT1-ZDT6 bi-objective and DTLZ1-DTLZ7 many-objective benchmark functions as separate sub-modules, all implementing `BenchmarkFn`, with 23 unit tests verifying known Pareto properties, optimum values, dimension validation, and bounds correctness.

## Completed Tasks

### Task 1: ZDT1-ZDT6 in src/benchmarks/zdt.rs

- Created `src/benchmarks/zdt.rs` with 6 structs: `ZDT1`, `ZDT2`, `ZDT3`, `ZDT4`, `ZDT5`, `ZDT6`
- All implement `BenchmarkFn` with `evaluate(&[f64]) -> Vec<f64>` returning `vec![f1, f2]`
- ZDT1 (n=30): Convex Pareto front, f2 = g * (1 - sqrt(x0/g))
- ZDT2 (n=30): Non-convex Pareto front, f2 = g * (1 - (x0/g)^2)
- ZDT3 (n=30): Disconnected front (5 segments), f2 = g * (1 - sqrt(x0/g) - (x0/g)*sin(10*pi*x0))
- ZDT4 (n=10): Multimodal front with mixed bounds ([0,1] first var, [-5,5] rest)
- ZDT5 (n=11): Continuous relaxation of original binary problem (11 real vars in [0,1])
- ZDT6 (n=10): Non-convex front with biased density
- Updated `src/benchmarks/mod.rs` with `pub mod zdt` and re-exports

### Task 2: DTLZ1-DTLZ7 in src/benchmarks/dtlz.rs

- Created `src/benchmarks/dtlz.rs` with 7 structs: `DTLZ1` through `DTLZ7`
- All implement `BenchmarkFn` with `new(n_vars, n_obj)` constructor (no Default)
- DTLZ1: Linear hyperplane front with Rastrigin-like multimodal g (3^k local fronts)
- DTLZ2: Unit sphere Pareto front (quadratic g)
- DTLZ3: Unit sphere front with DTLZ1's multimodal g (combines DTLZ1 g + DTLZ2 f)
- DTLZ4: Sphere front with biased density via alpha parameter (default alpha=100)
- DTLZ5: Degenerate curve front via theta transformation
- DTLZ6: Degenerate curve with cubic g function
- DTLZ7: Disconnected Pareto front with sin-based h function
- Shared `evaluate_dtlz2_like_core()` across DTLZ2/3/4/5/6 to avoid code duplication
- Updated `src/benchmarks/mod.rs` with `pub mod dtlz` and re-exports

### Task 3: WASM compilation check

- `cargo check --target wasm32-unknown-unknown --features benchmarks` fails with 4 errors from `getrandom 0.3.1` (transitive dependency of `rand`), identical to the pre-existing failure from Phase 44-01
- The ZDT and DTLZ code is pure math (f64 arithmetic, sin, cos, sqrt, exp) with zero WASM-incompatible patterns
- Verified: No `std::time::Instant`, no `rayon`, no `std::thread` in either file

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check --features benchmarks` | Passed (0 errors, 0 warnings on new files) |
| `cargo test --features benchmarks -- zdt dtlz` | 24 passed (10 ZDT + 13 DTLZ + 1) |
| `cargo clippy --features benchmarks` | No warnings on new files |
| `cargo check --target wasm32-unknown-unknown --features benchmarks` | Pre-existing failure (getrandom, not benchmark code) |
| No WASM-incompatible patterns in benchmark code | Verified (no Instant, no rayon, no std::time) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing correctness] DTLZ shared evaluate_dtlz2_like_core extracted as module-level function**

- **Found during:** Task 2
- **Issue:** DTLZ2, DTLZ3, DTLZ4, DTLZ5, and DTLZ6 all share the same f_i product/trigonometric structure for computing objective values from position variables and g, leading to near-identical code.
- **Fix:** Extracted `evaluate_dtlz2_like_core(x_pos, g, m)` as an associated function on `DTLZ2` that takes pre-computed position angles and g value, avoiding 5x code duplication. Each struct computes its own g and position transformation (DTLZ2/3/4 use `x*pi/2`, DTLZ4 applies alpha, DTLZ5/6 use theta), then delegates to the shared core.

## Key Decisions

- ZDT5 uses a continuous relaxation (11 real variables in [0,1]) instead of the original binary encoding, matching the pure `&[f64]` `BenchmarkFn` interface. Each variable maps to an integer via `z_i = floor(1 + k_i * x_i)` where `k_0 = 30` and `k_i = 5` for i >= 1.
- DTLZ functions take `(n_vars, n_obj)` constructor parameters with no `Default` impl, as both dimensions are semantically required.
- ZDT4 stores heterogeneous bounds (first var [0,1], rest [-5,5]) as a `Vec<(f64, f64)>` built in `new()`.
- `evaluate_dtlz2_like_core` lives as an associated function on `DTLZ2` rather than a standalone function, keeping namespace clean while allowing reuse by DTLZ3-6.

## Known Stubs

None. All 13 benchmark functions are fully implemented with verified evaluations.

## Threat Flags

None. The threat model accepted T-44-02 (DoS via large evaluate input) as a design trade-off; all 13 functions are O(n) in decision variables.

## Self-Check

```
FOUND: src/benchmarks/zdt.rs
FOUND: src/benchmarks/dtlz.rs
FOUND: a6ca76c
FOUND: cc32f2d
```

## Self-Check: PASSED
