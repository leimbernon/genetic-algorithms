---
phase: 47-architecture-audit-chromosomet-split
plan: 04
subsystem: [configuration, types]
tags: [rust, breaking-change, ChromosomeLength, enum-type]

requires:
  - phase: 47-03
    provides: "LinearChromosome bound upgrade across all engines"
provides:
  - "ChromosomeLength enum with Fixed and Variable variants"
  - "LimitConfiguration uses ChromosomeLength instead of genes_per_chromosome"
  - "Removed needs_unique_ids and alleles_can_be_repeated fields"
  - "Initializer signatures cleaned (dropped _needs_unique_ids parameter)"
affects: [47-05, 47-06, 52-variable-length]

tech-stack:
  added: []
  patterns: ["Standalone public enum with cfg_attr serde derive", "pub(crate) field encapsulation"]

key-files:
  created:
    - "src/chromosomes/length.rs"
    - "tests/test_chromosome_length.rs"
  modified:
    - "src/configuration.rs"
    - "src/traits/configuration.rs"
    - "src/chromosomes/mod.rs"
    - "src/lib.rs"
    - "src/initializers/binary_initializer.rs"
    - "src/initializers/range_initializer.rs"
    - "src/initializers/list_initializer.rs"
    - "src/initializers/mod.rs"

key-decisions:
  - "ChromosomeLength is Copy + Clone + Debug + PartialEq + Default (Fixed(0))"
  - "Field is pub(crate) on LimitConfiguration — external access via builder only"
  - "InitializationFn type alias drops _needs_unique_ids parameter (3-arg → 2-arg fn pointer)"

patterns-established:
  - "ChromosomeLength pattern: enum-first configuration type replacing raw usize"

requirements-completed: [ARCH-04, ARCH-05]

duration: 15min
completed: 2026-05-20
---

# Phase 47 Plan 04: ChromosomeLength Enum + LimitConfiguration Cleanup Summary

**Introduced ChromosomeLength enum, removed deprecated LimitConfiguration fields, cleaned initializer signatures**

## Accomplishments
- Created `ChromosomeLength` enum with `Fixed(usize)` and `Variable { min, max }` variants
- Replaced `genes_per_chromosome: usize` with `chromosome_length: ChromosomeLength` in LimitConfiguration
- Removed `needs_unique_ids` and `alleles_can_be_repeated` fields from LimitConfiguration
- Dropped `_needs_unique_ids` parameter from all 3 initializer functions
- Updated `InitializationFn<G>` type alias from 3-arg to 2-arg
- Added `with_chromosome_length` builder method replacing 3 removed methods
- Wave 0 tests pass on native and WASM

## Task Commits

1. **Task 1: ChromosomeLength enum + tests** — `bdb16fa` (feat)
2. **Task 2: LimitConfiguration migration** — `433fc49` (refactor)
3. **Wave 0 tests** — `09bc4ad` (test)

## Files Created/Modified
- `src/chromosomes/length.rs` — ChromosomeLength enum (new)
- `src/chromosomes/mod.rs` — pub mod length + re-export
- `src/lib.rs` — ChromosomeLength re-export at crate root
- `src/configuration.rs` — LimitConfiguration field swap
- `src/traits/configuration.rs` — with_chromosome_length builder method
- `src/initializers/*.rs` — dropped _needs_unique_ids parameter
- `tests/test_chromosome_length.rs` — variants, default, serde roundtrip tests

## Decisions Made
- Default is `Fixed(0)` — safe zero-value for Default-deriving config structs
- `pub(crate)` visibility on field — forces builder API usage externally
- Intentional compile breakage at downstream call sites (fixed in 47-05/47-06)

## Deviations from Plan
None

## Issues Encountered
None

## Next Phase Readiness
- ChromosomeLength available for Phase 52 (Variable variant)
- Downstream caller migration in 47-05 (StoppingCriteria) and 47-06 (engine call sites)

---
*Phase: 47-architecture-audit-chromosomet-split*
*Completed: 2026-05-20*
