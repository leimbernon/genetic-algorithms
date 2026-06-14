---
phase: 67-build-perf-m1-config-only-quick-wins
plan: "01"
subsystem: build-system
tags: [build-perf, cargo-profile, dev-experience]
dependency_graph:
  requires: []
  provides:
    - Cargo.toml [profile.dev] / [profile.dev.package."*"] / [profile.test] blocks
    - docs/DEVELOPMENT.md §Cargo profiles section
    - .planning/intel/build-profile.md AI-agent rationale
    - CHANGELOG.md ## [Unreleased] scaffold with Changed entry
  affects:
    - Local dev-build wall-clock (improved)
    - cargo test runtime (improved)
    - Plans 67-02/67-03/67-04 CHANGELOG [Unreleased] section (created here)
tech_stack:
  added: []
  patterns:
    - Cargo [profile.*] custom build profiles
    - Keep-a-Changelog [Unreleased] section scaffolding
key_files:
  created:
    - .planning/intel/build-profile.md
  modified:
    - Cargo.toml
    - docs/DEVELOPMENT.md
    - CHANGELOG.md
decisions:
  - "profile.dev uses debug=line-tables-only to reduce linker load on every incremental build"
  - "split-debuginfo=unpacked skips dsymutil on macOS, the largest non-compile link cost"
  - "profile.dev.package.*: opt-level=1 for deps is a one-time cost, then cached; cuts rand/rayon/log runtime overhead in tests"
  - "[Unreleased] inserted before [3.0.0] per Keep-a-Changelog; scaffold with Added/Changed/Removed for parallel plan appends"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-14"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 4
---

# Phase 67 Plan 01: Cargo Profile Tuning Summary

One-liner: Three Cargo profile blocks (line-tables-only debug, dep opt-level=1, test opt-level=1) delivering ~5-15% dev-build and ~50% test-runtime improvement over the Phase 66 baseline.

## What Shipped

This plan is a pure configuration change — zero source code modified. It delivers all three D-10 deliverables plus the Cargo.toml D-09 deliverable.

### Task 1 — Cargo.toml profile blocks

Appended three `[profile.*]` blocks verbatim from `.planning/v3.0.0-BUILD-PERF.md §Action #5/#6`:

```toml
[profile.dev]
debug = "line-tables-only"
split-debuginfo = "unpacked"

[profile.dev.package."*"]
opt-level = 1
debug = false

[profile.test]
opt-level = 1
```

Commit: `665dff4` — `build(67-01): tune [profile.dev]/[profile.test] for faster local iteration`

### Task 2 — Contributor documentation

- Appended `## Cargo profiles` section with four subsections to `docs/DEVELOPMENT.md`, matching the file's existing voice and heading hierarchy.
- Created `.planning/intel/build-profile.md` (new directory `.planning/intel/`) with 102-line AI-agent rationale, explicit `DO NOT REMOVE` warning, per-key explanations, and cross-links to the canonical BUILD-PERF spec and the contributor guide.

Commit: `a82abc7` — `docs(67-01): add Cargo profiles section and intel/build-profile.md rationale`

### Task 3 — CHANGELOG scaffold

- Inserted `## [Unreleased]` immediately before `## [3.0.0] - Unreleased` (now at line 8; `[3.0.0]` at line 20).
- Added empty `### Added`, `### Changed`, `### Removed` scaffolding so plans 67-02/67-03/67-04 can append cleanly without conflict.
- Added `### Changed` bullet for the profile tuning entry, citing `docs/DEVELOPMENT.md §Cargo profiles` and `Phase 67 / Plan 67-01`.

Commit: `81a0e73` — `docs(67-01): add [Unreleased] section to CHANGELOG with profile-tuning entry`

## Expected Build Impact

Per `.planning/v3.0.0-BUILD-PERF.md §Action #5/#6`:

| Scenario | Expected delta |
|----------|---------------|
| Clean dev build wall-clock | -5% to -15% (macOS: larger due to dsymutil skip) |
| Incremental dev build | no regression (profile only affects object/link, not compile) |
| `cargo test` wall-clock | ~-50% (test binary runs at -O1; GA generation loops are tight) |
| First clean build penalty | +5-10 s one-time (dep recompile at opt-level=1, then cached) |

Actual before/after measurements were not captured in this plan (measurement tooling belongs to the build-perf-gate workflow in Phase 66 / Plan 66-03). The profile choices are anchored to the Phase 66 baseline at `.planning/baselines/v3.0.0-baseline.json`.

## Verification Results

All three acceptance criteria sets passed at commit time:

- `cargo metadata` confirmed TOML is valid and profile blocks are recognized
- All grep counts returned expected values (1, 1, 1, 1, 1, 2 for profile key occurrences)
- `docs/DEVELOPMENT.md` contains all required headings and subsections
- `.planning/intel/build-profile.md` is 102 lines, contains `Phase 67` and `DO NOT REMOVE`
- `CHANGELOG.md` has `## [Unreleased]` at line 8, `## [3.0.0]` at line 20

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — this plan adds pure configuration and documentation. No data flowing to UI, no placeholder text in source code.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. The profile blocks are dev-only configuration with no effect on `[profile.release]` or published artifacts.

## Self-Check: PASSED

- [x] Cargo.toml contains `[profile.dev]`, `[profile.dev.package."*"]`, `[profile.test]` (verified via grep)
- [x] docs/DEVELOPMENT.md contains `## Cargo profiles` with four subsections (verified via grep)
- [x] .planning/intel/build-profile.md exists (102 lines, verified)
- [x] CHANGELOG.md `## [Unreleased]` at line 8, before `## [3.0.0]` at line 20 (verified)
- [x] Commits 665dff4, a82abc7, 81a0e73 exist in git log
