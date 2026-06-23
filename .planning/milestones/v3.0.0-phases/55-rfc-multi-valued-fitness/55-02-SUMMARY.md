---
phase: 55-rfc-multi-valued-fitness
plan: 02
subsystem: chromosomes
tags: [rust, chromosomes, vector-fitness, multi-objective, serde]

requires:
  - phase: 55-01
    provides: VectorFitness trait definition in src/traits/vector_fitness.rs

provides:
  - fitness_values: Vec<f64> field on all 7 built-in chromosome types
  - VectorFitness impl for Binary, Range<T>, ListChromosome<T>, UniqueChromosome<T>, MultiRangeChromosome<T>, MultiUniqueChromosome<T>, GpChromosome<N>
  - Serde-compatible new field via serde(default) on all 7 types

affects: [55-03, 55-04, 55-05, nsga2, lexicase-selection, mo-engines]

tech-stack:
  added: []
  patterns:
    - "VectorFitness explicit per-type impl — no blanket impl (D-01 opt-in pattern)"
    - "serde(default) on Vec<f64> fields for checkpoint backward compatibility"
    - "Private fitness_values field in GpChromosome matches its encapsulation style"

key-files:
  created: []
  modified:
    - src/types/chromosomes/binary.rs
    - src/types/chromosomes/range.rs
    - src/types/chromosomes/list.rs
    - src/types/chromosomes/unique.rs
    - src/types/chromosomes/multi_range.rs
    - src/types/chromosomes/multi_unique.rs
    - src/engines/gp/chromosome.rs

key-decisions:
  - "Used serde(default) not serde(skip) so fitness_values persists through checkpoint save/restore"
  - "GpChromosome fitness_values is private (no pub) to preserve its encapsulation style"
  - "VectorFitness impl generic bounds match the ChromosomeT impl bounds for each type"

patterns-established:
  - "fitness_values field placed immediately after age field in struct definition"
  - "VectorFitness impl block placed after ChromosomeT impl block, before LinearChromosome"

requirements-completed: [TRAITS-01]

duration: 12min
completed: 2026-05-30
---

# Phase 55 Plan 02: VectorFitness Impl on All Built-in Chromosome Types Summary

**`fitness_values: Vec<f64>` field and explicit `VectorFitness` impl added to all 7 built-in chromosome types with serde(default) for checkpoint backward compatibility**

## Performance

- **Duration:** 12 min
- **Started:** 2026-05-30T00:00:00Z
- **Completed:** 2026-05-30T00:12:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added `fitness_values: Vec<f64>` field to all 6 flat chromosome types (Binary, Range, List, Unique, MultiRange, MultiUnique) with `serde(default)` annotation
- Added explicit `VectorFitness` impl for each flat chromosome type after its `ChromosomeT` impl block
- Added `fitness_values: Vec<f64>` as private field to `GpChromosome<N>`, updated Clone/Default/with_root constructors, and added `VectorFitness` impl

## Task Commits

Each task was committed atomically:

1. **Task 1: 6 flat chromosomes** - `ad76258` (feat)
2. **Task 2: GpChromosome** - `0d4bef1` (feat)
3. **Fix: multi_unique line-wrap** - `63da2bd` (fix — reformatted VectorFitness impl header to single line for grep compatibility)

**Plan metadata:** _(docs commit follows)_

## Files Created/Modified

- `src/types/chromosomes/binary.rs` — Added `fitness_values: Vec<f64>` field + `VectorFitness` impl
- `src/types/chromosomes/range.rs` — Added `fitness_values: Vec<f64>` field + `VectorFitness` impl
- `src/types/chromosomes/list.rs` — Added `fitness_values: Vec<f64>` field + `VectorFitness` impl
- `src/types/chromosomes/unique.rs` — Added `fitness_values: Vec<f64>` field + `VectorFitness` impl
- `src/types/chromosomes/multi_range.rs` — Added `fitness_values: Vec<f64>` field + `VectorFitness` impl
- `src/types/chromosomes/multi_unique.rs` — Added `fitness_values: Vec<f64>` field + `VectorFitness` impl
- `src/engines/gp/chromosome.rs` — Added private `fitness_values: Vec<f64>`, updated Clone/Default/with_root, added `VectorFitness` impl

## Decisions Made

- `serde(default)` (not `serde(skip)`) so `fitness_values` is serialized in checkpoints and deserializes to `Vec::new()` when absent in old payloads
- `GpChromosome::fitness_values` kept private (no `pub`) to match existing encapsulation style of that type's other fields (`fitness`, `age`, `fitness_fn`)
- Each `impl VectorFitness for <Type>` uses the same generic bounds as the `impl ChromosomeT for <Type>` block above it

## Deviations from Plan

None — plan executed exactly as written. The minor reformatting of `multi_unique.rs` to put the `VectorFitness` impl header on a single line was a stylistic fix for grep compatibility, not a behavioral deviation.

## Issues Encountered

- `multi_unique.rs` VectorFitness impl was initially line-wrapped following the existing `LinearChromosome` style, which caused the grep verification pattern `impl.*VectorFitness for` to not match. Reformatted to single line.

## Known Stubs

None — `fitness_values` is a real storage field initialized to `Vec::new()`. No placeholder data.

## Threat Flags

None — these changes only add a data field and trait impl to existing chromosome types. No new network endpoints, auth paths, or trust boundary surface introduced.

## Self-Check

- [x] `grep -l "impl.*VectorFitness for" src/types/chromosomes/*.rs src/engines/gp/chromosome.rs` lists all 7 files
- [x] `cargo check --features serde` — zero errors from these 7 files (pre-existing `MultiCaseFitness` errors in `ga.rs`, `selection.rs`, `selection/lexicase.rs` are unrelated Wave 1 renames)
- [x] Commits `ad76258`, `0d4bef1`, `63da2bd` exist in git log

## Self-Check: PASSED

## Next Phase Readiness

All 7 built-in chromosome types now implement `VectorFitness`. Wave 3 (Plans 04+05) can add `U: VectorFitness` bounds to the MO engine APIs (`Nsga2Ga`, etc.) without requiring users to change their chromosome types.

---
*Phase: 55-rfc-multi-valued-fitness*
*Completed: 2026-05-30*
