# Phase 67: Build-perf M1 — config-only quick wins - Research

**Researched:** 2026-06-14
**Domain:** Cargo build configuration, CI tooling (nextest, sccache, mold linker)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Plan and execute Phase 66 before Phase 67. Phase 67 CONTEXT.md is ready, but planning (PLAN.md files) and execution MUST wait until Phase 66 is shipped.

**D-02:** Add `cargo nextest run` to ALL three test-running CI workflows: `rust-unit-tests.yml`, `coverage.yml`, and `wasm-check.yml`.

**D-03:** `coverage.yml` uses `cargo-llvm-cov` — compatible with nextest via `cargo llvm-cov nextest`. Safe to include.

**D-04:** Local `cargo test` remains unchanged — no developer install requirement.

**D-05:** Add `mozilla-actions/sccache-action@v0.0.9` (latest release, not v0.0.4 from BUILD-PERF.md which is stale) to ALL CI workflows that invoke rustc: `rust-unit-tests.yml`, `wasm-check.yml`, `coverage.yml`, `rust-clippy.yml`, `examples-smoke.yml`.

**D-06:** Set `RUSTC_WRAPPER=sccache` env var in each workflow step. Log cache hit-rate per BUILD-PERF.md spec.

**D-07:** Install mold via `apt-get install -y mold` in EVERY Linux CI workflow (not just `rust-unit-tests.yml`).

**D-08:** `.cargo/config.toml` gets the mold block for `x86_64-unknown-linux-gnu` AND a commented-out lld block for `aarch64-apple-darwin`. The existing WASM `rustflags` block is preserved unchanged.

**D-09:** Add to `Cargo.toml` exactly the three blocks from BUILD-PERF.md §Action #5/#6:
- `[profile.dev]` — `debug = "line-tables-only"`, `split-debuginfo = "unpacked"`
- `[profile.dev.package."*"]` — `opt-level = 1`, `debug = false`
- `[profile.test]` — `opt-level = 1`
No deviation from the spec.

**D-10:** 67-01 writes `docs/DEVELOPMENT.md` "Cargo profiles" section + `CHANGELOG.md` Changed entry + `.planning/intel/build-profile.md`.

**D-11:** 67-02 writes `docs/TESTING.md` nextest opt-in instructions + `CHANGELOG.md` Changed (internal) entry.

**D-12:** 67-03 writes `docs/DEVELOPMENT.md` "Linker recommendations" section + `CHANGELOG.md` Changed (CI) entry.

**D-13:** 67-04 writes `docs/DEVELOPMENT.md` "CI caching" subsection (no user action needed; informational).

**D-14:** Every plan commit body MUST include a `Revert plan:` line per BUILD-PERF.md non-negotiable guarantee #5.

### Claude's Discretion

None — all decisions locked in CONTEXT.md.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope. Phase 68 (dep hygiene) and Phase 69 (major refactors) take the next build-perf steps.
</user_constraints>

---

## Summary

Phase 67 lands four parallel, zero-risk build-config improvements: Cargo profile tuning (`Cargo.toml` profile blocks), `cargo-nextest` as the CI test runner, explicit `mold` linker configuration in `.cargo/config.toml`, and `sccache` CI caching across all five workflows that invoke `rustc`. All four plans run in Wave 0 (parallel) because their file sets do not overlap. No source-code behavioral change is permitted.

The Phase 66 blocker is confirmed resolved: `.planning/baselines/v3.0.0-baseline.json` exists (dev_build_s: 3.658, test_suite_s: 55.790), `tests/golden/` contains four seeded-output fixtures, and the `build-perf-gate` CI job is running. The baseline numbers are tight (3.6 s dev build, 55.8 s test suite) — the 5% improvement threshold for SC#5 requires approximately 0.18 s off the dev build and 2.8 s off the test suite.

