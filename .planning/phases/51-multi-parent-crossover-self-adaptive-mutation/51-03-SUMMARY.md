---
phase: 51-multi-parent-crossover-self-adaptive-mutation
plan: "03"
subsystem: mutation
tags:
  - self-adaptive
  - evolution-strategy
  - gaussian
  - mutation
  - any-downcast

# Dependency graph
requires:
  - phase: 51-01
    provides: SelfAdaptive trait with adapt_strategy_params default, Mutation::SelfAdaptiveGaussian enum variant, MutationConfiguration self_adaptive_tau/tau_prime/sigma_min fields, RangeChromosome strategy_params field + SelfAdaptive impl, Wave 0 test stubs
provides:
  - self_adaptive_gaussian_mutation function (src/operations/mutation/self_adaptive_gaussian.rs)
  - try_self_adaptive dispatcher (src/operations/mutation.rs) covering f64/f32/i32/i64
  - Mutation::SelfAdaptiveGaussian match arm with tau/tau_prime/sigma_min defaults
affects:
  - src/engines/ga.rs (ga.rs will add SelfAdaptiveGaussian branch in wave 3/4)
  - tests/operations/test_mutation_self_adaptive.rs (GREEN when test_operations compiles after Plan 02 lands)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Any-downcast dispatcher pattern (mirrors try_cauchy/try_polynomial) for type-specialized mutation
    - Concrete struct field access (individual.dna, individual.strategy_params) to avoid associated-type ambiguity when SelfAdaptive where clause is present
    - Box-Muller N(0, sigma) from gaussian.rs — reused convention

key-files:
  created:
    - src/operations/mutation/self_adaptive_gaussian.rs
  modified:
    - src/operations/mutation.rs

key-decisions:
  - "Access RangeChromosome concrete fields (individual.dna, individual.strategy_params) directly in self_adaptive_gaussian_mutation rather than via LinearChromosome trait methods — avoids Rust associated-type ambiguity when RangeChromosome<T>: SelfAdaptive is in the where clause"
  - "tau default = 1/sqrt(2*n_hint), tau_prime default = 1/sqrt(2*sqrt(n_hint)) where n_hint = individual.dna().len().max(1) — ES literature defaults computed from dna length since generic U may not implement SelfAdaptive"
  - "sigma_min hardcoded to 1e-5 in the mutate arm (non-configurable path through factory_with_params); configurable path goes through ga.rs special-case branch reading mutation_configuration.sigma_min"

patterns-established:
  - "try_self_adaptive: Any-downcast dispatcher covering f64/f32/i32/i64 — exact mirror of try_cauchy/try_polynomial/try_levy/try_uniform pattern"
  - "Concrete field access for RangeChromosome in operator files that carry SelfAdaptive bounds (works around Rust's associated-type resolution limitation)"

requirements-completed:
  - MUT-05
  - TRAITS-02

# Metrics
duration: 17min
completed: "2026-05-23"
---

# Phase 51 Plan 03: SelfAdaptiveGaussian Mutation Operator — Summary

**`self_adaptive_gaussian_mutation` for `RangeChromosome<T>` with `try_self_adaptive` Any-downcast dispatcher, sigma_min floor, and tau/tau_prime ES defaults wired into `Mutation::SelfAdaptiveGaussian` match arm.**

## Performance

