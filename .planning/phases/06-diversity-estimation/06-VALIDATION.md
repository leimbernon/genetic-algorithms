---
phase: 6
slug: diversity-estimation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-20
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` |
| **Config file** | none — `cargo test` |
| **Quick run command** | `cargo test test_stats test_ga test_extension test_mutation_dynamic` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test test_stats test_ga test_extension test_mutation_dynamic`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 6-01-01 | 01 | 1 | DIV-01 | unit | `cargo test test_stats` | ✅ `tests/test_stats.rs` | ⬜ pending |
| 6-01-02 | 01 | 1 | DIV-01 | unit | `cargo test --features serde serde_generation_stats` | ✅ `tests/test_serde.rs` | ⬜ pending |
| 6-01-03 | 01 | 1 | DIV-02 | integration | `cargo test test_extension` | ✅ `tests/test_extension.rs` | ⬜ pending |
| 6-01-04 | 01 | 1 | DIV-03 | unit | `cargo test test_mutation_dynamic` | ✅ `tests/operations/test_mutation_dynamic.rs` | ⬜ pending |
| 6-01-05 | 01 | 1 | DIV-01 | integration | `cargo test test_ga` | ✅ `tests/test_ga.rs` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test files needed; existing files need targeted additions only.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `diversity` value is semantically meaningful (non-zero, varies with population state) | DIV-01 | Requires GA run with observed population diversity changes | Run the OneMax example for 50 generations and verify `stats[*].diversity` varies |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
