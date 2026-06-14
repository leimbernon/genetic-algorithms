---
phase: 67-build-perf-m1-config-only-quick-wins
reviewed: 2026-06-14T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - .cargo/config.toml
  - .github/workflows/coverage.yml
  - .github/workflows/examples-smoke.yml
  - .github/workflows/rust-clippy.yml
  - .github/workflows/rust-unit-tests.yml
  - .github/workflows/wasm-check.yml
  - CHANGELOG.md
  - Cargo.toml
  - docs/DEVELOPMENT.md
  - docs/TESTING.md
findings:
  critical: 4
  warning: 5
  info: 2
  total: 11
status: issues_found
---

# Phase 67: Code Review Report

**Reviewed:** 2026-06-14T00:00:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 67 introduces Cargo profile tuning, nextest adoption, mold linker integration, and sccache
across CI workflows. The changes are broadly well-structured and the documentation is thorough.
However, four blockers exist: two runtime-package entries in `[dependencies]` that will ship
unwanted crates to library consumers; a non-existent `actions/checkout@v5` tag that will break CI
on every PR to `main`; and a `--ignore-filename-regex` pattern that uses negative-lookahead
syntax unsupported by Rust's regex engine, making the coverage gate logically inoperative.
Five warnings cover CI scope gaps, deprecated actions, and a doc/reality mismatch.

---

## Critical Issues

### CR-01: `mold` and `cargo-nextest` in `[dependencies]` instead of build tooling

**File:** `Cargo.toml:50-51`
**Issue:** Both `mold = "0.0.1"` and `cargo-nextest = "0.9.96"` are declared under
`[dependencies]`. This means they are compiled into the library crate and listed as required
runtime dependencies in the published `Cargo.toml` on crates.io. Every downstream user of
`genetic_algorithms` will pull in a linker binary and a test-runner binary as transitive Rust
library dependencies — which is semantically wrong and will bloat the dependency tree of anyone
who adds this crate. `mold` is a linker invoked by the OS/shell, not a Rust library; it has no
meaningful Rust API surface. `cargo-nextest` is a Cargo subcommand, not a library. Neither
should be a `[dependencies]` entry at all.

`mold` should be installed at the OS/CI level (`sudo apt-get install -y mold`) and referenced
only in `.cargo/config.toml`. `cargo-nextest` should be installed via
`taiki-e/install-action@nextest` in CI and `cargo install cargo-nextest --locked` locally; it
does not belong in `Cargo.toml` in any section.

**Fix:**
```toml
# REMOVE these two lines entirely from [dependencies]:
# mold = "0.0.1"
# cargo-nextest = "0.9.96"

# mold is already wired via .cargo/config.toml + CI apt-get install — no Cargo entry needed.
# cargo-nextest is already installed via taiki-e/install-action@nextest in CI — no entry needed.
```

---

### CR-02: `actions/checkout@v5` does not exist — CI will fail on every PR to `main`

**File:** `.github/workflows/rust-unit-tests.yml:21`
**Issue:** The step `uses: actions/checkout@v5` references a tag that does not exist in the
`actions/checkout` repository. The latest major version is `v4`. GitHub Actions will fail to
resolve the action and the entire workflow will error before running a single test. This means
the `rust-unit-tests.yml` gate — the only workflow that runs `cargo nextest run` against `main`
— is completely broken.

**Fix:**
```yaml
- uses: actions/checkout@v4   # v5 does not exist; latest major is v4
```

---

### CR-03: `--ignore-filename-regex` in coverage gate uses negative-lookahead (unsupported in Rust regex)

**File:** `.github/workflows/coverage.yml:46-48`
**Issue:** The `cargo llvm-cov nextest` command passes:
```
--ignore-filename-regex '^(?!.*(src/engines/|src/operations/)).*$'
```
The pattern `(?!...)` is a negative lookahead. Rust's `regex` crate (which `cargo-llvm-cov`
uses internally to match filenames) explicitly does not support lookahead or lookbehind
assertions. When this regex is compiled, `cargo-llvm-cov` will either panic/error with
"regex parse error: look-around" or silently fall back to unfiltered behaviour depending on
the version. Either outcome defeats the gate: the 80 % threshold is enforced against either
the wrong file set or no file set.

