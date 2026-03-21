---
phase: 07-list-genotype
plan: 02
subsystem: operations
tags: [genetic-algorithms, rust, mutation, initializer, list-genotype, crossover, integration-tests]

# Dependency graph
requires:
  - phase: 07-list-genotype-plan-01
    provides: List<T> gene and ListChromosome<T> with ChromosomeT trait
provides:
  - Mutation::ListValue operator (list_value_mutation fn)
  - ValueMutable impl for ListChromosome<T>
  - list_random_initialization (with replacement)
  - list_random_initialization_without_repetitions (permutation semantics)
  - Integration test suite proving ListChromosome works with full operator pipeline
affects: [future-list-ga-examples, operator-docs, usability-milestone]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ValueMutable impl placed in operator file (list_value.rs) to avoid circular imports"
    - "Generic T impl for ValueMutable (vs monomorphized Range impls)"
    - "Fisher-Yates shuffle for no-repetition initializer"
    - "Guard gene.alleles.len() < 2 before looping to avoid infinite loops"

key-files:
  created:
    - src/operations/mutation/list_value.rs
    - src/initializers/list_initializer.rs
    - tests/chromosomes/test_list.rs
  modified:
    - src/operations.rs
    - src/operations/mutation.rs
    - src/initializers.rs
    - tests/test_chromosomes.rs

key-decisions:
  - "ValueMutable impl for ListChromosome<T> lives in list_value.rs (not chromosomes/list.rs) to avoid circular import between chromosomes and operations crates"
  - "Generic T impl (not monomorphized) for ValueMutable on ListChromosome — list mutation needs no numeric ops so one impl covers all T"
  - "list_random_initialization uses first template's alleles for without-repetitions; with-repetitions picks random template per gene"
  - "Single-allele guard (alleles.len() < 2) returns early to avoid infinite rejection loop in ListValue mutation"

patterns-established:
  - "ValueMutable impl in mutation operator file when circular imports would occur"
  - "Operator module re-export: pub mod list_value in mutation.rs, pub mod list_initializer in initializers.rs"

requirements-completed: [LIST-03, LIST-04]

# Metrics
duration: 63min
completed: 2026-03-21
---

# Phase 07 Plan 02: List Genotype — Mutation, Initializer, and Integration Tests Summary

**Mutation::ListValue operator plus list_random_initialization functions completing ListChromosome<T> for GA use with full operator pipeline integration tests**

## Performance

- **Duration:** 63 min
- **Started:** 2026-03-21T15:00:00Z
- **Completed:** 2026-03-21T16:03:00Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added `Mutation::ListValue` enum variant and `list_value_mutation` function that replaces exactly one gene's allele index with a different one, with guard for empty chromosomes and single-allele genes
- Implemented `ValueMutable` for `ListChromosome<T>` (generic over T) in `list_value.rs` to avoid circular imports
- Created `list_random_initialization` (with replacement) and `list_random_initialization_without_repetitions` (Fisher-Yates permutation) matching the GA initializer API signature
- Proved ListChromosome works end-to-end: swap/inversion/scramble/insertion/ListValue mutations, SinglePoint/Uniform crossover, initialization, serde, and a 5-generation full GA run — all passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Mutation::ListValue operator + ValueMutable impl** - `c151d35` (feat)
2. **Task 2: List population initializer functions** - `55455c9` (feat)
3. **Task 3: Integration tests** - `67202d6` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/operations/mutation/list_value.rs` — list_value_mutation fn + ValueMutable impl for ListChromosome<T>, 8 unit tests
- `src/operations.rs` — Added `ListValue` variant to `Mutation` enum
- `src/operations/mutation.rs` — Added `pub mod list_value`, `Mutation::ListValue` dispatch arm, factory_non_value error arm
- `src/initializers/list_initializer.rs` — list_random_initialization + list_random_initialization_without_repetitions, 9 unit tests
- `src/initializers.rs` — Added `pub mod list_initializer` + `pub use list_initializer::*`
- `tests/chromosomes/test_list.rs` — 9 integration tests covering all operator types and full GA run
- `tests/test_chromosomes.rs` — Registered `mod test_list`

## Decisions Made

- ValueMutable impl for ListChromosome<T> lives in `list_value.rs` (not in `chromosomes/list.rs`) to avoid circular imports between the `chromosomes` and `operations` modules.
- Generic T impl (not monomorphized) for ValueMutable on ListChromosome — list mutation needs no numeric operations so one impl covers all T types.
- `list_random_initialization_without_repetitions` uses the first template's allele set for the permutation source; this matches typical usage where one template carries the full allele alphabet.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Swap mutation test assertion corrected to account for same-position swaps**
- **Found during:** Task 3 (integration tests)
- **Issue:** Initial test asserted "exactly 2 genes changed" after swap, but swap can pick the same index twice (valid no-op) or pick identical id values, yielding 0 observable changes
- **Fix:** Rewrote assertion to verify the sorted multiset of gene ids is unchanged (swap preserves alleles, never adds/removes), which is the correct invariant
- **Files modified:** tests/chromosomes/test_list.rs
- **Verification:** cargo test test_list_swap_mutation passes
- **Committed in:** 67202d6 (Task 3 commit)

**2. [Rule 1 - Bug] Fixed closure lifetime in full GA run test**
- **Found during:** Task 3 (integration tests)
- **Issue:** `with_fitness_fn` closure borrowed local `alleles` Vec which didn't satisfy `'static` bound; also E0716 temporary dropped while borrowed
- **Fix:** Inlined the allele literal in the closure (`g.value == 'a'`) and assigned GA to a `let mut ga` binding before calling `.run()`
- **Files modified:** tests/chromosomes/test_list.rs
- **Verification:** cargo test test_list_full_ga_run passes
- **Committed in:** 67202d6 (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (2 x Rule 1 - Bug)
**Impact on plan:** Both fixes were in test code only; no production code changed. No scope creep.

## Issues Encountered

None beyond the two auto-fixed test bugs above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- LIST-01 through LIST-04 requirements all complete
- ListChromosome<T> is fully usable in GA runs: initialization, all standard mutations, all crossover operators, serde, and end-to-end GA
- Phase 07 (list-genotype) is complete — ready for PR to milestone branch
- No blockers for subsequent phases

---
*Phase: 07-list-genotype*
*Completed: 2026-03-21*
