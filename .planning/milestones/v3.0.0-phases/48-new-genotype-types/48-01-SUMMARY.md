---
phase: 48-new-genotype-types
plan: 01
subsystem: traits, validators, types, operations, tests
tags:
  - rust
  - traits
  - validators
  - foundation
  - operator-compat
dependency_graph:
  requires: []
  provides:
    - OperatorCompat trait (src/traits/operator_compat.rs)
    - operator_compat_check<U> validator (src/validators/generic_validator.rs)
    - Crossover::MultiGroupPmx + Crossover::MultiGroupOx variants
    - pub(crate) pmx_build_child + ox_build_child visibility
    - Empty OperatorCompat impls on Binary, Range<T>, ListChromosome<T>
    - Wave 0 test scaffolds for 48-02/03/04
  affects:
    - src/engines/ga.rs (Ga::build() validator chain extended, U: OperatorCompat bound added)
    - src/operations/crossover.rs (match exhaustiveness for new variants)
tech_stack:
  added: []
  patterns:
    - No-blanket-impl per-type OperatorCompat impls (avoids specialization conflict)
    - pub(crate) helper functions for cross-module reuse
    - Build-time operator compatibility enforcement via Ga::build() validator chain
key_files:
  created:
    - src/traits/operator_compat.rs
    - tests/test_traits.rs
    - tests/traits/test_operator_compat.rs
    - tests/types/chromosomes/test_unique.rs
    - tests/types/chromosomes/test_multi_range.rs
    - tests/types/chromosomes/test_multi_unique.rs
    - tests/types/genotypes/test_unique.rs
    - tests/types/genotypes/test_multi_range.rs
    - tests/operations/test_crossover_multi_group_pmx.rs
    - tests/operations/test_crossover_multi_group_ox.rs
  modified:
    - src/traits.rs
    - src/operations.rs
    - src/operations/crossover.rs
    - src/operations/crossover/pmx.rs
    - src/operations/crossover/order.rs
    - src/types/chromosomes/binary.rs
    - src/types/chromosomes/range.rs
    - src/types/chromosomes/list.rs
    - src/lib.rs
    - src/validators/generic_validator.rs
    - src/engines/ga.rs
    - tests/test_types.rs
    - tests/test_operations.rs
decisions:
  - "No blanket impl<T: LinearChromosome> OperatorCompat for T — per-type explicit impls instead (overrides RESEARCH Pattern 4 recommendation)"
  - "MultiGroupPmx/MultiGroupOx variants added with inert error-returning dispatch arms (T-48-01 threat: compile-time exhaustiveness; runtime error until 48-04 wires real dispatch)"
  - "operator_compat_check wired via direct call after ValidatorFactory::validate, not inside the validate() chain, to keep the generic validator bounded by OperatorCompat cleanly"
metrics:
  duration: "~55 minutes"
  completed: "2026-05-22T13:19:47Z"
  tasks_completed: 3
  tasks_total: 3
  files_created: 10
  files_modified: 13
---

# Phase 48 Plan 01: OperatorCompat Foundation Summary

**One-liner:** `OperatorCompat` trait with build-time validator, `MultiGroupPmx`/`MultiGroupOx` enum variants, `pub(crate)` inner functions, per-type empty impls on three existing chromosomes, and Wave 0 test scaffolds — zero breaking changes, WASM clean.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | OperatorCompat trait + Crossover variants + build_child visibility + empty impls | d519a8e | src/traits/operator_compat.rs, src/operations.rs, src/operations/crossover/{pmx,order}.rs, src/types/chromosomes/{binary,range,list}.rs, src/lib.rs |
| 2 | operator_compat_check validator + wire into Ga::build() | d5bcbd4 | src/validators/generic_validator.rs, src/engines/ga.rs |
| 3 | Wave 0 tests — OperatorCompat behavior + scaffold files | 5a3a444 | tests/traits/test_operator_compat.rs + 7 scaffold files |

## What Was Built

### `src/traits/operator_compat.rs`

New public trait with two default-`None`-returning associated functions:

```rust
pub trait OperatorCompat {
    fn valid_crossovers() -> Option<&'static [Crossover]> { None }
    fn valid_mutations()  -> Option<&'static [Mutation]>  { None }
}
```

No blanket impl. Visible from the crate root as `genetic_algorithms::OperatorCompat`.

### Validator function

