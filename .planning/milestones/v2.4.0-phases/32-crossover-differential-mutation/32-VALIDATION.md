---
phase: 32
slug: crossover-differential-mutation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-04
---

# Phase 32 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde && cargo clippy`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 32-01-01 | 01 | 1 | CRS-01 | — | N/A | unit | `cargo test edge_recombination` | ✅ | ⬜ pending |
| 32-01-02 | 01 | 1 | CRS-01 | — | N/A | unit | `cargo test edge_recombination` | ❌ W0 | ⬜ pending |
| 32-02-01 | 02 | 1 | MUT-04 | — | N/A | unit | `cargo test differential` | ❌ W0 | ⬜ pending |
| 32-02-02 | 02 | 1 | MUT-04 | — | N/A | unit | `cargo test differential` | ❌ W0 | ⬜ pending |
| 32-03-01 | 03 | 2 | CRS-01, MUT-04 | — | N/A | integration | `cargo test && cargo test --features serde` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_crossover_edge_recombination.rs` — stubs for CRS-01
- [ ] `tests/test_mutation_differential.rs` — stubs for MUT-04

*Existing `cargo test` infrastructure covers all phase requirements — no additional framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ERX offspring visually preserves adjacency relationships from parents | CRS-01 | Algorithm correctness requires human inspection of gene ordering | Run `cargo test edge_recombination -- --nocapture` and verify printed output shows expected neighbor preservation |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
