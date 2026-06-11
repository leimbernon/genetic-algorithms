---
phase: 47
plan: 07
subsystem: reporter-removal-migration-guide
tags:
  - rust
  - reporter-removal
  - breaking-change
  - documentation
  - migration
dependency_graph:
  requires:
    - 47-06
  provides:
    - arch03-satisfied
    - reporter-removed
    - migration-guide-published
  affects:
    - 47-08
tech_stack:
  added: []
  patterns:
    - MIGRATION.md at crate root (breaking-change guide pattern)
    - Cargo.toml include array (explicit package manifest)
    - GaObserver replaces Reporter at all call sites
key_files:
  created:
    - MIGRATION.md
  modified:
    - src/lib.rs
    - src/engines/ga.rs
    - src/observe/observer/mod.rs
    - src/observe/observer/log.rs
    - tests/test_observe.rs
    - tests/observe/test_serde.rs
    - tests/engines/warm_starting/test_warm_starting.rs
    - README.md
    - Cargo.toml
  deleted:
    - src/observe/reporter/mod.rs
    - src/observe/reporter/duration.rs
    - src/observe/reporter/noop.rs
    - src/observe/reporter/simple.rs
    - tests/observe/reporter/test_reporter.rs
decisions:
  - D-10 (Reporter<U> trait removed entirely — with_reporter builder + 4 fire points deleted)
  - D-11 (MIGRATION.md published with all 7 v3.0.0 breaking change sections)
  - T-47-19 mitigated (Reporter removal documented in MIGRATION.md)
  - T-47-20 mitigated (cargo package --list confirms MIGRATION.md ships; include array replaces exclude)
  - T-47-21 accepted (reporter tests deleted — target trait no longer exists)
metrics:
  completed_date: "2026-05-21"
  duration_mins: 55
  tasks_completed: 2
  files_changed: 14
---

# Phase 47 Plan 07: Reporter Removal + Migration Guide Summary

Remove the deprecated `Reporter<U>` trait, all 4 fire points, `with_reporter()` builder, and module re-exports. Publish `MIGRATION.md` at the crate root covering all Phase 47 v3.0.0 breaking changes. First plan of PR 3 — satisfies ARCH-03.

## Tasks Completed

| Task | Commit | Description |
|------|--------|-------------|
| Task 1: Reporter removal | 8383a99 | Delete reporter module, clean ga.rs, remove test file |
| Task 2: MIGRATION.md + package | 85cd65a | Create MIGRATION.md, update README + Cargo.toml, fix serde tests |

## What Was Built

### Task 1 — Reporter Removal

Deleted `src/observe/reporter/` directory (4 files: mod.rs, duration.rs, noop.rs, simple.rs).

Cleaned `src/engines/ga.rs`:
- Removed `#[allow(deprecated)] use crate::reporter::Reporter;` import
- Removed `reporter: Option<Box<dyn Reporter<U> + Send>>` struct field
- Removed `reporter: None` from `Default` impl
- Removed `#[allow(deprecated)]` from `configuration()` accessor impl block
- Removed `with_reporter()` builder method (including deprecated + allow annotations)
- Removed 4 reporter fire points: `on_start` (~line 1447), `on_generation_complete` (~line 1974), `on_new_best` (~line 2060), `on_finish` (~line 2125)

Cleaned `src/lib.rs`: removed `#[path = "observe/reporter/mod.rs"] pub mod reporter;` lines.

Deleted `tests/observe/reporter/test_reporter.rs` (251 lines of tests for removed trait).

Removed `mod reporter { mod test_reporter; }` from `tests/test_observe.rs`.

### Task 2 — MIGRATION.md + Package

Created `MIGRATION.md` at crate root with 7 breaking-change sections:
1. Trait split: ChromosomeT + LinearChromosome (D-01/D-02)
2. LinearChromosome: `default()` renamed to `reset()` (D-03)
3. Reporter removed — use GaObserver (D-10)
4. ChromosomeLength replaces genes_per_chromosome (D-07)
5. Flat stopping builders replace StoppingCriteria struct (D-08)
6. LimitConfiguration field removals (D-06)
7. GaConfiguration field access → accessor methods (D-09)

