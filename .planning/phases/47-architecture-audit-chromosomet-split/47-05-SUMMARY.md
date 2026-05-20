---
phase: 47
plan: 05
subsystem: configuration
tags:
  - rust
  - configuration
  - stopping-criteria
  - encapsulation
  - wasm
dependency_graph:
  requires:
    - 47-04
  provides:
    - flat-stopping-fields
    - gaconfiguration-encapsulated
  affects:
    - 47-06
tech_stack:
  added: []
  patterns:
    - pub(crate) field + public accessor pattern (D-09)
    - flat stopping fields replacing StoppingCriteria struct (D-08)
    - WASM gate at call site only (field un-gated)
key_files:
  created:
    - tests/test_stopping_config.rs
  modified:
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/engines/ga.rs
decisions:
  - D-08 (StoppingCriteria dissolved into 3 flat pub(crate) fields on GaConfiguration)
  - D-09 (GaConfiguration fields pub(crate) with sub-struct-level public accessors)
  - WASM gate preserved at call site only — field is un-gated Option<f64>
metrics:
  duration_secs: 522
  completed_date: "2026-05-20"
  tasks_completed: 2
  files_changed: 4
---

# Phase 47 Plan 05: StoppingCriteria Dissolution + GaConfiguration Encapsulation Summary

Dissolve `StoppingCriteria` struct into 3 flat `pub(crate)` fields on `GaConfiguration`, add flat builder methods and public accessors, encapsulate all remaining `GaConfiguration` sub-struct fields with `pub(crate)`, add sub-struct-level read-only accessors, and update `ga.rs` to use flat field paths.

## Tasks Completed

| Task | Commit | Description |
|------|--------|-------------|
| Wave 0 tests (RED) | a37770c | `tests/test_stopping_config.rs` — 4 failing tests for flat stopping builders |
| Task 1: StoppingCriteria flatten + builders + encapsulation | 643f7b9 | Remove struct, add flat fields + accessors + trait changes |
| Task 2: ga.rs path update + Ga.configuration() accessor | b745315 | Replace `stopping_criteria.X` paths; add public accessor method |

## What Was Built

### StoppingCriteria Dissolution (D-08)
- Removed `StoppingCriteria` struct definition entirely from `src/configuration.rs`
- Added 3 flat `pub(crate)` fields on `GaConfiguration`: `stagnation_generations: Option<usize>`, `convergence_threshold: Option<f64>`, `max_duration_secs: Option<f64>`
- `max_duration_secs` field is intentionally **un-gated** — the WASM `#[cfg]` gate exists only at the call site in `ga.rs` (preserved per RESEARCH.md Pitfall 3 and PATTERNS.md WASM Gate Pattern)
- Added 3 public accessor methods on `impl GaConfiguration`: `stagnation_generations()`, `convergence_threshold()`, `max_duration_secs()`

### StoppingConfig Trait Changes (ARCH-06)
- Removed `fn with_stopping_criteria(self, criteria: StoppingCriteria) -> Self` from trait
- Added: `fn with_stagnation_limit(self, n: usize) -> Self`
- Added: `fn with_convergence_threshold(self, threshold: f64) -> Self`
- Added: `fn with_max_duration_secs(self, secs: f64) -> Self` (no `#[cfg]` on this method)
- Updated `impl StoppingConfig for GaConfiguration` and `impl StoppingConfig for Ga<U>` accordingly

### GaConfiguration Encapsulation (D-09 / ARCH-04)
Changed all top-level `GaConfiguration` fields from `pub` to `pub(crate)`:
- `adaptive_ga`, `number_of_threads`, `limit_configuration`, `selection_configuration`, `crossover_configuration`, `mutation_configuration`, `survivor`, `log_level`, `save_progress_configuration`, `elitism_count`, `niching_configuration`, `extension_configuration`, `rng_seed`, `crossover_portfolio`, `mutation_portfolio`, `aos_strategy`, `aos_reward_window`, `local_search_configuration`

