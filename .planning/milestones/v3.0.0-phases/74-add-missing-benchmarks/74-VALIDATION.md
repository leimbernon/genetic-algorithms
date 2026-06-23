---
phase: 74
slug: add-missing-benchmarks
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-18
---

# Phase 74 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust / divan (already in dev-dependencies) |
| **Config file** | Cargo.toml `[[bench]]` entries |
| **Quick run command** | `cargo bench --no-run` |
| **Full suite command** | `cargo bench --no-run && cargo test` |
| **Estimated runtime** | ~30 seconds (compile check only) |

---

## Sampling Rate

- **After every task commit:** Run `cargo bench --no-run`
- **After every plan wave:** Run `cargo bench --no-run && cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 74-01-01 | 01 | 1 | #267 | — | N/A | compile | `cargo bench --no-run` | ❌ W0 | ⬜ pending |
| 74-01-02 | 01 | 1 | #267 | — | N/A | compile | `cargo bench --no-run` | ❌ W0 | ⬜ pending |
| 74-01-03 | 01 | 1 | #267 | — | N/A | compile | `cargo bench --no-run` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `benches/pso.rs` — PSO engine benchmark stub
- [ ] `benches/cma_es.rs` — CMA-ES engine benchmark stub
- [ ] `benches/eda.rs` — EDA engine benchmark stub
- [ ] `benches/alps.rs` — already exists (template)
- [ ] `benches/island.rs` — Island GA benchmark stub
- [ ] `benches/gp.rs` — GP engine benchmark stub
- [ ] `benches/aos.rs` — AOS feature benchmark stub
- [ ] `benches/surrogate.rs` — Surrogate feature benchmark stub
- [ ] `benches/batch_fitness.rs` — Batch fitness feature benchmark stub
- [ ] `Cargo.toml` — `[[bench]]` entries for each new file

*All new bench files must be created; compile check (`cargo bench --no-run`) validates they are wired.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Benchmark groups produce meaningful output | #267 | Requires actual benchmark run | Run `cargo bench` and verify each engine group produces timing output |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