Updated `README.md`: added `> **v3.0.0 users:** see [MIGRATION.md](./MIGRATION.md)...` notice at top.

Updated `Cargo.toml`: switched from `exclude` array to explicit `include` array listing all publishable artifacts including `"MIGRATION.md"`. Verified `cargo package --list` includes MIGRATION.md.

Fixed rustdoc unresolved links in:
- `src/observe/observer/mod.rs`: removed `[Reporter]: crate::reporter::Reporter` reference, rewrote comparison table
- `src/observe/observer/log.rs`: removed `SimpleReporter` cross-reference

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unresolved rustdoc links in observer module**
- **Found during:** Task 2 `cargo doc --no-deps --all-features`
- **Issue:** `src/observe/observer/mod.rs` had `[Reporter]: crate::reporter::Reporter` and `GaObserver vs Reporter` table referencing the removed module. `src/observe/observer/log.rs` referenced `SimpleReporter`.
- **Fix:** Rewrote module-level doc to remove dead link; updated `# Differences from Reporter` section to `# GaObserver vs the removed Reporter trait`
- **Files modified:** `src/observe/observer/mod.rs`, `src/observe/observer/log.rs`
- **Commit:** 85cd65a

**2. [Rule 1 - Bug] Fixed pre-existing serde test failures**
- **Found during:** Task 2 verification (`cargo test --features serde`)
- **Issue:** `tests/observe/test_serde.rs` still imported `StoppingCriteria` (removed in 47-05), used `genes_per_chromosome` field, 3-arg init closures, and directly constructed `GaConfiguration` with `pub(crate)` fields. `tests/engines/warm_starting/test_warm_starting.rs` had 3 `range_random_initialization` calls with 3 args and direct `configuration.selection_configuration.method` access (pub(crate) field).
- **Root cause:** These test files were not migrated in Plan 47-06 because they are `#[cfg(feature = "serde")]`-gated and the 47-06 worktree ran without `--features serde`.
- **Fix:** Rewrote `serde_ga_configuration_with_values` to use Ga builder + `ga.configuration.clone()`. Renamed `serde_stopping_criteria` to `serde_stopping_criteria_flat` testing the new flat fields. Fixed 3 `range_random_initialization` 3-arg calls in warm_starting to 2-arg. Fixed 3 direct `configuration.selection/crossover/mutation_configuration.method` accesses to use `configuration().selection()/crossover()/mutation().method`.
- **Files modified:** `tests/observe/test_serde.rs`, `tests/engines/warm_starting/test_warm_starting.rs`
- **Commit:** 85cd65a

## Verification Gates

All 5 gates GREEN:
- `cargo test` — 966 passed, 28 ignored
- `cargo test --features serde` — 1003 passed, 28 ignored
- `cargo clippy --all-features` — 0 errors, 1 warning (pre-existing: unused `alleles` param in generic_validator.rs)
- `cargo check --target wasm32-unknown-unknown` — 0 errors
- `cargo doc --no-deps --all-features` — 0 warnings

## Known Stubs

None — MIGRATION.md is complete documentation, no placeholder content.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced.
MIGRATION.md is a static documentation file. T-47-19 and T-47-20 mitigated as planned.

## Self-Check: PASSED

- `src/observe/reporter/` directory does not exist: CONFIRMED
- `src/lib.rs` contains 0 occurrences of `pub mod reporter`: CONFIRMED
- `src/engines/ga.rs` contains 0 reporter references: CONFIRMED
- `tests/` contains 0 reporter trait references: CONFIRMED
- `MIGRATION.md` exists at crate root with 7 `##` sections: CONFIRMED
- `Cargo.toml` contains `"MIGRATION.md"` in include array: CONFIRMED
- `README.md` links to MIGRATION.md: CONFIRMED
- `cargo package --list` includes MIGRATION.md: CONFIRMED
- All 5 verification gates GREEN: CONFIRMED
- Commits 8383a99 and 85cd65a exist: CONFIRMED
