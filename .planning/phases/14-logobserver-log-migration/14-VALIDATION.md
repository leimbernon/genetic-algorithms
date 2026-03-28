---
phase: 14
slug: logobserver-log-migration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — uses Cargo.toml test discovery |
| **Quick run command** | `cargo test test_log_observer` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test test_log_observer`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 14-W0-01 | W0 | 0 | LOG-01, LOG-02 | unit/integration | `cargo test test_log_observer` | ❌ W0 | ⬜ pending |
| 14-01-01 | 01 | 1 | LOG-01 | unit | `cargo test test_log_observer_implements_trait` | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | LOG-01 | integration | `cargo test test_log_observer_attaches` | ❌ W0 | ⬜ pending |
| 14-01-03 | 01 | 1 | LOG-01 | unit | `cargo test test_log_observer_is_send_sync` | ❌ W0 | ⬜ pending |
| 14-01-04 | 01 | 1 | LOG-02 | unit | `cargo test test_ga_has_no_direct_log_calls` | ❌ W0 | ⬜ pending |
| 14-01-05 | 01 | 1 | LOG-03 | compile | `cargo build` | ✅ existing | ⬜ pending |
| 14-01-06 | 01 | 1 | LOG-03 | unit | `cargo test test_log_observer_is_unit_struct` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_log_observer.rs` — new test file with stubs for LOG-01 (attaches, implements trait, Send+Sync), LOG-02 (grep check or structural verification), LOG-03 (unit struct check)

*Note: `tests/test_observer.rs` already exists from Phase 13; LogObserver tests go in a separate `tests/test_log_observer.rs` file for clarity.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Log output format matches v2.1.0 exactly | LOG-01 | Requires visual comparison of log output text, targets, and KV fields | Run a GA with LogObserver attached + `env_logger` configured; compare output against recorded v2.1.0 baseline |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
