---
phase: 24-minor-improvements
plan: 01
subsystem: ga-core
tags: [performance, clone-elimination, selection, stats]
dependency_graph:
  requires: []
  provides: [push-last-stats, best-scan-deduplication, o-n-truncation]
  affects: [src/ga.rs, src/operations/selection/truncation.rs]
tech_stack:
  added: []
  patterns: [select_nth_unstable_by, fold-based-argmax, push-last-ownership]
key_files:
  created: []
  modified:
    - src/ga.rs
    - src/operations/selection/truncation.rs
key_decisions:
  - GenerationStats moved (not cloned) into self.stats via push-last ownership
  - dynamic_mutation_probability set directly on gen_stats before push (no last_mut)
  - best_chromosome_index function removed as dead code after fitness_values scan replaces it
  - notify_stats still requires one clone due to &self borrow conflict with self.notify()
  - truncation_size computed before indexed Vec so select_nth_unstable_by index is available
metrics:
  duration: 41m
  completed: "2026-04-05"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
  commits: 2
---

# Phase 24 Plan 01: Push-last stats, best-scan dedup, O(n) truncation Summary

One-liner: GenerationStats moved via push-last ownership, best-chromosome scan deduped to fitness_values fold, truncation selection replaced sort_by with select_nth_unstable_by.

## What Was Built

Three micro-optimizations targeting the hot generation loop and truncation selection:

1. **MISC-01 — Push-last stats ownership**: `GenerationStats` is now moved (not cloned) into `self.stats`. The `dynamic_mutation_probability` field is set directly on `gen_stats` before push, eliminating the prior `self.stats.last_mut()` pattern. All post-push references use `self.stats.last().unwrap()`.

2. **MISC-03 — Best-scan deduplication**: The `best_chromosome_index()` helper function (which traversed `chromosomes` again) was replaced with an inline `fold` over `fitness_values` (already computed earlier in the loop). The helper function was then removed as dead code.

3. **MISC-02 — O(n) truncation partitioning**: `indexed.sort_by(...)` (O(n log n)) in truncation selection replaced with `indexed.select_nth_unstable_by(truncation_size - 1, ...)` (O(n)). Elite trace log updated to drop the rank number (`Elite member -> index X fitness Y`).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Push-last stats pattern and best-scan deduplication | a03ab12 | src/ga.rs |
| 2 | O(n) truncation selection partitioning | ab64279 | src/operations/selection/truncation.rs, src/ga.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed dead best_chromosome_index function**
- **Found during:** Task 2 (clippy check)
- **Issue:** After Task 1 replaced the generation loop call, `best_chromosome_index` became unreachable dead code, causing a clippy warning
- **Fix:** Removed the function entirely (24 lines)
- **Files modified:** src/ga.rs
- **Commit:** ab64279

### Notes

- `notify_stats` at line 1045 still uses `.clone()` — this is intentional. `self.notify()` takes `&self`, so a shared borrow into `self.stats` cannot coexist with the `&self` receiver. The plan acknowledged this borrow checker constraint.
- `truncation_size` was computed before the `indexed` Vec build (reordered from original code) so the value is available for `select_nth_unstable_by`.

## Verification

- `cargo test`: 22 passed, 0 failed
- `cargo test --features serde`: 22 passed, 0 failed
- `cargo clippy`: 1 pre-existing warning (too many arguments), 0 new warnings
- All truncation tests pass including `test_truncation_selection_selects_only_from_top_half`

## Self-Check: PASSED

Files exist:
- src/ga.rs — FOUND
- src/operations/selection/truncation.rs — FOUND

Commits exist:
- a03ab12 — FOUND
- ab64279 — FOUND
