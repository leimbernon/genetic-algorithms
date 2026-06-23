---
phase: 51-multi-parent-crossover-self-adaptive-mutation
plan: "02"
subsystem: crossover-operators
tags:
  - crossover
  - multi-parent
  - real-valued
  - UNDX
  - SPX
  - PCX
  - factory-dispatch
dependency_graph:
  requires:
    - 51-01 (RealValued trait, Crossover::Undx/Spx/Pcx enum variants, Wave 0 test stubs)
  provides:
    - src/operations/crossover/undx.rs (undx() function)
    - src/operations/crossover/spx.rs (spx() function)
    - src/operations/crossover/pcx.rs (pcx() function)
    - src/operations/crossover.rs factory_multi_parent (public dispatch function)
    - src/operations/crossover.rs try_undx/try_spx/try_pcx (private dispatchers)
    - src/operations/mutation/self_adaptive_gaussian.rs (stub for Plan 03)
  affects:
    - src/operations/crossover.rs (new mods, new functions, updated match arms)
    - src/operations/mutation.rs (new pub mod self_adaptive_gaussian stub)
    - tests/operations/test_crossover_undx.rs (now GREEN)
    - tests/operations/test_crossover_spx.rs (now GREEN)
    - tests/operations/test_crossover_pcx.rs (now GREEN)
tech_stack:
  added: []
  patterns:
    - Multi-parent crossover operator pattern (mirrors sbx.rs structure)
    - Box-Muller N(0,sigma) sampling inline per operator file
    - Per-type downcast macro try_type!($t:ty) (mirrors try_sbx/try_blend pattern)
    - factory_multi_parent with RealValued bound (mirrors factory_lexicase pattern)
    - Gene clamping from sbx.rs (dna0[i].ranges[0] when ranges is non-empty)
key_files:
  created:
    - src/operations/crossover/undx.rs
    - src/operations/crossover/spx.rs
    - src/operations/crossover/pcx.rs
    - src/operations/mutation/self_adaptive_gaussian.rs
  modified:
    - src/operations/crossover.rs (pub mod declarations, try_* dispatchers, factory_multi_parent, match arm updates)
    - src/operations/mutation.rs (pub mod self_adaptive_gaussian stub declaration)
decisions:
  - Box-Muller sampling written inline per operator file (no shared helper) as specified
  - UNDX uses one global eta sample + per-gene xi samples (matches UNDX literature)
  - SPX spread computation moved to iterator map (avoids needless_range_loop clippy)
  - PCX orthogonal noise scales by per-gene spread across all parents
  - self_adaptive_gaussian stub created to unblock test_operations compilation (Rule 3)
  - Crossover match arms consolidated to single arm for Undx|Spx|Pcx in both impls
metrics:
  duration_seconds: 780
  completed_date: "2026-05-23T15:10:28Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 4
  files_modified: 2
---

# Phase 51 Plan 02: UNDX/SPX/PCX Operator Implementations — Summary

**One-liner:** Three multi-parent crossover operator files (UNDX, SPX, PCX) with Box-Muller sampling, plus `factory_multi_parent<U: LinearChromosome + RealValued>` dispatcher via per-type downcast macros covering f64/f32/i32/i64.

## Tasks Completed

| # | Task | Commit | Status |
|---|------|--------|--------|
| 1 | Implement undx(), spx(), pcx() operator functions + Rule 3 stub | e60bee7 | Done |
| 2 | Add factory_multi_parent + try_* dispatchers + mod declarations | 7eadb4c | Done |

## What Was Built

### Operator Files

**`src/operations/crossover/undx.rs`** — UNDX implementation:
- `pub fn undx<T: SbxConvertible>(parents: &[&RangeChromosome<T>], _num_parents: usize) -> Result<Vec<RangeChromosome<T>>, GaError>`
- Algorithm: centroid + `eta * dir_normalized + xi_per_gene` perturbation
- Parameters: `sigma_xi = 0.35 / sqrt(n-1)`, `sigma_eta = 0.35 / sqrt(n)`
- One global eta sample, per-gene xi via Box-Muller; dir_norm floored at 1e-14

**`src/operations/crossover/spx.rs`** — SPX implementation:
- `pub fn spx<T: SbxConvertible>(parents: &[&RangeChromosome<T>], _num_parents: usize) -> Result<Vec<RangeChromosome<T>>, GaError>`
- Algorithm: expand simplex by `epsilon = sqrt(n+2)`, sample via iterative r_k combination
- r_k drawn as `U(0,1)^(1/(n-1-k))` for k in 0..n-1, then r[n-1]=1.0
- Iterates backwards from last expanded vertex to produce uniform interior sample

**`src/operations/crossover/pcx.rs`** — PCX implementation:
- `pub fn pcx<T: SbxConvertible>(parents: &[&RangeChromosome<T>], _num_parents: usize) -> Result<Vec<RangeChromosome<T>>, GaError>`
- Algorithm: centered on parents[0], directional noise toward each other parent + orthogonal spread noise
- Parameters: `sigma_eta = 0.1`, `sigma_zeta = 0.1`; spread = max-min across all parents per gene

