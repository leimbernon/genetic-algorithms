---
phase: 63
slug: visualization-pareto-front-plotting-example-images
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-09
---

# Phase 63 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none |
| **Quick run command** | `cargo test --features visualization` |
| **Full suite command** | `cargo test --features visualization && cargo test --features "visualization,serde"` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features visualization`
- **After every plan wave:** Run `cargo test --features visualization && cargo clippy --features visualization`
- **Before `/gsd:verify-work`:** Full suite + smoke examples + wasm check must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| SC-1a | 01 | 1 | SC-1 | — | N/A | unit | `cargo test --features visualization test_plot_pareto_front_2d` | ❌ W0 | ⬜ pending |
| SC-1b | 01 | 1 | SC-1 | — | N/A | unit | `cargo test --features visualization test_plot_pareto_front_3d` | ❌ W0 | ⬜ pending |
| SC-1c | 01 | 1 | SC-1 | — | N/A | unit | `cargo test --features visualization test_plot_true_fitness_calls` | ❌ W0 | ⬜ pending |
| SC-1d | 01 | 1 | SC-1 | — | N/A | unit | `cargo test --features visualization test_pareto_error_cases` | ❌ W0 | ⬜ pending |
| SC-2 | 02 | 2 | SC-2 | — | N/A | smoke | `cargo run --example nsga2_zdt1 --features visualization -- --plot` | ✅ existing | ⬜ pending |
| SC-4 | 01 | 1 | SC-4 | — | N/A | compile | `cargo check --target wasm32-unknown-unknown --lib --features visualization` | ✅ CI updated | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/observe/visualization/test_visualization.rs` — unit tests for `plot_pareto_front_2d`, `plot_pareto_front_3d`, `plot_true_fitness_calls`, and error cases (SC-1) — stubs added in Plan 01 Task 1

*These test stubs must be created before Wave 1 execution begins.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| PNG files committed to `docs/images/` and linked from `README.md` | SC-3 | File commit + markdown link verification | Run `ls docs/images/` and check README links |
| Pareto front image visually correct | SC-2 | Visual inspection of PNG output | Open `docs/images/nsga2_zdt1.png` and verify Pareto front shape |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