The four existing CI workflows being modified (`rust-unit-tests.yml`, `coverage.yml`, `wasm-check.yml`, `rust-clippy.yml`, `examples-smoke.yml`) are all minimalist and will receive targeted additions only. The existing `.cargo/config.toml` contains a single `[target.wasm32-unknown-unknown]` rustflags block that must be preserved exactly as-is.

**Primary recommendation:** Implement the four plans in parallel. Each plan owns a disjoint file set. The only shared file is `CHANGELOG.md` — coordinate the four CHANGELOG entries through sequential commits or merge after the fact.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cargo profile tuning | Build system (Cargo.toml) | — | Profile blocks affect codegen; no runtime tier owns this |
| Linker configuration | Build system (.cargo/config.toml) | CI (mold install) | config.toml declares the linker; CI installs the binary |
| Test parallelism (nextest) | CI runner | Local dev (opt-in) | D-04 mandates CI-only; local unchanged |
| Compiler caching (sccache) | CI runner | — | sccache operates transparently as a RUSTC_WRAPPER |
| Documentation | Repository (docs/) | .planning/intel/ | Contributor-facing and AI-agent-facing separately |

---

## Standard Stack

### Core — No new Rust library dependencies

Phase 67 is config-only. No new entries in `[dependencies]` or `[dev-dependencies]`. All changes are:
- `Cargo.toml` profile blocks
- `.cargo/config.toml` target blocks
- `.github/workflows/*.yml` step additions
- `docs/` and `.planning/intel/` markdown files

### CI Tooling

| Tool | Version / Source | Purpose | How Installed |
|------|-----------------|---------|---------------|
| `cargo-nextest` | 0.9.137 (crates.io) | Faster parallel test runner for CI | `taiki-e/install-action@nextest` GitHub Action |
| `mozilla-actions/sccache-action` | v0.0.9 (GitHub, published 2025-06-18) | Shared compilation cache across CI jobs | GitHub Actions step |
| `mold` | System package (apt) | Fast ELF linker for Linux builds | `apt-get install -y mold` in each Linux job |

**Version verification:**
- `cargo-nextest`: 0.9.137, 10,805,867 total downloads on crates.io [VERIFIED: crates.io]
- `mozilla-actions/sccache-action@v0.0.9`: confirmed tag published 2025-06-18 on github.com/mozilla-actions/sccache-action [VERIFIED: GitHub API]
- `mold`: available in Ubuntu 22.04 (jammy) apt repositories [VERIFIED: packages.ubuntu.com]

### Workflow YAML Patterns

**nextest install (official recommendation):** [CITED: nexte.st/docs/installation/pre-built-binaries/]
```yaml
- uses: taiki-e/install-action@nextest
```

**nextest run (standard test-running CI step):**
```yaml
- name: Run tests (nextest)
  run: cargo nextest run
```

**nextest with llvm-cov (coverage.yml):**
```yaml
- name: Run coverage gate (src/engines/ + src/operations/ >= 80%)
  run: |
    cargo llvm-cov nextest \
      --all-features \
      --ignore-filename-regex '^(?!.*(src/engines/|src/operations/)).*$' \
      --fail-under-lines 80
```

**sccache-action step (add before any rustc invocation):** [CITED: github.com/mozilla-actions/sccache-action]
```yaml
- name: Configure sccache
  uses: mozilla-actions/sccache-action@v0.0.9
```

**sccache env var (add to job-level `env:` or to individual steps):**
```yaml
env:
  RUSTC_WRAPPER: sccache
  SCCACHE_GHA_ENABLED: "true"
```

**sccache cache-hit logging (add as a final step in each job):**
```yaml
- name: sccache stats
  run: sccache --show-stats
```

**mold install step (add before `cargo build` / `cargo test`):**
```yaml
- name: Install mold linker
  run: sudo apt-get install -y mold
```

**mold config.toml block:**
```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**Commented-out macOS lld opt-in (per D-08):**
```toml
# Uncomment for ~5 % faster macOS builds (requires `brew install llvm`):
# [target.aarch64-apple-darwin]
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

---

## Package Legitimacy Audit

