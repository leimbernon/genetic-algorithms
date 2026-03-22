---
phase: 11
slug: advanced-mode-examples
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test + cargo build |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo build --example <name>` |
| **Full suite command** | `cargo test && cargo build --example nsga2_zdt1 --example island_model --example job_scheduling` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo build --example <name>` for the example being built
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | EX-02 | build | `cargo build --example nsga2_zdt1` | ❌ W0 | ⬜ pending |
| 11-02-01 | 02 | 1 | EX-03 | build | `cargo build --example island_model` | ❌ W0 | ⬜ pending |
| 11-03-01 | 03 | 1 | EX-04 | build | `cargo build --example job_scheduling` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements (no new test files needed — validation is `cargo build` + `cargo clippy`).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `nsga2_zdt1` prints Pareto front with trade-off shape | EX-02 | Stochastic output — requires visual inspection | Run `cargo run --example nsga2_zdt1`, confirm (f1, f2) pairs show f1 increasing and f2 decreasing |
| `island_model` prints per-island best + global best | EX-03 | Stochastic output — requires visual inspection | Run `cargo run --example island_model`, confirm each migration round shows per-island values |
| `job_scheduling` prints job order + makespan | EX-04 | Stochastic output — requires visual inspection | Run `cargo run --example job_scheduling`, confirm sequence and makespan are printed |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
