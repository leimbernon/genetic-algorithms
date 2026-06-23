---
phase: 82-per-engine-convergence-integration-tests-issue-284
plan: 01
subsystem: testing
tags: [convergence, integration-tests, sphere, de, scatter, cellular, alps, cma, pso]

# Dependency graph
requires:
  - phase: 56-cma-es
    provides: CmaEngine with IPOP restart support
  - phase: 57-pso
    provides: PsoEngine with observer hooks
  - phase: 23-de-engine
    provides: DeEngine with 5 mutation strategies
  - phase: 24-scatter-engine
    provides: ScatterEngine with local search
  - phase: 25-cellular-engine
    provides: CellularEngine with 4 neighborhoods
  - phase: 26-alps-engine
    provides: AlpsEngine with 3 age schemes
provides:
  - "7 convergence regression tests across 6 engine test files"
  - "Sphere function convergence assertion < 1.0 on 5D for all single-objective engines"
  - "CMA IPOP restart convergence test with SpyObserver assertion"
affects: [engine-reliability, regression-prevention]

# Tech tracking
tech-stack:
  added: []
  patterns: [convergence-regression-test, local-search-for-scatter-stability]

key-files:
  created: []
  modified:
    - tests/engines/de/test_de.rs
    - tests/engines/scatter/test_scatter.rs
    - tests/engines/cellular/test_cellular.rs
    - tests/engines/alps/test_alps.rs
    - tests/engines/cma/test_cma.rs
    - tests/engines/pso/test_pso.rs

key-decisions:
  - "Scatter needs local search for reliable convergence on 5D sphere"
  - "CMA IPOP test must omit fitness_target to allow restarts to fire"
  - "Stagnation threshold of 100 ensures restart triggers before convergence"

patterns-established:
  - "Convergence regression test: each engine gets test_<engine>_convergence asserting best_fitness < 1.0"

requirements-completed: [ISSUE-284]

# Metrics
duration: 7min
completed: 2026-06-22
---

# Phase 82 Plan 01: Per-Engine Convergence Integration Tests Summary

**7 convergence regression tests for all single-objective engines (DE, Scatter, Cellular, ALPS, CMA, PSO) asserting sphere minimum < 1.0 on 5D**

## Performance

- **Duration:** 7 min
- **Started:** 2026-06-22T21:19:00Z
- **Completed:** 2026-06-22T21:26:36Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Added 7 convergence regression tests across 6 engine test files (Closes #284)
- All tests use fixed RNG seed 42 for determinism
- CMA IPOP test validates both convergence and restart triggering via SpyObserver
- Scatter test uses local search to ensure reliable convergence

## Task Commits

Each task was committed atomically:

1. **Task 1: DE, Scatter, Cellular, ALPS convergence tests** - `18bd5b3` (test)
2. **Task 2: CMA, PSO convergence tests (including IPOP restart)** - `84f1531` (test)
3. **Fix: Scatter convergence stabilization** - `d898d60` (fix)

## Files Created/Modified
- `tests/engines/de/test_de.rs` - Added `test_de_convergence` (DE/rand/1/binomial, < 1.0 threshold)
- `tests/engines/scatter/test_scatter.rs` - Added `test_scatter_convergence` (with local search for stability)
- `tests/engines/cellular/test_cellular.rs` - Added `test_cellular_convergence` (6x6 grid, Moore neighborhood)
- `tests/engines/alps/test_alps.rs` - Added `test_alps_convergence` (4 layers, Linear age scheme)
- `tests/engines/cma/test_cma.rs` - Added `test_cma_convergence` and `test_cma_ipop_convergence`
- `tests/engines/pso/test_pso.rs` - Added `test_pso_convergence` (30 particles, 5D)

## Decisions Made
- Scatter engine needs local search enabled for reliable convergence on 5D sphere — without it, the reference set sometimes stalls above threshold
- CMA IPOP convergence test omits `fitness_target` to prevent early stopping before restarts can fire
- CMA IPOP uses `stagnation_threshold: 100` (higher than existing tests) to ensure restart triggers during 500-generation run

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Scatter convergence test was flaky at 300 iterations**
- **Found during:** Task 1 verification (`cargo test --features serde`)
- **Issue:** Scatter engine sometimes converged to best_fitness ~1.05 within 300 iterations, just above threshold
- **Fix:** Added local search (10 steps, 0.5 step size) and increased reference set from 6 to 10
- **Files modified:** tests/engines/scatter/test_scatter.rs
- **Verification:** 10 consecutive test runs with 0 failures
- **Committed in:** d898d60

**2. [Rule 1 - Bug] CMA IPOP test failed — fitness_target caused early stopping**
- **Found during:** Task 2 verification
- **Issue:** `with_fitness_target(1.0)` caused CMA to stop before stagnation threshold was reached, preventing restarts from firing
- **Fix:** Removed `with_fitness_target(1.0)` from IPOP test config, increased stagnation_threshold to 100
- **Files modified:** tests/engines/cma/test_cma.rs
- **Verification:** test_cma_ipop_convergence passes consistently
- **Committed in:** 84f1531

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both deviations necessary for test reliability. No scope creep — all 7 planned convergence tests delivered.

## Issues Encountered
- Pre-existing failure in `test_multi_parent_integration::end_to_end_self_adaptive_gaussian_sigmas_evolve` (not related to this phase)

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 7 convergence tests pass with `cargo test` and `cargo test --features serde`
- No existing tests broken
- Phase 82 complete, ready for next phase

---
*Phase: 82-per-engine-convergence-integration-tests-issue-284*
*Completed: 2026-06-22*

## Self-Check: PASSED

- [x] tests/engines/de/test_de.rs contains `fn test_de_convergence`
- [x] tests/engines/scatter/test_scatter.rs contains `fn test_scatter_convergence`
- [x] tests/engines/cellular/test_cellular.rs contains `fn test_cellular_convergence`
- [x] tests/engines/alps/test_alps.rs contains `fn test_alps_convergence`
- [x] tests/engines/cma/test_cma.rs contains `fn test_cma_convergence`
- [x] tests/engines/cma/test_cma.rs contains `fn test_cma_ipop_convergence`
- [x] tests/engines/pso/test_pso.rs contains `fn test_pso_convergence`
- [x] All 7 tests pass with `cargo test`
- [x] All 7 tests pass with `cargo test --features serde`
- [x] Commits: 18bd5b3, 84f1531, d898d60
