---
phase: 72
slug: audit-ignored-doctests
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-18
---

# Phase 72 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test --doc` (Rust built-in) |
| **Config file** | none — uses standard cargo |
| **Quick run command** | `cargo test --doc` |
| **Full suite command** | `cargo test --doc --all-features` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --doc`
- **After every plan wave:** Run `cargo test --doc --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 72-01-01 | 01 | 1 | — | — | N/A | smoke | `cargo test --doc 2>&1 \| grep -c "ignored"` returns 0 | ✅ | ⬜ pending |
| 72-01-02 | 01 | 1 | — | — | N/A | smoke | `cargo test --doc` exits 0 | ✅ | ⬜ pending |
| 72-01-03 | 01 | 1 | — | — | N/A | grep | `grep -rn '```ignore\|```rust,ignore' src/` returns empty | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing infrastructure (`cargo test --doc`) covers all phase requirements.

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
