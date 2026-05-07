---
phase: 33-scalar-mutation-operators
plan: "01"
subsystem: mutation-operators
tags: [mutation, cauchy, range, operator, config]
dependency_graph:
  requires: []
  provides: [Mutation::Cauchy, cauchy_scale config, levy_alpha config, six-engine dispatch routing]
  affects: [src/operations, src/configuration, src/traits/configuration, src/engines/ga, src/engines/nsga2, src/engines/cellular, src/engines/island, src/engines/alps]
tech_stack:
  added: []
  patterns: [enum+factory dispatch, try_type! downcast macro, Option<f64> config field + builder method]
key_files:
  created:
    - src/operations/mutation/cauchy.rs
    - tests/operations/test_mutation_cauchy_levy_uniform.rs
  modified:
    - src/operations.rs
    - src/operations/mutation.rs
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/engines/ga.rs
    - src/engines/nsga2/mod.rs
    - src/engines/cellular/engine.rs
    - src/engines/island/mod.rs
    - src/engines/island/nsga2.rs
    - src/engines/alps/engine.rs
    - tests/test_operations.rs
    - tests/observe/test_serde.rs
decisions:
  - "Cauchy inverse-CDF uses EPSILON-bounded range to prevent tan(+-pi/2) = +-inf (T-33-01 mitigation)"
  - "LevyFlight and Uniform added as placeholder variants with unimplemented!() arms to keep workspace compiling after Task 1 engine dispatch is wired"
  - "Cellular and Alps engines use mutation_step/mutation_sigma as Cauchy scale/LevyFlight alpha passthrough (flat config structs, no cauchy_scale/levy_alpha fields)"
  - "Serde test_serde.rs required struct literal update for new MutationConfiguration fields plus new variants in serde round-trip array"
metrics:
  duration_minutes: 30
  completed_date: "2026-05-07"
  tasks_completed: 3
  files_changed: 12
---

# Phase 33 Plan 01: Cauchy Mutation Operator Summary

Adds the `Mutation::Cauchy` operator (MUT-01) with heavy-tailed real-valued perturbation using the inverse-CDF Cauchy distribution, plus the parameter-routing infrastructure (`cauchy_scale`/`levy_alpha` config fields, builder methods, six-engine dispatch) that Plans 02 and 03 will reuse for LevyFlight and Uniform operators.

## What Was Built

**Mutation::Cauchy** — inverse-CDF Cauchy perturbation for `Range<T>` chromosomes:
- `noise = cauchy_scale * tan(pi * (u - 0.5))` where `u ~ Uniform(EPSILON, 1-EPSILON)`
- EPSILON-bounded range prevents `tan(+-pi/2) = +-inf` (T-33-01 threat mitigation)
- Result clamped to gene's declared `[lo, hi]` range
- Supports `f64`, `f32`, `i32`, `i64` via `GaussianConvertible` trait
- Returns `GaError::MutationError` for non-Range (Binary, List) chromosomes

**Config infrastructure:**
- `cauchy_scale: Option<f64>` and `levy_alpha: Option<f64>` on `MutationConfiguration`
- `with_cauchy_scale(f64)` and `with_levy_alpha(f64)` builder methods on `MutationConfig` trait, `GaConfiguration`, and `Ga<U>`

**Placeholder variants** — `Mutation::LevyFlight` and `Mutation::Uniform` added with `unimplemented!()` arms to keep the workspace compiling with the six-engine dispatch already wired (Plans 02-03 replace these).

**Six-engine dispatch** — All engines route `cauchy_scale` (as `step`) and `levy_alpha` (as `sigma`) to `factory_with_params` for the respective variants.

**Tests** — 8 active Cauchy/builder tests + 10 ignored scaffolded tests (5 LevyFlight + 5 Uniform) for Plans 02-03 to activate.

## Deviations from Plan

**[Rule 1 - Bug] Fixed serde test struct literal missing new fields**
- **Found during:** Task 3 (`cargo test --features serde`)
- **Issue:** `tests/observe/test_serde.rs` had a `MutationConfiguration { ... }` struct literal that became non-exhaustive after adding `cauchy_scale` and `levy_alpha` fields
- **Fix:** Added `cauchy_scale: None, levy_alpha: None` to the literal; also added `Mutation::Cauchy`, `Mutation::LevyFlight`, `Mutation::Uniform` to the serde round-trip variants array (consistent with D-11 from context)
- **Files modified:** `tests/observe/test_serde.rs`
- **Commit:** 969d8c6

## Known Stubs

- `Mutation::LevyFlight` — `unimplemented!()` arm in `MutationOperator::mutate` and `factory_non_value`. Plan 02 replaces with real implementation.
- `Mutation::Uniform` — `unimplemented!()` arm in `MutationOperator::mutate` and `factory_non_value`. Plan 03 replaces with real implementation.

These stubs are intentional per plan design (Plan 01 lays the routing infrastructure; Plans 02-03 add the implementations).

## Pre-existing Issues (Out of Scope)

- `test_reporter_on_new_best_fires` — intermittently fails under `--features serde` due to random timing; passes when run in isolation. Pre-existing flaky test, not caused by this plan's changes.
- Rustdoc warning: `unresolved link to SelectionConfiguration::niche_radius` — pre-existing, not introduced by this plan.

## Self-Check: PASSED

Files exist:
- `src/operations/mutation/cauchy.rs` — FOUND
- `tests/operations/test_mutation_cauchy_levy_uniform.rs` — FOUND

Commits exist:
- `efc5e52` (feat: tasks 1+2) — FOUND
- `969d8c6` (test: task 3) — FOUND

Build and test:
- `cargo build` — PASSED
- `cargo build --features serde` — PASSED
- `cargo test` — 744 passed (+ 33 ignored)
- `cargo test --features serde` — 774 passed (+ 33 ignored)
- `cargo clippy --all-targets -- -D warnings` — CLEAN
