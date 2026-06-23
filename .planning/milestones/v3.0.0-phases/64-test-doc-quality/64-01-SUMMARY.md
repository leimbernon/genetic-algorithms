---
phase: 64-test-doc-quality
plan: "01"
subsystem: ci/coverage
tags: [coverage, ci, llvm-cov, baseline]
dependency_graph:
  requires: []
  provides: [64-COVERAGE-BASELINE.md, coverage.yml]
  affects: [.github/workflows/]
tech_stack:
  added: [cargo-llvm-cov 0.8.7]
  patterns: [negative-lookahead regex for --ignore-filename-regex]
key_files:
  created:
    - .planning/phases/64-test-doc-quality/64-COVERAGE-BASELINE.md
    - .github/workflows/coverage.yml
  modified: []
decisions:
  - "Negative lookahead regex `^(?!.*(src/engines/|src/operations/)).*$` used with --ignore-filename-regex to scope gate to exactly two subtrees without enumerating all excluded dirs"
  - "Coverage workflow is a separate job from rust-unit-tests.yml to avoid slowing PR feedback"
  - "Cargo-llvm-cov installed per-run via cargo install --locked; not added to Cargo.toml"
metrics:
  duration_minutes: 7
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 0
  completed_date: "2026-06-11"
---

# Phase 64 Plan 01: Coverage Infrastructure and Baseline Summary

cargo-llvm-cov 0.8.7 installed, per-module baseline captured for all src/engines/ and src/operations/ files, lowest-5 modules identified for Plan 3, and a CI workflow created to gate PRs at ≥80% line coverage scoped to those two subtrees.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Install cargo-llvm-cov and capture coverage baseline | f10d3e4 | .planning/phases/64-test-doc-quality/64-COVERAGE-BASELINE.md |
| 2 | Create coverage.yml GitHub Actions workflow | b63e09b | .github/workflows/coverage.yml |

## Lowest 5 Modules (from 64-COVERAGE-BASELINE.md)

These are the data-driven targets for Plan 3 test writing:

1. `src/engines/gp/chromosome.rs` — **49.15%** (118 lines, 60 missed)
2. `src/engines/gp/primitives.rs` — **49.61%** (129 lines, 65 missed)
3. `src/operations/mutation/differential.rs` — **50.00%** (26 lines, 13 missed)
4. `src/operations/crossover.rs` — **50.42%** (355 lines, 176 missed)
5. `src/engines/gp/configuration.rs` — **54.07%** (135 lines, 62 missed)

## Current Coverage for Target Subtrees

Overall project line coverage (all-features, excluding tests/): **85.73%**

Notable modules currently above 80% gate:
- `src/engines/`: many modules at 90-100%; key low outliers are GP subsystem and `engines/ga.rs` (67.49%)
- `src/operations/`: majority above 80%; low outliers are `crossover.rs` (50.42%) and `mutation/differential.rs` (50.00%)

## Exact --ignore-filename-regex Used in coverage.yml

```
^(?!.*(src/engines/|src/operations/)).*$
```

This negative lookahead matches any file path that does NOT contain `src/engines/` or `src/operations/`. Files that match the regex are excluded from the coverage gate. Files in the two target subtrees do not match and are therefore retained for the `--fail-under-lines 80` threshold computation.

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The workflow adds a `cargo install` step, which is mitigated by using `--locked` (T-64-01 in the plan's threat model).

## Self-Check: PASSED

- [x] `.planning/phases/64-test-doc-quality/64-COVERAGE-BASELINE.md` exists
- [x] `64-COVERAGE-BASELINE.md` contains `## Lowest 5 Modules` section with 5 entries
- [x] `64-COVERAGE-BASELINE.md` references `src/engines/` and `src/operations/`
- [x] `.github/workflows/coverage.yml` exists and is valid YAML
- [x] `coverage.yml` uses `cargo-llvm-cov`, `--all-features`, `--fail-under-lines 80`, `--ignore-filename-regex`, and `llvm-tools-preview`
- [x] `coverage.yml` triggers on `pull_request` to `main` and `milestone/**`
- [x] `64-COVERAGE-BASELINE.md` contains `## Final CI Regex` section documenting the regex
- [x] Task 1 commit f10d3e4 exists
- [x] Task 2 commit b63e09b exists
