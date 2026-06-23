---
phase: 71-per-operator-mutation-params
plan: "01"
subsystem: operations
tags: [rust, enum-refactoring, mutation, parameter-structs, breaking-change]

# Dependency graph
requires:
  - phase: 70-replace-operator-downcasting
    provides: RealValuedMutation trait dispatch replacing downcasting (prerequisite cleanup)
provides:
  - "8 named parameter structs (CreepParams, GaussianParams, PolynomialParams, NonUniformParams, DifferentialParams, CauchyParams, LevyFlightParams, SelfAdaptiveGaussianParams) in src/operations.rs"
  - "Mutation enum variants reshaped from inline-struct to tuple form (e.g. Mutation::Gaussian(GaussianParams))"
  - "factory_with_params removed from src/operations/mutation.rs"
  - "factory_with_chromosome_length simplified to 3-argument signature"
  - "Engine call sites updated: generation.rs (2x Differential arm, 2x factory_with_chromosome_length), island/mod.rs (2x), alps/configuration.rs, cellular/configuration.rs"
affects:
  - 71-02-PLAN
  - 71-03-PLAN

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Named parameter structs for enum variants: each parameterized Mutation variant now carries a dedicated Params struct (e.g. GaussianParams { pub sigma: Option<f64> }) with Default derive and serde cfg_attr annotations"
    - "Tuple variant dispatch: match arms destructure via (ParamStruct { field }) syntax rather than inline struct fields"

key-files:
  created: []
  modified:
    - src/operations.rs
    - src/operations/mutation.rs
    - src/engines/ga/generation.rs
    - src/engines/alps/configuration.rs
    - src/engines/cellular/configuration.rs
    - src/engines/island/mod.rs

key-decisions:
  - "D-01: Param structs use Option<f64> fields — None means use operator's documented default; defaults remain at dispatch"
  - "D-02: All param structs live in src/operations.rs alongside the Mutation enum"
  - "D-03: 8 parameterized variants become tuple variants; 10 unit variants untouched"
  - "D-05: factory_with_params deleted entirely (v3.0.0 breaking change)"
  - "D-06: factory_with_chromosome_length simplified to 3 args; 4 engine call sites updated"
  - "Added Default derive to all 8 param structs for ergonomic GaussianParams::default() shorthand"

patterns-established:
  - "Param struct pattern: #[derive(Debug, Clone, PartialEq, Default)] + serde cfg_attr on struct + pub Option<f64> fields with serde(default)"
  - "Wildcard tuple arms: Mutation::NonUniform(..) and Mutation::Differential(..) in match arms"

requirements-completed: []

# Metrics
duration: 50min
completed: 2026-06-18
status: complete
---

# Phase 71 Plan 01: Per-Operator Mutation Params Summary

**8 named parameter structs added to `src/operations.rs` and Mutation enum reshaped from inline-struct to tuple variants; factory_with_params deleted; factory_with_chromosome_length simplified to 3 args**

## Performance

- **Duration:** 50 min
- **Started:** 2026-06-18T09:44:34Z
- **Completed:** 2026-06-18T10:14:55Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Defined 8 public parameter structs (`CreepParams`, `GaussianParams`, `PolynomialParams`, `NonUniformParams`, `DifferentialParams`, `CauchyParams`, `LevyFlightParams`, `SelfAdaptiveGaussianParams`) in `src/operations.rs` — each with `pub` fields, `Default` derive, and `serde(default)` annotations
- Reshaped all 8 parameterized `Mutation` variants from inline-struct form to tuple form; 10 unit variants untouched
- Updated `MutationOperator::mutate` match arms and all factory functions in `mutation.rs` to destructure tuple variants; removed `factory_with_params` function; simplified `factory_with_chromosome_length` to 3-arg signature
- Updated 4 engine call sites: `generation.rs` (Differential arm + 2x factory_with_chromosome_length), `island/mod.rs` (2x), `alps/configuration.rs`, `cellular/configuration.rs`
- `cargo build` and `cargo test --lib` (56 tests) both pass with zero errors/warnings

## Task Commits

Each task was committed atomically:

1. **Task 1: Define 8 param structs and reshape Mutation enum variants to tuple form** - `706f6af` (refactor)
2. **Task 2: Update mutation.rs dispatch, remove factory_with_params, simplify factory_with_chromosome_length** - `4857f3e` (refactor)

## Files Created/Modified

- `src/operations.rs` — Added 8 param structs before Mutation enum; reshaped 8 variants to tuple form; updated enum docstring example
- `src/operations/mutation.rs` — Updated all match arms to tuple destructuring; deleted factory_with_params; simplified factory_with_chromosome_length; updated factory_self_adaptive and factory_non_value
- `src/engines/ga/generation.rs` — Updated Differential match arm to DifferentialParams; dropped None, None args from 2x factory_with_chromosome_length calls
- `src/engines/alps/configuration.rs` — Updated Gaussian construction default to tuple syntax
- `src/engines/cellular/configuration.rs` — Updated Gaussian construction default to tuple syntax
- `src/engines/island/mod.rs` — Dropped None, None args from 2x factory_with_chromosome_length calls

## Decisions Made

- Added `Default` derive to all 8 param structs (D-01 agent discretion) — enables ergonomic `GaussianParams::default()` shorthand at zero cost
- `DifferentialParams` and `NonUniformParams` imports omitted from `mutation.rs` since their wildcards `(..)` don't require destructuring — compiler confirmed clean
- `moead/mod.rs`, `nsga2/mod.rs`, `nsga3/mod.rs` still use `Mutation::Differential { .. }` in `matches!` macro — in Rust 1.94 this compiles without error or warning; these are updated in Plans 02/03 as part of the tests/integration pass

## Deviations from Plan

None - plan executed exactly as written. The engine files `moead`, `nsga2`, `nsga3` that use `{ .. }` in `matches!` are out of scope for Plan 01 per the plan's explicit note that "external tests/examples are updated in plans 02/03".

## Issues Encountered

None — `cargo build` and `cargo test --lib` passed cleanly after Task 2. No unexpected behavior changes.

## Next Phase Readiness

- Plan 01 complete. Plans 02 and 03 can now proceed to update all remaining test/example files and integration call sites
- Key remaining work: `tests/` construction syntax migration, `examples/` migration, `moead`/`nsga2`/`nsga3` wildcard guard updates
- All defaults preserved: Creep 0.01, Gaussian 0.1, Polynomial 20.0, Cauchy 1.0, LevyFlight 1.5, SelfAdaptive tau/tau_prime/sigma_min formulas

## Self-Check: PASSED

- `src/operations.rs` — FOUND, 8 param structs present, 8 tuple variants, no inline-struct syntax
- `src/operations/mutation.rs` — FOUND, factory_with_params gone, 3-arg factory_with_chromosome_length
- `src/engines/ga/generation.rs` — FOUND, Differential arm + factory calls updated
- `src/engines/alps/configuration.rs` — FOUND, Gaussian default updated
- `src/engines/cellular/configuration.rs` — FOUND, Gaussian default updated
- `src/engines/island/mod.rs` — FOUND, factory calls updated
- Task commits `706f6af` and `4857f3e` — VERIFIED in git log
- `cargo build` — CLEAN (0 errors, 0 warnings)
- `cargo test --lib` — 56/56 passed

---
*Phase: 71-per-operator-mutation-params*
*Completed: 2026-06-18*
