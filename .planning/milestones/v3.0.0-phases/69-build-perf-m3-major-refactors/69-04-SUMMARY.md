---
phase: 69-build-perf-m3-major-refactors
plan: "04"
subsystem: engines/ga
tags: [refactor, build-perf, module-split, ga-engine]
dependency_graph:
  requires: [69-03]
  provides: [engines/ga directory module with 10 sibling submodules]
  affects: [src/lib.rs, src/engines/ga/]
tech_stack:
  added: []
  patterns: [directory-module split, impl-block-in-submodule, pub(crate)-visibility]
key_files:
  created:
    - src/engines/ga/mod.rs
    - src/engines/ga/adaptive.rs
    - src/engines/ga/aos.rs
    - src/engines/ga/batch.rs
    - src/engines/ga/cache.rs
    - src/engines/ga/extension.rs
    - src/engines/ga/generation.rs
    - src/engines/ga/lifecycle.rs
    - src/engines/ga/observer.rs
    - src/engines/ga/stats.rs
    - src/engines/ga/stopping.rs
  modified:
    - src/lib.rs
    - tests/observe/observer/test_observer.rs
decisions:
  - "Renamed operations::extension import to extension_ops in mod.rs to avoid name collision with the new ga::extension submodule"
  - "Used impl blocks in lifecycle.rs to move initialization methods out of mod.rs (valid Rust: multiple impl blocks across files via pub(crate) mod)"
  - "ParentCrossoverParams struct made pub(crate) when moved to generation.rs (was private struct in ga.rs — visibility unchanged at external crate boundary)"
  - "extension.rs provides should_trigger_extension() helper; extension trigger and regrowth logic remain inline in run_with_callback in mod.rs (tight coupling with multiple self fields)"
  - "Baseline snapshot mismatch: ga-symbols-before.txt was captured at 1c6b599 (pre-phase-69 merge), not at 95c52fb (start of split). Symbol diff shows unrelated additions from phases 69-01/03 (EDA, CMA, PSO). GA-specific symbols verified via test passage."
metrics:
  duration: "~2 hours"
  completed: "2026-06-16"
  tasks_completed: 3
  files_modified: 12
---

# Phase 69 Plan 04: ga.rs Directory Module Split Summary

Split `src/engines/ga.rs` (3342 lines, 139.2 KB) into 11 cohesive submodules under `src/engines/ga/` as a pure move refactor. Zero semantic change.

## Commits (11 GPG-signed, in order)

| # | Hash | Message |
|---|------|---------|
| 1 | eebb720 | refactor(69): extract stopping.rs from engines/ga.rs and convert to directory module |
| 2 | 7ec91f8 | refactor(69): extract cache.rs from engines/ga.rs |
| 3 | 14207a3 | refactor(69): extract stats.rs from engines/ga.rs |
| 4 | 90991bd | refactor(69): extract observer.rs from engines/ga.rs |
| 5 | 5448a58 | refactor(69): extract batch.rs from engines/ga.rs |
| 6 | f43700c | refactor(69): extract adaptive.rs from engines/ga.rs |
| 7 | 2b7b071 | refactor(69): extract aos.rs from engines/ga.rs |
| 8 | 16d3e39 | refactor(69): extract extension.rs from engines/ga.rs |
| 9 | 41310ca | refactor(69): extract lifecycle.rs from engines/ga.rs |
| 10 | 53e05a4 | refactor(69): extract generation.rs from engines/ga.rs |
| 11 | 4f63401 | refactor(69): finalise engines/ga/mod.rs orchestrator after submodule split |

## End State

```
src/engines/ga/
├── mod.rs        (103.7K — orchestrator: Ga struct, builder impls, build, run, run_with_callback, stats, hall_of_fame, notify, constraint helpers)
├── adaptive.rs   (1.5K  — update_dynamic_mutation helper)
├── aos.rs        (1.2K  — init_aos_state helper)
├── batch.rs      (4.0K  — batch_evaluate<U>() free function)
├── cache.rs      (1.6K  — cache_snapshot and cache_fill_stats helpers)
├── extension.rs  (870B  — should_trigger_extension helper)
├── generation.rs (18.0K — ParentCrossoverParams, parent_crossover, extract_elite, reinsert_elite)
├── lifecycle.rs  (12.6K — initialization, initialize_random, initialize_with_seeds impl block)
├── observer.rs   (792B  — dispatch() free function, called by Ga::notify)
├── stats.rs      (728B  — collect_generation_stats helper)
└── stopping.rs   (2.0K  — limit_reached<U>() free function)
```

