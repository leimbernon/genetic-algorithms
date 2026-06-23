---
phase: 71
slug: per-operator-mutation-params
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-18
---

# Phase 71 — Validation Strategy

> All tasks carry inline `<automated>` verify commands. No Wave 0 setup needed — existing cargo test infrastructure covers all phase requirements.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo build` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30–60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo build`
- **After every plan wave:** Run full CI suite
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

All tasks have inline `<automated>` verify commands in their PLAN.md files. Per-task coverage:

| Task ID | Plan | Wave | Test Type | Automated Command |
|---------|------|------|-----------|-------------------|
| 71-01-01 | 01 | 1 | compile | `cargo build` |
| 71-02-01 | 02 | 2 | compile | `cargo build` |
| 71-03-01 | 03 | 2 | integration | `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps && cargo check --target wasm32-unknown-unknown && cargo bench --no-run` |

*All phase behaviors have automated verification via `cargo test`.*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements. No additional test setup needed.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify commands in PLAN.md
- [x] Sampling continuity: every task has automated verify
- [x] No Wave 0 gaps
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-18
