---
phase: 18
slug: observer-api-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --test test_observer --test test_tracing_observer --test test_composite_observer` |
| **Full suite command** | `cargo test && cargo test --features observer-tracing && cargo test --features observer-metrics && cargo test --features "observer-tracing,observer-metrics"` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run quick run command
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 18-01-01 | 01 | 1 | TRAC-01, COMP-01 | compile | `cargo build --features observer-tracing` | ✅ | ⬜ pending |
| 18-01-02 | 01 | 1 | TRAC-01, COMP-01 | integration | `cargo test --test test_tracing_observer --features observer-tracing` | ✅ | ⬜ pending |
| 18-01-03 | 01 | 1 | LOG-01, OBS-01 | integration | `cargo test --test test_observer` | ✅ | ⬜ pending |
| 18-01-04 | 01 | 1 | OBS-01, COMP-02 | integration | `cargo test --test test_observer && cargo test --test test_metrics_observer --features observer-metrics` | ✅ | ⬜ pending |
| 18-02-01 | 02 | 2 | OBS-02, OBS-01 | compile | `cargo build` | ✅ | ⬜ pending |
| 18-02-02 | 02 | 2 | OBS-02, OBS-01 | integration | `cargo test --test test_observer_reexports` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_observer_reexports.rs` — stubs for OBS-01, OBS-02 re-export tests (NoopObserver, ExtensionEvent, TerminationCause accessible from crate root)

*All other tests use existing test files — no new stubs required.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| LogObserver output order matches pre-v2.2.0 | LOG-01 | Extension event log ordering requires visual inspection against known pre-v2.2.0 output | Run a GA with LogObserver and an extension trigger; confirm extension log appears before generation-end log in the output |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
