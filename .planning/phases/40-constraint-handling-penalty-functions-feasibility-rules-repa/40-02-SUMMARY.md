---
phase: 40-constraint-handling-penalty-functions-feasibility-rules-repa
plan: 02
subsystem: constraint-handling, nsga2
tags: [nsga2, constraints, integration-test, test]
requires: [40-01]
provides: [CNS-01, CNS-02]
affects: [tests/test_engines.rs, tests/engines/nsga2/test_nsga2_constraints.rs]
tech-stack:
  added: []
  patterns: [Nsga2Ga constraint integration test pattern]
key-files:
  created:
    - tests/engines/nsga2/test_nsga2_constraints.rs
  modified:
    - tests/test_engines.rs
decisions:
  - "Nsga2Configuration holds NSGA2-specific config (num_objectives, population_size, max_generations) while GaConfiguration (via ConfigurationT) holds generic GA operator config (selection, crossover, mutation, genes_per_chromosome)"
  - "ParetoFront::individuals is the correct accessor for run results, not Population fields"
metrics:
  duration: ~15 minutes
  completed: 2026-05-11T20:13:01Z
---

# Phase 40 Plan 02: NSGA-II Constraint Integration Tests

Add NSGA-II constraint integration test module. One new test file exercising `Nsga2Ga::with_constraint_fns()` with a simple constraint, verifying a full NSGA-II run completes with constraints configured.

## Completed Tasks

| #  | Task                       | Commit   | Files                                                                  |
|----|----------------------------|----------|------------------------------------------------------------------------|
| 1  | Register test module       | ed4c95d  | tests/test_engines.rs                                                  |
| 2  | Create constraint test file | d3a6846  | tests/engines/nsga2/test_nsga2_constraints.rs                          |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] API mismatch in plan's sample test code**
- **Found during:** Task 2
- **Issue:** The plan's sample test code chained `with_genes_per_chromosome()`, `with_max_generations()`, `with_selection_method()`, `with_crossover_method()`, `with_mutation_method()` directly on the `Nsga2Ga` builder, but these methods exist on `GaConfiguration` (via `ConfigurationT`), not on `Nsga2Ga`. Additionally, `Ns2Ga::run()` returns `Result<ParetoFront<U>, GaError>`, so assertions needed to use `front.individuals` instead of `pop.chromosomes`.
- **Fix:** Moved GA operator config to `GaConfiguration` builder chain, kept NSGA2-specific config on `Nsga2Configuration`, and adjusted assertions to match `ParetoFront` API.
- **Files modified:** tests/engines/nsga2/test_nsga2_constraints.rs
- **Commit:** d3a6846

**2. [Rule 2 - Misleading] Dereference syntax for RangeGene::value()**
- **Found during:** Task 2 compilation (3 errors)
- **Issue:** The plan's sample code used `*g.value()` to dereference, but `RangeGene::value()` returns `T` by value (`i32`), not `&T`. The dereference caused `type i32 cannot be dereferenced` errors.
- **Fix:** Changed `*g.value()` to `g.value()` and `*dna[0].value()` to `dna[0].value()`.
- **Files modified:** tests/engines/nsga2/test_nsga2_constraints.rs
- **Commit:** d3a6846

**3. [Rule 4 - Scope] Unused import: StoppingConfig**
- **Found during:** Task 2 compilation
- **Issue:** `StoppingConfig` was imported but `with_max_generations` is handled directly on `Nsga2Configuration`, not via the trait, so `StoppingConfig` was unused.
- **Fix:** Removed `StoppingConfig` from imports.
- **Files modified:** tests/engines/nsga2/test_nsga2_constraints.rs
- **Commit:** d3a6846

## Known Stubs

None. The test file is self-contained and fully functional.

## Threat Flags

No new threat surface introduced. Test code only, no new network endpoints, auth paths, or schema changes.

## Verification

- `cargo test --test test_engines nsga2::test_nsga2_constraints` - PASSED (1 passed, 277 filtered)
- `cargo test --test test_constraints` - PASSED (all 8 tests)
- `cargo clippy --tests` - zero new warnings in test files
- `cargo check` - library code unchanged, compiles cleanly

## Self-Check: PASSED

All created/modified files exist and compile. All commits are present in git log.
