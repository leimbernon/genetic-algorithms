---
phase: 52
slug: variable-length-chromosomes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-24
---

# Phase 52 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --test '*variable*' 2>/dev/null || cargo test` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 52-01-01 | 01 | 0 | MUT-06, CHR-01 | — | N/A | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 52-02-01 | 02 | 1 | MUT-06 | — | N/A | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 52-02-02 | 02 | 1 | CHR-01 | — | N/A | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 52-03-01 | 03 | 2 | CHR-01 | — | N/A | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 52-03-02 | 03 | 2 | CHR-02 | — | N/A | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 52-04-01 | 04 | 3 | CHR-01, CHR-02, MUT-06 | — | N/A | integration | `cargo test && cargo test --features serde` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_variable_length.rs` — stubs for MUT-06, CHR-01, CHR-02
- [ ] Wave 0 stubs for: Mutation::Insertion, Mutation::Deletion, Mutation::PermutationInsert rename, Crossover::VariableLength, check_compatible_length guard, parsimony pressure

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Length grows/shrinks across generations | CHR-01 | Statistical — verify visually in multi-generation run | Run example with Variable { min: 2, max: 10 } and print chromosome lengths per generation |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