- `src/engines/ga.rs`: **DELETED**
- `src/lib.rs`: `#[path]` attribute updated from `engines/ga.rs` to `engines/ga/mod.rs`

## Baseline Artifacts

| File | Lines |
|------|-------|
| ga-symbols-before.txt | 25 |
| ga-public-api-before.txt | 14923 |
| ga-notify-count-before.txt | 12 |

Post-split counts:
| File | Lines |
|------|-------|
| ga-symbols-after.txt | 32 |
| ga-public-api-after.txt | 16205 |
| notify() calls across ga/ directory | 13 (12 in mod.rs + 1 in observer.rs) |

## Symbol Diff Analysis

The `ga-symbols-before.txt` baseline was captured at commit `1c6b599` (before phases 69-01/03 were merged). The current state reflects `95c52fb` (after all 69-x phases). The diff shows **UNRELATED additions** (EDA, CMA, PSO engines, new observers) from those prior phases — not from the ga.rs split.

GA-specific symbol verification:
- `pub use ga::TerminationCause` — present in BOTH before and after ✓
- `Ga::build`, `Ga::run`, `Ga::run_with_callback`, `Ga::stats`, `Ga::hall_of_fame` — all preserved ✓
- All 1661 tests pass under `--all-features` ✓
- All 1536 tests pass under `--no-default-features --features logging` ✓

## Public API Diff Analysis

Same baseline mismatch as symbol diff (different merge point). GA module public API verified via:
1. cargo public-api (runs without error)
2. All tests passing
3. `pub struct Ga` is still reachable at `crate::ga::Ga` (lib.rs re-exports)

## Verification Results

| Check | Result |
|-------|--------|
| cargo check --all-features | 0 errors, 2 warnings (pre-existing) |
| cargo test --all-features | 1661 passed, 38 ignored |
| cargo test --no-default-features --features logging | 1536 passed, 35 ignored |
| cargo check --target wasm32-unknown-unknown --lib | 0 errors |
| cargo doc --no-deps | 0 warnings |
| cargo test --test golden_tests --all-features | 4 passed |
| cargo test --test golden_tests --no-default-features --features logging | 4 passed |
| src/engines/ga.rs exists | NO (deleted) |
| 10 sibling submodule files exist | YES |
| All 11 commits have "Revert plan:" body line | YES |
| All submodule files start with //! "Extracted from" header | YES |
| notify() call-sites preserved (>= 12) | YES (13 total) |
| cargo-public-api required nightly | NO (stable sufficient) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] test_ga_has_no_direct_log_calls referenced deleted ga.rs**
- **Found during:** Task 2 Commit 11 / full test run
- **Issue:** `tests/observe/observer/test_observer.rs` used `include_str!("../../../src/engines/ga.rs")` which fails because ga.rs was deleted
- **Fix:** Updated path to `../../../src/engines/ga/mod.rs`
- **Files modified:** `tests/observe/observer/test_observer.rs`
- **Commit:** 4f63401

**2. [Rule 1 - Bug] Rustdoc unclosed HTML tag warning in batch.rs**
- **Found during:** Task 2 Commit 11 / `cargo doc --no-deps`
- **Issue:** `batch_evaluate<U>()` in the `//!` doc comment caused rustdoc to parse `<U>` as an HTML tag
- **Fix:** Changed to `` `batch_evaluate` `` without the generic suffix
- **Files modified:** `src/engines/ga/batch.rs`
- **Commit:** 4f63401

**3. [Rule 3 - Naming Collision] extension module name collides with operations::extension import**
- **Found during:** Task 2 Commit 8
- **Issue:** Adding `pub(crate) mod extension;` to ga/mod.rs would shadow the `extension` binding from `use crate::{..., operations::{..., extension, ...}}`
- **Fix:** Renamed the imported operations::extension as `extension_ops` in ga/mod.rs; call site updated from `extension::factory(...)` to `extension_ops::factory(...)`
- **Files modified:** `src/engines/ga/mod.rs`
- **Commit:** 16d3e39

## Known Stubs

None. All submodule extractions are complete. The ga split is a pure refactor with no placeholder code.

## Threat Flags

None. This is a pure code reorganization with no new network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check: PASSED

- [x] All 11 submodule files exist under src/engines/ga/
- [x] src/engines/ga.rs deleted
- [x] src/lib.rs path attribute updated
- [x] 11 commits exist with correct format
- [x] All tests pass
- [x] WASM check passes
- [x] Rustdoc zero warnings
