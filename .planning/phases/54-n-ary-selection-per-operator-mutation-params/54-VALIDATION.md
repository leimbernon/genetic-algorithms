---
phase: 54
slug: n-ary-selection-per-operator-mutation-params
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-28
---

# Phase 54 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 54-01-01 | 01 | 1 | N-ary selection | — | N/A | compile+unit | `cargo test` | ✅ | ⬜ pending |
| 54-01-02 | 01 | 1 | N-ary selection | — | N/A | compile+unit | `cargo test` | ✅ | ⬜ pending |
| 54-02-01 | 02 | 2 | Mutation params | — | N/A | compile+unit | `cargo test && cargo test --features serde` | ✅ | ⬜ pending |
| 54-02-02 | 02 | 2 | Mutation params | — | N/A | compile+unit | `cargo test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test files needed — all changes are verified by the existing test suite via compile-error-driven coverage. The compiler catches all call site breakages when trait signatures and enum variants change.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WASM target compiles | WASM compat | No wasm test in CI for this phase | `cargo check --target wasm32-unknown-unknown` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
