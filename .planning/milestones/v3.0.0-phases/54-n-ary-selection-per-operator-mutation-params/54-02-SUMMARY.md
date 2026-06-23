---
phase: 54-n-ary-selection-per-operator-mutation-params
plan: 02
subsystem: mutation-operators
tags: [mutation, enum-params, trait-dispatch, config-slim, breaking-change]
dependency_graph:
  requires: [54-01]
  provides: [parameterized-mutation-enum, slim-mutation-config, trait-dispatch]
  affects: [ga, island, nsga2, nsga3, moead, spea2, ibea, sms-emoa, alps, cellular]
tech_stack:
  added: []
  patterns: [enum-struct-variants, trait-dispatch, option-default-params]
key_files:
  created: []
  modified:
    - src/operations.rs
    - src/traits/operators.rs
    - src/operations/mutation.rs
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/engines/ga.rs
    - src/engines/alps/configuration.rs
    - src/engines/alps/engine.rs
    - src/engines/cellular/configuration.rs
    - src/engines/cellular/engine.rs
    - src/engines/nsga2/mod.rs
    - src/engines/nsga3/mod.rs
    - src/engines/island/mod.rs
    - src/engines/island/nsga2.rs
    - src/engines/ibea/mod.rs
    - src/engines/moead/mod.rs
    - src/engines/sms_emoa/mod.rs
    - src/engines/spea2/mod.rs
    - tests/operations/test_mutation_creep_gaussian.rs
    - tests/operations/test_mutation_cauchy_levy_uniform.rs
    - tests/operations/test_mutation_self_adaptive.rs
    - tests/operations/test_mutation.rs
    - tests/observe/test_serde.rs
    - tests/test_multi_parent_integration.rs
    - tests/engines/alps/test_alps.rs
    - tests/engines/cellular/test_cellular.rs
    - tests/engines/local_search.rs
    - tests/engines/moead/test_moead.rs
    - tests/engines/test_strategy_trait.rs
    - tests/gp.rs
    - tests/traits/test_self_adaptive.rs
    - tests/types/chromosomes/test_multi_range.rs
    - tests/types/chromosomes/test_multi_unique.rs
    - tests/types/chromosomes/test_unique.rs
    - examples/constrained_g1.rs
    - examples/island_model.rs
    - examples/memetic_rastrigin.rs
    - examples/niching.rs
    - examples/rastrigin.rs
decisions:
  - "NonUniform returns GaError from trait path instead of special-cased in GA loop (non_uniform_mutation takes concrete type, not generic U)"
  - "factory_with_params retained (still referenced by chromosome-length path); not deprecated in this phase"
  - "ALPS/Cellular deprecated fields mutation_step/mutation_sigma kept for backwards compat; engine code uses trait call"
  - "Differential default f=0.5, Cauchy default scale=1.0, LevyFlight default alpha=1.5, Gaussian default sigma=0.1 (breaking change from 1.0), Creep default step=0.01 (breaking change from 1.0)"
metrics:
  duration: "2 sessions"
  completed: "2026-05-29"
  tasks: 2
  files: 39
---

# Phase 54 Plan 02: Parameterize Mutation Enum + Collapse GA Dispatch Summary

**One-liner:** Mutation enum struct variants with inline `Option<f64>` params replacing global config fields; MutationOperator trait takes `&Mutation`; GA dispatch collapsed from ~60-line if/else to a 3-arm match.

## What Was Built

### Task 1: Parameterized Mutation Enum, Slimmed Config, Updated Trait

**Mutation enum changes (`src/operations.rs`):**
- `Copy` derive removed; `Clone` kept
- 8 unit variants converted to struct variants: `Gaussian { sigma: Option<f64> }`, `Creep { step: Option<f64> }`, `Polynomial { eta: Option<f64> }`, `NonUniform { b: Option<f64> }`, `Differential { f: Option<f64> }`, `Cauchy { scale: Option<f64> }`, `LevyFlight { alpha: Option<f64> }`, `SelfAdaptiveGaussian { tau, tau_prime, sigma_min, sigma_max: Option<f64> }`
- `#[serde(default)]` on all new fields for backward-compatible deserialization

**MutationOperator trait (`src/traits/operators.rs`):**
- `mutate(&self, &mut U, step, sigma)` → `mutate(&self, &mut U, mutation: &Mutation)`
- Each variant extracts its own param: `sigma.unwrap_or(0.1)` for Gaussian, `step.unwrap_or(0.01)` for Creep, etc.
- `NonUniform` and `Differential` return `GaError::MutationError` from the trait path (they require external context)

**MutationConfiguration (`src/configuration.rs`):**
- Removed 10 operator-specific fields: step, sigma, polynomial_eta, non_uniform_b, differential_f, cauchy_scale, levy_alpha, self_adaptive_tau, self_adaptive_tau_prime, sigma_min, sigma_max
- Retained 6 operator-agnostic fields: probability_max, probability_min, method, dynamic_mutation, target_cardinality, probability_step
- `Copy` removed (method: Mutation is no longer Copy)
- Corresponding 10 builder methods removed from `MutationConfig` trait and `GaConfiguration` impl

