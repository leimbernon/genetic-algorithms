---
phase: 74-add-missing-benchmarks
plan: 03
subsystem: benches
tags: [divan, aos, surrogate, batch-fitness, benchmark]

# Dependency graph
requires:
  - phase: 74-add-missing-benchmarks
    provides: "Engine benchmark infrastructure (divan, Cargo.toml [[bench]] entries)"
provides:
  - "AOS on/off divan benchmark (benches/aos.rs)"
  - "Surrogate on/off divan benchmark (benches/surrogate.rs)"
  - "Batch fitness vs per-call divan benchmark (benches/batch_fitness.rs)"
  - "Three [[bench]] entries in Cargo.toml"
affects: [74-add-missing-benchmarks]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Feature on/off two-variant bench pattern (mirrors metrics_observer.rs)"]

key-files:
  created:
    - benches/aos.rs
    - benches/surrogate.rs
    - benches/batch_fitness.rs
  modified:
    - Cargo.toml

key-decisions:
  - "Used MultiPoint instead of TwoPoint (TwoPoint variant does not exist in Crossover enum)"
  - "Explicit type annotation Ga::<RangeChromosome<f64>>::new() required for type inference with closures"
  - "AosConfig trait does not exist; AOS methods are on ConfigurationT (RESEARCH.md hypothesis incorrect)"

patterns-established:
  - "Feature on/off bench: two #[divan::bench] fns in a mod, one with feature, one without"
  - "Batch evaluator: implement BatchFitnessEvaluator with positional contract (evaluate_batch returns Vec of same length)"
  - "Surrogate: implement SurrogateModel with negated l1-norm for minimization prescreening"

requirements-completed: []

# Metrics
duration: 2min
completed: 2026-06-19
---

# Phase 74 Plan 03: Add Missing Feature Benchmarks Summary

**AOS, surrogate, and batch fitness on/off divan benchmarks closing feature-coverage gap for three GA engine features**

## Performance

- **Duration:** 2 min
- **Started:** 2026-06-19T06:58:17Z
- **Completed:** 2026-06-19T07:00:14Z
- **Tasks:** 3
- **Files created:** 3
- **Files modified:** 1

## Accomplishes

- AOS benchmark comparing Ga with crossover portfolio + AosStrategy vs plain Ga on Rastrigin 10D
- Surrogate benchmark comparing surrogate-assisted Ga vs plain Ga with LinearSurrogate prescreening
- Batch fitness benchmark comparing BatchFitnessEvaluator vs per-call fitness closure on sphere

## Task Commits

Each task was committed atomically:

1. **Task 1: Create benches/aos.rs** - `99a81c8` (feat)
2. **Task 2: Create benches/surrogate.rs** - `e30d223` (feat)
3. **Task 3: Create benches/batch_fitness.rs** - `a8d40af` (feat)

## Files Created/Modified

- `benches/aos.rs` - AOS on/off divan benchmark (Rastrigin 10D, pop 100, 30 gens)
- `benches/surrogate.rs` - Surrogate on/off divan benchmark (LinearSurrogate, Rastrigin 10D)
- `benches/batch_fitness.rs` - Batch vs per-call fitness divan benchmark (sphere, 10D)
- `Cargo.toml` - Three new `[[bench]]` entries: aos, surrogate, batch_fitness

## Decisions Made

- Used `Crossover::MultiPoint` instead of `TwoPoint` (TwoPoint variant does not exist in the Crossover enum)
- Added explicit `Ga::<RangeChromosome<f64>>::new()` type annotation (required for inference with initialization closures)
- AOS methods (`with_crossover_portfolio`, `with_aos_strategy`, `with_reward_window`) are on `ConfigurationT` trait, not a separate `AosConfig` trait as RESEARCH.md hypothesized

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Crossover::TwoPoint → MultiPoint**
- **Found during:** Task 1 (AOS bench compilation)
- **Issue:** `Crossover::TwoPoint` does not exist; the enum has `MultiPoint` for multi-point crossover
- **Fix:** Changed `Crossover::TwoPoint` to `Crossover::MultiPoint` in the AOS portfolio
- **Files modified:** benches/aos.rs
- **Verification:** `cargo bench --no-run --bench aos` exits 0
- **Committed in:** 99a81c8

**2. [Rule 3 - Blocking] Added explicit type annotation for Ga::new()**
- **Found during:** Tasks 1-3 (all bench compilations)
- **Issue:** `Ga::new()` type parameter `U` cannot be inferred when using closure-based initialization functions
- **Fix:** Changed `Ga::new()` to `Ga::<RangeChromosome<f64>>::new()` in all three bench files
- **Files modified:** benches/aos.rs, benches/surrogate.rs, benches/batch_fitness.rs
- **Verification:** All three `cargo bench --no-run` exit 0
- **Committed in:** 99a81c8, e30d223, a8d40af

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both auto-fixes necessary for compilation. No scope creep.

## Issues Encountered

None — all tasks compiled and verified on first successful attempt after fixes.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- All three feature benchmarks compile and are ready for measurement
- Phase 74 feature benchmark coverage is complete (AOS, surrogate, batch fitness)
- Phase ready for verification via `/gsd-verify-work 74`

---
*Phase: 74-add-missing-benchmarks*
*Completed: 2026-06-19*
