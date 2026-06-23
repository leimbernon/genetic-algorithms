---
phase: 67
slug: build-perf-m1-config-only-quick-wins
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-14
---

# Phase 67 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test / cargo nextest / cargo clippy |
| **Config file** | Cargo.toml, .cargo/config.toml, .github/workflows/*.yml |
| **Quick run command** | `cargo test --quiet` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy --all-targets -D warnings && cargo doc --no-deps` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --quiet`
- **After every plan wave:** Run full suite + `cargo check --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full suite must be green + golden tests byte-identical
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 67-01-01 | 01 | 0 | SC#1 | — | N/A | build | `cargo build --quiet` | ✅ | ⬜ pending |
| 67-01-02 | 01 | 0 | SC#1 | — | N/A | test | `cargo test --quiet` | ✅ | ⬜ pending |
| 67-01-03 | 01 | 0 | SC#1 | — | N/A | wasm | `cargo check --target wasm32-unknown-unknown` | ✅ | ⬜ pending |
| 67-02-01 | 02 | 0 | SC#2 | — | N/A | ci-check | CI workflow file contains `nextest` step | ✅ | ⬜ pending |
| 67-02-02 | 02 | 0 | SC#2 | — | N/A | test | `cargo test --quiet` (local unchanged) | ✅ | ⬜ pending |
| 67-03-01 | 03 | 0 | SC#3 | — | N/A | config | `.cargo/config.toml` has mold block + WASM block intact | ✅ | ⬜ pending |
| 67-03-02 | 03 | 0 | SC#3 | — | N/A | build | `cargo build --quiet` (no linker errors) | ✅ | ⬜ pending |
| 67-04-01 | 04 | 0 | SC#4 | — | N/A | ci-check | CI workflow files contain sccache-action step | ✅ | ⬜ pending |
| 67-04-02 | 04 | 0 | SC#4 | — | N/A | test | `cargo test --quiet` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] All four plans run in parallel (Wave 0 — no inter-plan dependencies)
- [ ] Existing test suite (`cargo test`, `cargo test --features serde`) must remain green after each plan
- [ ] WASM check (`cargo check --target wasm32-unknown-unknown`) must remain green after each plan
- [ ] Golden tests (`tests/golden/`) must remain byte-identical after each plan

*Existing infrastructure covers all phase requirements — no new test framework needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Build-perf-gate ≥5% improvement | SC#5 | Requires CI run with baseline comparison | Push branch and verify build-perf-gate.yml CI job passes |
| Cache hit-rate logged in CI | SC#4 | Requires live CI run | Check CI run log for sccache cache hit/miss output |
| CI nextest swap works on all 3 workflows | SC#2 | Requires CI run | Push branch and verify rust-unit-tests.yml, coverage.yml, wasm-check.yml all use nextest |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
