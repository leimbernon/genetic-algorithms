---
phase: 67-build-perf-m1-config-only-quick-wins
plan: "04"
subsystem: ci
tags: [build-perf, ci, sccache, cache, workflow]
dependency_graph:
  requires: [67-01, 67-02, 67-03]
  provides: [sccache-ci-caching]
  affects: [.github/workflows/rust-unit-tests.yml, .github/workflows/coverage.yml, .github/workflows/wasm-check.yml, .github/workflows/rust-clippy.yml, .github/workflows/examples-smoke.yml]
tech_stack:
  added: [mozilla-actions/sccache-action@v0.0.9]
  patterns: [sccache-as-RUSTC_WRAPPER, GHA-cache-backend, job-level-env-block]
key_files:
  modified:
    - .github/workflows/rust-unit-tests.yml
    - .github/workflows/rust-clippy.yml
    - .github/workflows/examples-smoke.yml
    - .github/workflows/coverage.yml
    - .github/workflows/wasm-check.yml
    - docs/DEVELOPMENT.md
    - CHANGELOG.md
decisions:
  - "sccache pinned to v0.0.9 (not @latest/@main) per D-05 supply-chain hygiene"
  - "build-perf-gate.yml explicitly excluded: sccache would corrupt cold-build timing measurements (Pitfall 4)"
  - "SCCACHE_GHA_ENABLED and RUSTC_WRAPPER always set together: setting one without the other breaks caching silently (Anti-Pattern)"
  - "rust-unit-tests.yml uses top-level env block (merged with CARGO_TERM_COLOR); other four workflows use job-level env blocks"
  - "sccache --show-stats appended as final step in every job for D-06 cache hit-rate observability"
metrics:
  duration: "~8 minutes"
  completed: "2026-06-14"
  tasks_completed: 3
  files_modified: 7
---

# Phase 67 Plan 04: sccache CI Caching Integration Summary

One-liner: `mozilla-actions/sccache-action@v0.0.9` wired as `RUSTC_WRAPPER` across five CI workflows with `SCCACHE_GHA_ENABLED=true` and per-job `sccache --show-stats` hit-rate logging; `build-perf-gate.yml` intentionally excluded.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Wire sccache into rust-unit-tests.yml, rust-clippy.yml, examples-smoke.yml | 17c9b14 | 3 workflow files |
| 2 | Wire sccache into coverage.yml and wasm-check.yml | 4ec70ef | 2 workflow files |
| 3 | Append CI caching subsection to docs/DEVELOPMENT.md and CHANGELOG | 6bbc626 | docs/DEVELOPMENT.md, CHANGELOG.md |

## What Was Done

### Task 1: rust-unit-tests.yml, rust-clippy.yml, examples-smoke.yml

For each file:
- **rust-unit-tests.yml**: Merged `RUSTC_WRAPPER: sccache` and `SCCACHE_GHA_ENABLED: "true"` into the existing top-level `env:` block (which already contained `CARGO_TERM_COLOR: always`). Inserted `Configure sccache` step immediately after `actions/checkout@v5`. Appended `sccache stats` step as the final step after `Run tests (nextest)`.
- **rust-clippy.yml**: Added a new job-level `env:` block under `clippy_check:`. Inserted `Configure sccache` step after `actions/checkout@v2` and before `actions-rs/toolchain@v1`. Appended `sccache stats` as final step after `Upload SARIF file`.
- **examples-smoke.yml**: Added job-level `env:` block under `examples-smoke:`. Inserted `Configure sccache` step after `actions/checkout@v4` and before `dtolnay/rust-toolchain@stable`. Appended `sccache stats` as final step after `Run example (smoke)` (executes per matrix entry — intended per D-06).

### Task 2: coverage.yml and wasm-check.yml

