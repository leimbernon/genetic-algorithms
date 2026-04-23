---
phase: 22-survivor-extension-optimization
plan: "02"
subsystem: performance
tags: [atomics, rayon, rng, concurrency, parallelism]

# Dependency graph
requires:
  - phase: 22-survivor-extension-optimization/22-01
    provides: survivor and extension operator optimizations
provides:
  - Relaxed atomic orderings in make_rng hot path (Acquire/Release/Relaxed instead of SeqCst)
  - Parallel extension population regrow via rayon into_par_iter
affects: [ga, rng, performance, extension]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Release/Acquire pairing for atomic seed visibility: Release in set_seed, Acquire in make_rng load"
    - "Relaxed ordering for monotonic counter used only for uniqueness, not synchronization"
    - "rayon into_par_iter collect-then-extend for parallel chromosome creation without sequential push"

key-files:
  created: []
  modified:
    - src/rng.rs
    - src/ga.rs

key-decisions:
  - "Acquire for SEED.load pairs with Release in set_seed — ensures seed visibility without full SeqCst barrier"
  - "Relaxed for COUNTER.fetch_add — counter only needs monotonic uniqueness per thread, not cross-thread ordering"
  - "Extract alleles_ref, genes_per_chromosome, ff before rayon closure to avoid borrowing self inside parallel iterator"

patterns-established:
  - "Minimum-correct atomic ordering: Release/Acquire pair for producer/consumer, Relaxed for monotonic counters"
  - "Parallel collect-then-extend: (0..n).into_par_iter().map(|_| {...}).collect() then vec.extend(results)"

requirements-completed:
  - CONC-01
  - CONC-02

# Metrics
duration: 11min
completed: "2026-03-31"
---

# Phase 22 Plan 02: RNG Atomic Orderings and Parallel Extension Regrow Summary

**RNG hot path reduced from SeqCst to Acquire/Relaxed and extension population regrow parallelized with rayon par_iter**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-31T16:57:15Z
- **Completed:** 2026-03-31T17:07:30Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- `make_rng()` now uses `Ordering::Acquire` for the seed load and `Ordering::Relaxed` for the counter increment — eliminates unnecessary full memory barriers on every per-operator RNG creation call
- `set_seed()` uses `Ordering::Release` for all stores — correctly pairs with Acquire in `make_rng` while avoiding SeqCst overhead
- Extension population regrow replaced sequential `for _ in 0..deficit` push loop with `(0..deficit).into_par_iter().map(...).collect()` + `extend` — chromosome creation now scales with available rayon threads
- All existing tests pass including serde feature; RNG determinism tests unchanged

## Task Commits

Each task was committed atomically:

1. **Task 1: Relax RNG atomic orderings** - `fb72bc6` (perf)
2. **Task 2: Parallelize extension population regrow with rayon** - `a257617` (perf)

## Files Created/Modified
- `src/rng.rs` - Changed 5 `Ordering::SeqCst` to `Acquire` (load), `Release` (stores), `Relaxed` (counter fetch_add)
- `src/ga.rs` - Replaced sequential regrow for-loop with rayon parallel collect-then-extend pattern

## Decisions Made
- Used `Acquire` for `SEED.load` to pair with `Release` in `set_seed` — the Release/Acquire pair provides the required happens-before guarantee that a seed written by `set_seed` is visible to any subsequent `make_rng` call
- Used `Relaxed` for `COUNTER.fetch_add` because the counter's only purpose is uniqueness (each thread gets a different value); exact ordering between threads is irrelevant
- Extracted `genes_per_chromosome`, `alleles_can_be_repeated`, `alleles_ref`, and `ff` before the rayon closure to satisfy the borrow checker — `self` cannot be borrowed inside a `into_par_iter` closure that needs `Send`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - the borrow extraction before the rayon closure was anticipated in the plan's action description and worked exactly as specified.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 22 complete — both plans executed, all 6 performance optimization issues addressed across phases 19-22
- All changes are internal optimizations with no public API changes
- Benchmark runs can now be compared against pre-milestone baseline to quantify improvements

---
*Phase: 22-survivor-extension-optimization*
*Completed: 2026-03-31*
