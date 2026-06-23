---
phase: 75-reduce-clones-in-generation-loop-reusable-offspring-buffers
plan: "02"
subsystem: engines/ga
tags: [performance, clone-reduction, output-buffer, behavioral-change, v3.0.0]
dependency_graph:
  requires: [75-01]
  provides: [offspring-buffer-reuse, no-crossover-skip, parent2-fallback, mutation-copy-callsites]
  affects: [src/engines/ga/generation.rs, src/engines/ga/mod.rs]
tech_stack:
  added: []
  patterns: [output-buffer-api, copy-instead-of-clone, early-return-empty]
key_files:
  created: []
  modified:
    - src/engines/ga/generation.rs
    - src/engines/ga/mod.rs
decisions:
  - "D-03: Replace all 4 Mutation.clone() call sites (3 in generation.rs, 1 in mod.rs) with Copy — zero-cost, no behavior change"
  - "D-04/D-05: Uncrossed pairs return Ok(Vec::new()) — offspring count = crossed_pairs * 2 (intentional v3.0.0 behavioral change)"
  - "D-06: 1-child fallback uses parent_2.clone() instead of parent_1.clone() — avoids asymmetry where both children equal parent_1"
  - "D-07: parent_crossover signature adds out: &mut Vec<U> and returns Result<(), GaError> — pub(crate) only, no public API break"
  - "D-08: offspring_buf allocated once with Vec::with_capacity(population_size * 2) before the generation loop — reused every generation"
  - "D-09: parent_crossover calls out.clear() at entry — stale contents from prior generation are always erased"
metrics:
  duration: "~7 minutes"
  completed: "2026-06-19T09:34:39Z"
status: complete
---

# Phase 75 Plan 02: Output-Buffer Offspring API and Clone Elimination Summary

Convert `parent_crossover()` to a caller-owned output-buffer API, remove parent-cloning on uncrossed pairs, fix the 1-child fallback to use `parent_2`, and delete four `Mutation` clones — eliminating the largest source of per-generation heap allocations in the GA hot path.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | No-crossover skip + parent_2 1-child fallback + remove Mutation clones in generation.rs (D-03, D-04, D-06) | 36b15cf | src/engines/ga/generation.rs |
| 2 | Convert parent_crossover to &mut Vec<U> output buffer (D-07, D-09) | b6c689a | src/engines/ga/generation.rs |
| 3 | Allocate offspring_buf once before loop; remove mod.rs Mutation clone (D-03, D-08) | f2f0384 | src/engines/ga/mod.rs |

## What Was Built

### generation.rs changes

- **No-crossover skip (D-04/D-05):** the `else` branch that previously cloned both parents now returns `Ok(Vec::new())` immediately. Per-generation offspring count = (crossed_pairs * 2) rather than (all_pairs * 2). This is a deliberate v3.0.0 behavioral change — documented in the PR body.
- **1-child fallback (D-06):** `children.pop().unwrap_or_else(|| parent_2.clone())` instead of `parent_1.clone()`. Avoids the asymmetry in multi-parent crossover paths where both children would be copies of parent_1.
- **Mutation clones removed (D-03):** three call sites replaced:
  - `configuration.mutation_configuration.method.clone()` → `.unwrap_or(...)` (Copy)
  - `mutation::factory_with_chromosome_length(mutation_method.clone(), ...)` × 2 → pass Copy value directly
  - `portfolio[op_idx].clone()` in AOS mutation selection → Copy indexing
- **Output-buffer signature (D-07):** `parent_crossover` now takes `out: &mut Vec<U>` as a fifth parameter and returns `Result<(), GaError>`.
- **Buffer clear (D-09):** `out.clear()` at function entry ensures no stale offspring from a prior generation survive into the current one.

### mod.rs changes

- **Buffer allocation (D-08):** `let mut offspring_buf: Vec<U> = Vec::with_capacity(self.configuration.limit_configuration.population_size * 2)` placed before the `for i in start_gen..total_gens` loop — a single heap allocation for the lifetime of the run.
- **Call-site update:** `generation::parent_crossover(..., &mut offspring_buf)?` with no `let mut offspring =` binding; all 26 downstream references to `offspring` renamed to `offspring_buf` (surrogate prescreening, batch eval, repair operator, constraint penalty, local search, `add_chromosomes`).
- **mod.rs Mutation clone removed (D-03):** `let builder_mutation = self.configuration.mutation_configuration.method;` (Copy, no clone) in the checkpoint hybrid-config block.

### Threat mitigations verified

- **T-75-02 (offspring_buf reuse):** `out.clear()` at entry + `add_chromosomes(&mut buf)` drains buffer → no stale offspring leak between generations. Confirmed by 1550-test suite (multi-generation runs).
- **T-75-03 (smaller offspring pool):** intentional v3.0.0 semantic. Survivor selection and elitism handle smaller pools without panic — covered by existing engine tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Refactor] Restructured if/else into early-return for no-crossover skip**
- **Found during:** Task 1
- **Issue:** The plan described removing the `else` branch body (parent clones) and replacing it with `return Ok(Vec::new())`, but the original code declared `let mut child_1: U; let mut child_2: U;` before the `if/else`, which would leave them uninitialized when the `else` returns early. Rust would reject this as "possibly uninitialized variable".
- **Fix:** Inverted the condition to an early-return `if crossover_probability > effective_crossover_prob { return Ok(Vec::new()); }` and moved `let mut child_1` / `let mut child_2` declarations inside the crossed branch, keeping them as `let mut child_1 = children.pop()...` and `let mut child_2 = children.pop()...` (inlined from the removed outer declarations).
- **Files modified:** src/engines/ga/generation.rs
- **Commit:** 36b15cf

**2. [Rule 2 - Missing clone removal] portfolio[op_idx].clone() in AOS mutation selection**
- **Found during:** Task 1
- **Issue:** The plan listed 3 Mutation clones in generation.rs to remove, but the AOS path also had `portfolio[op_idx].clone()` (indexing a `&Vec<Mutation>`) which was not explicitly listed. Since `Mutation: Copy`, this is also an unnecessary clone.
- **Fix:** Changed to `portfolio[op_idx]` (Copy by index). This is consistent with D-03's intent.
- **Files modified:** src/engines/ga/generation.rs
- **Commit:** 36b15cf

## Verification Results

- `cargo build` — zero errors
- `cargo clippy --all-targets` — zero errors
- `cargo test` — 1550 passed, 6 ignored
- `cargo test --features serde` — 1595 passed, 6 ignored
- `grep -c 'parent_1\.clone()' src/engines/ga/generation.rs` = 0
- `grep -c 'mutation_method\.clone()' src/engines/ga/generation.rs` = 0
- `grep -c 'mutation_configuration\.method\.clone()' src/engines/ga/mod.rs` = 0
- `grep -c 'out: &mut Vec<U>' src/engines/ga/generation.rs` = 1
- `grep -c 'out\.clear()' src/engines/ga/generation.rs` = 1
- `grep -c 'offspring_buf' src/engines/ga/mod.rs` = 26

## Known Stubs

None.

## Threat Flags

None. Internal hot-path refactor only; no external input, no new I/O, no new dependencies, no new public symbols.

## Self-Check: PASSED

- src/engines/ga/generation.rs modified: FOUND (36b15cf, b6c689a)
- src/engines/ga/mod.rs modified: FOUND (f2f0384)
- All 3 commits exist in git log: CONFIRMED
- cargo test 1550 passed: CONFIRMED
- Zero Mutation clones remaining: CONFIRMED
