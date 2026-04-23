---
phase: 20-crossover-algorithm-optimization
plan: 01
subsystem: operations
tags: [crossover, pmx, hashmap, algorithm-optimization, permutation]

# Dependency graph
requires:
  - phase: 19-clone-elimination
    provides: "set_gene()/dna_mut() patterns and U::new() child construction used as reference"
provides:
  - "O(n) PMX crossover with HashMap position map — eliminates O(n^2) linear scan"
  - "ALGO-01 and ALGO-02 requirements satisfied"
affects: [21-allocation-reduction, benchmarks]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pre-build HashMap<i32, usize> from slice once before hot loop — replaces .iter().position() O(n) scans"
    - "Pre-fill child Vec<G> from other.to_vec(), overwrite segment — eliminates Vec<Option<G>> + unwrap pattern"

key-files:
  created: []
  modified:
    - src/operations/crossover/pmx.rs
    - .planning/REQUIREMENTS.md

key-decisions:
  - "Pre-fill child from other.to_vec() + clone_from_slice for segment copy — clippy-safe and allocation-efficient"
  - "ALGO-01 (OX) satisfied by prior commit ca5bb76, no Phase 20 code change needed"

patterns-established:
  - "HashMap position map pattern: build once, look up in O(1) — applicable to any operator with gene-position mapping"

requirements-completed: [ALGO-01, ALGO-02]

# Metrics
duration: 5min
completed: 2026-03-30
---

# Phase 20 Plan 01: Crossover Algorithm Optimization Summary

**PMX crossover rewritten from O(n^2) to O(n) using HashMap position map and direct Vec<G> child construction**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-30T18:35:35Z
- **Completed:** 2026-03-30T18:40:52Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Replaced O(n) linear scan (`.iter().position()`) in the chain-following loop with O(1) HashMap lookup, making total PMX complexity O(n)
- Eliminated `Vec<Option<G>>` child construction and `.unwrap()` epilogue by pre-filling child from `other.to_vec()` and overwriting the segment with `clone_from_slice`
- Marked ALGO-01 complete in REQUIREMENTS.md, referencing prior commit ca5bb76 where OX was already fixed
- All 6 existing PMX tests pass with no behavioral changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor pmx_build_child to HashMap position map and direct Vec construction** - `392c1a1` (feat)
2. **Task 2: Mark ALGO-01 complete in REQUIREMENTS.md** - `2f222e6` (chore)

**Plan metadata:** (docs commit to follow)

## Files Created/Modified

- `src/operations/crossover/pmx.rs` — pmx_build_child rewritten: HashMap lookup replaces linear scan, Vec<G> pre-fill replaces Vec<Option<G>>
- `.planning/REQUIREMENTS.md` — ALGO-01 marked complete with ca5bb76 reference; traceability table updated

## Decisions Made

- Pre-fill child from `other.to_vec()` and use `clone_from_slice` for the segment copy. This avoids the `Vec<Option<G>>` wrapper and satisfies clippy's `manual_memcpy` lint in one move.
- ALGO-01 (Order Crossover) was already satisfied by commit ca5bb76 in Phase 19. No new OX code change required — only REQUIREMENTS.md update needed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced for-loop segment copy with clone_from_slice to satisfy clippy**
- **Found during:** Task 1 verification (`cargo clippy -- -D warnings`)
- **Issue:** Initial `for i in start..=end { child[i] = donor[i].clone(); }` triggered `clippy::manual_memcpy` warning
- **Fix:** Replaced with `child[start..=end].clone_from_slice(&donor[start..=end]);`
- **Files modified:** src/operations/crossover/pmx.rs
- **Verification:** `cargo clippy` produces no errors for pmx.rs; all 6 tests still pass
- **Committed in:** 392c1a1 (Task 1 commit, amended before commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - clippy lint fix)
**Impact on plan:** Minor — the fix is a direct clippy improvement with identical behavior. No scope creep.

## Issues Encountered

- Pre-existing `clippy::too-many-arguments` error in `src/ga.rs:1216` surfaces when running `cargo clippy -- -D warnings`. This is out of scope (pre-existing, not introduced by this plan) and documented here for awareness.

## Next Phase Readiness

- Phase 20 plan 01 complete: ALGO-01 and ALGO-02 satisfied
- Phase 21 (allocation-reduction) can proceed independently
- PMX HashMap pattern is a template for any future operator needing gene-position maps

---
*Phase: 20-crossover-algorithm-optimization*
*Completed: 2026-03-30*
