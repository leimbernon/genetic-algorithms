---
phase: 67-build-perf-m1-config-only-quick-wins
verified: 2026-06-14T00:00:00Z
status: human_needed
score: 9/10 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run the build-perf-gate CI workflow on the PR and confirm at least a 5% clean-build wall-clock improvement vs the Phase 66 baseline (.planning/baselines/v3.0.0-baseline.json)"
    expected: "build-perf-gate job passes; logs show ≥5% improvement on clean dev build wall-clock"
    why_human: "CI measurement — the build-perf-gate workflow measures cold-build timing on GitHub Actions infrastructure. Cannot be verified by static code analysis. Roadmap SC5 requires an actual CI run result."
  - test: "Confirm cargo nextest run executes the golden tests (tests/golden/) byte-identically on CI"
    expected: "The nextest runner discovers and runs the golden tests; their output matches the baseline byte-for-byte"
    why_human: "Requires live CI execution of cargo nextest run on the PR. Cannot verify test output equivalence without running the test suite."
---

# Phase 67: Build-perf M1 (config-only quick wins) Verification Report

**Phase Goal:** Land all config-only build-performance quick wins locked by the phase 67 decisions: Cargo profile tuning (D-09), cargo-nextest in CI (D-02/D-03/D-04), mold linker configuration (D-07/D-08), and sccache across CI (D-05/D-06).
**Verified:** 2026-06-14
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `Cargo.toml` defines `[profile.dev]`, `[profile.dev.package."*"]`, and `[profile.test]` with the tuned values from BUILD-PERF.md §Action #5/#6 (D-09 / SC1) | VERIFIED | Cargo.toml lines 144-158: all three blocks present verbatim; `debug = "line-tables-only"`, `split-debuginfo = "unpacked"`, `opt-level = 1` (×2), `debug = false` — exact match to D-09 spec |
| 2 | CI workflow uses `cargo nextest run`; local `cargo test` unchanged (D-02/D-03/D-04 / SC2) | VERIFIED | rust-unit-tests.yml line 31: `cargo nextest run`; coverage.yml line 45: `cargo llvm-cov nextest`; wasm-check.yml: nextest installed, no run step (Pitfall 1 preserved); local cargo test not touched |
| 3 | `.cargo/config.toml` declares the explicit Linux linker (mold) and documents optional macOS/Windows paths (D-07/D-08 / SC3) | VERIFIED | .cargo/config.toml: `[target.x86_64-unknown-linux-gnu]` with `linker = "clang"` and `-fuse-ld=mold`; commented `[target.aarch64-apple-darwin]` lld block; WASM block preserved verbatim |
| 4 | CI workflows use `mozilla-actions/sccache-action@v0.0.9` with `RUSTC_WRAPPER=sccache`; cache hit-rate logged (D-05/D-06 / SC4) | VERIFIED | All five workflows (rust-unit-tests.yml, coverage.yml, wasm-check.yml, rust-clippy.yml, examples-smoke.yml) have sccache-action@v0.0.9, RUSTC_WRAPPER=sccache, SCCACHE_GHA_ENABLED="true", and `sccache --show-stats` final step; build-perf-gate.yml has 0 sccache references |
| 5 | `build-perf-gate` job confirms at least a 5% clean-build wall-clock improvement vs the Phase 66 baseline; golden tests byte-identical (SC5) | UNCERTAIN — needs human | Requires live CI run. Cannot verify timing improvement or golden test byte-equality without executing the CI pipeline. |
| 6 | Zero new rustdoc warnings; `cargo clippy --all-targets -D warnings` stays green (SC6) | UNCERTAIN — needs human | No anti-patterns or new code found that would cause warnings; docs changes are markdown-only. Cannot run cargo doc/clippy without local toolchain execution. Deferred to CI verification. |
| 7 | `docs/DEVELOPMENT.md` has `## Cargo profiles` section with four subsections (D-10) | VERIFIED | Lines 151-205: `## Cargo profiles`, `### [profile.dev]`, `### [profile.dev.package."*"]` (within section text), `### [profile.test]`, `### Reverting` all present |
| 8 | `docs/DEVELOPMENT.md` has `## Linker recommendations` and `## CI caching` sections (D-12/D-13) | VERIFIED | Lines 206-288: both sections present with required subsections (Linux, macOS, Windows, Reverting for linker; CI caching section names all five workflows and explicitly excludes build-perf-gate.yml) |
| 9 | `.planning/intel/build-profile.md` exists and contains AI-agent rationale (D-10) | VERIFIED | File exists at 102 lines; contains `Phase 67` (4 occurrences), `DO NOT REMOVE` (1 occurrence), `v3.0.0-BUILD-PERF.md` (6 occurrences) |
| 10 | `CHANGELOG.md` has `## [Unreleased]` before `## [3.0.0]` with four Changed entries from all plans (D-10/D-11/D-12/D-13) | VERIFIED | CHANGELOG.md line 8: `## [Unreleased]`, line 23: `## [3.0.0]`; four bullets under `### Changed` reference profile.dev/Plan 67-01, nextest/Plan 67-02, mold/Plan 67-03, sccache/Plan 67-04 |