Added sub-struct-level public accessors on `impl GaConfiguration`:
- `limit() -> &LimitConfiguration`
- `selection() -> &SelectionConfiguration`
- `crossover() -> &CrossoverConfiguration`
- `mutation() -> &MutationConfiguration`
- `survivor() -> Survivor`
- `extension() -> Option<&ExtensionConfiguration>`
- `log() -> LogLevel`
- `adaptive_ga() -> bool`
- `number_of_threads() -> usize`
- `elitism_count() -> usize`
- `save_progress() -> &SaveProgressConfiguration`

### ga.rs Path Updates
- Replaced all `self.configuration.stopping_criteria.X` paths with `self.configuration.X` (flat paths)
- Added `Ga::configuration() -> &GaConfiguration` public method for test/external access
- Updated `impl StoppingConfig for Ga<U>` to use the 3 new flat builder methods
- WASM gates preserved exactly as before:
  - `#[cfg(target_arch = "wasm32")]` warn-once block at run start
  - `#[cfg(not(target_arch = "wasm32"))]` gate around `max_duration_secs` usage

### Wave 0 Tests
`tests/test_stopping_config.rs` contains 4 tests:
- `test_stopping_config_with_stagnation_limit` — asserts `.with_stagnation_limit(50)` → accessor reads `Some(50)`
- `test_stopping_config_with_convergence_threshold` — asserts `.with_convergence_threshold(0.001)` → accessor reads `Some(0.001)`
- `test_stopping_config_with_max_duration_secs` — asserts `.with_max_duration_secs(10.0)` → accessor reads `Some(10.0)`
- `test_stopping_config_default_is_none` — asserts all three stopping fields default to `None`

## Remaining Compile Errors (Hand-off to 47-06)

The following 44 pre-existing errors remain, all inherited from plan 47-04 and scoped to caller migrations:

| Category | Count | Files |
|----------|-------|-------|
| `no field 'genes_per_chromosome'` | 12 | `src/engines/ga.rs`, `src/engines/island/nsga2.rs`, `src/engines/nsga2/mod.rs`, and 5 others |
| `no field 'alleles_can_be_repeated'` | 10 | same multi-obj engine files |
| `wrong arity on InitializationFn` | 9 | initializer call sites in engine files |
| `wrong argument count (7 vs 6)` | 9 | crossover/mutation calls in engine files |
| `no field 'needs_unique_ids'` | 2 | `src/engines/ga.rs` |
| `wrong arity (3 vs 2)` | 2 | initializer call sites |

Additionally, the following test files will fail to compile until 47-06 migrates them from `with_stopping_criteria(StoppingCriteria{...})` to flat builder methods:
- `tests/engines/test_ga.rs` lines 772, 812, 854
- `tests/observe/test_serde.rs` lines 170, 202
- `tests/wasm_smoke.rs` line 38

None of these errors are attributable to plan 47-05 changes.

## Deviations from Plan

None — plan executed exactly as written.

The one clarification: PATTERNS.md showed `#[cfg(not(target_arch = "wasm32"))]` on `with_max_duration_secs` in the trait, but the plan's `<interfaces>` section and RESEARCH.md Pitfall 3 explicitly state NOT to gate the builder method. The plan's specification was followed: the field and builder are un-gated; only the ga.rs call site is gated.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced by this plan.

T-47-13 (WASM regression): Verified mitigated — `#[cfg(not(target_arch = "wasm32"))]` gate preserved around `max_duration_secs` usage at line 2106 of `ga.rs`.

## Known Stubs

None.

## Self-Check: PASSED

- `tests/test_stopping_config.rs` exists: FOUND
- `src/configuration.rs` has no `StoppingCriteria` struct: CONFIRMED (grep returns 0)
- `src/configuration.rs` has 3 flat stopping fields: CONFIRMED
- `src/traits/configuration.rs` has 3 flat builder methods: CONFIRMED
- `src/engines/ga.rs` has 0 occurrences of `stopping_criteria`: CONFIRMED
- Commits a37770c, 643f7b9, b745315: FOUND in git log
