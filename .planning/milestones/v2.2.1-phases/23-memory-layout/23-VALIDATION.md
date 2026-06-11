---
phase: 23
slug: memory-layout
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-03
---

# Phase 23 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 23-01-01 | 01 | 1 | MEM-01 | compile+unit | `cargo test` | ✅ | ⬜ pending |
| 23-01-02 | 01 | 1 | MEM-02 | compile+unit | `cargo test` | ✅ | ⬜ pending |
| 23-01-03 | 01 | 1 | MEM-03 | compile+serde | `cargo test && cargo test --features serde` | ✅ | ⬜ pending |
| 23-01-04 | 01 | 1 | MEM-04 | compile | `cargo build` | ✅ | ⬜ pending |
| 23-01-05 | 01 | 1 | MEM-05 | unit | `cargo test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

All MEM-01 through MEM-05 changes are internal to existing files with existing test coverage. No new test stubs required before execution.

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