- **coverage.yml**: Added job-level `env:` block under `coverage:`. Inserted `Configure sccache` after `actions/checkout@v4`, before `dtolnay/rust-toolchain@stable`. Appended `sccache stats` as final step after `Run coverage gate`. Pre-existing `--fail-under-lines 80` threshold preserved.
- **wasm-check.yml**: Added job-level `env:` block under `wasm-check:`. Inserted `Configure sccache` after `actions/checkout@v4`, before `dtolnay/rust-toolchain@stable`. Appended `sccache stats` as final step. All three `cargo check --target wasm32-unknown-unknown` steps preserved.
- **build-perf-gate.yml**: Confirmed NOT modified. Contains zero `sccache-action` references. This is intentional — sccache caching would invalidate the cold-build timing baseline (Pitfall 4 in 67-RESEARCH.md).

### Task 3: Documentation

- **docs/DEVELOPMENT.md**: Appended `## CI caching` section after `## Linker recommendations`. Section covers: sccache-action version pin (v0.0.9), all five affected workflows by name, explicit exclusion of `build-perf-gate.yml` with rationale, informational note that contributors need no local setup, hit-rate monitoring via `sccache --show-stats` output, and version-update guidance.
- **CHANGELOG.md**: Appended bullet under `## [Unreleased]` → `### Changed` referencing `mozilla-actions/sccache-action@v0.0.9`, `RUSTC_WRAPPER=sccache`, all five workflows, `sccache --show-stats`, `build-perf-gate.yml` exclusion, and `(Phase 67 / Plan 67-04)` tag.

## Security: Threat Model Compliance

| Threat ID | Mitigation Applied |
|-----------|-------------------|
| T-67-04-T | Version pinned exactly to `v0.0.9` in all five workflows; acceptance criteria assert exact string; stale v0.0.4 confirmed absent |
| T-67-04-D | sccache binary absence causes graceful fallback to direct rustc; `--show-stats` runs at end of job only |
| T-67-04-SC | Exact tag pin; Mozilla-maintained action; no new crate dependency added |

## Success Criteria Verified

1. Five workflows configure `mozilla-actions/sccache-action@v0.0.9` with `RUSTC_WRAPPER=sccache` + `SCCACHE_GHA_ENABLED=true` — PASS
2. Each of the five workflows has a final `sccache --show-stats` step — PASS
3. `build-perf-gate.yml` does NOT have sccache wired (Pitfall 4) — PASS
4. All five workflow files parse as valid YAML — PASS
5. No file uses sccache-action `v0.0.4` (stale version forbidden per D-05) — PASS
6. Commit body contains a `Revert plan:` line (D-14) — documented in output spec below

## Revert Plan (D-14)

In each of the five `.github/workflows/*.yml` files:
1. Remove the job-level `env:` keys `RUSTC_WRAPPER` and `SCCACHE_GHA_ENABLED`.
2. Remove the `Configure sccache` step (uses: `mozilla-actions/sccache-action@v0.0.9`).
3. Remove the `sccache stats` step (`sccache --show-stats`).
4. For `rust-unit-tests.yml`, restore the top-level `env:` block to contain only `CARGO_TERM_COLOR: always`.
5. Remove the `## CI caching` section from `docs/DEVELOPMENT.md`.
6. Remove the Plan 67-04 bullet from `CHANGELOG.md` `### Changed`.

No source-code change is required. No Cargo.toml change is required.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — this plan modifies only CI workflow YAML and documentation files. No new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

- `.github/workflows/rust-unit-tests.yml`: exists, contains sccache-action@v0.0.9, RUSTC_WRAPPER, SCCACHE_GHA_ENABLED, show-stats
- `.github/workflows/rust-clippy.yml`: exists, contains all four sccache markers
- `.github/workflows/examples-smoke.yml`: exists, contains all four sccache markers
- `.github/workflows/coverage.yml`: exists, contains all four sccache markers
- `.github/workflows/wasm-check.yml`: exists, contains all four sccache markers
- `docs/DEVELOPMENT.md`: contains `## CI caching` section
- `CHANGELOG.md`: contains `Phase 67 / Plan 67-04` reference
- Commits 17c9b14, 4ec70ef, 6bbc626 exist in git log
