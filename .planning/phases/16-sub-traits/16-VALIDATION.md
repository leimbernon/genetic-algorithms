---
phase: 16
slug: sub-traits
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-26
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `Cargo.toml` — existing test configuration |
| **Quick run command** | `cargo test 2>&1 \| tail -5` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test 2>&1 | tail -5`
- **After every plan wave:** Run full suite
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 16-01-01 | 01 | 1 | SUB-01, SUB-02 | build | `cargo build 2>&1 \| grep -c error` → 0 | ✅ | ⬜ pending |
| 16-01-02 | 01 | 1 | SUB-01 | build | `cargo build 2>&1 \| grep -c error` → 0 | ✅ | ⬜ pending |
| 16-01-03 | 01 | 1 | SUB-02 | build | `cargo build 2>&1 \| grep -c error` → 0 | ✅ | ⬜ pending |
| 16-02-01 | 02 | 2 | SUB-03 | build | `cargo build 2>&1 \| grep -c error` → 0 | ✅ | ⬜ pending |
| 16-03-01 | 03 | 3 | SUB-01, SUB-02, SUB-03 | unit | `cargo test test_island_observer test_nsga2_observer test_logobserver_all_traits` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_sub_trait_observers.rs` — stubs for SUB-01, SUB-02, SUB-03 integration tests

*Note: Existing `tests/` directory has `test_observer.rs` — new sub-trait tests follow the same pattern.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| None | — | All behaviors have automated verification | — |

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