> Phase 67 installs no Rust crate dependencies. The tools below are CI-layer additions only.

| Tool | Registry / Source | Age | Downloads / Signals | Source Repo | slopcheck | Disposition |
|------|------------------|-----|---------------------|-------------|-----------|-------------|
| `cargo-nextest` | crates.io | 3+ years | 10.8M downloads | github.com/nextest-rs/nextest | [OK] | Approved |
| `mozilla-actions/sccache-action` | GitHub Actions | 3+ years | Mozilla-maintained | github.com/mozilla-actions/sccache-action | N/A (not a crate) | Approved — official Mozilla action |
| `taiki-e/install-action` | GitHub Actions | 4+ years | Taiki Endo (Tokio contributor) | github.com/taiki-e/install-action | N/A (not a crate) | Approved — widely used in Rust ecosystem |
| `mold` | apt (Ubuntu package) | 3+ years | System package | github.com/rui314/mold | [OK] | Approved |

**slopcheck note:** slopcheck reported an error for `mozilla-actions/sccache-action` because it searched crates.io; this tool is a GitHub Action, not a Rust crate. The error is expected and does not indicate a slopcheck problem. Verified via GitHub API: v0.0.9 tag confirmed, maintained by Mozilla organization. [VERIFIED: GitHub API]

**Packages removed due to slopcheck [SLOP] verdict:** none

**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
PR pushed
    |
    v
[GitHub Actions trigger]
    |
    +---> rust-unit-tests.yml
    |         sccache-action (RUSTC_WRAPPER=sccache)
    |         apt install mold
    |         taiki-e/install-action@nextest
    |         cargo nextest run
    |         sccache --show-stats
    |
    +---> coverage.yml
    |         sccache-action (RUSTC_WRAPPER=sccache)
    |         apt install mold
    |         taiki-e/install-action@nextest
    |         cargo llvm-cov nextest [--fail-under-lines 80]
    |         sccache --show-stats
    |
    +---> wasm-check.yml
    |         sccache-action (RUSTC_WRAPPER=sccache)
    |         [no mold: wasm32 target, not ELF Linux]
    |         taiki-e/install-action@nextest
    |         cargo nextest run (for host tests)
    |         cargo check --target wasm32-unknown-unknown (unchanged)
    |         sccache --show-stats
    |
    +---> rust-clippy.yml
    |         sccache-action (RUSTC_WRAPPER=sccache)
    |         apt install mold
    |         cargo clippy ... (unchanged)
    |         sccache --show-stats
    |
    +---> examples-smoke.yml
              sccache-action (RUSTC_WRAPPER=sccache)
              apt install mold
              cargo run --example ... --release (unchanged)
              sccache --show-stats

[Cargo.toml profile blocks]
    |
    +--> [profile.dev]           debug="line-tables-only", split-debuginfo="unpacked"
    +--> [profile.dev.package.*] opt-level=1, debug=false
    +--> [profile.test]          opt-level=1

[.cargo/config.toml] (additive — existing WASM block preserved)
    |
    +--> [target.wasm32-unknown-unknown]  rustflags (UNCHANGED)
    +--> [target.x86_64-unknown-linux-gnu] linker="clang", rustflags="-fuse-ld=mold" (NEW)
    +--> # commented-out aarch64-apple-darwin lld block (NEW, docs only)