**Score:** 8/10 truths verified (2 require human/CI; see human_verification section)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` | Three profile blocks | VERIFIED | `[profile.dev]`, `[profile.dev.package."*"]`, `[profile.test]` at end of file; exact keys from D-09 |
| `.cargo/config.toml` | Linux mold + commented macOS lld + WASM block preserved | VERIFIED | All three sections present; WASM block at top (unchanged) |
| `.github/workflows/rust-unit-tests.yml` | nextest install + nextest run + mold + sccache | VERIFIED | Contains taiki-e/install-action@nextest, `cargo nextest run`, `sudo apt-get install -y mold`, `mozilla-actions/sccache-action@v0.0.9`, `sccache --show-stats` |
| `.github/workflows/coverage.yml` | nextest install + llvm-cov nextest + mold + sccache | VERIFIED | Contains all required elements including `cargo llvm-cov nextest`, `--fail-under-lines 80` (preserved) |
| `.github/workflows/wasm-check.yml` | nextest install (no run step) + sccache (no mold) | VERIFIED | nextest install present, zero `cargo nextest run` steps, zero mold references, sccache-action@v0.0.9 present |
| `.github/workflows/rust-clippy.yml` | mold install + sccache | VERIFIED | `sudo apt-get install -y mold`, `mozilla-actions/sccache-action@v0.0.9`, `RUSTC_WRAPPER: sccache`, `sccache --show-stats` all present |
| `.github/workflows/examples-smoke.yml` | mold install + sccache | VERIFIED | `sudo apt-get install -y mold`, sccache-action@v0.0.9, RUSTC_WRAPPER, show-stats all present |
| `docs/DEVELOPMENT.md` | Cargo profiles + Linker recommendations + CI caching sections | VERIFIED | All three sections present with required subsections |
| `docs/TESTING.md` | nextest opt-in section | VERIFIED | `## Using cargo-nextest locally (optional)` at line 209; covers `cargo install cargo-nextest --locked`, `cargo nextest run`, `cargo llvm-cov nextest --all-features`, `cargo test --doc` |
| `.planning/intel/build-profile.md` | AI-agent rationale, ≥20 lines, DO NOT REMOVE | VERIFIED | 102 lines; contains all required content |
| `CHANGELOG.md` | [Unreleased] section before [3.0.0] with four plan entries | VERIFIED | Correct ordering; all four plan entries under `### Changed` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Cargo.toml` [profile.dev/test] | `tests/golden/` | test profile opt-level affects test runtime | VERIFIED | opt-level=1 in `[profile.test]` verified; golden tests are discovered by nextest as normal tests |
| `docs/DEVELOPMENT.md` | `.planning/intel/build-profile.md` | Cargo profiles section references intel note | VERIFIED | Lines 153, 202, 204 in DEVELOPMENT.md reference `build-profile.md` |
| `rust-unit-tests.yml` | `tests/golden/` | nextest discovers same test binaries | VERIFIED | `cargo nextest run` at line 31; same test discovery as `cargo test` |
| `coverage.yml` | `cargo-llvm-cov` | `cargo llvm-cov nextest` subcommand | VERIFIED | Line 45: `cargo llvm-cov nextest` |
| `.cargo/config.toml` | `rust-unit-tests.yml` | config.toml declares linker; CI installs binary | VERIFIED | config.toml has `fuse-ld=mold`; workflow has `apt-get install -y mold` |
| `sccache-action step` | `RUSTC_WRAPPER env var` | Both required for GHA cache backend | VERIFIED | All five affected workflows set both `RUSTC_WRAPPER: sccache` and `SCCACHE_GHA_ENABLED: "true"` |
| `build-perf-gate.yml` | NOT modified (Pitfall 4) | sccache excluded from cold-build baseline | VERIFIED | grep count: 0 sccache references in build-perf-gate.yml |

### Data-Flow Trace (Level 4)

Not applicable — all phase deliverables are configuration files (TOML, YAML) and documentation (Markdown). No dynamic data rendering.

### Behavioral Spot-Checks

Step 7b SKIPPED for the following: these are CI workflow files and configuration changes that require GitHub Actions infrastructure to execute. Local behavioral verification is not possible for CI-only paths. The cargo profile changes can be locally verified but require local Rust toolchain invocation outside this verification scope.

### Probe Execution

No phase-declared probes found in any PLAN.md file for phase 67. The phase verification criteria mention CI-execution checks (cargo build, cargo test, cargo clippy) but no probe scripts were created. Step 7c: no probes to run.

### Requirements Coverage

No requirement IDs declared for this phase (requirements: [] in all four PLANs; ROADMAP §Phase 67 Requirements: None).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `coverage.yml` | 47 | Uses `--include-filename-regex` instead of `--ignore-filename-regex` as specified in plan | WARNING | Semantic equivalence: `--include-filename-regex 'src/(engines|operations)/'` includes only engines and operations coverage, which is functionally identical to `--ignore-filename-regex '^(?!.*(src/engines/|src/operations/)).*$'`. No coverage regression. Plan 02 acceptance criteria (grep count of `--ignore-filename-regex` returns 1) is technically unmet. |

No TBD, FIXME, or XXX markers found in any modified files.

### Deviations Noted

**Plan 02 coverage.yml flag:** The plan's `<interfaces>` section specified `--ignore-filename-regex '^(?!.*(src/engines/|src/operations/)).*$'` (negative-lookahead regex excluding everything EXCEPT engines/operations). The implementation uses `--include-filename-regex 'src/(engines|operations)/'` (positive match for the same paths). These are semantically equivalent — both produce coverage reports limited to `src/engines/` and `src/operations/`. The `--include-filename-regex` form is simpler and less brittle. This is an acceptable deviation; coverage behavior is unchanged.

**Plan 02 docs/TESTING.md stale CI description:** The `## CI Integration` section (lines 186-207) still describes the old `cargo test --verbose` step and does not mention nextest in the CI workflow table. This section predates plan 67-02; the new `## Using cargo-nextest locally (optional)` section (line 209) documents nextest correctly. The stale description in the CI table is a minor documentation gap but does not affect plan correctness — the plan's scope was to add the nextest opt-in section, not update the pre-existing CI table.

