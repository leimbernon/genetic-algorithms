---
phase: 57
slug: pso-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-02
---

# Phase 57 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test pso` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test pso`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| PSO-01 | 01 | 1 | RealGene bounds | — | N/A | unit | `cargo test real_gene` | ❌ W0 | ⬜ pending |
| PSO-02 | 01 | 1 | PsoState/PsoEngine | — | N/A | unit | `cargo test pso` | ❌ W0 | ⬜ pending |
| PSO-03 | 01 | 1 | PsoConfiguration | — | N/A | unit | `cargo test pso_config` | ❌ W0 | ⬜ pending |
| PSO-04 | 02 | 2 | Velocity update + boundary | — | N/A | unit | `cargo test pso` | ❌ W0 | ⬜ pending |
| PSO-05 | 02 | 2 | gbest topology | — | N/A | unit | `cargo test pso` | ❌ W0 | ⬜ pending |
| PSO-06 | 02 | 2 | ring topology | — | N/A | unit | `cargo test pso` | ❌ W0 | ⬜ pending |
| PSO-07 | 03 | 3 | Observer hooks | — | N/A | integration | `cargo test pso` | ❌ W0 | ⬜ pending |
| PSO-08 | 03 | 3 | WASM compile | — | N/A | build | `cargo check --target wasm32-unknown-unknown` | ✅ | ⬜ pending |
| PSO-09 | 03 | 3 | pso_rastrigin convergence | — | N/A | integration | `cargo run --example pso_rastrigin` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/pso.rs` — stub tests for PSO-01 through PSO-09 (with `#[ignore]` gates; un-ignore per wave)
- [ ] `src/traits/real_gene.rs` — add `bounds()` method before any PSO code compiles

*Existing infrastructure (`cargo test`, `wasm32` check, serde feature) covers all other phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `pso_rastrigin` visual convergence | D-10 | Example output requires human inspection | Run `cargo run --example pso_rastrigin` and verify fitness improves toward 0 over generations |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