The intent is to measure coverage *only* for `src/engines/` and `src/operations/`. The correct
approach is to use `--include-filename-regex` (an allowlist) rather than a deny-list lookahead.

**Fix:**
```yaml
- name: Run coverage gate (src/engines/ + src/operations/ ≥ 80%)
  run: |
    cargo llvm-cov nextest \
      --all-features \
      --include-filename-regex '(src/engines/|src/operations/)' \
      --fail-under-lines 80
```

---

### CR-04: Unit test CI never runs against milestone-branch PRs

**File:** `.github/workflows/rust-unit-tests.yml:5-6`
**Issue:** The workflow triggers only on:
```yaml
on:
  pull_request:
    branches: [ "main" ]
```
All feature and fix work targets `milestone/**` branches (per CLAUDE.md and the branching
strategy). PRs to `milestone/**` branches are where all code review and testing must happen
before work is merged upstream to `main`. With the current trigger, `cargo nextest run` never
executes on any feature PR — only on the final merge PR to `main`. This leaves milestone-branch
PRs completely untested by the unit-test gate.

**Fix:**
```yaml
on:
  pull_request:
    branches: [ "main", "milestone/**" ]
```

---

## Warnings

### WR-01: `rust-clippy.yml` uses deprecated `actions-rs/toolchain@v1` (archived, node12)

**File:** `.github/workflows/rust-clippy.yml:19-23`
**Issue:** `actions-rs/toolchain@v1` is archived and runs on the deprecated Node.js 12 runtime.
GitHub has been warning about this for over a year and will eventually refuse to run node12
actions. The recommended replacement is `dtolnay/rust-toolchain@stable`, which is already used
in the other four workflows in this changeset (consistent with the rest of the repo). Using two
different action wrappers for the same task introduces unnecessary divergence.

Additionally, `actions/checkout@v2` on line 16 and `github/codeql-action/upload-sarif@v2` on
line 32 are also pinned to old major versions (latest is v4 and v3 respectively).

**Fix:**
```yaml
# Replace:
- uses: actions-rs/toolchain@v1
  with:
    profile: minimal
    toolchain: stable
    components: clippy,rustfmt
    override: true

# With:
- name: Install stable toolchain
  uses: dtolnay/rust-toolchain@stable
  with:
    components: clippy,rustfmt

# Also update:
- uses: actions/checkout@v4           # was @v2
# and:
  uses: github/codeql-action/upload-sarif@v3   # was @v2
```

---

### WR-02: Unit test CI does not run `--features serde` — serde tests always skipped in CI

**File:** `.github/workflows/rust-unit-tests.yml:31`
**Issue:** `cargo nextest run` is invoked with no feature flags. Per CLAUDE.md: "All PRs must
pass: `cargo test`, `cargo test --features serde`". The serde feature gates checkpoint
serialization tests (`tests/observe/test_checkpoint.rs`, `tests/observe/test_serde.rs`, and
serde blocks in chromosome tests). These tests are never executed in CI, so serde regressions
can land undetected.

**Fix:**
```yaml
- name: Run tests (nextest)
  run: cargo nextest run

- name: Run tests with serde feature (nextest)
  run: cargo nextest run --features serde
```

---

### WR-03: `nextest` installed in `wasm-check.yml` but never used

**File:** `.github/workflows/wasm-check.yml:34-35`
**Issue:** The step `uses: taiki-e/install-action@nextest` installs nextest, but no subsequent
step invokes `cargo nextest`. All actual checks are `cargo check --target wasm32-unknown-unknown
--lib [--features ...]`. The WASM target cannot run tests (no `std::thread`, no runtime), so
nextest install here is dead code that adds install latency to every wasm-check run with zero
benefit.