- **Duration:** ~17 min
- **Started:** 2026-05-23T14:54:00Z
- **Completed:** 2026-05-23T15:11:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Implemented two-step ES mutation in `self_adaptive_gaussian.rs`: log-normal sigma update via `adapt_strategy_params`, then single-gene Box-Muller perturbation clamped to declared range
- Wired `try_self_adaptive` dispatcher (f64/f32/i32/i64) into `mutation.rs` alongside existing `try_cauchy`/`try_polynomial` dispatchers
- Replaced the Wave 0 placeholder error arm for `Mutation::SelfAdaptiveGaussian` with real dispatch that computes tau/tau_prime defaults and falls back to `GaError::MutationError` for non-SelfAdaptive chromosomes
- All gates pass: `cargo build`, `cargo build --features serde`, `cargo clippy -- -D warnings`, `cargo check --target wasm32-unknown-unknown`
- `cargo test --test test_traits` passes (9 tests, including 4 SelfAdaptive trait tests from Plan 01)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement self_adaptive_gaussian_mutation operator function** - `9d16013` (feat)
2. **Task 2: Wire SelfAdaptiveGaussian into mutation.rs via try_self_adaptive dispatcher** - `ea45d0a` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/operations/mutation/self_adaptive_gaussian.rs` — Self-adaptive Gaussian mutation operator: two-step ES mutation (sigma update + single-gene Gaussian perturbation), WASM-safe, no rayon/std::time
- `src/operations/mutation.rs` — Added `pub mod self_adaptive_gaussian`, `try_self_adaptive` dispatcher, and `Mutation::SelfAdaptiveGaussian` real dispatch arm with ES defaults

## Decisions Made

1. **Concrete field access over trait methods in operator file:** When `RangeChromosome<T>: SelfAdaptive` is in the where clause, the Rust compiler cannot resolve `<RangeChromosome<T> as ChromosomeT>::Gene` from the abstract `LinearChromosome::dna()` return type. Solution: access `individual.dna` (the concrete `pub Vec<RangeGenotype<T>>` field) and `individual.strategy_params` directly. This matches the internal API usage in `range.rs` itself.

2. **tau/tau_prime defaults in `mutate` arm:** Since `mutate(&mut U, step, sigma)` doesn't receive the full `MutationConfiguration`, tau defaults are computed from `individual.dna().len().max(1)` (accessible via `LinearChromosome` which `U` must implement). The `ga.rs` special-case branch (Wave 3) will read `mutation_configuration.self_adaptive_tau/tau_prime/sigma_min` to allow user overrides.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Rust associated-type ambiguity with SelfAdaptive where clause**
- **Found during:** Task 1 (implementation)
- **Issue:** The plan specified using `individual.dna()[idx].clone()` (trait method call) to get the gene. When `RangeChromosome<T>: SelfAdaptive` is in the where clause, Rust cannot resolve `<RangeChromosome<T> as ChromosomeT>::Gene` to the concrete `RangeGenotype<T>` type, making `.value` and `.ranges` field access fail to compile. The `gaussian.rs` and `cauchy.rs` analogues don't have this issue because they don't carry the `SelfAdaptive` where bound.
- **Fix:** Access the concrete public fields `individual.dna` (`Vec<RangeGenotype<T>>`) and `individual.strategy_params` (`Vec<f64>`) directly instead of going through trait methods. Write back via `individual.dna[idx] = gene` instead of `individual.set_gene(idx, gene)`.
- **Files modified:** `src/operations/mutation/self_adaptive_gaussian.rs`
- **Verification:** `cargo build` exits 0; `cargo check --target wasm32-unknown-unknown` exits 0
- **Committed in:** 9d16013 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in implementation approach)
**Impact on plan:** Fix necessary for compilation. No change to observable behavior — equivalent API access via concrete public fields. No scope creep.

## Issues Encountered

**Wave 0 test compilation blocked by Plan 02 stubs:** The `test_operations` binary cannot compile because `tests/operations/test_crossover_undx.rs`, `test_crossover_spx.rs`, and `test_crossover_pcx.rs` reference `crossover::undx::undx`, `crossover::spx::spx`, `crossover::pcx::pcx` — modules implemented by Plan 02. Since Plans 02 and 03 run in parallel worktrees in Wave 2, neither alone can make the full `test_operations` binary compile. The three `test_mutation_self_adaptive` tests (sigma_min, sigma_spread_evolves, returns_error_for_non_self_adaptive) will be GREEN once Plan 02 lands and both are merged to the feature branch.

## Known Stubs

None — all plan goals are fully implemented. The Wave 0 test compilation dependency on Plan 02 is a structural property of the parallel wave, not a stub.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes. All new surface is in-process operator functions.

## Next Phase Readiness

- `self_adaptive_gaussian_mutation` is production-ready for `RangeChromosome<f64|f32|i32|i64>`
- `Mutation::SelfAdaptiveGaussian` dispatch is live in `factory_with_params` — the Wave 0 test `returns_error_for_non_self_adaptive` verifies this path
- Plan 04 (ga.rs integration) can wire a special-case branch reading `mutation_configuration.self_adaptive_tau/tau_prime/sigma_min` for user-configurable ES parameters

---
*Phase: 51-multi-parent-crossover-self-adaptive-mutation*
*Completed: 2026-05-23*
