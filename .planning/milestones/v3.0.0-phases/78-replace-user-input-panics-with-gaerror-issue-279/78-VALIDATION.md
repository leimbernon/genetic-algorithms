---
phase: 78
slug: replace-user-input-panics-with-gaerror-issue-279
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-19
---

# Phase 78 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 78-01-01 | 01 | 1 | GaError::InternalError | — | N/A | unit | `cargo test` | ✅ | ⬜ pending |
| 78-01-02 | 01 | 1 | EDA/PSO/CMA empty pop | — | N/A | unit | `cargo test` | ✅ | ⬜ pending |
| 78-01-03 | 01 | 1 | Cellular/ALPS new() | — | N/A | unit | `cargo test` | ✅ | ⬜ pending |
| 78-01-04 | 01 | 1 | OX crossover | — | N/A | unit | `cargo test` | ✅ | ⬜ pending |
| 78-01-05 | 01 | 1 | selection.rs Lexicase | — | N/A | unit | `cargo test` | ✅ | ⬜ pending |
| 78-01-06 | 01 | 1 | generation.rs mutex | — | N/A | unit | `cargo test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