**Fix:** Remove the nextest install step from `wasm-check.yml`:
```yaml
# DELETE this step entirely:
# - name: Install nextest
#   uses: taiki-e/install-action@nextest
```

---

### WR-04: `docs/DEVELOPMENT.md` CI table still documents old `cargo test --verbose` commands

**File:** `docs/DEVELOPMENT.md:140-143`
**Issue:** The CI Workflows table at line 140 states that `rust-unit-tests.yml` runs
"`cargo build --verbose`, `cargo test --verbose`". Since Phase 67, the workflow now runs
`cargo nextest run` and no longer runs `cargo test --verbose`. The table is factually wrong and
will mislead contributors who look there to understand what CI checks.

**Fix:** Update the table row:
```markdown
| `rust-unit-tests.yml` | PR targeting `main` | `cargo build --verbose`, `cargo nextest run` |
```

---

### WR-05: `log` crate `kv_unstable` feature flag — stability risk on version bumps

**File:** `Cargo.toml:43`
**Issue:** `log = { version = "0.4.22", features = ["std", "serde", "kv_unstable"] }`. The
`kv_unstable` feature is explicitly marked unstable in the `log` crate and its API has changed
between minor versions of `log 0.4`. Since `Cargo.toml` specifies `"0.4.22"` with SemVer
compatible (`^0.4.22`), any `0.4.x` patch release could modify the `kv_unstable` API and
silently break compilation for all users of this crate. This is a published library — the
stability risk flows to every downstream consumer.

**Fix:** Either pin `log` to an exact version (`= "0.4.22"`) or remove `kv_unstable` and use
only the stable structured logging API. The `kv_unstable` feature is used for structured
key-value log pairs (e.g., `info!(target="ga_events", key = val)`); if that syntax is required,
pin the exact version and document the intentional instability:
```toml
log = { version = "=0.4.22", features = ["std", "serde", "kv_unstable"] }
```

---

## Info

### IN-01: `examples-smoke.yml` missing `memetic_rastrigin`, `cma_es_rastrigin`, `ipop_rastrigin`, `pso_rastrigin`, `eda_trap`, `surrogate_rastrigin` from smoke matrix

**File:** `.github/workflows/examples-smoke.yml:22-31`
**Issue:** `Cargo.toml` declares six `[[example]]` entries added in v3.0.0
(`memetic_rastrigin`, `cma_es_rastrigin`, `ipop_rastrigin`, `pso_rastrigin`, `eda_trap`,
`surrogate_rastrigin`) but none appear in the `matrix.example` list. Two of these
(`sms_emoa_zdt1`, `ibea_zdt1`) require `--features benchmarks` which may justify omission, but
`memetic_rastrigin`, `cma_es_rastrigin`, `pso_rastrigin`, `eda_trap`, and `surrogate_rastrigin`
have no required-features guard. A broken example in these will not be caught by smoke tests.

**Fix:** Add the unconstrained examples to the matrix, or open a tracking issue if they are
intentionally excluded (with a comment in the workflow).

---

### IN-02: CHANGELOG `[Unreleased]` section is nested above `[3.0.0] - Unreleased` without a version anchor

**File:** `CHANGELOG.md:8-21` and `23`
**Issue:** The file has two "unreleased" sections: a top-level `## [Unreleased]` (Phase 67
entries) and a `## [3.0.0] - Unreleased` section below it. The `[Unreleased]` comparison link
at line 567 points to `compare/2.4.0...HEAD`. If Phase 67 changes are released as part of
v3.0.0, the `[Unreleased]` block content must be migrated into `[3.0.0]` before tagging, or
the Phase 67 entries will be orphaned above the version that ships them. The current structure
risks changelog entries being omitted from the v3.0.0 release notes.

**Fix:** Merge the `[Unreleased]` Phase 67 entries into the `## [3.0.0] - Unreleased` section
now, since v3.0.0 is the next planned release and Phase 67 is part of that milestone.

---

_Reviewed: 2026-06-14T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
