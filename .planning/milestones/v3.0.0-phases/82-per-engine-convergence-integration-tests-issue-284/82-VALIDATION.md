---
phase: 82
slug: per-engine-convergence-integration-tests-issue-284
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-22
---

# Phase 82 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --test test_de --test test_scatter --test test_cellular --test test_alps --test test_cma --test test_pso` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test <engine_test_file>`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 82-01-01 | 01 | 1 | SC-1 | — | N/A | integration | `cargo test --test test_de test_de_convergence` | ❌ W0 | ⬜ pending |
| 82-01-02 | 01 | 1 | SC-2 | — | N/A | integration | `cargo test --test test_scatter test_scatter_convergence` | ❌ W0 | ⬜ pending |
| 82-01-03 | 01 | 1 | SC-3 | — | N/A | integration | `cargo test --test test_cellular test_cellular_convergence` | ❌ W0 | ⬜ pending |
| 82-01-04 | 01 | 1 | SC-4 | — | N/A | integration | `cargo test --test test_alps test_alps_convergence` | ❌ W0 | ⬜ pending |
| 82-01-05 | 01 | 1 | SC-5 | — | N/A | integration | `cargo test --test test_cma test_cma_convergence` | ❌ W0 | ⬜ pending |
| 82-01-06 | 01 | 1 | SC-5 | — | N/A | integration | `cargo test --test test_cma test_cma_ipop_restart_convergence` | ❌ W0 | ⬜ pending |
| 82-01-07 | 01 | 1 | SC-6 | — | N/A | integration | `cargo test --test test_pso test_pso_convergence` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements. Each engine test file already has `sphere` and `random_pop` helpers.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `cargo test --features serde` passes | SC-9 | Feature-gated tests require explicit flag | Run `cargo test --features serde` and verify 0 failures |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
