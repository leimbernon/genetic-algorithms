---
phase: 24
slug: minor-improvements
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-04
---

# Phase 24 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test 2>&1 \| tail -5` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test 2>&1 | tail -5`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 24-01-01 | 01 | 1 | MISC-01 | unit | `cargo test && cargo test --features serde` | ✅ | ⬜ pending |
| 24-01-02 | 01 | 1 | MISC-02 | unit | `cargo test test_truncation` | ✅ | ⬜ pending |
| 24-01-03 | 01 | 1 | MISC-03 | unit | `cargo test` | ✅ | ⬜ pending |
| 24-02-01 | 02 | 1 | MISC-04 | unit | `cargo test migration` | ✅ | ⬜ pending |
| 24-02-02 | 02 | 1 | MISC-05 | unit | `cargo test migration` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. All test files already exist — no new test stubs needed. All changes are internal refactors verified by existing tests.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