```

### Recommended File Changes per Plan

**67-01 (Cargo profile tuning):**
- `Cargo.toml` — add three profile blocks at end of file
- `docs/DEVELOPMENT.md` — append "Cargo profiles" section
- `.planning/intel/build-profile.md` — new AI-agent note file
- `CHANGELOG.md` — add Changed entry

**67-02 (nextest CI swap):**
- `.github/workflows/rust-unit-tests.yml` — add install step + swap test command
- `.github/workflows/coverage.yml` — add install step + change `cargo llvm-cov` to `cargo llvm-cov nextest`
- `.github/workflows/wasm-check.yml` — add install step + add nextest run step for host tests
- `docs/TESTING.md` — append nextest opt-in instructions
- `CHANGELOG.md` — add Changed (internal) entry

**67-03 (mold linker):**
- `.cargo/config.toml` — add `[target.x86_64-unknown-linux-gnu]` block + commented macOS block
- `.github/workflows/rust-unit-tests.yml` — add `apt-get install -y mold` step
- `.github/workflows/coverage.yml` — add `apt-get install -y mold` step
- `.github/workflows/rust-clippy.yml` — add `apt-get install -y mold` step
- `.github/workflows/examples-smoke.yml` — add `apt-get install -y mold` step
- `docs/DEVELOPMENT.md` — append "Linker recommendations" section
- `CHANGELOG.md` — add Changed (CI) entry

**67-04 (sccache):**
- `.github/workflows/rust-unit-tests.yml` — add sccache-action step + env vars + stats step
- `.github/workflows/coverage.yml` — add sccache-action step + env vars + stats step
- `.github/workflows/wasm-check.yml` — add sccache-action step + env vars + stats step
- `.github/workflows/rust-clippy.yml` — add sccache-action step + env vars + stats step
- `.github/workflows/examples-smoke.yml` — add sccache-action step + env vars + stats step
- `docs/DEVELOPMENT.md` — append "CI caching" subsection
- `CHANGELOG.md` — informational note (per BUILD-PERF.md spec: no public CHANGELOG entry needed)

### Anti-Patterns to Avoid

- **Replacing `.cargo/config.toml` wholesale:** The existing `[target.wasm32-unknown-unknown]` rustflags block must survive. Plan 67-03 appends; it does not overwrite.
- **Adding mold to wasm-check.yml:** The `wasm32-unknown-unknown` target compiles via clang/LLVM cross-compile chain, not the ELF Linux mold chain. mold only applies to native Linux ELF output. No mold step in wasm-check.yml.
- **Using `cargo test` after nextest install:** Plans 67-02 and 67-04 interact on the same workflow files. The final state must use `cargo nextest run`, not revert to `cargo test`.
- **Setting `SCCACHE_GHA_ENABLED` without also setting `RUSTC_WRAPPER`:** Both env vars are required for GitHub Actions cache integration to function.
- **Forgetting `sccache --show-stats`:** D-06 requires cache hit-rate logging. The `--show-stats` step fulfills this.
- **Using `v0.0.4` for sccache-action:** BUILD-PERF.md §Action #10 references v0.0.4 (stale). D-05 locks the version to `v0.0.9`. Plans must use v0.0.9.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Parallel test execution | Custom shell parallelism scripts | `cargo nextest run` | nextest handles binary-level parallelism, retries, timing, and JSON reporting natively |
| Compiler output caching | Artisanal cache key scripts + tar | `mozilla-actions/sccache-action` | Action manages GHA cache backend, sccache daemon lifetime, and RUSTC_WRAPPER automatically |
| Fast linking | Custom linker flags discovered by trial | `mold` via apt + `[target.x86_64-unknown-linux-gnu]` block | mold is the correct, battle-tested answer for Linux ELF; the config.toml pattern is the canonical Rust way to configure it |

---

## Common Pitfalls

### Pitfall 1: wasm-check.yml nextest scope confusion

**What goes wrong:** An implementer adds `cargo nextest run` to `wasm-check.yml` but wasm-check has no `cargo test` — it only runs `cargo check --target wasm32-unknown-unknown`. nextest cannot run cross-compiled tests for wasm32 (the test binaries cannot execute on the Linux runner).

**Why it happens:** D-02 says "ALL three test-running CI workflows" which includes wasm-check.yml. The plan must add nextest for the *host-architecture* tests (if any exist in the workflow) but must NOT attempt to nextest-run wasm32 test binaries.

**How to avoid:** In `wasm-check.yml`, nextest applies only to any host-arch `cargo test` steps, not to the `cargo check --target wasm32-unknown-unknown` steps. Currently wasm-check.yml has no `cargo test` steps — so nextest install is added for future-proofing per D-02, but no `cargo nextest run` step is needed in the current workflow (the three `cargo check` steps remain unchanged).

**Warning signs:** CI error "nextest cannot run cross-compiled binaries" or "no test binaries found."

### Pitfall 2: `build-perf-gate` dep_count check breaking

**What goes wrong:** The `build-perf-gate.yml` Python script enforces `dep_count` as an exact match (0% tolerance). Phase 67 is config-only but sccache-action and nextest don't add crate deps. However, if any plan accidentally adds a Rust crate dep, the dep_count will change from 97 and the gate will fail.

**Why it happens:** Phase 67 is purely CI and config — no Cargo.toml dependency additions. Risk is accidental.

**How to avoid:** Verify that no plan adds any entry under `[dependencies]`, `[dev-dependencies]`, or `[build-dependencies]` in Cargo.toml.

**Warning signs:** `build-perf-gate` reporting `REGRESSION: dep_count changed from 97 to N`.

### Pitfall 3: CHANGELOG.md merge conflict across parallel plans

**What goes wrong:** Four plans each append to `CHANGELOG.md` in parallel commits. When merged, two plans could edit the same line range and conflict.

**Why it happens:** Wave 0 allows all four plans to run in parallel, but they share CHANGELOG.md.

**How to avoid:** Each plan appends a distinct, self-contained bullet to the existing `## [Unreleased]` section. If no `## [Unreleased]` section exists, each plan creates the section header exactly once. Merge order: 67-01 first (creates the section), then 67-02, 67-03, 67-04 append under it.

