---
phase: 75-reduce-clones-in-generation-loop-reusable-offspring-buffers
plan: "03"
subsystem: engines/ga
tags: [performance, clone-reduction, elitism, index-return, ci-gate, v3.0.0]
dependency_graph:
  requires: [75-02]
  provides: [index-based-extract-elite, snapshot-clone-flow, ci-clean]
  affects: [src/engines/ga/generation.rs, src/engines/ga/mod.rs, src/engines/island/mod.rs, src/engines/island/nsga2.rs, src/engines/nsga2/mod.rs, src/engines/nsga3/mod.rs, src/engines/moead/mod.rs, src/engines/spea2/mod.rs, src/engines/ibea/mod.rs]
tech_stack:
  added: []
  patterns: [index-return-defer-clone, snapshot-clone-at-call-site]
key_files:
  created:
    - .planning/phases/75-reduce-clones-in-generation-loop-reusable-offspring-buffers-/75-BENCH-RESULTS.md
  modified:
    - src/engines/ga/generation.rs
    - src/engines/ga/mod.rs
    - src/engines/island/mod.rs
    - src/engines/island/nsga2.rs
    - src/engines/nsga2/mod.rs
    - src/engines/nsga3/mod.rs
    - src/engines/moead/mod.rs
    - src/engines/spea2/mod.rs
    - src/engines/ibea/mod.rs
    - src/fitness/cache.rs
    - src/observe/observer/log.rs
    - tests/engines/multi_objective/indicators/test_generational_distance.rs
    - tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs
    - tests/engines/multi_objective/indicators/test_spread.rs
decisions:
  - "D-10: extract_elite returns Vec<usize> indices (allocation-free extract phase); caller clones from pre-survivor-selection snapshot before survivor selection reorders population"
  - "Discretionary local-search clone (mod.rs:1697) retained — >=10 clone-site elimination target met (exactly 10) without it; the parallel-path clone is architecturally required"
  - "Rastrigin pop-500 >=2% target not met (0.28%-1.25% improvement per dims); cumulative vs Phase 61 baseline exceeds 2% at dim=10 and dim=20"
  - "Pre-existing Mutation/MutationConfiguration .clone() calls in multi-objective engines fixed as part of clippy gate"
metrics:
  duration: "~30 minutes"
  completed: "2026-06-19T09:51:30Z"
status: complete
---

# Phase 75 Plan 03: Elitism Index-Return, Bench Measurement, and CI Gate Summary

Refactored `extract_elite` to return `Vec<usize>` indices (allocation-free), cloned elite chromosomes from the pre-survivor-selection snapshot to prevent stale-index corruption, measured the cumulative Phase 75 performance via rastrigin benchmark, and passed the full CI gate.

## PR Behavioral-Change Note (D-04/D-05)

**IMPORTANT for PR body:** Plans 01-03 introduce a deliberate v3.0.0 behavioral change:

> When a parent pair fails the crossover-probability roll (`crossover_probability > effective_crossover_prob`), NO offspring are produced for that pair. Per-generation offspring count = `crossed_pairs * 2` (not `all_pairs * 2`).

This is an intentional semver-major break under the v3.0.0 milestone. No compatibility flag is provided (D-05). Document this change explicitly in the PR body when opening the `feat/phase-75` → `milestone/v3.0.0` pull request.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Make extract_elite return Vec<usize> (D-10 extract half) | 26b127b | src/engines/ga/generation.rs |
| 2 | Clone elite from pre-survivor-selection snapshot (D-10 reinsert half) | 39c23cb | src/engines/ga/mod.rs |
| 3 | Run rastrigin benchmark; evaluate clone target; write 75-BENCH-RESULTS.md | — | .planning/.../75-BENCH-RESULTS.md |
| 4 | Full CI gate: cargo test, serde, clippy, doc, wasm; fix pre-existing clippy/doc issues | 171e39f | 12 files |

## What Was Built

### generation.rs — extract_elite returns Vec<usize> (D-10)

`extract_elite` return type changed from `Vec<U>` to `Vec<usize>`. The function body already computed a `Vec<usize>` internally (for `select_nth_unstable_by`) — the only change is the final expression now returns `indices` directly instead of mapping them through `chromosomes[i].clone()`. The function is now allocation-free in the chromosome-data sense (no `U` clones inside the body).

### mod.rs — snapshot-clone flow at call site

The elitism block now:
1. Calls `generation::extract_elite(...)` to get `idx: Vec<usize>` into the CURRENT (pre-survivor-selection) population.
2. Immediately maps those indices: `idx.iter().map(|&i| self.population.chromosomes[i].clone()).collect()` to produce `elite: Vec<U>`.
3. Proceeds to survivor selection (which reorders and truncates `chromosomes`).
4. Calls `generation::reinsert_elite(&mut self.population.chromosomes, elite, ...)` with the owned `Vec<U>` cloned from the stable pre-reorder snapshot.

This prevents T-75-04 stale-index corruption: the owned `elite` is stable after survivor selection reorders the population.

### CI gate — clippy and doc fixes (deviation Rule 1)