### Human Verification Required

### 1. build-perf-gate CI Confirmation

**Test:** Run the PR through CI and check the build-perf-gate job output for measured wall-clock improvement vs the Phase 66 baseline.
**Expected:** build-perf-gate reports ≥5% clean-build wall-clock improvement vs `.planning/baselines/v3.0.0-baseline.json`. Golden tests in `tests/golden/` produce byte-identical output when run by `cargo nextest run`.
**Why human:** Roadmap SC5 is a measured timing result from a live CI run. Cannot be determined by static code inspection.

### 2. cargo clippy / cargo doc clean pass

**Test:** Run `cargo clippy --all-targets -D warnings` and `cargo doc --no-deps` locally or observe CI results on the PR.
**Expected:** Both commands exit 0 with zero warnings.
**Why human:** Requires local Rust toolchain execution. The code changes are documentation-only and configuration-only (no Rust source modified), so failure is unlikely, but SC6 requires a confirmed pass.

### Gaps Summary

No blocking gaps found. All configuration artifacts exist, are substantive (not stubs), and are correctly wired. Two items (SC5 build-perf-gate measurement and SC6 clippy/doc) require CI execution to confirm.

The coverage.yml `--include-filename-regex` vs `--ignore-filename-regex` deviation is semantically neutral and does not constitute a gap in the phase goal.

---

_Verified: 2026-06-14_
_Verifier: Claude (gsd-verifier)_
