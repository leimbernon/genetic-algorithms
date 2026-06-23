---
phase: 51
slug: multi-parent-crossover-self-adaptive-mutation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-23
---

# Phase 51 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --test test_crossover_undx --test test_crossover_spx --test test_crossover_pcx` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run the per-task command from the table below
- **After every plan wave:** Run `cargo test && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green (including serde and wasm check)
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 51-01-01 | 01 | 0 | TRAITS-02 | — | N/A | unit | `cargo test --test test_self_adaptive` (in `tests/traits/`) | ❌ W0 | ⬜ pending |
| 51-02-01 | 02 | 1 | CRS-02 | — | N/A | unit | `cargo test --test test_crossover_undx` (in `tests/operations/`) | ❌ W0 | ⬜ pending |
| 51-02-02 | 02 | 1 | CRS-03 | — | N/A | unit | `cargo test --test test_crossover_spx` (in `tests/operations/`) | ❌ W0 | ⬜ pending |
| 51-02-03 | 02 | 1 | CRS-04 | — | N/A | unit | `cargo test --test test_crossover_pcx` (in `tests/operations/`) | ❌ W0 | ⬜ pending |
| 51-03-01 | 03 | 1 | MUT-05 | — | N/A | unit | `cargo test --test test_mutation_self_adaptive` (in `tests/operations/`) | ❌ W0 | ⬜ pending |
| 51-04-01 | 04 | 2 | CRS-02,CRS-03,CRS-04,MUT-05,TRAITS-02 | — | N/A | integration | `cargo test --test test_multi_parent_integration && cargo test --features serde && cargo check --target wasm32-unknown-unknown` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Note: `RealValued` (TRAITS-02) is a marker trait — compile-time only, no behavioral test required. Its compile-time enforcement is exercised transitively via the multi-parent integration test in Plan 04 (`Ga<RangeChromosome<f64>>` requires `RealValued`).*

---

## Wave 0 Requirements

- [ ] `tests/operations/test_crossover_undx.rs` — stubs for UNDX operator (CRS-02)
- [ ] `tests/operations/test_crossover_spx.rs` — stubs for SPX operator (CRS-03)
- [ ] `tests/operations/test_crossover_pcx.rs` — stubs for PCX operator (CRS-04)
- [ ] `tests/operations/test_mutation_self_adaptive.rs` — stubs for SelfAdaptiveGaussian (MUT-05)
- [ ] `tests/traits/test_self_adaptive.rs` — stubs for SelfAdaptive trait + RangeChromosome impl (TRAITS-02)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| UNDX sigma constants match algorithm literature | CRS-02 | Math constant verification requires domain knowledge | Run UNDX crossover on 2D Rastrigin, inspect offspring distribution shape |
| Self-adaptive sigma co-evolution convergence | MUT-05 | Stochastic behavior over many generations | Run 100-gen GA on sphere function, verify sigma mean decreases as fitness improves |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
</content>
</invoke>
