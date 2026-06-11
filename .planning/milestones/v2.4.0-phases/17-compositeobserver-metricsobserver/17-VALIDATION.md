---
phase: 17
slug: compositeobserver-metricsobserver
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 17 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo test --features observer-metrics && cargo clippy` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run full suite (`cargo test && cargo test --features serde && cargo test --features observer-metrics && cargo clippy`)
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 17-01-01 | 01 | 1 | COMP-01 | unit | `cargo test test_composite` | ⬜ pending |
| 17-01-02 | 01 | 1 | COMP-01 | compile | `cargo build` | ⬜ pending |
| 17-02-01 | 02 | 2 | COMP-02 | unit | `cargo test --features observer-metrics test_metrics` | ⬜ pending |
| 17-02-02 | 02 | 2 | COMP-02 | compile | `cargo build --features observer-metrics` | ⬜ pending |
| 17-02-03 | 02 | 2 | COMP-03 | compile | `cargo build` (default features, no observer-metrics) | ⬜ pending |
| 17-03-01 | 03 | 2 | COMP-01/02/03 | integration | `cargo test --features observer-metrics` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — no new test framework, config, or tooling needed. `cargo test` already handles unit + integration tests. The `observer-metrics` feature gate follows the established `observer-tracing` pattern.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No data races in island parallel execution with MetricsObserver | COMP-03 | Requires criterion benchmark or thread sanitizer run | Run `cargo test --features observer-metrics -- --test-threads=4` and confirm no panics; optionally run criterion bench for island + MetricsObserver |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