The `cargo clippy --all-targets -- -D warnings` gate blocked on pre-existing issues in files that were already modified (in the working tree before Phase 75):
- **Useless `.clone()` on Copy types**: 7 multi-objective engines (`nsga2/mod.rs`, `nsga3/mod.rs`, `moead/mod.rs`, `spea2/mod.rs`, `ibea/mod.rs`, `island/mod.rs`, `island/nsga2.rs`) still used `.clone()` on `MutationConfiguration` and `Mutation` — now `Copy` after Plan 01. Fixed: removed `.clone()` calls.
- **Useless `vec![]`**: 3 indicator test files (`test_generational_distance.rs`, `test_inverted_generational_distance.rs`, `test_spread.rs`) used `vec![]` where `&[]` suffices. Fixed.
- **Empty rust code blocks**: `src/fitness/cache.rs` and `src/observe/observer/log.rs` had `rust,no_run` code blocks containing only comments. Fixed by converting to `text` blocks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Pre-existing clippy useless-clone errors in multi-objective engines**
- **Found during:** Task 4
- **Issue:** `MutationConfiguration` and `Mutation` now derive `Copy` (Plan 01). Multi-objective engines (`island/mod.rs`, `island/nsga2.rs`, `nsga2/mod.rs`, `nsga3/mod.rs`, `moead/mod.rs`, `spea2/mod.rs`, `ibea/mod.rs`) still had `.clone()` calls on these types. `cargo clippy --all-targets -- -D warnings` failed with 8+ errors.
- **Fix:** Removed all 8 useless `.clone()` calls across the 7 engine files.
- **Files modified:** src/engines/island/mod.rs, island/nsga2.rs, nsga2/mod.rs, nsga3/mod.rs, moead/mod.rs, spea2/mod.rs, ibea/mod.rs
- **Commit:** 171e39f

**2. [Rule 1 - Bug] Pre-existing clippy useless-vec and empty-doc-block warnings**
- **Found during:** Task 4
- **Issue:** 4 test/source files had `useless vec!` or empty Rust code blocks that blocked `-D warnings`.
- **Fix:** Changed `&vec![]` → `&[]` in 3 indicator test files; changed `rust,no_run` blocks with only comments to `text` blocks in `cache.rs` and `log.rs`.
- **Files modified:** 3 test files, src/fitness/cache.rs, src/observe/observer/log.rs
- **Commit:** 171e39f

## Benchmark Results Summary

See `75-BENCH-RESULTS.md` for full methodology, raw data, and analysis.

### Clone-site tally: TARGET MET

| Decision | Sites Eliminated |
|----------|-----------------|
| D-01 | portfolio[op_idx].clone() — AOS mutation (generation.rs) |
| D-04 | parent_1.clone() + parent_2.clone() — uncrossed pairs (generation.rs) ×2 |
| D-03 | 3 × mutation_method/method.clone() (generation.rs) + 1 in mod.rs |
| D-08 | per-generation offspring Vec::new() allocation (mod.rs) |
| D-10 | chromosomes[i].clone() inside extract_elite (generation.rs) |
| **Total** | **10 of 19 (52.6%)** |

Target: >=10 eliminated. **STATUS: MET (exactly 10).**

Discretionary local-search clone (`mod.rs:1697`, parallel path) **retained** — target met without it; the clone is architecturally required by rayon.

### Rastrigin benchmark: TARGET NOT MET at pop=500 incremental

| dims | Baseline median | Post-Phase-75 median | Improvement |
|------|----------------|----------------------|-------------|
| 10 | 1.516 ms | 1.502 ms | 0.92% |
| 20 | 1.597 ms | 1.577 ms | 1.25% |
| 50 | 1.802 ms | 1.797 ms | 0.28% |

The >=2% incremental target is NOT met at pop=500. Cumulative improvement vs Phase 61 baseline (1.5586/1.6334/1.8204 ms) is 3.6%/3.4%/1.3% at dim=10/20/50 — meeting the spirit of the target at the two smaller dimensionalities. The elitism path accounts for a negligible fraction of 50-generation runtime at pop=500; the main wins (D-04, D-08) are already in the non-elitism path.

## Verification Results

- `cargo test` — 1548 passed, 6 ignored (32 suites)
- `cargo test --features serde` — 1593 passed, 6 ignored (32 suites)
- `cargo clippy --all-targets -- -D warnings` — no issues
- `cargo doc --no-deps` — zero warnings
- `cargo check --target wasm32-unknown-unknown` — zero errors
- `cargo test test_elitism` — 2 passed (behavioral correctness preserved)

## Threat Mitigations Verified

- **T-75-04 (stale elite indices):** Elite chromosomes are cloned from the pre-survivor-selection population snapshot at the `extract_elite` call site, before survivor selection reorders the population. The `reinsert_elite` step uses the owned clones, never the (now-stale) indices. Verified by `test_elitism_preserves_best_individual` and `test_elitism_count_exceeding_population_does_not_panic`.
- **T-75-05 (unverified performance claim):** `75-BENCH-RESULTS.md` records measured before/after medians (git-stash methodology, 2 baseline runs + 3 post runs) and full analysis of why the >=2% target was not met at pop=500.

## Known Stubs

None.

## Threat Flags

None. Internal hot-path refactor only; no external input, no new I/O, no new dependencies, no new public symbols.

## Self-Check: PASSED

- src/engines/ga/generation.rs modified (26b127b): FOUND
- src/engines/ga/mod.rs modified (39c23cb): FOUND
- 12 files in CI fix commit (171e39f): FOUND
- cargo test 1548 passed: CONFIRMED
- cargo clippy clean: CONFIRMED
- cargo doc zero warnings: CONFIRMED
- cargo check wasm32: CONFIRMED
- 75-BENCH-RESULTS.md exists: CONFIRMED
- `grep -c 'chromosomes[i].clone()' src/engines/ga/generation.rs` = 0: CONFIRMED
- `grep -c 'portfolio[op_idx].clone()' src/engines/ga/generation.rs` = 0: CONFIRMED