All three:
- Validate `parents.len() >= 3` returning `GaError::CrossoverError("... requires at least 3 parents")`
- Validate uniform DNA length across parents
- Handle empty DNA case (return `Ok(vec![RangeChromosome::new()]`)
- Use `crate::rng::make_rng()` (WASM-safe)
- Gene clamping via `dna0[i].ranges[0]` when ranges is non-empty
- Emit debug logs: `debug!(target: "crossover_events", method = "..."; ...)`
- No `rayon`, no `std::time::Instant`

### Factory Dispatcher

**`src/operations/crossover.rs`** additions:
- `pub mod undx;`, `pub mod spx;`, `pub mod pcx;` declarations (alphabetical)
- `fn try_undx`, `fn try_spx`, `fn try_pcx` — private dispatchers using `try_type!` macro covering f64/f32/i32/i64 (4 types × 3 dispatchers = 12+ downcast sites via macro expansion)
- `pub fn factory_multi_parent<U: LinearChromosome + RealValued>(parents: &[&U], configuration: CrossoverConfiguration) -> Result<Vec<U>, GaError>` — top-level dispatch with rustdoc
- Updated `Crossover::Undx | Spx | Pcx` match arms in both `impl CrossoverOperator for Crossover` and `impl CrossoverOperator for CrossoverConfiguration` to return "Multi-parent crossover variant invoked through 2-parent factory; use factory_multi_parent"

### Numeric types covered by try_* dispatchers
`f64`, `f32`, `i32`, `i64` — all four SbxConvertible concrete types.

### Tests now passing (Wave 0 → GREEN)

| Test | Location | Status |
|------|----------|--------|
| `undx_produces_one_offspring_within_bounds` | tests/operations/test_crossover_undx.rs | GREEN |
| `undx_rejects_fewer_than_three_parents` | tests/operations/test_crossover_undx.rs | GREEN |
| `spx_produces_one_offspring_within_bounds` | tests/operations/test_crossover_spx.rs | GREEN |
| `spx_offspring_within_expanded_simplex` | tests/operations/test_crossover_spx.rs | GREEN |
| `pcx_produces_one_offspring_within_bounds` | tests/operations/test_crossover_pcx.rs | GREEN |
| `pcx_offspring_biased_toward_primary_parent` | tests/operations/test_crossover_pcx.rs | GREEN |

## Verification Results

```
cargo build                           → exit 0
cargo build --features serde          → exit 0
cargo clippy -- -D warnings           → no issues
cargo test --test test_operations (undx filter) → 2 passed
cargo test --test test_operations (spx filter)  → 2 passed
cargo test --test test_operations (pcx filter)  → 2 passed
cargo check --target wasm32-unknown-unknown      → exit 0
git diff src/traits/operators.rs      → empty (trait unchanged)
git diff factory() function           → no deletions
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Created self_adaptive_gaussian stub to unblock test compilation**
- **Found during:** Task 1 — `cargo test --test test_operations` failed with E0432 on `genetic_algorithms::operations::mutation::self_adaptive_gaussian::self_adaptive_gaussian_mutation`. The `test_mutation_self_adaptive.rs` (Plan 03's Wave 0 RED test) is registered in `test_operations.rs` alongside the UNDX/SPX/PCX tests. Its compilation failure prevented all tests in the binary from running.
- **Fix:** Created `src/operations/mutation/self_adaptive_gaussian.rs` with a stub `self_adaptive_gaussian_mutation()` that returns `GaError::MutationError("not yet implemented (Plan 03 stub)")`. Added `pub mod self_adaptive_gaussian;` to `mutation.rs`.
- **Files modified:** `src/operations/mutation/self_adaptive_gaussian.rs` (new), `src/operations/mutation.rs` (mod decl)
- **Commit:** e60bee7
- **Plan 03 impact:** Plan 03 will replace the stub with the real implementation. The test file's expected behavior (sigma_min enforcement, sigma spread evolution) will become GREEN after Plan 03.

**2. [Rule 1 - Bug] Consolidated Crossover match arms to fix redundancy**
- **Found during:** Task 2 — Plan 01 added separate `Crossover::Undx { .. }`, `Crossover::Spx { .. }`, `Crossover::Pcx { .. }` arms in both `impl CrossoverOperator` blocks with the message "implemented in Plan 02." After Plan 02 ships, the message became stale.
- **Fix:** Consolidated to `Crossover::Undx { .. } | Crossover::Spx { .. } | Crossover::Pcx { .. }` with the canonical message specified by Plan 02: "Multi-parent crossover variant invoked through 2-parent factory; use factory_multi_parent".
- **Files modified:** `src/operations/crossover.rs`
- **Commit:** 7eadb4c

## Known Stubs

`src/operations/mutation/self_adaptive_gaussian.rs` — stub implementation returning an error. This is intentional: Plan 03 will replace with the real log-normal self-adaptive mutation. The stub allows `test_operations` to compile while Plan 03's Wave 0 test remains in RED state.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes. All new surface is in-process operator functions.

## Self-Check: PASSED

Files exist:
- `src/operations/crossover/undx.rs` — FOUND
- `src/operations/crossover/spx.rs` — FOUND
- `src/operations/crossover/pcx.rs` — FOUND
- `src/operations/mutation/self_adaptive_gaussian.rs` — FOUND

Commits verified:
- e60bee7 (feat(51-02): implement UNDX, SPX, and PCX...) — FOUND
- 7eadb4c (feat(51-02): add factory_multi_parent...) — FOUND
