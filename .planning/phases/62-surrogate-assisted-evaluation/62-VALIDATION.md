---
phase: 62
slug: surrogate-assisted-evaluation
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-09
revised: 2026-06-09
---

# Phase 62 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test test_surrogate` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test test_surrogate`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 62-01-01 | 01 | 1 | SC-1 (D-01, D-02) | — | N/A | compile | `cargo build --lib && cargo doc --no-deps --lib` | ✅ W0 | ⬜ pending |
| 62-01-02 | 01 | 1 | SC-2c (D-10) | — | N/A | compile | `cargo build --lib --features serde` | ✅ W0 | ⬜ pending |
| 62-01-03 | 01 | 1 | SC-1a, SC-1d, SC-1g, SC-2c | — | N/A | unit | `cargo test --test test_surrogate --features serde` | ✅ W0 | ⬜ pending |
| 62-02-01 | 02 | 2 | SC-1e, SC-1f (D-03) | — | N/A | unit | `cargo test --test test_surrogate invalid_fraction_zero_rejected invalid_fraction_over_one_rejected boundary_fraction_one_accepted` | ✅ W0 | ⬜ pending |
| 62-02-02 | 02 | 2 | D-04, D-05, D-06, D-08 (hot-path insertion) | — | N/A | compile + regression | `cargo build --lib && cargo clippy --lib -- -D warnings && cargo test --lib && cargo check --target wasm32-unknown-unknown` | ✅ W0 | ⬜ pending |
| 62-02-03 | 02 | 2 | SC-1b, SC-1c, SC-2a, SC-2b, SC-3 (D-09, D-10, D-11) | — | N/A | integration | `cargo test --test test_surrogate` | ✅ W0 | ⬜ pending |
| 62-03-01 | 03 | 3 | end-to-end demonstration | — | N/A | example | `cargo run --example surrogate_rastrigin --release` | ✅ W0 | ⬜ pending |
| 62-03-02 | 03 | 3 | full CI matrix + SUMMARY | — | N/A | full suite | `cargo test && cargo test --features serde && cargo clippy --all-targets -- -D warnings && cargo doc --no-deps && cargo check --target wasm32-unknown-unknown` | ✅ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/test_surrogate.rs` (flat path) — Plan 01 Task 3 creates four trait-only / pure-math / serde-only tests (SC-1a, SC-1d, SC-1g, SC-2c) with zero `#[ignore]`; Plan 02 appends engine-runtime tests (SC-1b, SC-1c, SC-1e, SC-1f, SC-2a, SC-2b, SC-3) — also zero `#[ignore]`.
- [x] `src/fitness/surrogate.rs` — trait definition created by Plan 01 Task 1.

*Existing infrastructure (`cargo test`) covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Surrogate-reduced fitness call count observable in real GA run | SC-2 | Requires an instrumented fitness function with a counter | Plan 03 Task 1: `examples/surrogate_rastrigin.rs` includes an embedded `assert!` for `gen_stats.true_fitness_calls < offspring_count` — promoted from manual to automated via the example smoke test |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (flat-path `tests/test_surrogate.rs`)
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved (revised 2026-06-09 to address checker feedback: 3-task Plan 01 structure, intermediate hot-path checkpoint in Plan 02, flat test path)