**All engine files updated:**
- Added `use crate::traits::MutationOperator;` where needed
- Added `.clone()` at all former implicit-Copy sites
- `nsga2`, `nsga3`, `moead`, `spea2`, `ibea`, `sms_emoa`, `island::mod`, `island::nsga2`: dispatch simplified to `mutation_config.method.mutate(child, &mutation_config.method)?`
- `alps`, `cellular` engines: use `self.config.mutation.mutate(&mut offspring, &self.config.mutation)?`
- `alps/configuration.rs`, `cellular/configuration.rs`: kept deprecated `mutation_step`/`mutation_sigma` fields with `#[deprecated(since = "3.0.0")]` annotations

### Task 2: Collapsed GA Loop Dispatch + Updated Tests

**GA loop dispatch (`src/engines/ga.rs`):**
```rust
match &mutation_method {
    Mutation::Differential { f } => {
        let f_val = f.unwrap_or(0.5);
        differential::differential_mutation(&mut child_1, chromosomes, key, f_val)?;
    }
    Mutation::Insertion | Mutation::Deletion => {
        mutation::factory_with_chromosome_length(mutation_method.clone(), &mut child_1, ...)?;
    }
    _ => {
        mutation_method.mutate(&mut child_1, &mutation_method)?;
    }
}
```

**Test files updated:**
- `test_mutation_creep_gaussian.rs`: all `factory_with_params` calls replaced with `factory(Mutation::Creep{step:Some(x)}, ...)` or `factory(Mutation::Gaussian{sigma:Some(x)}, ...)`; new tests for inline params and default values
- `test_mutation_cauchy_levy_uniform.rs`: struct-variant construction throughout; new `cauchy_inline_scale_applies` test
- `test_mutation_self_adaptive.rs`: struct-variant SelfAdaptiveGaussian; new `inline_params_work` and `default_params_stay_in_range` tests; `#[must_use]` warnings fixed
- `test_serde.rs`: `Mutation::Polynomial { eta: None }` for config round-trip test
- 15 other test/example files: unit-variant `Mutation::Gaussian` → `Mutation::Gaussian { sigma: None }` throughout

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] NonUniform not special-cased in GA loop**
- **Found during:** Task 2
- **Issue:** Plan said to special-case `NonUniform` in the GA match block and call `non_uniform_mutation()` helper. However, `non_uniform_mutation()` takes a concrete `&mut RangeChromosome<T>` not generic `&mut U`. Cannot call it in the generic dispatch.
- **Fix:** NonUniform falls through to the `_ =>` arm which calls `mutate()` on the trait impl. The trait impl returns `GaError::MutationError("NonUniform requires generation context...")`. This preserves the error behavior — NonUniform was non-functional via the old `factory_with_params` too (no step/sigma forwarding). Functionally equivalent.
- **Files modified:** src/operations/mutation.rs (NonUniform arm in trait impl)
- **Commit:** 6ccb11a

**2. [Rule 2 - Auto-add] Remove unused `mutation` imports in 5 engine files**
- **Found during:** Task 2 (clippy-level warnings)
- **Issue:** After switching to trait dispatch, `use crate::operations::{crossover, mutation}` had an unused `mutation` import in alps/engine.rs, cellular/engine.rs, nsga2, nsga3, island/nsga2
- **Fix:** Changed to `use crate::operations::crossover;`
- **Files modified:** 5 engine files
- **Commit:** a214a73

**3. [Rule 2 - Auto-add] Fix #[must_use] warnings in test_mutation_self_adaptive.rs**
- **Found during:** Task 2
- **Issue:** `self_adaptive_gaussian_mutation()` returns `Result` but two test calls ignored it
- **Fix:** Added `let _ =` prefix
- **Files modified:** tests/operations/test_mutation_self_adaptive.rs
- **Commit:** a214a73

**4. [Rule 1 - Bug] Fix unused ChromosomeT import in test_self_adaptive.rs**
- **Found during:** Task 2
- **Issue:** `use genetic_algorithms::traits::{ChromosomeT, LinearChromosome, SelfAdaptive}` — `ChromosomeT` unused
- **Fix:** Removed from import
- **Files modified:** tests/traits/test_self_adaptive.rs
- **Commit:** a214a73

**5. [Rule 1 - Bug] Fix unused mut variable in gp.rs**
- **Found during:** Task 2
- **Issue:** `let mut rng = SmallRng::seed_from_u64(99)` — not mutated
- **Fix:** Changed to `let rng`
- **Files modified:** tests/gp.rs
- **Commit:** a214a73

## Known Stubs

None — all mutation operators wire their parameters from the enum variant; no placeholder values in rendering paths.

## Threat Flags

No new security-relevant surface introduced. The serde deserialization threat (T-54-03) is mitigated by `#[serde(default)]` on all new `Option<f64>` fields — old checkpoint payloads without these fields deserialize to `None` and use the operator's built-in default.

## Self-Check: PASSED

- src/operations.rs: Mutation enum has `Gaussian { sigma: Option<f64> }`, Copy not in derive — VERIFIED
- src/traits/operators.rs: `mutation: &Mutation` in trait signature — VERIFIED
- src/configuration.rs: no `pub step`, `pub sigma`, `pub polynomial_eta`, etc. in MutationConfiguration — VERIFIED
- src/engines/ga.rs: `mutation_method.mutate(&mut child_1, &mutation_method)` present — VERIFIED
- Commits 6ccb11a and a214a73 exist in git log — VERIFIED
- cargo check: 0 errors — VERIFIED (cargo check --tests exits 0)
- cargo test --test test_operations: 365 passed — VERIFIED
- cargo test --test test_multi_parent_integration: 4 passed — VERIFIED
