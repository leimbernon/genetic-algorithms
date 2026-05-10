---
phase: 39
slug: multi-objective-quality-indicators-hypervolume-gd-igd-spread
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 39 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test --features serde` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test --features serde`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 39-01-01 | 01 | 1 | MOO-05 | — | N/A | unit | `cargo test --lib indicators` | ❌ W0 | ⬜ pending |
| 39-01-02 | 01 | 1 | MOO-05 | — | N/A | unit | `cargo test --lib indicators` | ❌ W0 | ⬜ pending |
| 39-01-03 | 01 | 1 | MOO-05 | — | N/A | unit | `cargo test --lib indicators` | ❌ W0 | ⬜ pending |
| 39-01-04 | 01 | 1 | MOO-05 | — | N/A | unit | `cargo test --lib indicators` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/engines/multi_objective/indicators/hypervolume_tests.rs` — stubs for MOO-05 hypervolume
- [ ] `tests/engines/multi_objective/indicators/generational_distance_tests.rs` — stubs for MOO-05 GD
- [ ] `tests/engines/multi_objective/indicators/inverted_generational_distance_tests.rs` — stubs for MOO-05 IGD
- [ ] `tests/engines/multi_objective/indicators/spread_tests.rs` — stubs for MOO-05 spread

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Algorithmic correctness of indicator formulas | MOO-05 | Formal verification of mathematical correctness requires domain expertise | Verify against published reference values: ZDT1 true Pareto front for GD/IGD, known hypervolume values for 2D test cases |

*Note: Automated tests cover analytical verification with inline reference data per D-04.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
