---
phase: 8
slug: reporter-trait
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test reporter` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test reporter`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | REP-01, REP-02 | unit | `cargo test reporter` | ❌ W0 | ⬜ pending |
| 08-01-02 | 01 | 1 | REP-01 | unit | `cargo test reporter` | ❌ W0 | ⬜ pending |
| 08-02-01 | 02 | 2 | REP-03 | unit | `cargo test simple_reporter` | ❌ W0 | ⬜ pending |
| 08-02-02 | 02 | 2 | REP-04 | unit | `cargo test duration_reporter` | ❌ W0 | ⬜ pending |
| 08-02-03 | 02 | 2 | REP-01, REP-02 | integration | `cargo test test_reporter` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/reporter/mod.rs` — `Reporter<U>` trait definition with default no-op impls
- [ ] `src/reporter/noop.rs` — `NoopReporter` struct
- [ ] `src/reporter/simple.rs` — `SimpleReporter` struct with unit tests
- [ ] `src/reporter/duration.rs` — `DurationReporter` struct with unit tests
- [ ] `tests/reporter/test_reporter.rs` — integration tests for hook invocation order and GA loop wiring

*All files are created as part of plan execution — tests are co-located with implementation (Wave 0 TDD pattern).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SimpleReporter stdout output visible | REP-03 | stdout capture in tests is possible but the exact format is best verified by eye | Run a short GA with `SimpleReporter::new(1)` attached; confirm one-line summary prints per generation |

*All other behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
