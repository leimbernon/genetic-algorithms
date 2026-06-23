---
phase: 66-build-perf-foundations-baseline-matrix-golden-tests
plan: "02"
subsystem: ci
tags: [ci, github-actions, feature-matrix, wasm32]
dependency_graph:
  requires: ["66-01"]
  provides: ["feature-matrix-ci"]
  affects: [".github/workflows/feature-matrix.yml"]
tech_stack:
  added: []
  patterns: ["GitHub Actions matrix.include strategy", "per-matrix-entry cache key"]
key_files:
  created:
    - .github/workflows/feature-matrix.yml
  modified: []
decisions:
  - "Use strategy.matrix.include (not matrix.feature) to allow wasm32 to run cargo check while all others run cargo test"
  - "No RUSTFLAGS env override needed in the workflow — .cargo/config.toml target-specific rustflags apply automatically when --target wasm32-unknown-unknown is passed"
  - "Trigger on push to main and milestone/** only (no pull_request) per D-01 — cost belongs at merge time"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-14"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 0
---

# Phase 66 Plan 02: Feature Matrix CI Workflow Summary

Feature-matrix CI workflow that runs `cargo test --quiet` for 7 feature combinations and `cargo check --target wasm32-unknown-unknown --lib` for wasm32, triggered on push to `main` and `milestone/**` only.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create feature-matrix CI workflow | c5b9686 | .github/workflows/feature-matrix.yml |
| 2 | Validate YAML syntax and commit | c5b9686 | .github/workflows/feature-matrix.yml |

## What Was Built

`.github/workflows/feature-matrix.yml` — a GitHub Actions workflow with 8 matrix entries:

| Entry | Feature flags | Command |
|-------|--------------|---------|
| default | (none) | `cargo test --quiet` |
| serde | `--features serde` | `cargo test --quiet --features serde` |
| visualization | `--features visualization` | `cargo test --quiet --features visualization` |
| benchmarks | `--features benchmarks` | `cargo test --quiet --features benchmarks` |
| observer-tracing | `--features observer-tracing` | `cargo test --quiet --features observer-tracing` |
| observer-metrics | `--features observer-metrics` | `cargo test --quiet --features observer-metrics` |
| all-features | `--all-features` | `cargo test --quiet --all-features` |
| wasm32 | (none, target-specific) | `cargo check --target wasm32-unknown-unknown --lib` |

Workflow uses `actions/checkout@v4`, `dtolnay/rust-toolchain@stable` (with `targets: wasm32-unknown-unknown`), and `Swatinem/rust-cache@v2` (keyed per matrix entry: `feature-matrix-${{ matrix.name }}`).

## Decisions Made

- `strategy.matrix.include` form chosen over `matrix.feature` to allow different `run` commands per entry (wasm32 needs `cargo check`, others need `cargo test`)
- No `RUSTFLAGS` env override added — `.cargo/config.toml` already sets `--cfg getrandom_backend="wasm_js"` for the `wasm32-unknown-unknown` target, applied automatically by cargo
- Trigger: `on: push: branches: [main, "milestone/**"]` with no `pull_request` key per D-01

## Verification Results

- YAML structure manually verified: 8 matrix entries, consistent 2-space indentation, all required keys present
- `grep -q 'pull_request'` returns non-zero (string absent) — no PR trigger
- wasm32 entry confirmed to use `cargo check --target wasm32-unknown-unknown --lib`
- All 5 feature flag names match exact Cargo.toml keys: `serde`, `visualization`, `benchmarks`, `observer-tracing`, `observer-metrics`
- Commit body includes `Revert plan: delete .github/workflows/feature-matrix.yml` per BUILD-PERF.md §Non-negotiable guarantee #5

## Deviations from Plan

None - plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced. The workflow file is version-controlled and changes require PR review (T-66-02-01 accepted).

## Self-Check: PASSED

- `.github/workflows/feature-matrix.yml` exists: FOUND
- Commit c5b9686 exists: FOUND
