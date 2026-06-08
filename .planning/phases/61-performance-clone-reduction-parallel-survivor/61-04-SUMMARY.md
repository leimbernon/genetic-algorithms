---
phase: 61-performance-clone-reduction-parallel-survivor
plan: "04"
subsystem: verification
tags: [benchmark, ci, phase-gate, verification]
dependency_graph:
  requires:
    - 61-01 (rastrigin benchmark harness)
    - 61-02 (parallel survivor sort)
    - 61-03 (observer &U signature change)
  provides:
    - 61-BENCH-RESULTS.md with baseline vs. post-change numbers
    - Full CI matrix verification for Phase 61
  affects:
    - .planning/phases/61-performance-clone-reduction-parallel-survivor/61-BENCH-RESULTS.md
    - src/engines/permutate/engine.rs (clippy fix)
tech_stack:
  added: []
  patterns:
    - "pre-phase worktree baseline measurement: git worktree add at pre-phase commit + bench copy"
    - "criterion median as primary benchmark metric"
key_files:
  created:
    - .planning/phases/61-performance-clone-reduction-parallel-survivor/61-BENCH-RESULTS.md
  modified:
    - src/engines/permutate/engine.rs
decisions:
  - "Baseline measured via pre-phase worktree (e3b0728): rastrigin.rs + Cargo.toml [[bench]] entry copied from current tree"
  - "ROADMAP ≥10% gate NOT MET: max improvement is 2.11% at dim=20; dim=50 regressed −0.38%"
  - "Root cause: par_sort overhead at pop=500 equals or exceeds sort savings; gains expected at pop=5000+"
  - "clippy --all-features surfaces pre-existing errors in metrics_observer bench + visualization test (not Phase 61 regressions)"
  - "Rule 1 auto-fix: permutate/engine.rs &candidate needless borrow fixed (missed by Plan 03)"
metrics:
  duration_minutes: 45
  completed_date: "2026-06-08"
  tasks_completed: 2
  tasks_total: 4
  files_modified: 2
  status: PAUSED_AT_CHECKPOINT_TASK_3
---

# Phase 61 Plan 04: Benchmark & CI Verification Summary (Tasks 1–2 Complete; Paused at Task 3)

Phase 61 verification gate: rastrigin wall-time measured before/after Plans 02/03. ROADMAP ≥10% gate NOT MET — actual improvement ≤2.11% at pop=500. Full CI matrix green. Paused at Task 3 human-verify checkpoint.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Capture baseline and post-change rastrigin bench numbers | c3a0c45 | 61-BENCH-RESULTS.md (created, 78 lines) |
| 2 | Run full CI matrix verification + auto-fix clippy needless borrow | 852628a | src/engines/permutate/engine.rs (1 line) |

## Paused At

Task 3 (checkpoint:human-verify) — human decision required on ROADMAP gate NOT MET result.

## What Was Built

### Task 1: Benchmark Measurement

Ran `cargo bench --bench rastrigin` at two tree states:

**Baseline** (pre-phase worktree at e3b0728 — Phase 60 tip, before any Phase 61 source changes):
- dim=10: 1.5586 ms
- dim=20: 1.6334 ms
- dim=50: 1.8204 ms

**Post-change** (worktree HEAD with Plans 01+02+03 applied):
- dim=10: 1.5375 ms (+1.35%)
- dim=20: 1.5990 ms (+2.11%)
- dim=50: 1.8274 ms (−0.38%)

**Headline: ROADMAP success criterion #1 is NOT MET.** The ≥10% wall-time reduction was not achieved at pop=500. Maximum improvement is 2.11% at dim=20.

Analysis: At pop=500 with max_generations=50, parallel sort (par_sort_unstable_by) introduces rayon thread-pool coordination overhead that equals or exceeds the sort savings. The observer &U change eliminates one clone per new-best event, but this is undetectable at 50 generations. Population sizes of 5000+ would be required for parallel sort to show net gains.

### Task 2: Full CI Matrix Verification

All gates green:

| Check | Command | Result |
|-------|---------|--------|
| cargo test | `cargo test` | 1176 passed, 0 failed |
| cargo test serde | `cargo test --features serde` | 1216 passed, 0 failed |
| cargo clippy | `cargo clippy --all-targets -- -D warnings` | No issues |
| cargo wasm32 | `cargo check --target wasm32-unknown-unknown` | PASS |
| cargo doc | `cargo doc --no-deps` | 0 rustdoc warnings |

Structural greps (all match expected counts):

| File | Grep | Expected | Actual |
|------|------|----------|--------|
| survivor/fitness.rs | par_sort_unstable_by | 2 | 2 |
| survivor/mu_plus_lambda.rs | par_sort_unstable_by | 2 | 2 |
| survivor/age.rs | par_sort_unstable_by | 1 | 1 |
| survivor/mu_comma_lambda.rs | par_sort_unstable_by | 2 | 2 |
| survivor/deterministic_crowding.rs | par_sort_unstable_by | 0 | 0 |
| observe/observer/mod.rs | fn on_new_best(&self, _: &U) | 1 | 1 |
| engines/ga.rs | on_new_best(i, &self.population.best_chromosome) | 1 | 1 |
| observe/observer/composite.rs | best.clone() | 0 | 0 |

Note: `cargo clippy --all-targets --all-features -- -D warnings` surfaces pre-existing compilation errors in `benches/metrics_observer.rs` (removed `with_genes_per_chromosome` API) and `tests/observe/visualization/test_visualization.rs` (missing `cache_hits`/`cache_misses` fields). These are out-of-scope pre-existing issues from prior phases — not introduced by Phase 61. They are tracked in deferred-items.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy needless borrow in permutate/engine.rs**
- **Found during:** Task 2 clippy run
- **Issue:** `&candidate` at line 102 creates `&&U` since `candidate` is already `&U` from `self.candidates.iter()`. This was supposed to be fixed in Plan 03 commit a986fb9, but the fix was not applied to this worktree's copy.
- **Fix:** Changed `&candidate` to `candidate` in `self.notify(|obs| obs.on_new_best(idx, candidate))`
- **Files modified:** src/engines/permutate/engine.rs
- **Commit:** 852628a

## Known Stubs

None — benchmark and verification only.

## Threat Flags

None — verification-only tasks, no new network endpoints or auth paths.

## Deferred Issues

Pre-existing compilation errors (out-of-scope per deviation scope boundary):
1. `benches/metrics_observer.rs:37` — `with_genes_per_chromosome` method no longer exists in `GaConfiguration`
2. `tests/observe/visualization/test_visualization.rs:15` — `GenerationStats` struct is missing `cache_hits` and `cache_misses` fields

These only surface with `--all-features`. They are from prior phase work and are not Phase 61 regressions.

## Self-Check: PASSED

- 61-BENCH-RESULTS.md exists: FOUND
- Commit c3a0c45 exists: FOUND
- Commit 852628a exists: FOUND
- All structural greps: VERIFIED
- All CI gates: VERIFIED
