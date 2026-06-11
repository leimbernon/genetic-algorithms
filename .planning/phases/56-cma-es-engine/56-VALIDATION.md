---
phase: 56
slug: cma-es-engine
status: draft
nyquist_compliant: false
wave_0_complete: true
created: 2026-06-01
---

# Phase 56 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `Cargo.toml` — existing test configuration |
| **Quick run command** | `cargo test --test test_engines cma` |
| **Full suite command** | `cargo test && cargo test --features serde` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test test_engines cma`
- **After every plan wave:** Run `cargo test && cargo test --features serde`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 56-rename-01 | rename | 1 | D-01/D-02 | — | N/A | unit | `cargo test --test test_de` | ✅ covered by Plan 02 Task 3 (Wave 0 scaffold) | ⬜ pending |
| 56-rename-02 | rename | 1 | D-01 | — | N/A | compile | `cargo check` | ✅ | ⬜ pending |
| 56-cma-01 | cma-engine | 2 | D-03/D-04/D-05 | — | N/A | integration | `cargo test --test test_engines cma` | ✅ covered by Plan 02 Task 3 (Wave 0 scaffold) | ⬜ pending |
| 56-cma-02 | cma-engine | 2 | D-06 | — | N/A | unit | `cargo test --test test_engines cma observer` | ✅ covered by Plan 02 Task 3 (Wave 0 scaffold) | ⬜ pending |
| 56-cma-03 | cma-engine | 2 | D-07 | — | N/A | example | `cargo run --example cma_es_rastrigin` | ✅ covered by Plan 02 Task 3 (Wave 0 scaffold) | ⬜ pending |
| 56-wasm-01 | wasm | 3 | WASM | — | N/A | compile | `cargo check --target wasm32-unknown-unknown` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Note: All ❌ W0 entries above have been resolved — Plan 02 Task 3 lands the Wave 0 test scaffolding (`tests/engines/cma/mod.rs`, `tests/engines/cma/test_cma.rs` with all 11 stubs, and the `mod cma { mod test_cma; }` entry in `tests/test_engines.rs`). Plan 03 Task 3 un-ignores them and turns Status from ⬜ pending into ✅ green.*

---

## Wave 0 Requirements

- [x] `tests/engines/cma/mod.rs` — module entry point for CMA-ES tests (delivered by Plan 02 Task 3)
- [x] `tests/engines/cma/test_cma.rs` — stubs for D-03 through D-07 (delivered by Plan 02 Task 3)
- [x] Entry added to `tests/test_engines.rs` (or equivalent integration test root) (delivered by Plan 02 Task 3)

*Existing infrastructure covers the rename validation via `cargo check` and existing DE tests.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CMA-ES converges on sphere function (5D) | D-03/D-05 | Convergence speed depends on RNG seed, loose threshold needed | Run `cargo test test_cma_sphere` and observe best_fitness < 1.0 after 500 generations |
| Observer hooks fire in correct order | D-06 | Ordering assertion requires mock observer | Run `cargo test test_cma_observer_fires` with CountingObserver and assert call counts |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
