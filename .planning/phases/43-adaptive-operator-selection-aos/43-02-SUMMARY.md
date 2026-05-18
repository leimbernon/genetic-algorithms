---
phase: 43-adaptive-operator-selection-aos
plan: 02
subsystem: core
tags: [aos, adaptive-operator-selection, ga-engine-integration, reward-accumulation]

# Dependency graph
requires:
  - phase: 43-01
    provides: AOS core state machine (AosState, AosStrategy), AOS configuration fields, builder methods, build validation
provides:
  - AOS runtime state initialization in GA loop when portfolios configured
  - AOS-aware offspring generation with per-couple operator selection
  - Reward accumulation via Mutex<Vec<(usize, f64)>> inside parallel offspring loop
  - Batch AOS state update after parallel loop completes
  - 5 GA integration tests covering PM, AP, MAB strategies and AOS+AGA coexistence
affects:
  - 43-03 (AOS lifecycle/advance_generation, if applicable)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AOS state as Option<Mutex<AosState>> on Ga struct for safe shared access across rayon threads"
    - "AOS operator selection inside process_pair closure with Mutex lock per couple"

key-files:
  modified:
    - src/engines/ga.rs
    - tests/engines/aos/test_aos.rs

key-decisions:
  - "AOS state stored as Option<Mutex<AosState>> on Ga struct — Mutex ensures safe shared access across rayon threads"
  - "parent_crossover receives Option<&Mutex<AosState>> references, not owned values"
  - "Two separate reward accumulators (crossover, mutation) each as Arc<Mutex<Vec<(usize, f64)>>>"
  - "Reward normalization uses is_maximization to swap parent/child ordering for correct sign"
  - "Crossover portfolio operators limited to RangeGene-compatible types (Uniform, SinglePoint, Clone)"
  - "Cycle crossover excluded from test portfolios (requires permutation chromosomes with unique gene IDs)"

requirements-completed:
  - AOS-01

# Metrics
duration: 19min
completed: 2026-05-13
---

# Phase 43 Plan 02: AOS GA Engine Integration

**Wired AOS runtime state into the GA generation loop: Mutex-wrapped AosState on Ga struct, per-couple operator dispatch via parent_crossover, Mutex-guarded reward accumulation, and batch AosState update after parallel loop**

## Performance

- **Duration:** 19 min
- **Started:** 2026-05-13T21:21:28Z
- **Completed:** 2026-05-13T21:40:16Z
- **Tasks:** 3
- **Files created/modified:** 2

## Accomplishments

- Added `aos_crossover: Option<Mutex<AosState>>` and `aos_mutation: Option<Mutex<AosState>>` fields on `Ga<U>` struct with Default impl
- Initialized AOS runtime state in `run_with_callback()` when `crossover_portfolio` or `mutation_portfolio` is configured
- Modified `parent_crossover` signature to accept AOS parameters: portfolios, AosState references, generation, best_fitness, is_maximization
- Added per-couple AOS operator selection inside the `process_pair` closure (Mutex lock per couple)
- Replaced single-operator crossover/mutation dispatch with AOS-aware dispatch
- Accumulated rewards via `Arc<Mutex<Vec<(usize, f64)>>>` shared across rayon threads
- Drained reward accumulators and called `AosState::record_rewards()` + `update()` after parallel loop
- Created 5 GA integration tests: crossover portfolio, both portfolios, MAB strategy, Adaptive Pursuit, AOS+Adaptive GA coexistence

## Task Commits

Each task was committed atomically:

1. **Task 1: Add AOS runtime fields to Ga struct, initialize AosState at run start** - `effd0d5` (feat)
2. **Task 2: Wire AOS dispatch into parent_crossover with reward accumulation** - `6f530af` (feat)
3. **Task 3: GA integration tests for AOS** - `1487cd9` (test)

## Files Created/Modified

- `src/engines/ga.rs` - **Modified**: Added AOS fields to Ga struct, AOS init in run_with_callback, AOS dispatch in parent_crossover, reward accumulation, updated call site (132 lines added)
- `tests/engines/aos/test_aos.rs` - **Modified**: Added 5 GA integration tests with GA builder + run patterns (188 lines added)

## Decisions Made

- AOS state stored as `Option<Mutex<AosState>>` (not `Option<AosState>`) for safe shared access across rayon threads without per-couple cloning
- Two separate reward accumulators (`crossover_reward_acc`, `mutation_reward_acc`) to keep crossover and mutation rewards segregated for their respective AosState instances
- For maximization: swapped parent/child arguments to `compute_normalized_reward` so better offspring always produces positive reward
- Used `Clone` crossover (works with any chromosome type) as third test portfolio operator instead of `Cycle` (requires permutation chromosomes)
- Using `n as usize` for `genes_per_chromosome` instead of `try_into().unwrap()` for simplicity

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Import/Type] Added missing Crossover and Mutation type imports in ga.rs**
- **Found during:** Task 2 (verification)
- **Issue:** `Mutation` type not in scope inside `parent_crossover` — `use crate::operations::{}` didn't include `Crossover` or `Mutation`
- **Fix:** Added `Crossover` and `Mutation` to the existing `crate::operations::{}` import
- **Files modified:** src/engines/ga.rs
- **Verification:** cargo check passes
- **Committed in:** 6f530af (Task 2 commit)

