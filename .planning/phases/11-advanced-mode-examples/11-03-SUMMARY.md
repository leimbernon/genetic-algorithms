---
phase: 11-advanced-mode-examples
plan: "03"
subsystem: examples
tags: [rust, genetic-algorithms, permutation, job-scheduling, combinatorial-optimization, order-crossover, insertion-mutation]

# Dependency graph
requires:
  - phase: 10-single-population-examples
    provides: RangeChromosome<i32> permutation pattern with range_random_initialization
provides:
  - examples/job_scheduling.rs -- permutation-based makespan minimization with Order crossover and Insertion mutation

affects:
  - future example phases referencing permutation encoding patterns

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Permutation encoding with RangeChromosome<i32> and range_random_initialization Some(false)
    - Crossover::Order (OX) + Mutation::Insertion for permutation-safe operators
    - Greedy FIFO heuristic as fitness function for scheduling problems

key-files:
  created:
    - examples/job_scheduling.rs
  modified: []

key-decisions:
  - "Used Crossover::Order (OX) and Mutation::Insertion as the permutation-safe operator pair -- both guarantee no duplicate job indices after recombination or mutation"
  - "Greedy FIFO heuristic assigns each job to the earliest-available machine -- O(N*M) per evaluation, fast enough for 15x5 problem"
  - "PROCESSING_TIMES declared as a const at module level (not inside main) to allow use inside the fitness closure without capture issues"

patterns-established:
  - "Permutation GA pattern: range_random_initialization with Some(false) + Crossover::Order + Mutation::Insertion"
  - "Scheduling fitness pattern: machine_finish array updated greedily per gene in permutation order"

requirements-completed: [EX-04]

# Metrics
duration: 2min
completed: 2026-03-22
---

# Phase 11 Plan 03: Job Scheduling Summary

**Permutation-encoded parallel machine scheduling with Order crossover (OX) and Insertion mutation minimizing makespan across 15 jobs x 5 machines**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-22T10:33:49Z
- **Completed:** 2026-03-22T10:35:01Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Created `examples/job_scheduling.rs` (173 lines) demonstrating permutation-encoded combinatorial optimization
- Implemented greedy FIFO scheduling heuristic as fitness function using a `machine_finish` array updated per gene
- Used Order crossover and Insertion mutation as permutation-safe operators that never produce duplicate job indices
- Example runs in under 2 seconds and converges from makespan 15 down to 13 within 500 generations

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Job Scheduling permutation example** - `45dab97` (feat)

**Plan metadata:** see final commit below

## Files Created/Modified

- `examples/job_scheduling.rs` -- Job scheduling makespan minimization example with permutation encoding, greedy FIFO fitness, Order crossover, and Insertion mutation

## Decisions Made

- `PROCESSING_TIMES` declared as a `const` at module level rather than inside `main()` to allow capture-free use inside the `fitness_fn` closure
- Used `Crossover::Order` + `Mutation::Insertion` as the permutation operator pair (both verified to exist in the codebase before writing)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Job scheduling example is complete and runnable
- Permutation encoding pattern (RangeChromosome<i32> + Order + Insertion) is now documented via a working example
- Phase 11 all 3 plans complete if plans 01 and 02 are done

---
*Phase: 11-advanced-mode-examples*
*Completed: 2026-03-22*
