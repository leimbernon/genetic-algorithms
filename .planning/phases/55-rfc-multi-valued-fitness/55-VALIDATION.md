---
phase: 55
slug: rfc-multi-valued-fitness
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-29
---

# Phase 55 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 55-01-01 | 01 | 1 | REQ-TBD | — | N/A | unit | `cargo test vector_fitness` | ❌ W0 | ⬜ pending |
| 55-01-02 | 01 | 1 | REQ-TBD | — | N/A | unit | `cargo test built_in_chromosomes` | ❌ W0 | ⬜ pending |
| 55-02-01 | 02 | 2 | REQ-TBD | — | N/A | integration | `cargo test mo_engines` | ❌ W0 | ⬜ pending |
| 55-03-01 | 03 | 3 | REQ-TBD | — | N/A | unit | `cargo test --features serde` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/vector_fitness.rs` — stubs for VectorFitness trait tests
- [ ] `tests/mo_engines_vector.rs` — stubs for MO engine integration with VectorFitness

*Existing infrastructure covers most phase requirements (cargo test is already in place).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WASM compilation | REQ-TBD | Requires wasm32 target | `cargo check --target wasm32-unknown-unknown` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