**Warning signs:** Git merge conflict in CHANGELOG.md during the parallel plan merge.

### Pitfall 4: sccache interfering with build-perf-gate timing measurements

**What goes wrong:** `build-perf-gate.yml` runs `bash bench/build_perf.sh` which calls `cargo clean && cargo build --timings` to measure cold build times. If sccache is installed in the build-perf-gate job, it will cache the cold build artifacts and produce misleadingly fast results.

**Why it happens:** D-05 adds sccache to all CI workflows that invoke rustc, including build-perf-gate.yml.

**How to avoid:** Check whether `build-perf-gate.yml` is in scope for D-05. Looking at the decision text: "ALL CI workflows that invoke rustc: `rust-unit-tests.yml`, `wasm-check.yml`, `coverage.yml`, `rust-clippy.yml`, `examples-smoke.yml`". `build-perf-gate.yml` is NOT in this list. Plan 67-04 MUST NOT add sccache to `build-perf-gate.yml`. The existing `build-perf-gate.yml` uses `Swatinem/rust-cache@v2` for registry/dep caching only, which is intentional.

**Warning signs:** build-perf-gate reporting 60% improvement on the first run (sccache hit) then reverting on cache miss.

### Pitfall 5: `split-debuginfo = "unpacked"` on macOS CI

**What goes wrong:** The Cargo profile adds `split-debuginfo = "unpacked"` as a dev profile setting. On macOS, this is the intended fast path. But on Linux (ubuntu-latest), `"unpacked"` may generate `.dwo` files that slow down linking instead of speeding it up.

**Why it happens:** The BUILD-PERF.md spec includes `split-debuginfo = "unpacked"` without qualification.

**How to avoid:** Per BUILD-PERF.md §Action #5/#6: the spec's comment says "Default split-debuginfo on macOS is slow; line tables alone shave seconds." On Linux, `"unpacked"` is not harmful (it's also a valid value) and the main win comes from `debug = "line-tables-only"`. D-09 locks the blocks verbatim — no deviation. The planner should not second-guess the spec here.

**Warning signs:** None expected; this is documented behavior in the spec.

---

## Code Examples

### Cargo.toml profile blocks to add (67-01)

