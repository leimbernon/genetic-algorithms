---
phase: 33-scalar-mutation-operators
plan: "02"
subsystem: mutation-operators
tags: [mutation, levy-flight, range, operator, mantegna, gamma]
dependency_graph:
  requires: [33-01]
  provides: [Mutation::LevyFlight, mantegna_sigma_u, gamma_approx, levy_flight_mutation, six levy tests active]
  affects: [src/operations/mutation, tests/operations/test_mutation_cauchy_levy_uniform]
tech_stack:
  added: []
  patterns: [Mantegna algorithm, Box-Muller normal sampling, alpha.clamp stability guard, range-scaled step, try_levy downcast helper]
key_files:
  created:
    - src/operations/mutation/levy_flight.rs
  modified:
    - src/operations/mutation.rs
    - tests/operations/test_mutation_cauchy_levy_uniform.rs
decisions:
  - "Mantegna step scaled by (hi - lo) so stability index alpha governs heavy-tail behavior independently of gene scale (Pitfall 3)"
  - "alpha.clamp(0.1, 1.99) applied inside levy_flight_mutation to bound Gamma function inputs to a stable region (T-33-07 mitigation)"
  - "gamma_approx uses Abramowitz & Stegun 6.1.36 polynomial on [1,2] with recursion for out-of-range inputs — avoids external crate dependency"
  - "sigma parameter slot carries levy_alpha value per Plan 01 engine routing convention (sigma = None defaults to alpha = 1.5)"
metrics:
  duration_minutes: 15
  completed_date: "2026-05-07"
  tasks_completed: 2
  files_changed: 3
---

# Phase 33 Plan 02: LevyFlight Mutation Operator Summary

Implements `Mutation::LevyFlight` (MUT-02) using Mantegna's algorithm with a recursive Gamma approximation and replaces the Plan 01 placeholder `unimplemented!()` arms with a real implementation. Activates five previously ignored LevyFlight behavioral tests and adds a sixth defaults test.

## What Was Built

**Mutation::LevyFlight** — Mantegna's Lévy step algorithm for `Range<T>` chromosomes:
- `step = σ_u * u / |v|^(1/α)` where `u ~ N(0, σ_u²)`, `v ~ N(0, 1)` (Mantegna's formula)
- σ_u computed via `mantegna_sigma_u(alpha)` using the exact Γ-based formula from Yang 2010
- `gamma_approx` — recursive Gamma approximation using Abramowitz & Stegun 6.1.36 polynomial on `[1, 2]`
- Step scaled by `(hi - lo)` so stability index α governs heavy-tail behavior independently of gene scale
- `alpha.clamp(0.1, 1.99)` prevents Gamma function instability at the extremes (T-33-07 mitigation)
- Result clamped to gene's declared `[lo, hi]` range
- Supports `f64`, `f32`, `i32`, `i64` via `GaussianConvertible` trait
- Returns `GaError::MutationError` for non-Range (Binary) chromosomes

**Dispatch wiring in `mutation.rs`:**
- `pub mod levy_flight;` module declaration added
- `try_levy` helper function following the `try_cauchy` pattern
- Placeholder `unimplemented!()` arms in `MutationOperator::mutate` and `factory_non_value` replaced with real dispatch

**Tests — 6 active LevyFlight tests:**
1. `levy_flight_mutation_via_factory_changes_value` — non-no-op across 200 iterations
2. `levy_flight_mutation_via_factory_stays_in_range` — clamp invariant on `[0.0, 100.0]`
3. `levy_flight_mutation_changes_at_most_one_gene` — single-gene-per-call invariant (D-02)
4. `levy_flight_mutation_works_on_i32` — correctness for `RangeChromosome<i32>`
5. `levy_flight_mutation_errors_on_binary_chromosome` — error path for non-Range types
6. `levy_flight_default_alpha_when_sigma_none` — sigma=None defaults to α=1.5

Only the 5 Uniform tests remain `#[ignore]` for Plan 03.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

- `Mutation::Uniform` — `unimplemented!()` arm in `MutationOperator::mutate` and `factory_non_value`. Plan 03 replaces with real implementation.

## Pre-existing Issues (Out of Scope)

- Rustdoc warning: `unresolved link to SelectionConfiguration::niche_radius` — pre-existing, not introduced by this plan. Documented in Plan 01 SUMMARY as pre-existing.

## Self-Check: PASSED

Files exist:
- `src/operations/mutation/levy_flight.rs` — FOUND
- `tests/operations/test_mutation_cauchy_levy_uniform.rs` — FOUND (modified)

Commits exist:
- `bfc1c21` (feat: Task 1) — FOUND
- `d646b8c` (test: Task 2) — FOUND

Build and test:
- `cargo build` — PASSED
- `cargo build --features serde` — PASSED
- `cargo test` — 752 passed (+ 28 ignored)
- `cargo test --features serde` — 782 passed (+ 28 ignored)
- `cargo clippy --all-targets -- -D warnings` — CLEAN
- `cargo doc --no-deps` — 1 pre-existing warning (niche_radius link, out of scope)
