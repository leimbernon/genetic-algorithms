---
phase: 15
slug: tracingobserver
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `Cargo.toml` — existing test configuration |
| **Quick run command** | `cargo test --features observer-tracing 2>&1 \| tail -5` |
| **Full suite command** | `cargo test --features observer-tracing && cargo test --features "observer-tracing,serde" && cargo clippy --features observer-tracing` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features observer-tracing 2>&1 | tail -5`
- **After every plan wave:** Run full suite
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 1 | TRAC-02 | build | `cargo build 2>&1 \| grep -c error` → 0 | ✅ | ⬜ pending |
| 15-01-02 | 01 | 1 | TRAC-02 | build | `cargo build --features observer-tracing 2>&1 \| grep -c error` → 0 | ✅ | ⬜ pending |
| 15-01-03 | 01 | 1 | TRAC-01 | unit | `cargo test --features observer-tracing test_tracing_observer` | ❌ W0 | ⬜ pending |
| 15-01-04 | 01 | 1 | TRAC-03 | integration | `cargo test --features observer-tracing test_logtracer_no_recursion` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_tracing_observer.rs` — stubs for TRAC-01 (observer emits events) and TRAC-03 (LogTracer coexistence)
- [ ] Or `src/observer/tracing_observer.rs` `#[cfg(test)] mod tests` — unit tests for TracingObserver hooks

*Note: Existing `tests/` directory has `test_observer.rs` — new tracing tests can follow the same pattern.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Spans visible in Jaeger/OTEL subscriber | TRAC-01 | Requires a live tracing subscriber (not easily unit-testable) | Attach `tracing_subscriber::fmt::init()`, run 5 generations, verify console output shows `ga_run` and `ga_generation` spans |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
