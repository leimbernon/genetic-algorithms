---
phase: 67-build-perf-m1-config-only-quick-wins
plan: "02"
subsystem: ci
tags: [build-perf, ci, nextest, testing]
depends_on: []
provides: [nextest-ci-runner]
affects: [rust-unit-tests.yml, coverage.yml, wasm-check.yml]

tech_stack:
  added:
    - taiki-e/install-action@nextest (GitHub Actions step — CI only)
  patterns:
    - cargo nextest run (CI test runner, replaces cargo test --verbose)
    - cargo llvm-cov nextest (coverage runner integration)

key_files:
  modified:
    - .github/workflows/rust-unit-tests.yml
    - .github/workflows/coverage.yml
    - .github/workflows/wasm-check.yml
    - docs/TESTING.md
    - CHANGELOG.md

decisions:
  - "Used taiki-e/install-action@nextest (not cargo install cargo-nextest --locked) in CI: faster install (~3s vs ~60s), pre-built binary, officially supported by the nextest project. cargo install path documented in TESTING.md as the local dev alternative."
  - "wasm-check.yml gets the install step but NO cargo nextest run step (Pitfall 1): wasm-check only runs cargo check for cross-compiled targets; nextest cannot execute wasm32 binaries on the host runner."
  - "coverage.yml uses cargo llvm-cov nextest (the cargo-llvm-cov subcommand integration), not a separate nextest run followed by llvm-cov. This is D-03 — the subcommand is purpose-built for this integration."
  - "Local cargo test is explicitly preserved as the default workflow (D-04). The TESTING.md section is titled (optional) to communicate this clearly."

metrics:
  duration_minutes: 8
  completed_date: "2026-06-14"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 5
---

# Phase 67 Plan 02: Swap cargo test -> cargo nextest run in CI Workflows Summary

CI test runner switched to `cargo nextest run` across all three test-running workflows (D-02 / D-03), with local `cargo test` preserved unchanged (D-04). Expected 30-50% wall-clock reduction in `rust-unit-tests.yml` and `coverage.yml` CI jobs via per-binary parallelism.

## What Was Built

Three GitHub Actions workflow files were modified to adopt nextest as the CI test runner:

**`rust-unit-tests.yml`** — Added `taiki-e/install-action@nextest` before the `Build` step; replaced `run: cargo test --verbose` with `run: cargo nextest run`. The `Build` step (`cargo build --verbose`) is unchanged. One install step added, one line changed.

**`coverage.yml`** — Added `taiki-e/install-action@nextest` after `Install cargo-llvm-cov`; replaced `cargo llvm-cov` with `cargo llvm-cov nextest` (D-03 requirement). All flags preserved: `--all-features`, `--ignore-filename-regex`, `--fail-under-lines 80`.

**`wasm-check.yml`** — Added `taiki-e/install-action@nextest` after the cache step for future-proofing (D-02). No `cargo nextest run` step added (Pitfall 1: the workflow only has `cargo check --target wasm32-unknown-unknown` steps; nextest cannot execute cross-compiled wasm32 binaries on the host runner).

**`docs/TESTING.md`** — New `## Using cargo-nextest locally (optional)` section added at the end. Covers: why CI uses nextest, how to install locally (`cargo install cargo-nextest --locked`), how to run (`cargo nextest run`), coverage mirroring (`cargo llvm-cov nextest --all-features`), and the doc-test gap (`cargo test --doc` still required).

**`CHANGELOG.md`** — Appended a `Changed` bullet under `[Unreleased]` documenting the CI-internal runner switch (all three workflows, Phase 67 / Plan 67-02).

## Key Decisions

### install-action vs cargo install

`taiki-e/install-action@nextest` was chosen over `cargo install cargo-nextest --locked` for CI because:
- Pre-built binary download: ~3 seconds vs ~60 seconds to compile from source
- Officially supported install path from the nextest project
- `@nextest` alias tracks the official stable nextest release channel

The `cargo install` path is documented in `TESTING.md` as the correct local dev install method.

### Pitfall 1: wasm-check.yml install-only

`wasm-check.yml` has no host-arch test steps — only `cargo check --target wasm32-unknown-unknown`. Nextest runs test binaries on the host; it cannot execute cross-compiled wasm32 binaries. Adding a `cargo nextest run` step here would either fail at runtime or run zero tests, providing no value. The install step is retained for future-proofing: if a host-arch `cargo test` step is ever added to `wasm-check.yml`, nextest will already be available without a separate PR.

### cargo llvm-cov nextest (D-03)

`cargo llvm-cov nextest` is a purpose-built subcommand of `cargo-llvm-cov` that instruments nextest runs under llvm-cov. This is the correct integration path (not `cargo nextest run` followed by a separate `cargo llvm-cov` invocation). The subcommand passes nextest's binary discovery through the llvm-cov instrumentation layer seamlessly.

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. The only new network access is `taiki-e/install-action@nextest` downloading a pre-built nextest binary from GitHub Releases (T-67-02-T: accepted, documented in plan threat register).

## Self-Check: PASSED

Files verified:
- `.github/workflows/rust-unit-tests.yml` — FOUND; contains `taiki-e/install-action@nextest` (1 match), `cargo nextest run` (1 match); no `cargo test --verbose`; valid YAML
- `.github/workflows/coverage.yml` — FOUND; contains `cargo llvm-cov nextest` (1 match), nextest install (1 match), `--fail-under-lines 80` (1 match), `--ignore-filename-regex` (1 match); valid YAML
- `.github/workflows/wasm-check.yml` — FOUND; contains nextest install (1 match); no `cargo nextest run`; 3 `cargo check --target wasm32-unknown-unknown` steps; valid YAML
- `docs/TESTING.md` — FOUND; `## Using cargo-nextest locally (optional)` heading present
- `CHANGELOG.md` — FOUND; `[Unreleased]` section present; nextest Changed bullet with `Phase 67 / Plan 67-02`

Commits verified:
- ec43b17 — ci(67-02): wire nextest into rust-unit-tests.yml
- 56f201a — ci(67-02): wire nextest into coverage.yml and wasm-check.yml
- 927e160 — docs(67-02): add nextest opt-in docs and CHANGELOG Changed entry
