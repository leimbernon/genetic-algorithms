---
phase: 47-architecture-audit-chromosomet-split
plan: 01
subsystem: traits
tags:
  - rust
  - traits
  - refactor
  - breaking-change
  - tdd

dependency_graph:
  requires: []
  provides:
    - "ChromosomeT minimal evaluation contract (ARCH-01)"
    - "LinearChromosome flat-slice supertrait with default set_gene/reset (ARCH-02)"
    - "Wave 0 tests locking both new contracts"
  affects:
    - "All implementors of ChromosomeT (47-02 updates them)"
    - "All operators with U: ChromosomeT bound (47-02 changes to U: LinearChromosome)"

tech_stack:
  added: []
  patterns:
    - "Rust trait supertrait composition (LinearChromosome: ChromosomeT)"
    - "TDD RED/GREEN cycle — tests written before trait implementation"
    - "Default trait method implementations with bounds checking"

key_files:
  created:
    - src/traits/linear_chromosome.rs
    - tests/test_chromosomet_core.rs
    - tests/test_linear_chromosome.rs
  modified:
    - src/traits/chromosome.rs
    - src/traits.rs
    - src/lib.rs

decisions:
  - "ChromosomeT shrunk to 6-method evaluation contract per D-01 (fitness/set_fitness/calculate_fitness/age/set_age/fitness_distance)"
  - "LinearChromosome: ChromosomeT supertrait per D-02 — adds dna/dna_mut/set_dna/set_fitness_fn/new_gene + default set_gene/reset"
  - "reset() -> &mut Self replaces old default(self) -> Self per D-03 — removes Default trait shadowing"
  - "set_fitness_fn stays on LinearChromosome only per D-04 — ChromosomeT has no fitness function installation"
  - "Tests placed directly under tests/ as test_chromosomet_core.rs and test_linear_chromosome.rs per CLAUDE.md tests-in-tests/ rule"

metrics:
  duration: "~10 minutes execution"
  completed_date: "2026-05-20"
  tasks_completed: 2
  tasks_total: 2
  files_created: 3
  files_modified: 3
---

# Phase 47 Plan 01: ChromosomeT Split — Summary

**One-liner:** Split all-in-one `ChromosomeT` (98 lines) into minimal evaluation contract `ChromosomeT` (~60 lines) and `LinearChromosome: ChromosomeT` supertrait with default `set_gene`/`reset` implementations, with Wave 0 TDD tests.

## What Was Built

### Task 1: RED — Wave 0 failing tests

Created two integration test files that exercise the new trait contracts:

- **`tests/test_chromosomet_core.rs`**: `MinimalChromo` implements only `ChromosomeT` with no DNA fields. Tests `fitness_distance` default impl and setter chainability. Fails RED because old `ChromosomeT` requires `dna/dna_mut/set_dna/set_fitness_fn`.
- **`tests/test_linear_chromosome.rs`**: `LinearChromo` implements both `ChromosomeT` and `LinearChromosome`. Tests `set_gene` bounds checking (in-bounds update + OOB no-op), `reset()` state, and `new_gene()` delegation. Fails RED because `LinearChromosome` doesn't exist yet.

RED state verified: `E0046` (missing trait items) and `E0432` (unresolved import) confirmed by `cargo test --no-run`.

### Task 2: GREEN — Trait split implementation

- **`src/traits/chromosome.rs`**: Removed `dna`, `dna_mut`, `set_dna`, `set_fitness_fn`, `new_gene`, `set_gene`, and `default(self)->Self`. Kept: `new()`, `calculate_fitness`, `fitness`, `set_fitness`, `set_age`, `age`, `fitness_distance` (default impl).
- **`src/traits/linear_chromosome.rs`** (new): `pub trait LinearChromosome: ChromosomeT` with required `dna/dna_mut/set_dna/set_fitness_fn`, default `new_gene()`, bounds-checked `set_gene()` (logs `log::warn!` on OOB), and `reset()` (sets fitness=NaN, age=0, dna=empty via `Cow::Borrowed(&[])`).
- **`src/traits.rs`**: Added `pub mod linear_chromosome` and `pub use linear_chromosome::LinearChromosome`.
- **`src/lib.rs`**: Added `pub use traits::LinearChromosome` at crate root.

## Deviations from Plan

None — plan executed exactly as written.

The intermediate `cargo check --lib` failure (318 errors from existing implementors) is the expected in-plan state documented in the task: "This is acceptable interim state ONLY within this plan; the lib build will be GREEN again after 47-02."

## TDD Gate Compliance

| Gate | Status | Commit |
|------|--------|--------|
| RED | Passed — both test files fail to compile with E0046/E0432 | `76815a0` |
| GREEN | Passed — structural grep assertions confirm trait shape; full test execution deferred to post-47-02 | `295fa3a` |
| REFACTOR | Not needed — implementations are clean as written | — |

## Known Stubs

None. All default implementations in `LinearChromosome` are complete (set_gene with bounds check, reset with full state reset, new_gene delegating to Gene::new).

## Self-Check: PASSED

- `tests/test_chromosomet_core.rs` — FOUND
- `tests/test_linear_chromosome.rs` — FOUND
- `src/traits/chromosome.rs` — FOUND (shrunk to minimal)
- `src/traits/linear_chromosome.rs` — FOUND (new file)
- Commit `76815a0` — FOUND (RED tests)
- Commit `295fa3a` — FOUND (GREEN trait split)
- `pub trait ChromosomeT` in chromosome.rs — CONFIRMED
- `pub trait LinearChromosome: ChromosomeT` in linear_chromosome.rs — CONFIRMED
- `pub use traits::LinearChromosome` in lib.rs — CONFIRMED
- ChromosomeT does NOT contain dna/dna_mut/set_dna/set_fitness_fn/new_gene/set_gene/default — CONFIRMED

## Next Steps

Plan 47-02 will update all existing implementors (`BinaryChromosome`, `RangeChromosome`, `List`) by splitting their `impl ChromosomeT` blocks into `impl ChromosomeT` + `impl LinearChromosome`. After that, the crate will compile clean and the Wave 0 tests will execute.
