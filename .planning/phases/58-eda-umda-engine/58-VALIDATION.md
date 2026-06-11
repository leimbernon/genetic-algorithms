---
phase: 58
slug: eda-umda-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-04
---

# Phase 58 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --test test_eda` |
| **Full suite command** | `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test test_eda`
- **After every plan wave:** Run `cargo test && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 58-01-01 | 01 | 1 | EDA engine module + lib.rs re-exports | — | N/A | unit + compile | `cargo test --test test_eda && cargo check` | ❌ W0 | ⬜ pending |
| 58-01-02 | 01 | 1 | Test scaffold (Bernoulli/Gaussian/observer stubs) | — | N/A | unit | `cargo test --test test_eda` | ❌ W0 | ⬜ pending |
| 58-02-01 | 02 | 2 | Bernoulli + Gaussian run() loops + observer wiring + WASM gate | — | N/A | unit + compile | `cargo test --test test_eda && cargo check --target wasm32-unknown-unknown` | ❌ W0 | ⬜ pending |
| 58-02-02 | 02 | 2 | Un-ignore engine runtime tests (convergence + observer hooks) | — | N/A | integration | `cargo test --test test_eda` | ❌ W0 | ⬜ pending |
| 58-03-01 | 03 | 3 | `eda_trap` example converges + integration test | — | N/A | integration | `cargo run --example eda_trap` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Note: lib.rs re-exports (previously row 58-04-01) are covered by Plan 01 Task 1's compile gate. WASM compatibility (previously row 58-05-01) is verified inside every plan's verify block via `cargo check --target wasm32-unknown-unknown`.*

---

## Wave 0 Requirements

- [ ] `tests/engines/eda/test_eda.rs` — test stubs for EDA engine (Bernoulli, Gaussian, observer)
- [ ] `tests/engines/eda/mod.rs` — module declaration

*Existing infrastructure (cargo test) covers the framework; only test file stubs are needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `eda_trap` converges to global optimum | Phase success criterion 4 | Stochastic convergence not deterministic | `cargo run --example eda_trap` — confirm best fitness approaches optimal within 200 generations |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
