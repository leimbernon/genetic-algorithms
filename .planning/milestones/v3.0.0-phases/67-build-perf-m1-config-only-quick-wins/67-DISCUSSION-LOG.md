# Phase 67: Build-perf M1 — config-only quick wins - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 67-build-perf-m1-config-only-quick-wins
**Areas discussed:** Phase 66 ordering, Nextest scope, sccache scope + version, mold installation strategy

---

## Phase 66 Ordering

| Option | Description | Selected |
|--------|-------------|----------|
| Plan Phase 66 first, then 67 | Correct sequencing — Phase 67 plans can't verify SC#5 without the baseline. Plan and execute Phase 66, then come back. | ✓ |
| Plan both in parallel | Write Phase 67 plans now, mark SC#5 as blocked. Execute Phase 66 first before merging Phase 67 PRs. | |
| Skip SC#5 for now | Proceed with Phase 67, omit build-perf-gate verification. Accept unquantified improvement. | |

**User's choice:** Plan Phase 66 first, then 67
**Notes:** Phase 66 has no directory and hasn't been started. Phase 67 is contextualized and ready to plan, but execution must wait for Phase 66.

---

## Nextest Scope

| Option | Description | Selected |
|--------|-------------|----------|
| rust-unit-tests.yml only | Matches BUILD-PERF.md spec. nextest/coverage tooling compatibility risk with coverage.yml. | |
| rust-unit-tests.yml + wasm-check.yml | Adds nextest to both test-heavy jobs; supports wasm32 cross-target. | |
| All test-running workflows | rust-unit-tests.yml + coverage.yml + wasm-check.yml. Requires verifying nextest + coverage compatibility first. | ✓ |

**User's choice:** All test-running workflows
**Notes:** Codebase check confirmed `coverage.yml` uses `cargo-llvm-cov`, which supports nextest via `cargo llvm-cov nextest`. All three workflows can safely use nextest. User also chose to include `wasm-check.yml`.

### Follow-up: wasm-check.yml inclusion

| Option | Description | Selected |
|--------|-------------|----------|
| wasm-check.yml stays on cargo test | WASM tests run under a cross-target emulator; nextest adds complexity with minimal gain. | |
| Add nextest to wasm-check.yml too | nextest supports cross-target builds. Consistent approach across all CI jobs. | ✓ |

---

## sccache Scope + Version

### Scope

| Option | Description | Selected |
|--------|-------------|----------|
| All workflows that invoke rustc | rust-unit-tests.yml, wasm-check.yml, coverage.yml, rust-clippy.yml, examples-smoke.yml. Maximum cache reuse. | ✓ |
| rust-unit-tests.yml + wasm-check.yml only | Matches BUILD-PERF.md spec. Simpler; avoids potential conflicts. | |
| rust-unit-tests.yml + wasm-check.yml + rust-clippy.yml | Three biggest-impact jobs; coverage excluded due to llvm-cov conflict risk. | |

### Version

| Option | Description | Selected |
|--------|-------------|----------|
| Use latest release (v0.0.9) | More bug fixes, updated sccache binary. Lower risk of stale action. | ✓ |
| Pin to v0.0.4 as specified | Matches BUILD-PERF.md exactly. | |
| Pin to a specific SHA | Security best practice for third-party GitHub Actions. | |

**User's choice:** All rustc-invoking workflows; v0.0.9
**Notes:** BUILD-PERF.md spec was written when v0.0.4 was current. v0.0.9 is preferred.

---

## Mold Installation Strategy

### CI installation scope

| Option | Description | Selected |
|--------|-------------|----------|
| Install in all Linux CI workflows | Every job that builds Rust on Linux benefits from mold. 5s apt-get overhead per job; net win positive. | ✓ |
| rust-unit-tests.yml only | Matches BUILD-PERF.md spec. Minimal surface area. | |
| Use a GitHub Action for mold | e.g. rui314/setup-mold@v1. Cleaner but adds a third-party action dependency. | |

### macOS section in .cargo/config.toml

| Option | Description | Selected |
|--------|-------------|----------|
| Include commented-out macOS lld block | Documents opt-in for local macOS devs. Matches BUILD-PERF.md spec. | ✓ |
| Linux-only, no macOS section | macOS 15+ fast linker is good enough. Avoids noise. | |

**User's choice:** All Linux CI workflows via apt-get; include commented-out macOS lld block
**Notes:** Consistent install approach across all Linux jobs. macOS comment documents the opt-in path without activating it.

---

## Claude's Discretion

None — all areas had clear user selections.

## Deferred Ideas

None — discussion stayed within Phase 67 scope.