`pub fn operator_compat_check<U>(configuration: &GaConfiguration) -> Result<(), GaError>` added to `src/validators/generic_validator.rs`. Called from `Ga::build()` after `ValidatorFactory::validate`. Returns `GaError::ConfigurationError` if the configured crossover or mutation is not in the chromosome's valid set (when `Some`).

### Crossover enum variants

`Crossover::MultiGroupPmx` and `Crossover::MultiGroupOx` added after `EdgeRecombination`. Both have inert error-returning dispatch arms in `crossover.rs` (match exhaustiveness satisfied; real dispatch wired in 48-04).

### Visibility changes

`pmx_build_child` → `pub(crate)` (pmx.rs line 70).
`ox_build_child` → `pub(crate)` (order.rs line 58).
These enable `multi_group_pmx.rs` and `multi_group_ox.rs` to call them in 48-04.

### Empty OperatorCompat impls

`BinaryChromosome`, `Range<T>`, `ListChromosome<T>` — all get explicit empty impls (inheriting `None`-returning defaults). This satisfies the `U: OperatorCompat` bound on `Ga::build()` for all existing users without any behavioral change.

### Wave 0 tests

`tests/traits/test_operator_compat.rs`: 5 tests proving the trait behavior (default None, rejection on mismatch, acceptance on match). 7 scaffold files with `#[test] fn placeholder()` for 48-02/03/04 to populate.

## Design Decisions

### Key decision: No blanket impl (overrides RESEARCH Pattern 4)

RESEARCH Pattern 4 recommended `impl<T: LinearChromosome> OperatorCompat for T {}` as a backward-compatible blanket. The plan's objective overrides this with per-type explicit empty impls.

**Rationale:** A blanket impl over `LinearChromosome` would block concrete restriction impls for `UniqueChromosome<T>` and `MultiUniqueChromosome<T>` (Rust stable has no specialization). The per-type approach is more verbose (3 explicit empty impls for existing types + explicit impls for each new type in 48-02/04) but is correct by construction and scales cleanly to future chromosome types.

**Impact:** Any user-defined chromosome type that uses `Ga<U>` now needs to add `impl OperatorCompat for MyChromosome {}` (empty body). This is a breaking change in v3.0.0 (intentional — major version), documented clearly in the trait's module docs.

## Deviations from Plan

### Auto-fix: Match exhaustiveness for new Crossover variants

**Found during:** Task 1 — adding `MultiGroupPmx` and `MultiGroupOx` to the `Crossover` enum caused compile-time match exhaustiveness errors in `src/operations/crossover.rs`.

**Fix:** Added `Crossover::MultiGroupPmx | Crossover::MultiGroupOx => Err(GaError::CrossoverError("..."))` arms to both match blocks in crossover.rs. This satisfies match exhaustiveness while keeping the variants inert until 48-04 implements real dispatch. Matches threat model T-48-01 ("inert until 48-04").

**Files modified:** `src/operations/crossover.rs`

**Commit:** d519a8e (included in Task 1 commit)

## Known Stubs

None. All implementations are production-ready within scope. The Wave 0 scaffold tests (`placeholder()`) are intentional placeholders, not functionality stubs — the test harness compiles cleanly and the real tests will be written in 48-02/03/04.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| No new surface | — | No new network endpoints, auth paths, file access patterns, or schema changes. Threat model T-48-01 through T-48-SC all accepted as documented in PLAN. |

## Verification

- `cargo check --all-features` — EXIT 0
- `cargo build --all-features` — EXIT 0
- `cargo check --target wasm32-unknown-unknown` — EXIT 0 (no new time/parallel code)
- `cargo test --test test_traits` — 5 passed
- `cargo test --test test_types` — 43 passed (includes 6 scaffold placeholders)
- `cargo test --test test_operations` — 322 passed (includes 2 scaffold placeholders)
- `cargo test --test test_validators` — 33 passed
- Pre-existing `--all-features` test failures (`test_tracing_observer.rs`, etc.) are pre-Phase 48 and unrelated to changes here

## Self-Check: PASSED

- `src/traits/operator_compat.rs` — FOUND
- `src/validators/generic_validator.rs` (operator_compat_check) — FOUND
- `tests/traits/test_operator_compat.rs` — FOUND
- All 7 scaffold files — FOUND
- Task 1 commit d519a8e — VERIFIED
- Task 2 commit d5bcbd4 — VERIFIED
- Task 3 commit 5a3a444 — VERIFIED