**2. [Rule 3 - Type inference] Fixed type annotation for Ga<RangeChromosome<i32>> in inline tests**
- **Found during:** Task 3 (verification)
- **Issue:** Compiler couldn't infer `U = RangeChromosome<i32>` from builder chain alone
- **Fix:** Added explicit `let mut ga: Ga<RangeChromosome<i32>>` type annotation to 3 inline tests
- **Files modified:** tests/engines/aos/test_aos.rs
- **Verification:** All tests compile and pass
- **Committed in:** 1487cd9 (Task 3 commit)

**3. [Rule 3 - Crossover type] Replaced Crossover::Cycle with Crossover::Clone in test portfolios**
- **Found during:** Task 3 (runtime test failure)
- **Issue:** `Cycle` crossover failed at runtime on `RangeGene<i32>` (requires permutation chromosomes with unique gene IDs)
- **Fix:** Replaced with `Crossover::Clone` which works with any chromosome type without configuration
- **Files modified:** tests/engines/aos/test_aos.rs
- **Verification:** All tests pass (327 total, 18 AOS tests)
- **Committed in:** 1487cd9 (Task 3 commit)

**4. [Rule 3 - Import] Added ConfigurationT and config trait imports to AOS test file**
- **Found during:** Task 3 (compilation)
- **Issue:** `Ga::new()` requires `ConfigurationT` trait in scope; builder methods require `SelectionConfig`, `CrossoverConfig`, `MutationConfig`, `StoppingConfig` in scope
- **Fix:** Added all 5 trait imports to test file
- **Files modified:** tests/engines/aos/test_aos.rs
- **Verification:** cargo test passes
- **Committed in:** 1487cd9 (Task 3 commit)

---

**Total deviations:** 4 auto-fixed (4 rule 3 blocking issues)
**Impact on plan:** All fixes necessary for compilation and correct test execution. No scope creep.

## Issues Encountered

- `Crossover::Cycle` failed on `RangeGene<i32>` at runtime — cycle crossover requires permutation chromosomes with unique gene IDs, but RangeGene i32 values are not unique. Fixed by using `Clone` crossover instead.
- `Crossover::MultiPoint` requires `number_of_points` to be configured; not suitable for portfolio tests without additional builder calls. Fixed by using `Clone` instead.
- Several trait imports needed to be added for the GA builder pattern: `ConfigurationT`, `SelectionConfig`, `CrossoverConfig`, `MutationConfig`, `StoppingConfig`, and `ChromosomeT`.

## Known Stubs

None — the AOS GA integration is fully functional. All AOS state machine behavior (select_operator, record_rewards, update) is tested at both unit and integration level.

## Threat Surface Scan

All planned mitigations from threat model in place:
- T-43-05: Mutex<AosState> lock contention — accepted, lock held briefly per couple
- T-43-06: Reward accumulator double-count — mitigated, drained once after parallel loop
- T-43-07: Information disclosure — accepted, best_fitness already observable
- T-43-08: Reward normalization — mitigated via EPSILON clamp in compute_normalized_reward

No new threat surface introduced.

## Self-Check

- [X] Task commits exist: effd0d5, 6f530af, 1487cd9
- [X] `cargo check` passes
- [X] `cargo test --test test_engines` — 327 passed, 2 ignored (5 new AOS tests, no regression)
- [X] `cargo test --test test_engines -- test_aos_` — 18/18 pass (13 Plan 01 + 5 Plan 02)
- [X] `cargo clippy` — zero new warnings
- [X] Ga struct has `aos_crossover: Option<Mutex<AosState>>` and `aos_mutation: Option<Mutex<AosState>>` fields
- [X] AOS state initialized at run start when portfolios configured
- [X] parent_crossover dispatches through AOS when portfolio configured
- [X] Rewards accumulated via Mutex and batch-updated after parallel loop
- [X] All AOS integration tests pass (PM, AP, MAB, both portfolios, + adaptive GA coexistence)
- [X] SUMMARY.md created in plan directory

## Verification

- **cargo check:** Pass
- **cargo clippy:** Zero new warnings
- **cargo test --test test_engines -- test_aos_:** 18 passed
- **cargo test --test test_engines (full suite):** 327 passed, 2 ignored

## Next Phase Readiness

- AOS GA integration is complete: state init, per-couple dispatch, reward accumulation, batch update
- 5 integration tests verify all AOS strategies (PM/AP/MAB) work in the full GA loop
- AOS + Adaptive GA coexistence verified (D-13)
- Ready for Phase 43 Plan 03 if needed: advance_generation hook, per-generation AOS lifecycle extensions

---
*Phase: 43-adaptive-operator-selection-aos*
*Completed: 2026-05-13*
