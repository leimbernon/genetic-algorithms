---
phase: 30
slug: observer-wiring-de-benchmark
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-28
---

# Phase 30 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + criterion benchmarks |
| **Config file** | `Cargo.toml` (existing) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 30-01-01 | 01 | 1 | OBS-01 | — | N/A | integration | `cargo test` | ✅ | ⬜ pending |
| 30-01-02 | 01 | 1 | OBS-02 | — | N/A | integration | `cargo test` | ✅ | ⬜ pending |
| 30-01-03 | 01 | 1 | OBS-03 | — | N/A | integration | `cargo test` | ✅ | ⬜ pending |
| 30-01-04 | 01 | 1 | OBS-04 | — | N/A | integration | `cargo test` | ✅ | ⬜ pending |
| 30-02-01 | 02 | 2 | OBS-05 | — | N/A | benchmark | `cargo bench --bench de` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. Tests will be added to `tests/` per project convention.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Benchmark output shows DE-vs-GA comparison report | OBS-05 | criterion output requires manual inspection | Run `cargo bench --bench de` and verify both DE and GA groups appear |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
