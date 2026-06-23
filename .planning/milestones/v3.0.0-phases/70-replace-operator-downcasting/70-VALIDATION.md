---
phase: 70
slug: replace-operator-downcasting
status: verified
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-18
---

# Phase 70 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Built-in `cargo test` (rustc test harness) |
| **Config file** | none — standard Cargo test layout |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` (includes doc-tests) |
| **Estimated runtime** | ~28 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test` + `cargo clippy` + `cargo fmt --check`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 28 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 70-01-01 | 01 | 1 | (no downcast calls) | — | N/A (internal refactor) | compilation | `cargo check` | ✅ | ✅ green |
| 70-01-02 | 01 | 1 | (all operators work) | — | N/A | integration | `cargo test` | ✅ (268 tests) | ✅ green |
| 70-01-03 | 01 | 1 | (wasm32 compiles) | — | N/A | compilation | `cargo check --target wasm32-unknown-unknown` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- None — existing test infrastructure covers all phase requirements. The 12 mutation test files already test the operators via the factory path, which exercises the exact code being refactored.

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | — | — | — |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 27s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** verified 2026-06-18