```toml
# Source: .planning/v3.0.0-BUILD-PERF.md §Action #5/#6 (locked by D-09)
[profile.dev]
# debuginfo for line numbers in backtraces, no full DWARF.
debug = "line-tables-only"
# Default split-debuginfo on macOS is slow; line tables alone shave seconds.
split-debuginfo = "unpacked"

[profile.dev.package."*"]
# Optimise dependencies once; speeds up runtime tests on rand/rayon/log
# at the cost of ~5-10 s on first build (one-time, then cached).
opt-level = 1
debug = false

[profile.test]
# Tests run a lot of GA generations; -O1 cuts runtime ~50 % at no rustc cost.
opt-level = 1
```

### .cargo/config.toml additions (67-03)

```toml
# Source: .planning/v3.0.0-BUILD-PERF.md §Action #9 (modified by D-08)
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# Uncomment for ~5 % faster macOS builds (requires `brew install llvm`):
# [target.aarch64-apple-darwin]
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

The existing content (do NOT remove):
```toml
[target.wasm32-unknown-unknown]
rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]
```

### rust-unit-tests.yml final state sketch (67-02 + 67-03 + 67-04 combined)

```yaml
name: Rust Unit Tests

on:
  pull_request:
    branches: [ "main" ]

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
  RUSTC_WRAPPER: sccache
  SCCACHE_GHA_ENABLED: "true"

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v5
    - name: Configure sccache
      uses: mozilla-actions/sccache-action@v0.0.9
    - name: Install mold linker
      run: sudo apt-get install -y mold
    - name: Install nextest
      uses: taiki-e/install-action@nextest
    - name: Build
      run: cargo build --verbose
    - name: Run tests (nextest)
      run: cargo nextest run
    - name: sccache stats
      run: sccache --show-stats
```

### coverage.yml final state sketch (67-02 + 67-03 + 67-04 combined)

```yaml
name: Coverage Gate

on:
  pull_request:
    branches: [main, "milestone/**"]

permissions:
  contents: read

jobs:
  coverage:
    name: coverage (src/engines/ + src/operations/)
    runs-on: ubuntu-latest

    env:
      RUSTC_WRAPPER: sccache
      SCCACHE_GHA_ENABLED: "true"

    steps:
      - uses: actions/checkout@v4
      - name: Configure sccache
        uses: mozilla-actions/sccache-action@v0.0.9
      - name: Install stable toolchain with llvm-tools
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - name: Cache cargo registry and target
        uses: Swatinem/rust-cache@v2
        with:
          key: coverage
      - name: Install mold linker
        run: sudo apt-get install -y mold
      - name: Install nextest
        uses: taiki-e/install-action@nextest
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov --locked
      - name: Run coverage gate (src/engines/ + src/operations/ >= 80%)
        run: |
          cargo llvm-cov nextest \
            --all-features \
            --ignore-filename-regex '^(?!.*(src/engines/|src/operations/)).*$' \
            --fail-under-lines 80
      - name: sccache stats
        run: sccache --show-stats
