---
phase: 47
plan: 08
subsystem: examples-ci-smoke-test
tags:
  - ci
  - github-actions
  - examples
  - phase-gate
dependency_graph:
  requires:
    - 47-07
  provides:
    - arch07-satisfied
    - examples-smoke-ci
    - phase-47-complete
  affects:
    - phase-48-onwards
tech_stack:
  added:
    - GitHub Actions matrix strategy (10-example smoke CI)
  patterns:
    - wasm-check.yml trigger/toolchain/cache pattern reused
    - fail-fast: false + timeout-minutes per job (T-47-22 mitigation)
key_files:
  created:
    - .github/workflows/examples-smoke.yml
  modified:
    - src/validators/generic_validator.rs
decisions:
  - D-12 (examples-smoke.yml: new CI workflow; triggers on push main/milestone/**/feat/**/fix** and PRs to main/milestone/**)
  - T-47-22 mitigated (timeout-minutes: 5 per matrix job prevents CI hang)
  - T-47-23 mitigated (workflow IS the regression detector for the 10 examples)
  - T-47-24 accepted (no secrets used in examples)
  - Rule 1 auto-fix (unused alleles param in generic_validator.rs prefixed with _ to clear clippy -D warnings)
metrics:
  completed_date: "2026-05-21"
  duration_mins: 20
  tasks_completed: 1
  files_changed: 2
---

# Phase 47 Plan 08: Examples Smoke Test CI Workflow + Phase 47 Final Gate Summary

Add `.github/workflows/examples-smoke.yml` that compiles and runs each of the 10 designated examples on every PR to the milestone branch and on every push to feat/fix branches. Satisfies ARCH-07. Closes Phase 47 with all 7 ARCH requirements complete and all verification gates GREEN.

## Tasks Completed

| Task | Commit | Description |
|------|--------|-------------|
| Task 1: examples-smoke.yml + clippy fix | 3cbe363 | Add CI workflow (10-example matrix); prefix unused `alleles` param with `_` to clear clippy -D warnings |

## What Was Built

### examples-smoke.yml Workflow

New file `.github/workflows/examples-smoke.yml`:

- **Triggers:** push to `main`, `milestone/**`, `feat/**`, `fix/**`; pull_request to `main`, `milestone/**`
- **Matrix:** 10 examples as parallel jobs
- **fail-fast:** false (all failures surface at once — faster diagnosis)
- **timeout-minutes:** 5 per matrix job (T-47-22 DoS mitigation)
- **Steps:** checkout → dtolnay/rust-toolchain@stable → Swatinem/rust-cache@v2 → cargo run --example ${{ matrix.example }} --release

### The 10 CI-Target Examples

All 10 examples verified to have corresponding `.rs` files in `examples/`. None require `--features` flags (sms_emoa_zdt1 and ibea_zdt1 require `--features benchmarks` but are excluded from the list). All run in 1–2 seconds locally.

| # | Example | Category | Local time |
|---|---------|----------|-----------|
| 1 | knapsack_binary | Single-objective | ~2s |
| 2 | onemax_binary | Single-objective | ~1s |
| 3 | onemax_extension | Single-objective + extension | ~1s |
| 4 | rastrigin | Continuous optimization | ~1s |
| 5 | nsga2_zdt1 | Multi-objective (NSGA-II) | ~1s |
| 6 | island_model | Island model | ~1s |
| 7 | job_scheduling | Combinatorial | ~2s |
| 8 | niching | Niching / fitness sharing | ~1s |
| 9 | hall_of_fame_demo | Hall of Fame | ~1s |
| 10 | aos_demo | Adaptive operator selection | ~1s |

### Auto-fix: Unused `alleles` parameter (Rule 1)

`src/validators/generic_validator.rs` line 35: the `alleles: Option<&[U::Gene]>` parameter in `validate()` was unused and produced a `clippy::unused_variables` warning. Under `-D warnings`, this becomes a hard error. Fixed by renaming to `_alleles` and updating the doc comment.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unused variable error in generic_validator.rs**
- **Found during:** Task 1 clippy verification (`cargo clippy --all-features -- -D warnings`)
- **Issue:** `alleles` parameter in `validate()` was unused, producing `error: unused variable: 'alleles'` under `-D warnings`. This was noted as a pre-existing warning in 47-07-SUMMARY.md but not fixed there.
- **Fix:** Renamed `alleles` to `_alleles` in the function signature and updated the doc comment.
- **Files modified:** `src/validators/generic_validator.rs`
- **Commit:** 3cbe363

**2. [Rule 3 - Blocking] Merged milestone/v3.0.0 into worktree branch before creating workflow**
- **Found during:** Pre-execution — worktree was behind milestone/v3.0.0 by all of plans 47-01 through 47-07
- **Fix:** `git merge milestone/v3.0.0` fast-forwarded the worktree-agent branch to include all prior Phase 47 work
- **Impact:** No code changes — purely a merge to get the correct base state

## Phase 47 Final Verification Gate

All 8 gates GREEN as of 2026-05-21:

### Gate 1: cargo test (default features)
- **Result:** 966 passed, 28 ignored — PASS

### Gate 2: cargo test --features serde
- **Result:** 1003 passed, 28 ignored — PASS

### Gate 3: cargo clippy --all-features -- -D warnings
- **Result:** No issues found — PASS
  (fixed unused `alleles` param first — see deviation above)

### Gate 4: cargo check --target wasm32-unknown-unknown
- **Result:** 0 errors, compilation complete — PASS

### Gate 5: cargo doc --no-deps --all-features
- **Result:** Zero warnings — PASS

### Gate 6: All 10 examples run under 60s each
- **Result:** All 10 examples ran in 1–2s each — PASS
  - knapsack_binary: PASS (2s)
  - onemax_binary: PASS (1s)
  - onemax_extension: PASS (1s)
  - rastrigin: PASS (1s)
  - nsga2_zdt1: PASS (1s)
  - island_model: PASS (1s)
  - job_scheduling: PASS (2s)
  - niching: PASS (1s)
  - hall_of_fame_demo: PASS (1s)
  - aos_demo: PASS (1s)

### Gate 7: cargo package --list includes MIGRATION.md
- **Result:** MIGRATION.md and README.md both listed — PASS

### Gate 8: ROADMAP.md Phase 47 success criteria demonstrably true
- ARCH-01: ChromosomeT split into core trait (confirmed in 47-01)
- ARCH-02: LinearChromosome supertrait + operator bounds updated (confirmed in 47-02, 47-03)
- ARCH-03: Reporter<U> removed; MIGRATION.md published (confirmed in 47-07)
- ARCH-04: ChromosomeLength enum introduced (confirmed in 47-04)
- ARCH-05: LimitConfiguration field removals (confirmed in 47-05)
- ARCH-06: GaConfiguration encapsulated + StoppingCriteria flattened (confirmed in 47-05, 47-06)
- ARCH-07: examples-smoke.yml CI workflow created (this plan) — PASS

**All 7 ARCH requirements satisfied. Phase 47 COMPLETE.**

## PR 3 Status

PR 3 (plans 47-07 + 47-08) is ready to merge against `milestone/v3.0.0`. Contains:
- Reporter<U> removal + MIGRATION.md (47-07)
- examples-smoke.yml CI workflow (47-08)

## Known Stubs

None — CI workflow and clippy fix are complete, no placeholder content.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced.
examples-smoke.yml triggers only on standard CI events; no secrets are used (pure compute).
T-47-22, T-47-23 mitigated; T-47-24 accepted.

## Self-Check: PASSED

- `.github/workflows/examples-smoke.yml` exists: CONFIRMED
- YAML parses via Python yaml.safe_load, examples-smoke job present: CONFIRMED
- 10 examples in matrix, all verified to correspond to real .rs files: CONFIRMED
- push.branches includes main, milestone/**, feat/**, fix/**: CONFIRMED
- pull_request.branches includes main, milestone/**: CONFIRMED
- fail-fast: false; timeout-minutes: 5: CONFIRMED
- All 5 standard verification gates GREEN: CONFIRMED
- All 10 examples run in <60s each: CONFIRMED
- MIGRATION.md in cargo package: CONFIRMED
- Commit 3cbe363 exists: CONFIRMED
- All 7 ARCH requirements satisfied: CONFIRMED