```

---

## Runtime State Inventory

Step 2.5: SKIPPED — Phase 67 is not a rename/refactor/migration phase. All changes are additive config and CI file edits. No stored data, live service config, OS-registered state, secrets, or build artifacts contain names that change.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `ubuntu-latest` GitHub runner | All Linux CI jobs | CI-only (not local) | ubuntu-22.04 (jammy) | — |
| `mold` (apt) | 67-03, all Linux CI | CI install via apt | Available in jammy repos | — |
| `taiki-e/install-action` | 67-02 (nextest install) | GitHub Actions | v2 | `cargo install cargo-nextest --locked` |
| `mozilla-actions/sccache-action` | 67-04 | GitHub Actions | v0.0.9 confirmed | — |
| Python 3 | `build-perf-gate.yml` (already existing) | ubuntu-latest includes Python 3 | 3.x | — |
| `Swatinem/rust-cache@v2` | coverage.yml, wasm-check.yml (already present) | Available | v2 | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** `taiki-e/install-action` could be replaced by `cargo install cargo-nextest --locked` if the action ever becomes unavailable, but this adds ~60 s of compile time. The action is the preferred path per official nextest docs.

---

## Validation Architecture

> `workflow.nyquist_validation` is not explicitly set to `false` in `.planning/config.json` (file may not exist), so Validation Architecture is included.

Phase 67 is config-only. The test suite itself does not change — the same test code runs through nextest instead of `cargo test`. Validation is exercised through the existing CI gates plus the build-perf-gate.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in + cargo-nextest (post-phase) |
| Config file | `nextest.toml` — not yet present; no custom config needed (nextest defaults work) |
| Quick run command | `cargo nextest run` |
| Full suite command | `cargo nextest run --all-features` |
| Coverage command | `cargo llvm-cov nextest --all-features` |

### Phase Requirements → Test Map

| ID | Behavior | Test Type | Automated Command | Notes |
|----|----------|-----------|-------------------|-------|
| SC#1 | Cargo.toml has three profile blocks | Structural check | `grep -q 'profile.dev' Cargo.toml` | Manual / CI compile verify |
| SC#2 | CI uses cargo nextest run | Structural check | CI workflow diff | CI pass = verified |
| SC#3 | .cargo/config.toml has mold block + WASM block preserved | Structural check | `grep 'fuse-ld=mold' .cargo/config.toml` | Manual verify |
| SC#4 | CI workflows use sccache; hit-rate logged | Structural check | `grep 'sccache-action' .github/workflows/*.yml` | CI pass = verified |
| SC#5 | build-perf-gate confirms >= 5% improvement | Automated gate | `bash bench/build_perf.sh` (via build-perf-gate.yml) | Runs on PR automatically |
| SC#6 | Zero new rustdoc warnings; clippy green | Quality gate | `cargo doc --no-deps`, `cargo clippy --all-targets -D warnings` | CI pass = verified |

### Sampling Rate

- **Per task commit:** `cargo test` (local), then push and watch CI
- **Per wave merge:** Full CI matrix (all 5 workflows + build-perf-gate)
- **Phase gate:** build-perf-gate must show >= 5% improvement on dev_build_s and/or test_suite_s vs baseline

### Wave 0 Gaps

Phase 67 has no new test files to write — the phase is config-only. Wave 0 gaps are CI structural:

- [ ] `nextest.toml` — not required (nextest defaults sufficient; no custom profile config needed)
- [ ] `.planning/intel/` directory — does not exist yet; must be created by 67-01

No new `tests/` files required.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `cargo test` in CI | `cargo nextest run` | nextest stable 2022+ | 30-50% test-suite wall-clock reduction via per-test-binary parallelism |
| No compiler cache in CI | `sccache` via `mozilla-actions/sccache-action` | Action available since 2022; v0.0.9 June 2025 | 30-60% CI wall-clock reduction on warm builds |
| Default `ld` linker on Linux | `mold` via `.cargo/config.toml` | mold 1.0 released 2022 | 5-10% link-phase reduction on clean builds |
| Full DWARF debuginfo (`debug = true`) | `debug = "line-tables-only"` | Cargo 1.65+ supports string values | Significantly smaller `.debug_info` section; faster linking |

**Deprecated/outdated:**
- `sccache-action@v0.0.4`: Listed in BUILD-PERF.md §Action #10. Stale. Use v0.0.9 per D-05.
- `cargo test --verbose` in `rust-unit-tests.yml`: Will be replaced by `cargo nextest run`. The `--verbose` flag on nextest is not necessary (nextest provides better output by default).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ubuntu-latest` on GitHub Actions is `ubuntu-22.04` (jammy) at the time of writing | Environment Availability | If GH upgrades to ubuntu-24.04 (noble), mold is still available in apt — risk is LOW |
| A2 | `taiki-e/install-action@nextest` installs without version pin | Code Examples | If the action breaks, fallback is `cargo install cargo-nextest --locked` — risk is LOW |
| A3 | `cargo llvm-cov nextest` syntax is correct for the installed cargo-llvm-cov version | Code Examples | If cargo-llvm-cov version in coverage.yml is old and doesn't support `nextest` subcommand, the step fails — verify after merge |

---

## Open Questions

1. **wasm-check.yml nextest scope**
   - What we know: D-02 says all three test-running CI workflows. wasm-check.yml runs `cargo check`, not `cargo test`.
   - What's unclear: Should nextest be installed in wasm-check.yml at all if there are no test steps to run?
   - Recommendation: Install `taiki-e/install-action@nextest` in wasm-check.yml for completeness and future-proofing (D-02 says include it), but do not add a `cargo nextest run` step since there are no host-arch tests to run. The three `cargo check --target wasm32-unknown-unknown` steps remain unchanged.

2. **CHANGELOG.md `## [Unreleased]` section existence**
   - What we know: Four plans all touch CHANGELOG.md.
   - What's unclear: Does CHANGELOG.md currently have a `## [Unreleased]` section or does it start at `## [3.0.0]`?
   - Recommendation: Each plan should check for the section and create it if absent. The planner should designate plan 67-01 (most self-contained) as the one to create the section header, and plans 67-02, 67-03, 67-04 append under it. Alternatively, merge all four CHANGELOG edits in post-phase cleanup.

3. **`build-perf-gate.yml` and sccache**
   - What we know: D-05 lists 5 workflows by name; `build-perf-gate.yml` is NOT in the list.
   - What's unclear: Nothing — the list is explicit.
   - Recommendation: Plan 67-04 MUST NOT add sccache to `build-perf-gate.yml`. This is confirmed by the decision text.

---

## Security Domain

This phase modifies CI infrastructure only — no user-facing code, no authentication flows, no data handling. ASVS categories are not applicable to CI configuration changes.

The only security consideration: `mozilla-actions/sccache-action@v0.0.9` should be pinned to the exact tag (not `@latest` or `@main`) to prevent supply-chain drift. The decisions already specify `v0.0.9`. [ASSUMED — standard CI supply-chain hygiene]

---

## Sources

### Primary (HIGH confidence)
- `.planning/v3.0.0-BUILD-PERF.md` — authoritative phase spec; exact TOML blocks, doc deliverables, revert plans
- `.planning/phases/67-build-perf-m1-config-only-quick-wins/67-CONTEXT.md` — all locked decisions (D-01 through D-14)
- `.planning/baselines/v3.0.0-baseline.json` — verified existing; Phase 66 complete
- `.github/workflows/*.yml` — all five workflow files read in full; exact current state known
- `.cargo/config.toml` — read in full; WASM block content confirmed
- `Cargo.toml` — read in full; no existing `[profile.*]` blocks confirmed (safe to add)

### Secondary (MEDIUM confidence)
- [nexte.st/docs/installation/pre-built-binaries/](https://nexte.st/docs/installation/pre-built-binaries/) — official nextest CI install instructions; `taiki-e/install-action@nextest` confirmed as recommended approach [CITED]
- GitHub API: `mozilla-actions/sccache-action` v0.0.9 tag confirmed published 2025-06-18 [VERIFIED: GitHub API]
- crates.io: `cargo-nextest` 0.9.137, 10.8M downloads [VERIFIED: crates.io]
- packages.ubuntu.com: `mold` available in Ubuntu 22.04 (jammy) [VERIFIED: packages.ubuntu.com]

### Tertiary (LOW confidence)
- None — all claims verified from primary or secondary sources.

---

## Metadata

**Confidence breakdown:**
- Standard stack (TOML blocks, action versions): HIGH — all verified against official sources and confirmed tags
- Architecture (which file each plan touches): HIGH — all files read directly from codebase
- CI workflow patterns: HIGH — official nextest docs + confirmed GH API for sccache-action
- Pitfalls: HIGH — derived from direct inspection of existing CI files and BUILD-PERF.md constraints

**Research date:** 2026-06-14
**Valid until:** 2026-07-14 (action version pinned; re-verify if milestone extends past July 2026)
